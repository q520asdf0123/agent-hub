//! REST API handlers 与应用状态（CONTRACT §3.4）。路由固定在 main.rs。

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::RwLock;

use crate::config::{self, Config};
use crate::types::{ChatReq, ProjectInfo, SessionSummary};

pub struct AppState {
    pub config: RwLock<Config>,
    pub runs: crate::run::RunRegistry,
    pub instance_id: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(config::load()),
            runs: crate::run::RunRegistry::default(),
            instance_id: format!(
                "{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|duration| duration.as_millis())
                    .unwrap_or(0)
            ),
        }
    }
}

// ---------- 静态文件（编译期内嵌） ----------

pub async fn index_html() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        include_str!("../static/index.html"),
    )
}

pub async fn app_js() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "application/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        include_str!("../static/app.js"),
    )
}

pub async fn sage_scheduler_js() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "application/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        include_str!("../static/sage-scheduler.js"),
    )
}

pub async fn favicon_svg() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "image/svg+xml"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        include_str!("../static/favicon.svg"),
    )
}

pub async fn style_css() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        include_str!("../static/style.css"),
    )
}

// ---------- GET /api/status ----------

pub async fn status() -> impl IntoResponse {
    Json(crate::cli::status().await)
}

pub async fn instance(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(json!({ "instance_id": state.instance_id }))
}

// ---------- GET /api/skills ----------

#[derive(Deserialize)]
pub struct SkillsQuery {
    pub project: Option<String>,
}

pub async fn skills(Query(q): Query<SkillsQuery>) -> Response {
    match tokio::task::spawn_blocking(move || crate::skills::list_skills(q.project.as_deref()))
        .await
    {
        Ok(list) => Json(list).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("技能扫描失败: {e}") })),
        )
            .into_response(),
    }
}

// ---------- POST /api/sage（SAGE 智能路由决策） ----------

#[derive(Deserialize)]
pub struct SageReq {
    pub prompt: String,
    pub agent: Option<String>,
    /// 本任务此前已失败过的 agent（ExecutionState.failed_agents，触发失败重路由）
    pub failed: Option<Vec<String>>,
    /// 运行期确认当前 provider 不提供的执行体（model_not_found 等）；同样硬排除，
    /// 但不计入本任务的 failure_count。
    pub unavailable: Option<Vec<String>>,
    /// 官方 ExecutionState 的前端可观测部分。
    pub state: Option<serde_json::Value>,
    /// Task 约束；服务端会覆盖 available_agents，避免选择未安装 CLI。
    pub constraints: Option<serde_json::Value>,
}

pub async fn sage_route(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SageReq>,
) -> Response {
    let failed = body.failed.unwrap_or_default();
    let unavailable = body.unavailable.unwrap_or_default();
    let status = crate::cli::status().await;
    let mut available = Vec::new();
    if status.claude.installed {
        available.push("claude");
    }
    if status.codex.installed {
        available.push("codex");
    }
    let mut constraints = match body.constraints.unwrap_or_else(|| json!({})) {
        serde_json::Value::Object(map) => map,
        _ => serde_json::Map::new(),
    };
    constraints.insert("available_agents".to_string(), json!(available));
    let mut active_counts: HashMap<String, usize> = HashMap::new();
    for (_, run) in state.runs.list_all() {
        if !run.is_done() {
            *active_counts.entry(run.agent.clone()).or_default() += 1;
        }
    }
    let loads = active_counts
        .into_iter()
        .map(|(agent, count)| (agent, (count as f64 / 4.0).min(1.0)))
        .collect::<HashMap<_, _>>();
    constraints.insert("loads".to_string(), json!(loads));
    if let Ok(models) = tokio::task::spawn_blocking(crate::models::discover).await {
        constraints.insert("model_catalog".to_string(), json!(models));
    }
    match crate::sage::route(
        &body.prompt,
        body.agent.as_deref().unwrap_or("claude"),
        &failed,
        &unavailable,
        body.state.unwrap_or_else(|| json!({})),
        serde_json::Value::Object(constraints),
    )
    .await
    {
        Ok(v) => Json(v).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response(),
    }
}

// ---------- POST /api/sage/outcome（执行结果回喂，驱动 SAGE 在线学习） ----------

#[derive(Deserialize)]
pub struct SageOutcomeReq {
    pub decision_blob: serde_json::Value,
    /// 0.0 ~ 1.0
    pub success: f64,
    pub actual_cost: Option<f64>,
    pub actual_latency_ms: Option<f64>,
    /// 分工模式：agent id → 0..1
    pub agent_scores: Option<serde_json::Value>,
    /// 分工模式：需求名 → 0..1
    pub requirement_scores: Option<serde_json::Value>,
}

pub async fn sage_outcome(Json(body): Json<SageOutcomeReq>) -> Response {
    match crate::sage::outcome(
        body.decision_blob,
        body.success,
        body.actual_cost,
        body.actual_latency_ms,
        body.agent_scores,
        body.requirement_scores,
    )
    .await
    {
        Ok(v) => Json(v).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response(),
    }
}

// ---------- POST /api/open（用系统默认程序打开本地文件 / 目录） ----------

#[derive(Deserialize)]
pub struct OpenReq {
    pub path: String,
    pub project: Option<String>,
    /// "reveal" = 在资源管理器中定位该文件；缺省 = 智能打开
    pub mode: Option<String>,
    /// 指定受支持的本机应用 ID；缺省时沿用原有智能打开逻辑。
    pub app: Option<String>,
}

/// 代码 / 文本类扩展名：进编辑器（支持跳行）；其余（Excel/Word/图片/PDF 等）走系统默认程序。
const TEXT_EXTS: &[&str] = &[
    "rs", "py", "js", "ts", "jsx", "tsx", "vue", "java", "php", "go", "c", "cpp", "cc", "h",
    "hpp", "cs", "rb", "lua", "swift", "kt", "dart", "sh", "bash", "bat", "cmd", "ps1", "sql",
    "html", "htm", "css", "scss", "less", "json", "jsonl", "ndjson", "yaml", "yml", "toml",
    "xml", "md", "markdown", "txt", "ini", "cfg", "conf", "env", "log", "properties", "gradle",
    "proto", "diff", "patch", "gitignore", "dockerfile", "makefile", "tf", "svelte", "astro",
];

/// 末尾 `:<行号>` 拆出（跳过盘符冒号）；`D:\a\b.py:42` → (path, Some(42))
fn split_line_suffix(raw: &str) -> (String, Option<u32>) {
    if let Some(idx) = raw.rfind(':') {
        if idx > 1 {
            let digits = &raw[idx + 1..];
            if !digits.is_empty() && digits.len() <= 7 && digits.bytes().all(|b| b.is_ascii_digit())
            {
                return (raw[..idx].to_string(), digits.parse().ok());
            }
        }
    }
    (raw.to_string(), None)
}

fn is_text_file(p: &Path) -> bool {
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    if let Some(e) = ext {
        return TEXT_EXTS.contains(&e.as_str());
    }
    // 无扩展名：嗅探前 2KB，无 NUL 字节视为文本
    std::fs::read(p)
        .map(|b| !b.iter().take(2048).any(|&c| c == 0))
        .unwrap_or(false)
}

// ---------- GET /api/editors（本机已安装编辑器 / IDE） ----------

pub async fn editors() -> Response {
    match tokio::task::spawn_blocking(crate::editors::list).await {
        Ok(list) => Json(list).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("编辑器检测失败: {e}")})),
        )
            .into_response(),
    }
}

pub async fn open_path(Json(body): Json<OpenReq>) -> Response {
    let raw = body.path.trim().replace('/', "\\");
    // 末尾 :行号（若带行号的原样路径正好存在则不拆，Windows 文件名不含冒号，通常不会）
    let (stripped, line) = split_line_suffix(&raw);
    let chosen = if Path::new(&raw).exists() { raw.clone() } else { stripped };
    let mut p = std::path::PathBuf::from(&chosen);
    if p.is_relative() {
        if let Some(proj) = body.project.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            p = Path::new(proj).join(&p);
        }
    }
    if !p.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("文件不存在: {}", p.display())})),
        )
            .into_response();
    }

    // reveal 模式：资源管理器中定位文件
    if body.mode.as_deref() == Some("reveal") {
        let spawned = std::process::Command::new("explorer.exe")
            .arg(format!("/select,{}", p.display()))
            .spawn();
        return match spawned {
            Ok(_) => Json(json!({"ok": true})).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("打开失败: {e}")})),
            )
                .into_response(),
        };
    }

    // 项目目录右键菜单指定的打开方式。应用 ID 只能映射到后端已检测到的程序，
    // 不接受前端传入任意可执行路径。
    if let Some(app) = body.app.as_deref() {
        let spawned = match app {
            "explorer" => std::process::Command::new("explorer.exe").arg(&p).spawn(),
            "terminal" => std::process::Command::new("wt.exe")
                .arg("-d")
                .arg(&p)
                .spawn()
                .or_else(|_| {
                    let cwd = if p.is_dir() {
                        p.as_path()
                    } else {
                        p.parent().unwrap_or(&p)
                    };
                    std::process::Command::new("powershell.exe")
                        .arg("-NoExit")
                        .current_dir(cwd)
                        .spawn()
                }),
            editor_id => {
                let Some(exe) = crate::editors::executable(editor_id) else {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(json!({"error": format!("未检测到可用应用: {editor_id}")})),
                    )
                        .into_response();
                };
                std::process::Command::new(exe).arg(&p).spawn()
            }
        };
        return match spawned {
            Ok(_) => Json(json!({"ok": true, "app": app})).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("打开失败: {e}")})),
            )
                .into_response(),
        };
    }

    // 代码/文本 + 检测到 VS Code → 编辑器打开并跳行；其余走系统默认程序
    let spawned = if p.is_file() && is_text_file(&p) {
        if let Some(code) = crate::editors::executable("vscode") {
            let target = match line {
                Some(n) => format!("{}:{}", p.display(), n),
                None => p.display().to_string(),
            };
            std::process::Command::new(code).arg("-g").arg(target).spawn()
        } else {
            std::process::Command::new("explorer.exe").arg(&p).spawn()
        }
    } else {
        // 目录 → 资源管理器；Excel/Word/PDF/图片等 → 系统默认程序
        std::process::Command::new("explorer.exe").arg(&p).spawn()
    };
    match spawned {
        Ok(_) => Json(json!({"ok": true, "line": line})).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("打开失败: {e}")})),
        )
            .into_response(),
    }
}

// ---------- 文件改动统计 / 差异 / 内容（「已编辑文件」卡片） ----------

fn resolve_in_project(path: &str, project: Option<&str>) -> std::path::PathBuf {
    let raw = path.trim().replace('/', "\\");
    let p = std::path::PathBuf::from(&raw);
    if p.is_relative() {
        if let Some(proj) = project.map(str::trim).filter(|s| !s.is_empty()) {
            return Path::new(proj).join(&p);
        }
    }
    p
}

/// 文件实际所属的 git 仓库根（支持项目内嵌套仓库）；找不到则退回给定项目。
fn repo_root_for(p: &Path, fallback: &str) -> String {
    let dir = if p.is_dir() { p } else { p.parent().unwrap_or(p) };
    if let Some(out) = git_in(&dir.display().to_string(), &["rev-parse", "--show-toplevel"]) {
        let root = out.trim().replace('/', "\\");
        if !root.is_empty() {
            return root;
        }
    }
    fallback.to_string()
}

fn git_in(project: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(project)
        .args(args)
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        None
    }
}

#[derive(Deserialize)]
pub struct FileStatReq {
    pub project: String,
    pub files: Vec<String>,
}

/// 每个文件的 +增 -删 行数（git numstat 对 HEAD；未跟踪文件按整文件行数计增）。
pub async fn filestat(Json(body): Json<FileStatReq>) -> Response {
    let res = tokio::task::spawn_blocking(move || {
        // 按文件所属仓库根分组取 numstat（支持项目内嵌套仓库），根级缓存
        let mut numstat_cache: HashMap<String, Vec<(String, i64, i64)>> = HashMap::new();
        let mut numstat_for = |root: &str| -> Vec<(String, i64, i64)> {
            if let Some(v) = numstat_cache.get(root) {
                return v.clone();
            }
            let raw = git_in(root, &["diff", "--numstat", "HEAD"]).unwrap_or_default();
            let mut v = Vec::new();
            for line in raw.lines() {
                let mut it = line.splitn(3, '\t');
                let a = it.next().unwrap_or("").trim();
                let d = it.next().unwrap_or("").trim();
                let f = it.next().unwrap_or("").trim();
                if f.is_empty() {
                    continue;
                }
                v.push((
                    f.replace('\\', "/"),
                    a.parse().unwrap_or(0),
                    d.parse().unwrap_or(0),
                ));
            }
            numstat_cache.insert(root.to_string(), v.clone());
            v
        };
        let out: Vec<serde_json::Value> = body
            .files
            .iter()
            .map(|f| {
                let p = resolve_in_project(f, Some(&body.project));
                let root = repo_root_for(&p, &body.project);
                let git_map = numstat_for(&root);
                let norm = f.replace('\\', "/");
                let hit = git_map
                    .iter()
                    .find(|(g, _, _)| norm.ends_with(g.as_str()) || g.ends_with(norm.as_str()));
                if let Some((_, a, d)) = hit {
                    return json!({"file": f, "adds": a, "dels": d});
                }
                let pathspec = p.display().to_string();
                // 工作区干净（改动已提交）→ 最近一次触及该文件的提交统计
                if let Some(logstat) = git_in(
                    &root,
                    &["log", "-1", "--numstat", "--format=", "--", &pathspec],
                ) {
                    if let Some(l) = logstat.lines().find(|l| l.contains('\t')) {
                        let mut it = l.splitn(3, '\t');
                        let a: i64 = it.next().unwrap_or("").trim().parse().unwrap_or(0);
                        let d: i64 = it.next().unwrap_or("").trim().parse().unwrap_or(0);
                        return json!({"file": f, "adds": a, "dels": d});
                    }
                }
                // 未跟踪的新文件：整文件行数计增
                if p.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&p) {
                        return json!({"file": f, "adds": content.lines().count(), "dels": 0});
                    }
                }
                json!({"file": f, "adds": null, "dels": null})
            })
            .collect();
        out
    })
    .await;
    match res {
        Ok(v) => Json(v).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("统计失败: {e}")})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct DiffQuery {
    pub project: String,
    pub file: String,
}

/// 单文件对 HEAD 的差异；未跟踪文件合成「全新增」差异。
pub async fn diff(Query(q): Query<DiffQuery>) -> Response {
    let res = tokio::task::spawn_blocking(move || {
        let p = resolve_in_project(&q.file, Some(&q.project));
        let root = repo_root_for(&p, &q.project);
        let pathspec = p.display().to_string();
        let d = git_in(&root, &["diff", "HEAD", "--", &pathspec]).unwrap_or_default();
        if !d.trim().is_empty() {
            return json!({"diff": d});
        }
        // 未跟踪的新文件 → 合成差异
        let status = git_in(
            &root,
            &["status", "--porcelain", "--untracked-files=all", "--", &pathspec],
        )
        .unwrap_or_default();
        if status.starts_with("??") && p.is_file() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                let mut buf = format!("+++ {}（新文件）\n", q.file);
                for line in content.lines().take(4000) {
                    buf.push('+');
                    buf.push_str(line);
                    buf.push('\n');
                }
                return json!({"diff": buf});
            }
        }
        // 工作区干净（已提交）→ 展示最近一次触及该文件的提交差异
        if let Some(hash) = git_in(&root, &["log", "-1", "--format=%H", "--", &pathspec])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            if let Some(d2) = git_in(&root, &["show", "--format=", &hash, "--", &pathspec]) {
                if !d2.trim().is_empty() {
                    let short: String = hash.chars().take(8).collect();
                    return json!({"diff": d2, "source": format!("来自最近提交 {short}")});
                }
            }
        }
        json!({"diff": ""})
    })
    .await;
    match res {
        Ok(v) => Json(v).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("差异获取失败: {e}")})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct FileTextQuery {
    pub path: String,
    pub project: Option<String>,
}

/// 文本内容（复制文件内容用），上限 2MB。
pub async fn filetext(Query(q): Query<FileTextQuery>) -> Response {
    let p = resolve_in_project(&q.path, q.project.as_deref());
    match tokio::fs::read(&p).await {
        Ok(bytes) if bytes.len() <= 2 * 1024 * 1024 => {
            Json(json!({"text": String::from_utf8_lossy(&bytes)})).into_response()
        }
        Ok(_) => (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({"error": "文件超过 2MB"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("读取失败: {e}")})),
        )
            .into_response(),
    }
}

// ---------- GET /api/file（本地图片，转录内联展示用） ----------

const IMG_EXTS: [(&str, &str); 6] = [
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("bmp", "image/bmp"),
];

#[derive(Deserialize)]
pub struct FileQuery {
    pub path: String,
}

pub async fn file(Query(q): Query<FileQuery>) -> Response {
    let path = std::path::PathBuf::from(q.path.trim());
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    let Some((_, mime)) = IMG_EXTS.iter().find(|(e, _)| *e == ext.as_str()) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "仅支持图片文件"})),
        )
            .into_response();
    };
    match tokio::fs::read(&path).await {
        Ok(bytes) if bytes.len() <= 30 * 1024 * 1024 => (
            [
                (header::CONTENT_TYPE, *mime),
                (header::CACHE_CONTROL, "max-age=3600"),
            ],
            bytes,
        )
            .into_response(),
        Ok(_) => (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({"error": "图片文件过大"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("读取失败: {e}")})),
        )
            .into_response(),
    }
}

// ---------- POST /api/upload（输入框粘贴/选择的图片存入临时目录） ----------

#[derive(Deserialize)]
pub struct UploadQuery {
    pub name: Option<String>,
}

pub async fn upload(Query(q): Query<UploadQuery>, body: axum::body::Bytes) -> Response {
    if body.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "空文件"}))).into_response();
    }
    let ext = q
        .name
        .as_deref()
        .and_then(|n| n.rsplit('.').next())
        .map(|e| e.to_ascii_lowercase())
        .filter(|e| IMG_EXTS.iter().any(|(x, _)| *x == e.as_str()))
        .unwrap_or_else(|| "png".to_string());
    let dir = std::env::temp_dir().join("agent-hub-uploads");
    if let Err(e) = tokio::fs::create_dir_all(&dir).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("创建上传目录失败: {e}")})),
        )
            .into_response();
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = dir.join(format!("img-{stamp}.{ext}"));
    match tokio::fs::write(&path, &body).await {
        Ok(()) => Json(json!({"path": path.to_string_lossy()})).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("保存失败: {e}")})),
        )
            .into_response(),
    }
}

// ---------- GET /api/models ----------

pub async fn models() -> Response {
    match tokio::task::spawn_blocking(crate::models::discover).await {
        Ok(resp) => Json(resp).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("模型发现失败: {e}") })),
        )
            .into_response(),
    }
}

// ---------- GET /api/projects ----------

pub async fn projects(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(build_projects(&state, false).await)
}

/// 历史发现的全部项目（供「导入项目」选择面板用），含 pinned 标记。
pub async fn discover_projects(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(build_projects(&state, true).await)
}

/// 从已导入列表移除（不动任何历史数据）。
pub async fn remove_project(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddProjectReq>,
) -> Response {
    let want = crate::history::claude::normalize_path(body.path.trim());
    {
        let mut cfg = state.config.write().await;
        let before = cfg.projects.len();
        cfg.projects
            .retain(|p| crate::history::claude::normalize_path(p) != want);
        if cfg.projects.len() != before {
            if let Err(e) = config::save(&cfg) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("保存配置失败: {e}")})),
                )
                    .into_response();
            }
        }
    }
    Json(build_projects(&state, false).await).into_response()
}

// ---------- POST /api/projects ----------

#[derive(Deserialize)]
pub struct AddProjectReq {
    pub path: String,
}

pub async fn add_project(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddProjectReq>,
) -> Response {
    let path = crate::history::claude::normalize_path(body.path.trim());
    if path.is_empty() || !Path::new(&path).is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("目录不存在: {}", body.path)})),
        )
            .into_response();
    }
    {
        let mut cfg = state.config.write().await;
        let already = cfg
            .projects
            .iter()
            .any(|p| crate::history::claude::normalize_path(p) == path);
        if !already {
            cfg.projects.push(path.clone());
            if let Err(e) = config::save(&cfg) {
                cfg.projects.pop();
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("保存配置失败: {e}")})),
                )
                    .into_response();
            }
        }
    }
    Json(build_projects(&state, false).await).into_response()
}

// ---------- POST /api/pick-folder（弹出系统文件夹选择框，返回所选绝对路径） ----------

#[cfg(windows)]
pub async fn pick_folder() -> Response {
    use std::sync::atomic::{AtomicBool, Ordering};
    static PICKING: AtomicBool = AtomicBool::new(false);
    if PICKING.swap(true, Ordering::SeqCst) {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error": "已有选择窗口打开，请先处理该窗口"})),
        )
            .into_response();
    }
    // 对话框需要 STA COM 线程；spawn_blocking 线程会被复用，故再起独立线程
    let result = tokio::task::spawn_blocking(|| {
        std::thread::spawn(|| crate::dialog::pick_folder("选择项目根目录"))
            .join()
            .unwrap_or_else(|_| Err("对话框线程异常退出".to_string()))
    })
    .await
    .unwrap_or_else(|e| Err(format!("对话框任务失败: {e}")));
    PICKING.store(false, Ordering::SeqCst);
    match result {
        Ok(path) => Json(json!({ "path": path })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))).into_response(),
    }
}

#[cfg(not(windows))]
pub async fn pick_folder() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "当前平台不支持系统文件夹选择框"})),
    )
        .into_response()
}

// ---------- GET /api/sessions ----------

#[derive(Deserialize)]
pub struct SessionsQuery {
    pub project: Option<String>,
    pub q: Option<String>,
    pub limit: Option<usize>,
}

pub async fn sessions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SessionsQuery>,
) -> impl IntoResponse {
    let mut list = collect_all_sessions().await;
    apply_title_overrides(&mut list);
    if let Some(p) = query
        .project
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        let want = crate::history::claude::normalize_path(p);
        list.retain(|s| s.project == want);
    } else {
        // 未指定项目：只显示已导入项目的会话（不自动导入全部历史）
        let pinned: HashSet<String> = state
            .config
            .read()
            .await
            .projects
            .iter()
            .map(|p| crate::history::claude::normalize_path(p))
            .collect();
        list.retain(|s| pinned.contains(&s.project));
    }
    if let Some(q) = query.q.as_deref().map(str::trim).filter(|q| !q.is_empty()) {
        let ql = q.to_lowercase();
        list.retain(|s| s.title.to_lowercase().contains(&ql));
    }
    list.sort_by(|a, b| b.updated.cmp(&a.updated));
    list.truncate(query.limit.unwrap_or(200));
    Json(list)
}

// ---------- GET /api/session ----------

#[derive(Deserialize)]
pub struct SessionQuery {
    pub agent: String,
    pub id: String,
    pub project: Option<String>,
}

pub async fn session_detail(Query(q): Query<SessionQuery>) -> Response {
    let result = match q.agent.as_str() {
        "claude" => {
            let Some(project) = q
                .project
                .clone()
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
            else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "claude 会话必须提供 project 参数"})),
                )
                    .into_response();
            };
            let id = q.id.clone();
            tokio::task::spawn_blocking(move || crate::history::claude::transcript(&project, &id))
                .await
                .unwrap_or_else(|e| Err(format!("读取转录任务失败: {e}")))
        }
        "codex" => {
            let id = q.id.clone();
            tokio::task::spawn_blocking(move || crate::history::codex::transcript(&id))
                .await
                .unwrap_or_else(|e| Err(format!("读取转录任务失败: {e}")))
        }
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("未知 agent: {other}")})),
            )
                .into_response();
        }
    };
    match result {
        Ok(mut t) => {
            // 分叉等场景的标题覆盖
            if let Some(ov) = titles_load()
                .get(&format!("{}:{}", t.agent, t.id))
                .and_then(|v| v.as_str())
            {
                t.title = ov.to_string();
            }
            Json(t).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"error": e}))).into_response(),
    }
}

// ---------- POST /api/chat ----------

pub async fn chat(State(state): State<Arc<AppState>>, Json(req): Json<ChatReq>) -> Response {
    crate::run::stream_chat(&state.runs, req).await
}

// ---------- 后台运行：重连 / 列表 / 停止 ----------

#[derive(Deserialize)]
pub struct RunQuery {
    pub id: String,
}

/// 重连某次运行：只推送订阅之后的新事件（历史部分由转录重载补齐）。
pub async fn run_attach(State(state): State<Arc<AppState>>, Query(q): Query<RunQuery>) -> Response {
    match state.runs.get(&q.id) {
        Some(rs) => crate::run::attach(rs, None, usize::MAX),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "运行不存在或已清理"})),
        )
            .into_response(),
    }
}

/// 后台运行列表：running=true 为进行中；false 时带 ok/error（10 分钟内结束的）。
pub async fn runs_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let list: Vec<serde_json::Value> = state
        .runs
        .list_all()
        .into_iter()
        .map(|(id, r)| {
            let outcome = r.outcome();
            json!({
                "run_id": id,
                "agent": r.agent,
                "project": r.project,
                "prompt": r.prompt,
                "sage": r.sage.clone(),
                "session_id": r.session_id.lock().unwrap().clone(),
                "running": !r.is_done(),
                "ok": outcome.as_ref().map(|o| o.0),
                "error": outcome.and_then(|o| o.1),
            })
        })
        .collect();
    Json(list)
}

#[derive(Deserialize)]
pub struct StopReq {
    pub run_id: String,
}

// ---------- POST /api/fork-at（中点分叉：从指定消息处分叉出新会话） ----------

#[derive(Deserialize)]
pub struct ForkAtReq {
    pub agent: String,
    pub id: String,
    /// codex=消息来源行 ordinal（数字）；claude=行 uuid（字符串）；null=会话末尾
    pub pos: Option<serde_json::Value>,
}

pub async fn fork_at(Json(body): Json<ForkAtReq>) -> Response {
    let result = match body.agent.as_str() {
        "codex" => {
            crate::history::codex::fork_at(&body.id, body.pos.as_ref().and_then(|v| v.as_u64()))
        }
        "claude" => {
            crate::history::claude::fork_at(&body.id, body.pos.as_ref().and_then(|v| v.as_str()))
        }
        _ => Err("未知 agent".to_string()),
    };
    match result {
        Ok(new_id) => {
            // 分叉标题：父标题 + " (n)"（Codex Desktop 同款习惯）
            let mut list = collect_all_sessions().await;
            apply_title_overrides(&mut list);
            let parent_title = list
                .iter()
                .find(|s| s.agent == body.agent && s.id == body.id)
                .map(|s| s.title.trim().to_string())
                .unwrap_or_default();
            if !parent_title.is_empty() {
                // 去掉父标题自身的 " (n)" 后缀作为基名
                let base = match parent_title.rfind(" (") {
                    Some(i)
                        if parent_title.ends_with(')')
                            && parent_title[i + 2..parent_title.len() - 1]
                                .chars()
                                .all(|c| c.is_ascii_digit()) =>
                    {
                        parent_title[..i].to_string()
                    }
                    _ => parent_title.clone(),
                };
                let taken: std::collections::HashSet<String> =
                    list.iter().map(|s| s.title.trim().to_string()).collect();
                let mut n = 2;
                while taken.contains(&format!("{base} ({n})")) {
                    n += 1;
                }
                title_override_set(&body.agent, &new_id, &format!("{base} ({n})"));
            }
            invalidate_sessions_cache();
            Json(json!({ "ok": true, "id": new_id })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response(),
    }
}

// ---------- POST /api/session/delete（会话移入回收目录，可恢复） ----------

#[derive(Deserialize)]
pub struct DeleteSessionReq {
    pub agent: String,
    pub id: String,
}

pub async fn delete_session(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DeleteSessionReq>,
) -> Response {
    // 运行中的会话拒绝删除
    let running = state.runs.list_all().iter().any(|(_, r)| {
        !r.is_done() && r.session_id.lock().unwrap().as_deref() == Some(body.id.as_str())
    });
    if running {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "error": "会话有任务在运行，先停止再删除" })),
        )
            .into_response();
    }
    let path = match body.agent.as_str() {
        "claude" => crate::history::claude::session_file_for(&body.id),
        "codex" => crate::history::codex::rollout_path_for(&body.id),
        _ => None,
    };
    let Some(path) = path else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "未找到会话文件" })),
        )
            .into_response();
    };
    // 不硬删：移入 ~/.agenthub/trash/，误删可手动恢复
    let trash = dirs::home_dir()
        .map(|h| h.join(".agenthub").join("trash"))
        .unwrap_or_default();
    if std::fs::create_dir_all(&trash).is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "创建回收目录失败" })),
        )
            .into_response();
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| body.id.clone());
    let dest = trash.join(format!("{}-{}", body.agent, name));
    match std::fs::rename(&path, &dest) {
        Ok(_) => {
            invalidate_sessions_cache();
            Json(json!({ "ok": true, "trash": dest.to_string_lossy() })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("移动到回收目录失败: {e}") })),
        )
            .into_response(),
    }
}

pub async fn stop_run(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StopReq>,
) -> impl IntoResponse {
    let ok = state.runs.stop(&body.run_id);
    Json(json!({ "ok": ok, "status": if ok { "stopping" } else { "not_found" } }))
}

// ---------- 标题覆盖（分叉会话等取不到标题的场景） ----------

/// ~/.agenthub/titles.json：{"<agent>:<id>": "标题"}
fn titles_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".agenthub").join("titles.json"))
}

fn titles_load() -> serde_json::Map<String, serde_json::Value> {
    titles_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

fn titles_save(map: &serde_json::Map<String, serde_json::Value>) {
    if let Some(p) = titles_path() {
        let _ = p.parent().map(std::fs::create_dir_all);
        let _ = std::fs::write(p, serde_json::Value::Object(map.clone()).to_string());
    }
}

pub fn title_override_set(agent: &str, id: &str, title: &str) {
    let mut map = titles_load();
    map.insert(format!("{agent}:{id}"), json!(title));
    titles_save(&map);
}

fn apply_title_overrides(list: &mut [SessionSummary]) {
    let map = titles_load();
    if map.is_empty() {
        return;
    }
    for s in list.iter_mut() {
        if let Some(t) = map.get(&format!("{}:{}", s.agent, s.id)).and_then(|v| v.as_str()) {
            s.title = t.to_string();
        }
    }
}

// ---------- 内部工具 ----------

/// 会话扫描结果缓存：页面加载并发打 3-4 个接口，共享一次全量扫描。
static SESSIONS_CACHE: std::sync::OnceLock<
    std::sync::Mutex<Option<(std::time::Instant, Vec<SessionSummary>)>>,
> = std::sync::OnceLock::new();
static SCAN_LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();

const SESSIONS_TTL_SECS: u64 = 5;

fn sessions_cache_get() -> Option<Vec<SessionSummary>> {
    let cache = SESSIONS_CACHE.get_or_init(|| std::sync::Mutex::new(None));
    let guard = cache.lock().unwrap();
    match guard.as_ref() {
        Some((at, v)) if at.elapsed().as_secs() < SESSIONS_TTL_SECS => Some(v.clone()),
        _ => None,
    }
}

/// 启动预热：后台先扫一遍，首次页面加载直接命中缓存。
pub async fn warm_sessions() {
    let _ = collect_all_sessions().await;
}

/// 运行结束后调用：立即失效缓存，让新会话马上出现在侧栏。
pub fn invalidate_sessions_cache() {
    if let Some(cache) = SESSIONS_CACHE.get() {
        *cache.lock().unwrap() = None;
    }
}

/// 两侧历史会话合并（5 秒 TTL 缓存 + 并发合流：同一时刻只扫一次盘）。
async fn collect_all_sessions() -> Vec<SessionSummary> {
    if let Some(v) = sessions_cache_get() {
        return v;
    }
    let _g = SCAN_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await;
    // 拿到扫描锁后二次检查（可能已被并发请求填充）
    if let Some(v) = sessions_cache_get() {
        return v;
    }
    let v = tokio::task::spawn_blocking(|| {
        let mut v = crate::history::claude::all_sessions();
        v.extend(crate::history::codex::all_sessions());
        v
    })
    .await
    .unwrap_or_default();
    let cache = SESSIONS_CACHE.get_or_init(|| std::sync::Mutex::new(None));
    *cache.lock().unwrap() = Some((std::time::Instant::now(), v.clone()));
    v
}

/// 按项目聚合的会话统计。
struct Agg {
    claude: usize,
    codex: usize,
    last: Option<String>,
}

/// all=false：仅已导入（pinned）项目；all=true：并入历史发现的全部项目（导入面板用）。
/// 均按 last_active 降序。
async fn build_projects(state: &AppState, all: bool) -> Vec<ProjectInfo> {
    let sessions = collect_all_sessions().await;
    let pinned_raw: Vec<String> = state.config.read().await.projects.clone();
    let pinned: Vec<String> = pinned_raw
        .iter()
        .map(|p| crate::history::claude::normalize_path(p))
        .collect();

    let mut map: HashMap<String, Agg> = HashMap::new();
    for s in &sessions {
        let e = map.entry(s.project.clone()).or_insert(Agg {
            claude: 0,
            codex: 0,
            last: None,
        });
        if s.agent == "claude" {
            e.claude += 1;
        } else {
            e.codex += 1;
        }
        // ISO 8601 UTC 统一格式，字典序即时间序；None < Some。
        if s.updated > e.last {
            e.last = s.updated.clone();
        }
    }

    let mut out: Vec<ProjectInfo> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for p in &pinned {
        if p.is_empty() || !seen.insert(p.clone()) {
            continue;
        }
        out.push(make_info(p, map.get(p), true));
    }
    if all {
        for (path, agg) in &map {
            if seen.contains(path) {
                continue;
            }
            out.push(make_info(path, Some(agg), false));
        }
    }
    out.sort_by(|a, b| b.last_active.cmp(&a.last_active));
    out
}

fn make_info(path: &str, agg: Option<&Agg>, pinned: bool) -> ProjectInfo {
    let name = Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    ProjectInfo {
        path: path.to_string(),
        name,
        exists: Path::new(path).is_dir(),
        claude_sessions: agg.map(|a| a.claude).unwrap_or(0),
        codex_sessions: agg.map(|a| a.codex).unwrap_or(0),
        last_active: agg.and_then(|a| a.last.clone()),
        pinned,
    }
}

//! Claude Code 历史读取：`~/.claude/projects/<编码目录>/*.jsonl`（顶层）
//! + `~/.claude/history.jsonl`（标题来源）。契约见 docs/CONTRACT.md §2.1。
//!
//! 另导出通用工具：`normalize_path`（§1 路径规范化）与若干 pub(crate) 辅助
//! （ISO 8601 转换、字符安全截断），codex 侧复用。

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::types::{Block, ChatMessage, SessionSummary, Transcript};

/// 标题最大字符数（非字节）。
pub(crate) const TITLE_MAX: usize = 80;
/// 工具输入/输出摘要最大字符数。
pub(crate) const SUMMARY_MAX: usize = 400;

// ---------------------------------------------------------------------------
// 通用工具（codex.rs 复用）
// ---------------------------------------------------------------------------

/// 路径规范化：剥 `\\?\` 前缀、`/`→`\`、盘符大写、去尾部 `\`（保留盘符根 `D:\`）。
pub fn normalize_path(p: &str) -> String {
    let mut s = p.trim().to_string();
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        s = rest.to_string();
    }
    s = s.replace('/', "\\");
    let mut chars: Vec<char> = s.chars().collect();
    if chars.len() >= 2 && chars[1] == ':' && chars[0].is_ascii_lowercase() {
        chars[0] = chars[0].to_ascii_uppercase();
        s = chars.into_iter().collect();
    }
    while s.ends_with('\\') && s.chars().count() > 3 {
        s.pop();
    }
    s
}

/// 字符安全截断（绝不按字节切，避免多字节字符 panic）。
pub(crate) fn truncate_chars(s: &str, max: usize) -> String {
    match s.char_indices().nth(max) {
        None => s.to_string(),
        Some((idx, _)) => s[..idx].to_string(),
    }
}

/// 标题清洗：trim、控制字符替换为空格、截 TITLE_MAX 字符。
pub(crate) fn clean_title(s: &str) -> String {
    let t: String = s
        .trim()
        .chars()
        .map(|c| {
            if c == '\n' || c == '\r' || c == '\t' {
                ' '
            } else {
                c
            }
        })
        .collect();
    truncate_chars(t.trim(), TITLE_MAX)
}

/// Howard Hinnant civil_from_days：epoch 日数 → (年, 月, 日)。已用真实样本自测。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn format_iso(secs: i64, millis: u32) -> String {
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        y,
        m,
        d,
        sod / 3600,
        (sod % 3600) / 60,
        sod % 60,
        millis
    )
}

/// SystemTime → ISO 8601 UTC 字符串（毫秒精度）。
pub(crate) fn system_time_to_iso(t: SystemTime) -> Option<String> {
    let dur = t.duration_since(UNIX_EPOCH).ok()?;
    Some(format_iso(dur.as_secs() as i64, dur.subsec_millis()))
}

/// 文件 mtime → ISO 8601 UTC 字符串。
pub(crate) fn mtime_iso(path: &Path) -> Option<String> {
    system_time_to_iso(fs::metadata(path).ok()?.modified().ok()?)
}

/// 逐行读取（容忍非 UTF-8：lossy 转换；IO 错误即停止）。
pub(crate) fn for_each_line<F: FnMut(&str) -> bool>(path: &Path, mut f: F) {
    let Ok(file) = fs::File::open(path) else {
        return;
    };
    let reader = BufReader::new(file);
    for chunk in reader.split(b'\n') {
        let Ok(bytes) = chunk else { break };
        let line = String::from_utf8_lossy(&bytes);
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !f(line) {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// claude 专用
// ---------------------------------------------------------------------------

fn claude_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude"))
}

fn projects_root() -> Option<PathBuf> {
    claude_dir().map(|d| d.join("projects"))
}

/// 真实路径 → 项目目录名编码（`[^A-Za-z0-9]` → `-`，有损）。
fn encode_project_dir(p: &str) -> String {
    p.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// 斜杠命令记录（`<command-name>` / `<local-command...`）。
fn is_command_text(s: &str) -> bool {
    let t = s.trim_start();
    t.starts_with("<command-name>") || t.starts_with("<local-command")
}

// ---- history.jsonl 标题缓存（mtime 变化时整体重读） ----

struct HistoryCache {
    mtime: Option<SystemTime>,
    /// sessionId → (最早 epoch_ms, display)
    titles: HashMap<String, (i64, String)>,
}

fn history_cache() -> &'static Mutex<HistoryCache> {
    static CACHE: OnceLock<Mutex<HistoryCache>> = OnceLock::new();
    CACHE.get_or_init(|| {
        Mutex::new(HistoryCache {
            mtime: None,
            titles: HashMap::new(),
        })
    })
}

fn load_history_titles(path: &Path) -> HashMap<String, (i64, String)> {
    let mut map: HashMap<String, (i64, String)> = HashMap::new();
    for_each_line(path, |line| {
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            let display = v.get("display").and_then(Value::as_str).unwrap_or("");
            let sid = v.get("sessionId").and_then(Value::as_str).unwrap_or("");
            let ts = v
                .get("timestamp")
                .and_then(Value::as_i64)
                .unwrap_or(i64::MAX);
            if !sid.is_empty() && !display.trim().is_empty() && !display.starts_with('/') {
                match map.get(sid) {
                    Some((old_ts, _)) if *old_ts <= ts => {}
                    _ => {
                        map.insert(sid.to_string(), (ts, display.to_string()));
                    }
                }
            }
        }
        true
    });
    map
}

/// history.jsonl 中该 sessionId 最早的非 `/` 开头 display。
fn history_title(session_id: &str) -> Option<String> {
    let path = claude_dir()?.join("history.jsonl");
    let mtime = fs::metadata(&path).ok()?.modified().ok();
    let mut cache = history_cache().lock().ok()?;
    if cache.mtime != mtime || mtime.is_none() {
        cache.titles = load_history_titles(&path);
        cache.mtime = mtime;
    }
    cache.titles.get(session_id).map(|(_, d)| clean_title(d))
}

// ---- 项目目录 → 真实 cwd 缓存（仅缓存解析成功的目录） ----

fn dir_cwd_cache() -> &'static Mutex<HashMap<String, String>> {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 从目录内任一顶层 jsonl 前若干行的 `cwd` 字段恢复真实路径（规范化后返回）。
fn dir_real_cwd(dir: &Path) -> Option<String> {
    let key = dir.file_name()?.to_string_lossy().to_string();
    if let Ok(cache) = dir_cwd_cache().lock() {
        if let Some(c) = cache.get(&key) {
            return Some(c.clone());
        }
    }
    let mut found: Option<String> = None;
    let rd = fs::read_dir(dir).ok()?;
    'files: for entry in rd.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let mut cwd: Option<String> = None;
        let mut n = 0;
        for_each_line(&path, |line| {
            n += 1;
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                if let Some(c) = v.get("cwd").and_then(Value::as_str) {
                    if !c.trim().is_empty() {
                        cwd = Some(normalize_path(c));
                        return false;
                    }
                }
            }
            n < 20
        });
        if cwd.is_some() {
            found = cwd;
            break 'files;
        }
    }
    let found = found?;
    if let Ok(mut cache) = dir_cwd_cache().lock() {
        cache.insert(key, found.clone());
    }
    Some(found)
}

/// 扫描会话文件前若干行：created（第一个顶层 timestamp）与 fallback 标题
/// （第一条 type==user && !isSidechain && !isMeta 的文本）。
fn scan_session_file(path: &Path, need_title: bool) -> (Option<String>, Option<String>) {
    let mut created: Option<String> = None;
    let mut title: Option<String> = None;
    let mut n = 0;
    for_each_line(path, |line| {
        n += 1;
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            if created.is_none() {
                if let Some(ts) = v.get("timestamp").and_then(Value::as_str) {
                    created = Some(ts.to_string());
                }
            }
            if need_title && title.is_none() && is_transcript_line(&v, "user") {
                if let Some(text) = v
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(first_user_text)
                {
                    title = Some(clean_title(&text));
                }
            }
        }
        let done = created.is_some() && (!need_title || title.is_some());
        !done && n < 200
    });
    (created, title)
}

/// envelope 过滤：type 匹配且非 sidechain / meta。
fn is_transcript_line(v: &Value, want: &str) -> bool {
    v.get("type").and_then(Value::as_str) == Some(want)
        && v.get("isSidechain").and_then(Value::as_bool) != Some(true)
        && v.get("isMeta").and_then(Value::as_bool) != Some(true)
}

/// user 消息 content 中的第一段普通文本（跳过斜杠命令记录与空文本）。
fn first_user_text(content: &Value) -> Option<String> {
    match content {
        Value::String(s) => {
            if !s.trim().is_empty() && !is_command_text(s) {
                Some(s.clone())
            } else {
                None
            }
        }
        Value::Array(items) => items.iter().find_map(|item| {
            if item.get("type").and_then(Value::as_str) == Some("text") {
                let t = item.get("text").and_then(Value::as_str).unwrap_or("");
                if !t.trim().is_empty() && !is_command_text(t) {
                    return Some(t.to_string());
                }
            }
            None
        }),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// 公开 API
// ---------------------------------------------------------------------------

/// 全部项目全部会话。每次调用重扫目录（文件数少）；history.jsonl 与目录 cwd 有缓存。
pub fn all_sessions() -> Vec<SessionSummary> {
    let mut out = Vec::new();
    let Some(root) = projects_root() else {
        return out;
    };
    let Ok(rd) = fs::read_dir(&root) else {
        return out;
    };
    for dir_entry in rd.flatten() {
        let dir = dir_entry.path();
        if !dir.is_dir() {
            continue;
        }
        // 真实项目路径恢复失败（如仅含 memory/ 的空目录）则跳过该目录
        let Some(project) = dir_real_cwd(&dir) else {
            continue;
        };
        let Ok(files) = fs::read_dir(&dir) else {
            continue;
        };
        for f in files.flatten() {
            let path = f.path();
            if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|s| s.to_str()).map(String::from) else {
                continue;
            };
            let from_history = history_title(&id);
            let (created, fallback) = scan_session_file(&path, from_history.is_none());
            let title = from_history
                .or(fallback)
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| "(无标题)".to_string());
            out.push(SessionSummary {
                agent: "claude".to_string(),
                id,
                title,
                project: project.clone(),
                created,
                updated: mtime_iso(&path),
                archived: false,
            });
        }
    }
    out
}

/// 单会话完整转录。project 为真实路径（内部按有损编码定位目录，找不到则全局搜文件名）。
pub fn transcript(project: &str, session_id: &str) -> Result<Transcript, String> {
    let root = projects_root().ok_or_else(|| "无法定位用户主目录".to_string())?;
    let normalized = normalize_path(project);
    let mut path = root
        .join(encode_project_dir(&normalized))
        .join(format!("{session_id}.jsonl"));
    if !path.is_file() {
        // 有损编码可能对不上目录名：退化为全局按文件名搜索
        path = find_session_file(&root, session_id)
            .ok_or_else(|| format!("未找到 Claude 会话文件: {session_id}"))?;
    }

    let mut messages: Vec<ChatMessage> = Vec::new();
    let mut file_cwd: Option<String> = None;
    let mut fallback_title: Option<String> = None;
    let (mut u_in, mut u_out, mut u_cr, mut u_cw, mut u_ctx) = (0i64, 0i64, 0i64, 0i64, 0i64);
    let mut first_ts: Option<String> = None;
    let mut last_ts: Option<String> = None;
    let mut last_model: Option<String> = None;
    for_each_line(&path, |line| {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            return true; // 单行解析失败跳过，不中断
        };
        if file_cwd.is_none() {
            if let Some(c) = v.get("cwd").and_then(Value::as_str) {
                file_cwd = Some(normalize_path(c));
            }
        }
        let role = match v.get("type").and_then(Value::as_str) {
            Some(t @ ("user" | "assistant")) => t.to_string(),
            _ => return true,
        };
        if !is_transcript_line(&v, &role) {
            return true;
        }
        let Some(msg) = v.get("message") else {
            return true;
        };
        let blocks = if role == "user" {
            if fallback_title.is_none() {
                if let Some(t) = msg.get("content").and_then(first_user_text) {
                    fallback_title = Some(clean_title(&t));
                }
            }
            user_blocks(msg.get("content"))
        } else {
            // API error 等合成行跳过
            if msg.get("model").and_then(Value::as_str) == Some("<synthetic>") {
                return true;
            }
            // 整场用量累计（context = 最后一次调用的完整 prompt 规模）
            if let Some(u) = msg.get("usage") {
                let g = |k: &str| u.get(k).and_then(Value::as_i64).unwrap_or(0);
                let (i, cr, cw) = (
                    g("input_tokens"),
                    g("cache_read_input_tokens"),
                    g("cache_creation_input_tokens"),
                );
                u_in += i;
                u_cr += cr;
                u_cw += cw;
                u_out += g("output_tokens");
                if i + cr + cw > 0 {
                    u_ctx = i + cr + cw;
                }
            }
            if let Some(m) = msg.get("model").and_then(Value::as_str) {
                last_model = Some(m.to_string());
            }
            assistant_blocks(msg.get("content"))
        };
        let ts_s = v.get("timestamp").and_then(Value::as_str).map(String::from);
        if first_ts.is_none() {
            first_ts = ts_s.clone();
        }
        if ts_s.is_some() {
            last_ts = ts_s.clone();
        }
        if !blocks.is_empty() {
            messages.push(ChatMessage {
                role,
                ts: ts_s,
                blocks,
            });
        }
        true
    });
    let usage = if u_out > 0 {
        Some(serde_json::json!({
            "input": u_in, "output": u_out,
            "cache_read": u_cr, "cache_write": u_cw,
            "context": u_ctx, "first_ts": first_ts, "last_ts": last_ts,
            "model": last_model,
        }))
    } else {
        None
    };

    let title = history_title(session_id)
        .or(fallback_title)
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "(无标题)".to_string());
    Ok(Transcript {
        agent: "claude".to_string(),
        id: session_id.to_string(),
        project: file_cwd.unwrap_or(normalized),
        title,
        messages,
        usage,
    })
}

/// 在全部项目目录中按文件名搜索 `<session_id>.jsonl`（顶层）。
fn find_session_file(root: &Path, session_id: &str) -> Option<PathBuf> {
    let file_name = format!("{session_id}.jsonl");
    for dir in fs::read_dir(root).ok()?.flatten() {
        let candidate = dir.path().join(&file_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// 块映射
// ---------------------------------------------------------------------------

fn text_block(text: String) -> Block {
    Block {
        kind: "text".to_string(),
        text,
        name: None,
    }
}

fn user_blocks(content: Option<&Value>) -> Vec<Block> {
    let mut blocks = Vec::new();
    match content {
        Some(Value::String(s)) => {
            if !s.trim().is_empty() && !is_command_text(s) {
                blocks.push(text_block(s.clone()));
            }
        }
        Some(Value::Array(items)) => {
            for item in items {
                match item.get("type").and_then(Value::as_str) {
                    Some("text") => {
                        let t = item.get("text").and_then(Value::as_str).unwrap_or("");
                        if !t.trim().is_empty() && !is_command_text(t) {
                            blocks.push(text_block(t.to_string()));
                        }
                    }
                    Some("tool_result") => {
                        blocks.push(Block {
                            kind: "tool_result".to_string(),
                            text: truncate_chars(
                                &tool_result_text(item.get("content")),
                                SUMMARY_MAX,
                            ),
                            name: None,
                        });
                    }
                    Some("image") => {
                        blocks.push(image_block(item));
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    blocks
}

/// image 块 → data URL 直传前端 <img> 渲染；过大或非 base64 内联时退回占位符。
fn image_block(item: &Value) -> Block {
    let data_url = item.get("source").and_then(|s| {
        let media = s
            .get("media_type")
            .and_then(Value::as_str)
            .unwrap_or("image/png");
        let data = s.get("data").and_then(Value::as_str)?;
        if s.get("type").and_then(Value::as_str) == Some("base64")
            && media.starts_with("image/")
            && data.len() < 6_000_000
        {
            Some(format!("data:{media};base64,{data}"))
        } else {
            None
        }
    });
    Block {
        kind: "image".to_string(),
        text: data_url.unwrap_or_else(|| "[图片]".to_string()),
        name: None,
    }
}

/// tool_result 的 content：字符串直接用；嵌套数组取 text 块；只有图片时给占位。
fn tool_result_text(content: Option<&Value>) -> String {
    match content {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(items)) => {
            let texts: Vec<&str> = items
                .iter()
                .filter(|i| i.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|i| i.get("text").and_then(Value::as_str))
                .collect();
            let joined = texts.join("\n");
            if joined.trim().is_empty()
                && items
                    .iter()
                    .any(|i| i.get("type").and_then(Value::as_str) == Some("image"))
            {
                "[图片]".to_string()
            } else {
                joined
            }
        }
        _ => String::new(),
    }
}

fn assistant_blocks(content: Option<&Value>) -> Vec<Block> {
    let mut blocks = Vec::new();
    match content {
        Some(Value::String(s)) => {
            if !s.trim().is_empty() {
                blocks.push(text_block(s.clone()));
            }
        }
        Some(Value::Array(items)) => {
            for item in items {
                match item.get("type").and_then(Value::as_str) {
                    Some("text") => {
                        let t = item.get("text").and_then(Value::as_str).unwrap_or("");
                        if !t.trim().is_empty() {
                            blocks.push(text_block(t.to_string()));
                        }
                    }
                    Some("thinking") => {
                        let t = item.get("thinking").and_then(Value::as_str).unwrap_or("");
                        if !t.trim().is_empty() {
                            blocks.push(Block {
                                kind: "thinking".to_string(),
                                text: t.to_string(),
                                name: None,
                            });
                        }
                    }
                    Some("tool_use") => {
                        let name = item
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("tool")
                            .to_string();
                        // TodoWrite = 任务计划 → plan 块（前端渲染进度清单）
                        let plan = if name == "TodoWrite" {
                            crate::run::plan_items(item.pointer("/input/todos"))
                        } else {
                            Vec::new()
                        };
                        if !plan.is_empty() {
                            blocks.push(Block {
                                kind: "plan".to_string(),
                                text: serde_json::to_string(&plan).unwrap_or_default(),
                                name: None,
                            });
                        } else {
                            let input = item
                                .get("input")
                                .map(|i| serde_json::to_string(i).unwrap_or_default())
                                .unwrap_or_default();
                            let is_edit = matches!(
                                name.as_str(),
                                "Edit" | "Write" | "MultiEdit" | "NotebookEdit"
                            );
                            blocks.push(Block {
                                kind: "tool_use".to_string(),
                                text: truncate_chars(&input, SUMMARY_MAX),
                                name: Some(name),
                            });
                            if is_edit {
                                if let Some(fp) =
                                    item.pointer("/input/file_path").and_then(Value::as_str)
                                {
                                    blocks.push(Block {
                                        kind: "file_edit".to_string(),
                                        text: fp.to_string(),
                                        name: None,
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    blocks
}

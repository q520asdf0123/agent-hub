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

use crate::types::{Block, ChatMessage, SagePromptMeta, SessionSummary, Transcript};

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

/// SAGE 会把内部编排说明作为 CLI 的 user prompt 写入原生历史。
/// 历史/UI 只能暴露真实原始任务，不能把 HANDOFF/COLLABORATE 指令冒充用户输入。
/// 跨会话上下文转移块的起始标记：被移交/被拉进协作的 agent 拿到的是全新会话，
/// 前端把来源会话记录附在原始任务之后。标题与气泡只取任务，记录不入正文。
pub(crate) const SAGE_CONTEXT_MARKER: &str = "\n\n【来源会话上下文】";

pub(crate) fn sage_original_task(text: &str) -> Option<String> {
    let text = text.trim();
    let original = if text.starts_with("【SAGE HANDOFF】") {
        let rest = text.split_once("\n\n")?.1;
        match rest.find(SAGE_CONTEXT_MARKER) {
            Some(end) => &rest[..end],
            None => rest,
        }
    } else if text.starts_with("【SAGE COLLABORATE") {
        let (_, rest) = text.split_once("原始任务：")?;
        let rest = rest.trim_start_matches(['\r', '\n']);
        let end = [
            SAGE_CONTEXT_MARKER,
            "\n\n依赖节点产出：",
            "\n\n节点产出：",
            "\n\n请完成本节点",
        ]
        .iter()
        .filter_map(|marker| rest.find(marker))
        .min()
        .unwrap_or(rest.len());
        &rest[..end]
    } else {
        return None;
    };
    let original = original.trim();
    (!original.is_empty()).then(|| original.to_string())
}

pub(crate) fn sage_prompt_metadata(text: &str) -> Option<SagePromptMeta> {
    let text = text.trim();
    let (kind, requirement) = if text.starts_with("【SAGE HANDOFF】") {
        ("handoff", None)
    } else if let Some(header) = text.lines().next().and_then(|line| {
        line.strip_prefix("【SAGE COLLABORATE · ")
            .and_then(|value| value.strip_suffix('】'))
    }) {
        if header == "所有者汇总" {
            ("summary", None)
        } else {
            ("collaborate", Some(header.to_string()))
        }
    } else {
        return None;
    };
    let field = |prefix: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(prefix))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(String::from)
    };
    let source = field("来源会话：").or_else(|| field("主会话："));
    let (source_agent, source_session_id) = source
        .as_deref()
        .and_then(|value| value.split_once(':'))
        .map(|(agent, session_id)| {
            (
                (!agent.trim().is_empty()).then(|| agent.trim().to_string()),
                (!session_id.trim().is_empty()).then(|| session_id.trim().to_string()),
            )
        })
        .unwrap_or((None, None));
    Some(SagePromptMeta {
        kind: kind.to_string(),
        workflow_id: field("协作标识："),
        requirement,
        owner: field("任务所有者："),
        executor: field("当前执行者："),
        source_agent,
        source_session_id,
        original_task: sage_original_task(text),
    })
}

fn first_content_text(content: &Value) -> Option<&str> {
    match content {
        Value::String(value) => Some(value),
        Value::Array(items) => items.iter().find_map(|item| {
            (item.get("type").and_then(Value::as_str) == Some("text"))
                .then(|| item.get("text").and_then(Value::as_str))
                .flatten()
        }),
        _ => None,
    }
}

fn push_sage_meta(items: &mut Vec<SagePromptMeta>, text: &str) {
    if let Some(meta) = sage_prompt_metadata(text) {
        if !items.contains(&meta) {
            items.push(meta);
        }
    }
}

/// CLI/宿主以 user role 写入的系统上下文、通知和压缩续接摘要，不是用户输入。
pub(crate) fn injected_user_text(text: &str) -> bool {
    let text = text.trim_start();
    if text.starts_with('<')
        || text.starts_with("==")
        || text.starts_with("# AGENTS.md")
        || text.starts_with(
            "This session is being continued from a previous conversation that ran out of context.",
        )
    {
        return true;
    }
    let head: String = text.chars().take(600).collect();
    head.contains("<INSTRUCTIONS>")
        || head.contains("<user_instructions>")
        || head.contains("<workspace_roots>")
        || head.contains("<permission_profile")
        || text.contains("<environment_context>")
}

fn user_title_source(text: &str) -> String {
    let Some(original) = sage_original_task(text) else {
        return text.to_string();
    };
    original
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("请查看图片文件:"))
        .unwrap_or(original.trim())
        .to_string()
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
    cache.titles.get(session_id).and_then(|(_, display)| {
        (!injected_user_text(display)).then(|| clean_title(&user_title_source(display)))
    })
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
fn scan_session_file(
    path: &Path,
    need_title: bool,
) -> (Option<String>, Option<String>, Option<SagePromptMeta>) {
    let mut created: Option<String> = None;
    let mut title: Option<String> = None;
    let mut sage: Option<SagePromptMeta> = None;
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
            if sage.is_none() && is_transcript_line(&v, "user") {
                if let Some(text) = v
                    .get("message")
                    .and_then(|message| message.get("content"))
                    .and_then(first_content_text)
                {
                    sage = sage_prompt_metadata(text);
                }
            }
        }
        let basics = created.is_some() && (!need_title || title.is_some());
        !(basics && (sage.is_some() || n >= 30)) && n < 200
    });
    (created, title, sage)
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
            if !s.trim().is_empty() && !is_command_text(s) && !injected_user_text(s) {
                Some(user_title_source(s))
            } else {
                None
            }
        }
        Value::Array(items) => items.iter().find_map(|item| {
            if item.get("type").and_then(Value::as_str) == Some("text") {
                let t = item.get("text").and_then(Value::as_str).unwrap_or("");
                if !t.trim().is_empty() && !is_command_text(t) && !injected_user_text(t) {
                    return Some(user_title_source(t));
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
            let (created, fallback, sage) = scan_session_file(&path, from_history.is_none());
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
                sage,
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
    let mut sage: Vec<SagePromptMeta> = Vec::new();
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
        if role == "user" {
            if let Some(text) = msg.get("content").and_then(first_content_text) {
                push_sage_meta(&mut sage, text);
            }
        }
        let blocks = if role == "user" {
            if fallback_title.is_none() {
                if let Some(t) = msg.get("content").and_then(first_user_text) {
                    fallback_title = Some(clean_title(&t));
                }
            }
            normalize_sage_user_blocks(&messages, user_blocks(msg.get("content")))
        } else {
            // API error 等合成行：正文是报错文本，标成错误分隔线（不映射会
            // 导致报错在重开会话后"消失"）
            if msg.get("model").and_then(Value::as_str) == Some("<synthetic>") {
                let err_text = msg
                    .get("content")
                    .and_then(Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|b| b.get("text").and_then(Value::as_str))
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .unwrap_or_default();
                let trimmed = err_text.trim();
                if !trimmed.is_empty() && trimmed != "No response requested." {
                    messages.push(ChatMessage {
                        role: "system".to_string(),
                        ts: v.get("timestamp").and_then(Value::as_str).map(String::from),
                        blocks: vec![Block {
                            kind: "divider".to_string(),
                            text: format!("⚠ 运行报错：{}", truncate_chars(trimmed, SUMMARY_MAX)),
                            name: None,
                        }],
                        pos: None,
                    });
                }
                return true;
            }
            // 整场用量累计（context = 最后一次调用的完整 prompt 规模）。
            // 子链（subagent）调用的 ctx/model 不代表主对话——总量照计，
            // 但不得覆盖上下文与模型，否则刷新时占比在主/子链间跳变。
            let sidechain = v.get("isSidechain").and_then(Value::as_bool).unwrap_or(false);
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
                if !sidechain && i + cr + cw > 0 {
                    u_ctx = i + cr + cw;
                }
            }
            if !sidechain {
                if let Some(m) = msg.get("model").and_then(Value::as_str) {
                    last_model = Some(m.to_string());
                }
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
                // 中点分叉定位：该消息来源行的 uuid
                pos: v
                    .get("uuid")
                    .and_then(Value::as_str)
                    .map(|u| serde_json::json!(u)),
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

    attach_subagents(&path, &mut messages);

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
        sage,
        usage,
    })
}

/// 单个子 agent 转录 → sub_* 块（文本与工具调用；thinking 略去以免嵌套区过长）。
/// 上限防止超长子任务把响应撑爆——截断后补一条提示块。
const SUBAGENT_BLOCK_MAX: usize = 120;

fn subagent_blocks(path: &Path) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    let mut truncated = false;
    for_each_line(path, |line| {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            return true;
        };
        if v.get("type").and_then(Value::as_str) != Some("assistant") {
            return true;
        }
        let Some(content) = v.pointer("/message/content").and_then(Value::as_array) else {
            return true;
        };
        for b in content {
            if out.len() >= SUBAGENT_BLOCK_MAX {
                truncated = true;
                return false;
            }
            match b.get("type").and_then(Value::as_str).unwrap_or("") {
                "text" => {
                    if let Some(t) = b.get("text").and_then(Value::as_str) {
                        if !t.trim().is_empty() {
                            out.push(Block {
                                kind: "sub_text".to_string(),
                                text: truncate_chars(t, SUMMARY_MAX),
                                name: None,
                            });
                        }
                    }
                }
                "tool_use" => {
                    let input = b.get("input").map(|i| i.to_string()).unwrap_or_default();
                    out.push(Block {
                        kind: "sub_tool".to_string(),
                        text: truncate_chars(&input, SUMMARY_MAX),
                        name: b.get("name").and_then(Value::as_str).map(String::from),
                    });
                }
                _ => {}
            }
        }
        true
    });
    if truncated {
        out.push(Block {
            kind: "sub_text".to_string(),
            text: "…（子 agent 过程较长，仅展示前段）".to_string(),
            name: None,
        });
    }
    out
}

/// 会话的子 agent 转录：<会话文件同名目录>/subagents/**/*.jsonl。
/// 返回 (可用于定位的键集合, 该子 agent 的块)。键有两种来源：
/// 文件名里的 agentId（普通子 agent，主流的 tool_result 正文含它）与父目录名
/// （workflow 子 agent，目录名即 wf id，同样出现在 tool_result 里）。
fn collect_subagents(session_path: &Path) -> Vec<(Vec<String>, Vec<Block>)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().and_then(|x| x.to_str()) == Some("jsonl") {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    walk(&session_path.with_extension("").join("subagents"), &mut files);
    files.sort();
    files
        .into_iter()
        .filter_map(|p| {
            let mut keys: Vec<String> = Vec::new();
            // agent-<agentId>.jsonl → agentId
            if let Some(id) = p
                .file_stem()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_prefix("agent-"))
            {
                keys.push(id.to_string());
            }
            // 父目录名（workflow 的 wf id）；直接挂在 subagents/ 下的不算
            if let Some(d) = p.parent().and_then(|d| d.file_name()).and_then(|n| n.to_str()) {
                if d != "subagents" {
                    keys.push(d.to_string());
                }
            }
            if keys.is_empty() {
                return None;
            }
            let blocks = subagent_blocks(&p);
            (!blocks.is_empty()).then_some((keys, blocks))
        })
        .collect()
}

/// 把子 agent 过程插到触发它的工具调用之后：先按工作流 id 在 tool_result 里定位，
/// 再回溯最近的一次 tool_use（前端按紧邻的卡片归组）。定位不到则整组丢弃，
/// 避免以主助手口吻散落在正文里。
fn attach_subagents(session_path: &Path, messages: &mut [ChatMessage]) -> usize {
    let groups = collect_subagents(session_path);
    if groups.is_empty() {
        return 0;
    }
    // 定位键 → 主流中最近一次 tool_use 的 (消息下标, 块下标)
    let mut anchor: HashMap<String, (usize, usize)> = HashMap::new();
    let mut last_tool: Option<(usize, usize)> = None;
    for (mi, m) in messages.iter().enumerate() {
        for (bi, b) in m.blocks.iter().enumerate() {
            if b.kind == "tool_use" {
                last_tool = Some((mi, bi));
            } else if b.kind == "tool_result" {
                for (keys, _) in &groups {
                    for g in keys {
                        if !anchor.contains_key(g) && b.text.contains(g.as_str()) {
                            if let Some(pos) = last_tool {
                                anchor.insert(g.clone(), pos);
                            }
                        }
                    }
                }
            }
        }
    }
    // 同一锚点可能对应多个子 agent：按块下标倒序插入，避免下标漂移
    let mut pending: Vec<(usize, usize, Vec<Block>)> = groups
        .into_iter()
        .filter_map(|(keys, blocks)| {
            keys.iter()
                .find_map(|g| anchor.get(g))
                .map(|&(mi, bi)| (mi, bi, blocks))
        })
        .collect();
    pending.sort_by(|a, b| (b.0, b.1).cmp(&(a.0, a.1)));
    let n = pending.len();
    for (mi, bi, blocks) in pending {
        let at = bi + 1;
        messages[mi].blocks.splice(at..at, blocks);
    }
    n
}

/// 中点分叉：复制父会话文件到指定消息 uuid（含）截断，逐行替换 sessionId，
/// 生成可 --resume 的新会话文件。cut_uuid=None 表示复制全量。返回新会话 id。
pub fn fork_at(parent_id: &str, cut_uuid: Option<&str>) -> Result<String, String> {
    let path = session_file_for(parent_id).ok_or("未找到父会话文件")?;
    let new_id = crate::history::new_uuid(false);
    let mut out = String::new();
    let mut hit_cut = false;
    for_each_line(&path, |line| {
        out.push_str(&line.replace(parent_id, &new_id));
        out.push('\n');
        if let Some(cut) = cut_uuid {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                if v.get("uuid").and_then(Value::as_str) == Some(cut) {
                    hit_cut = true;
                    return false; // 截断：该消息之后不带入
                }
            }
        }
        true
    });
    if let Some(cut) = cut_uuid {
        if !hit_cut {
            return Err(format!("未找到截断消息 {cut}"));
        }
    }
    if out.is_empty() {
        return Err("父会话文件为空".to_string());
    }
    let dest = path
        .parent()
        .ok_or("父目录缺失")?
        .join(format!("{new_id}.jsonl"));
    fs::write(&dest, out).map_err(|e| format!("写入失败: {e}"))?;
    Ok(new_id)
}

/// 按会话 id 定位会话文件（删除等操作用）。
pub fn session_file_for(session_id: &str) -> Option<PathBuf> {
    let root = dirs::home_dir()?.join(".claude").join("projects");
    find_session_file(&root, session_id)
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
            if !s.trim().is_empty() && !is_command_text(s) && !injected_user_text(s) {
                blocks.push(text_block(s.clone()));
            }
        }
        Some(Value::Array(items)) => {
            for item in items {
                match item.get("type").and_then(Value::as_str) {
                    Some("text") => {
                        let t = item.get("text").and_then(Value::as_str).unwrap_or("");
                        if !t.trim().is_empty()
                            && !is_command_text(t)
                            && !injected_user_text(t)
                        {
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

fn normalize_sage_user_blocks(messages: &[ChatMessage], mut blocks: Vec<Block>) -> Vec<Block> {
    let Some(original) = blocks
        .iter()
        .filter(|block| block.kind == "text")
        .find_map(|block| sage_original_task(&block.text))
    else {
        return blocks;
    };
    let already_visible = messages.iter().any(|message| {
        message.role == "user"
            && message
                .blocks
                .iter()
                .any(|block| block.kind == "text" && block.text.trim() == original)
    });
    if already_visible {
        return Vec::new();
    }
    let mut replaced = false;
    blocks.retain_mut(|block| {
        if block.kind != "text" || sage_original_task(&block.text).is_none() {
            return true;
        }
        if replaced {
            return false;
        }
        block.text = original.clone();
        replaced = true;
        true
    });
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ORIGINAL: &str = "检查这个活动玩法，时间是不是有问题";

    fn handoff_prompt() -> String {
        format!(
            "【SAGE HANDOFF】路由判定你接管本任务的完整所有权。请独立完成并给出最终结果。\n协作标识：sage-handoff-1\n来源会话：claude:origin-session\n当前执行者：Codex · gpt-5.6-sol\n\n{ORIGINAL}\n\n请查看图片文件: C:\\temp\\shot.png"
        )
    }

    fn collaborate_prompt(node: &str) -> String {
        format!(
            "【SAGE COLLABORATE · {node}】\n协作标识：sage-flow-1\n主会话：codex:owner-session\n任务所有者：Codex\n当前执行者：Claude\n完整分工：analysis → Claude\n\n原始任务：\n{ORIGINAL}\n\n请完成本节点并给出可供下游节点直接使用的明确产出。"
        )
    }

    #[test]
    fn sage_prompts_expose_only_the_original_task() {
        let handoff = sage_original_task(&handoff_prompt()).expect("应提取移交原始任务");
        assert!(handoff.starts_with(ORIGINAL));
        assert!(handoff.contains("请查看图片文件:"));
        assert_eq!(user_title_source(&handoff_prompt()), ORIGINAL);
        let handoff_meta = sage_prompt_metadata(&handoff_prompt()).unwrap();
        assert_eq!(handoff_meta.workflow_id.as_deref(), Some("sage-handoff-1"));
        assert_eq!(handoff_meta.source_agent.as_deref(), Some("claude"));
        assert_eq!(handoff_meta.source_session_id.as_deref(), Some("origin-session"));
        assert_eq!(handoff_meta.executor.as_deref(), Some("Codex · gpt-5.6-sol"));
        assert_eq!(
            sage_original_task(&collaborate_prompt("analysis")).as_deref(),
            Some(ORIGINAL)
        );
        // 跨会话上下文转移：来源会话记录附在任务之后，不得混进标题与正文
        let handoff_ctx = format!(
            "{}{}你接手的是一个已在进行中的对话。\n\n〔用户〕上一轮问题\n\n〔Codex〕上一轮结论",
            handoff_prompt(),
            SAGE_CONTEXT_MARKER
        );
        assert_eq!(sage_original_task(&handoff_ctx).as_deref(), sage_original_task(&handoff_prompt()).as_deref());
        assert_eq!(user_title_source(&handoff_ctx), ORIGINAL);
        assert_eq!(
            sage_prompt_metadata(&handoff_ctx).unwrap().source_session_id.as_deref(),
            Some("origin-session")
        );
        let collab_ctx = collaborate_prompt("analysis").replace(
            "\n\n请完成本节点",
            &format!("{}来源记录\n\n请完成本节点", SAGE_CONTEXT_MARKER),
        );
        assert_eq!(sage_original_task(&collab_ctx).as_deref(), Some(ORIGINAL));

        let meta = sage_prompt_metadata(&collaborate_prompt("analysis")).unwrap();
        assert_eq!(meta.kind, "collaborate");
        assert_eq!(meta.workflow_id.as_deref(), Some("sage-flow-1"));
        assert_eq!(meta.requirement.as_deref(), Some("analysis"));
        assert_eq!(meta.owner.as_deref(), Some("Codex"));
        assert_eq!(meta.executor.as_deref(), Some("Claude"));
        assert_eq!(meta.source_agent.as_deref(), Some("codex"));
        assert_eq!(meta.source_session_id.as_deref(), Some("owner-session"));
        assert_eq!(meta.original_task.as_deref(), Some(ORIGINAL));

        let first = normalize_sage_user_blocks(
            &[],
            user_blocks(Some(&json!(collaborate_prompt("analysis")))),
        );
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].text, ORIGINAL);

        let existing = vec![ChatMessage {
            role: "user".to_string(),
            ts: None,
            blocks: first,
            pos: None,
        }];
        let duplicate = normalize_sage_user_blocks(
            &existing,
            user_blocks(Some(&json!(collaborate_prompt("review")))),
        );
        assert!(duplicate.is_empty());
    }

    #[test]
    fn system_injected_user_messages_are_hidden() {
        let notification = "<task-notification>\n<status>completed</status>\n</task-notification>";
        let continuation = "This session is being continued from a previous conversation that ran out of context. The summary below covers the earlier portion of the conversation.\n\nSummary:\nThe user (via a 【SAGE HANDOFF】 prompt) requested work.";

        assert!(injected_user_text(notification));
        assert!(injected_user_text(continuation));
        assert!(user_blocks(Some(&json!(notification))).is_empty());
        assert!(user_blocks(Some(&json!(continuation))).is_empty());
    }
}

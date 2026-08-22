//! Codex 历史读取：`~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`（递归）
//! + `~/.codex/archived_sessions/rollout-*.jsonl`（扁平，archived=true）。
//! 契约见 docs/CONTRACT.md §2.2。首行 session_meta 建索引，(path, mtime) 增量缓存。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use serde_json::Value;

use crate::history::claude::{
    clean_title, for_each_line, normalize_path, system_time_to_iso, truncate_chars, SUMMARY_MAX,
};
use crate::types::{Block, ChatMessage, SessionSummary, Transcript};

/// 索引时向后扫描找标题的最大行数（不含首行 session_meta）。
const TITLE_SCAN_LINES: usize = 30;

// ---------------------------------------------------------------------------
// (path, mtime) 增量索引
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct IndexEntry {
    id: String,
    /// 规范化 cwd
    cwd: String,
    /// session_meta.payload.timestamp
    created: Option<String>,
    title: String,
    /// thread_source=="subagent" 或首行不可解析：整个文件跳过（缓存负结果）
    skip: bool,
}

fn index() -> &'static Mutex<HashMap<PathBuf, (SystemTime, IndexEntry)>> {
    static INDEX: OnceLock<Mutex<HashMap<PathBuf, (SystemTime, IndexEntry)>>> = OnceLock::new();
    INDEX.get_or_init(|| Mutex::new(HashMap::new()))
}

fn codex_root() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".codex"))
}

fn is_rollout_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with("rollout-") && n.ends_with(".jsonl"))
        .unwrap_or(false)
}

/// 递归收集 rollout 文件（sessions 树）。
fn walk(dir: &Path, out: &mut Vec<(PathBuf, SystemTime)>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_dir() {
            walk(&path, out);
        } else if ft.is_file() && is_rollout_file(&path) {
            if let Ok(mt) = entry.metadata().and_then(|m| m.modified()) {
                out.push((path, mt));
            }
        }
    }
}

/// 当前存在的全部 rollout 文件：(path, mtime, archived)。
fn collect_files() -> Vec<(PathBuf, SystemTime, bool)> {
    let mut out = Vec::new();
    let Some(root) = codex_root() else { return out };
    let mut live = Vec::new();
    walk(&root.join("sessions"), &mut live);
    for (p, mt) in live {
        out.push((p, mt, false));
    }
    if let Ok(rd) = fs::read_dir(root.join("archived_sessions")) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_file() && is_rollout_file(&path) {
                if let Ok(mt) = entry.metadata().and_then(|m| m.modified()) {
                    out.push((path, mt, true));
                }
            }
        }
    }
    out
}

/// 按会话 id 定位 rollout 文件（文件名含 id；多处匹配取 mtime 最新）。
pub fn rollout_path_for(session_id: &str) -> Option<PathBuf> {
    if session_id.is_empty() {
        return None;
    }
    collect_files()
        .into_iter()
        .filter(|(p, _, _)| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.contains(session_id))
                .unwrap_or(false)
        })
        .max_by_key(|(_, mt, _)| *mt)
        .map(|(p, _, _)| p)
}

/// 文件尾部（64KB）最后一条 token_count 的 info（运行中实时用量旁路用）。
pub fn latest_token_count(path: &Path) -> Option<Value> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = fs::File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    let take = len.min(64 * 1024);
    f.seek(SeekFrom::End(-(take as i64))).ok()?;
    let mut buf = Vec::with_capacity(take as usize);
    f.read_to_end(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf);
    for line in text.lines().rev() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue; // 截断产生的半行或非 JSON 行
        };
        if v.get("type").and_then(Value::as_str) != Some("event_msg") {
            continue;
        }
        let Some(p) = v.get("payload") else { continue };
        if p.get("type").and_then(Value::as_str) == Some("token_count") {
            return match p.get("info") {
                Some(i) if !i.is_null() => Some(i.clone()),
                _ => Some(p.clone()), // 旧式扁平字段
            };
        }
    }
    None
}

/// 读首行 session_meta + 向后最多 30 行找标题，构建索引条目。
fn index_file(path: &Path) -> IndexEntry {
    let mut entry = IndexEntry {
        id: String::new(),
        cwd: String::new(),
        created: None,
        title: String::new(),
        skip: true,
    };
    let mut n: usize = 0;
    for_each_line(path, |line| {
        n += 1;
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            return n != 1; // 首行坏了直接放弃；标题行坏了继续
        };
        if n == 1 {
            if v.get("type").and_then(Value::as_str) != Some("session_meta") {
                return false;
            }
            let Some(payload) = v.get("payload") else {
                return false;
            };
            if payload.get("thread_source").and_then(Value::as_str) == Some("subagent") {
                return false;
            }
            let id = payload
                .get("id")
                .and_then(Value::as_str)
                .or_else(|| payload.get("session_id").and_then(Value::as_str))
                .unwrap_or("");
            if id.is_empty() {
                return false;
            }
            entry.id = id.to_string();
            entry.cwd = normalize_path(payload.get("cwd").and_then(Value::as_str).unwrap_or(""));
            entry.created = payload
                .get("timestamp")
                .and_then(Value::as_str)
                .map(String::from)
                .or_else(|| v.get("timestamp").and_then(Value::as_str).map(String::from));
            entry.skip = false;
            return true;
        }
        if let Some(t) = title_candidate(&v) {
            entry.title = clean_title(&t);
            return false;
        }
        n <= TITLE_SCAN_LINES
    });
    if !entry.skip && entry.title.is_empty() {
        entry.title = "(无标题)".to_string();
    }
    entry
}

/// 标题候选：第一条真实用户输入（response_item message/user 或 event_msg user_message）。
fn title_candidate(v: &Value) -> Option<String> {
    let payload = v.get("payload")?;
    let text = match v.get("type").and_then(Value::as_str) {
        Some("response_item") => {
            if payload.get("type").and_then(Value::as_str) != Some("message")
                || payload.get("role").and_then(Value::as_str) != Some("user")
            {
                return None;
            }
            content_text(payload.get("content"))
        }
        Some("event_msg") => {
            if payload.get("type").and_then(Value::as_str) != Some("user_message") {
                return None;
            }
            payload
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        }
        _ => return None,
    };
    clean_title_text(&text)
}

/// 提取标题：
/// 1) 包装器（Orca/Tutti 等）注入格式：真实请求在 "## My request:" 之后 → 取其后首行文字；
/// 2) 首个非空行以 # / < / == 开头 → 整条是注入文档（AGENTS.md 全文、
///    "# Files mentioned by the user" 等），返回 None 让调用方扫描下一条消息；
/// 3) 其余取首行真实文字。
fn clean_title_text(s: &str) -> Option<String> {
    if let Some(idx) = s.find("## My request:") {
        let after = &s[idx + "## My request:".len()..];
        for line in after.lines() {
            let l = line.trim();
            if l.is_empty() || l.starts_with('<') || l.starts_with('#') {
                continue;
            }
            return Some(l.to_string());
        }
        return None;
    }
    let first = s.lines().map(str::trim).find(|l| !l.is_empty())?;
    if first.starts_with('#') || first.starts_with('<') || first.starts_with("==") {
        return None;
    }
    Some(first.to_string())
}

/// 非 JSON 参数（Orca 等包装器的 JS 输入）里扫描 cmd:"..." / "cmd":"..."，
/// 取第一个命令串做摘要。
fn extract_cmd(args: &str) -> Option<String> {
    let idx = args.find("cmd")?;
    let rest = &args[idx + 3..];
    let colon = rest.find(':')?;
    if colon > 3 {
        return None; // cmd 与冒号之间只容忍引号
    }
    let after = rest[colon + 1..].trim_start();
    // 记住开引号类型，只在同类引号处结束（命令里常含另一种引号）
    let (after, quote) = if let Some(a) = after.strip_prefix('"') {
        (a, '"')
    } else if let Some(a) = after.strip_prefix('\'') {
        (a, '\'')
    } else {
        return None;
    };
    let mut out = String::new();
    let mut chars = after.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(n) = chars.next() {
                    if n == 'n' {
                        out.push(' ');
                    } else {
                        out.push(n);
                    }
                }
            }
            c if c == quote => break,
            _ => out.push(c),
        }
        if out.chars().count() > 300 {
            break;
        }
    }
    let t = out.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// content[] 中 input_text / output_text / text 的 text 拼接。
fn content_text(content: Option<&Value>) -> String {
    let Some(Value::Array(items)) = content else {
        return String::new();
    };
    let texts: Vec<&str> = items
        .iter()
        .filter(|i| {
            matches!(
                i.get("type").and_then(Value::as_str),
                Some("input_text" | "output_text" | "text")
            )
        })
        .filter_map(|i| i.get("text").and_then(Value::as_str))
        .filter(|t| !t.is_empty())
        .collect();
    texts.join("\n")
}

/// 刷新索引并返回当前存在文件的条目快照。
fn refresh_index() -> Vec<(PathBuf, SystemTime, bool, IndexEntry)> {
    let files = collect_files();
    let mut out = Vec::with_capacity(files.len());
    let Ok(mut idx) = index().lock() else {
        return out;
    };
    for (path, mtime, archived) in files {
        let entry = match idx.get(&path) {
            Some((cached_mtime, e)) if *cached_mtime == mtime => e.clone(),
            _ => {
                let e = index_file(&path);
                idx.insert(path.clone(), (mtime, e.clone()));
                e
            }
        };
        out.push((path, mtime, archived, entry));
    }
    out
}

// ---------------------------------------------------------------------------
// 公开 API
// ---------------------------------------------------------------------------

/// 全部会话（含 archived，archived=true）；过滤 thread_source=="subagent"。
pub fn all_sessions() -> Vec<SessionSummary> {
    refresh_index()
        .into_iter()
        .filter(|(_, _, _, e)| !e.skip)
        .map(|(_, mtime, archived, e)| SessionSummary {
            agent: "codex".to_string(),
            id: e.id,
            title: e.title,
            project: e.cwd,
            created: e.created,
            updated: system_time_to_iso(mtime),
            archived,
        })
        .collect()
}

/// 单会话完整转录：从索引反查文件路径（索引未建则先建）。
pub fn transcript(session_id: &str) -> Result<Transcript, String> {
    let snapshot = refresh_index();
    let (path, entry) = snapshot
        .into_iter()
        .find(|(_, _, _, e)| !e.skip && e.id == session_id)
        .map(|(p, _, _, e)| (p, e))
        .ok_or_else(|| format!("未找到 Codex 会话: {session_id}"))?;

    let mut messages: Vec<ChatMessage> = Vec::new();
    let mut usage_total: Option<Value> = None;
    let (mut u_ctx, mut u_win) = (0i64, 0i64);
    let mut first_ts: Option<String> = None;
    let mut last_ts: Option<String> = None;
    for_each_line(&path, |line| {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            return true; // 单行解析失败跳过，不中断
        };
        let ts = v.get("timestamp").and_then(Value::as_str).map(String::from);
        if first_ts.is_none() {
            first_ts = ts.clone();
        }
        if ts.is_some() {
            last_ts = ts.clone();
        }
        let Some(payload) = v.get("payload") else {
            return true;
        };
        // 用量：token_count 事件（新式 info.total_token_usage / 旧式扁平字段）
        if v.get("type").and_then(Value::as_str) == Some("event_msg")
            && payload.get("type").and_then(Value::as_str) == Some("token_count")
        {
            let info = payload.get("info").unwrap_or(payload);
            let tot = info.get("total_token_usage").unwrap_or(info);
            usage_total = Some(tot.clone());
            if let Some(last) = info.get("last_token_usage") {
                // OpenAI 语义 input_tokens 已含缓存 → 即当前上下文占用
                let c = last
                    .get("input_tokens")
                    .and_then(Value::as_i64)
                    .unwrap_or(0);
                if c > 0 {
                    u_ctx = c;
                }
            }
            if let Some(w) = info.get("model_context_window").and_then(Value::as_i64) {
                u_win = w;
            }
            return true;
        }
        match v.get("type").and_then(Value::as_str) {
            Some("response_item") => handle_response_item(&mut messages, payload, ts),
            Some("event_msg") => handle_event_msg(&mut messages, payload, ts),
            Some("compacted") => messages.push(ChatMessage {
                role: "system".to_string(),
                ts,
                blocks: vec![Block {
                    kind: "divider".to_string(),
                    text: "上下文已压缩".to_string(),
                    name: None,
                }],
            }),
            // session_meta / turn_context / 其他行跳过
            _ => {}
        }
        true
    });

    let usage = usage_total.map(|tot| {
        let g = |k: &str| tot.get(k).and_then(Value::as_i64).unwrap_or(0);
        // OpenAI 语义：input_tokens 已包含 cached_input_tokens → 拆出未命中部分，
        // 与 claude（input 不含缓存）统一口径，前端 input+cache_read 即总输入
        let cr = g("cached_input_tokens");
        let input = (g("input_tokens") - cr).max(0);
        serde_json::json!({
            "input": input, "output": g("output_tokens"),
            "cache_read": cr, "cache_write": g("cache_write_input_tokens"),
            "context": if u_ctx > 0 { u_ctx } else { input + cr },
            "window": if u_win > 0 { Some(u_win) } else { None },
            "first_ts": first_ts, "last_ts": last_ts,
        })
    });
    Ok(Transcript {
        agent: "codex".to_string(),
        id: entry.id,
        project: entry.cwd,
        title: entry.title,
        messages,
        usage,
    })
}

// ---------------------------------------------------------------------------
// 行 → 消息映射
// ---------------------------------------------------------------------------

fn push_block(messages: &mut Vec<ChatMessage>, role: &str, ts: Option<String>, block: Block) {
    messages.push(ChatMessage {
        role: role.to_string(),
        ts,
        blocks: vec![block],
    });
}

/// 系统注入的「用户」消息：以 < / == 开头，或前段含环境上下文 / 指令注入标签
/// （environment_context、user_instructions 等，可能带前缀文字混排）。
fn injected_user_text(text: &str) -> bool {
    let t = text.trim_start();
    if t.starts_with('<') || t.starts_with("==") || t.starts_with("# AGENTS.md") {
        return true;
    }
    let head: String = t.chars().take(600).collect();
    head.contains("<INSTRUCTIONS>")
        || head.contains("<user_instructions>")
        || head.contains("<workspace_roots>")
        || head.contains("<permission_profile")
        // env_context 可能拼接在长注入文本尾部，全文扫
        || text.contains("<environment_context>")
}

/// 文本消息入列；response_item 与 event_msg 重复表达同一消息时去重
/// （同 role、文本相同、相邻出现，保留先出现的）。
fn push_text_dedup(messages: &mut Vec<ChatMessage>, role: &str, ts: Option<String>, text: String) {
    if let Some(last) = messages.last() {
        if last.role == role
            && last.blocks.len() == 1
            && last.blocks[0].kind == "text"
            && last.blocks[0].text.trim() == text.trim()
        {
            return;
        }
    }
    push_block(
        messages,
        role,
        ts,
        Block {
            kind: "text".to_string(),
            text,
            name: None,
        },
    );
}

fn handle_response_item(messages: &mut Vec<ChatMessage>, payload: &Value, ts: Option<String>) {
    match payload.get("type").and_then(Value::as_str) {
        Some("message") => {
            let role = match payload.get("role").and_then(Value::as_str) {
                Some(r @ ("user" | "assistant")) => r,
                _ => return, // developer / system 跳过
            };
            let text = content_text(payload.get("content"));
            // content 里的 input_image（image_url 可能是 data URL 或本地路径）
            let mut images: Vec<String> = Vec::new();
            if let Some(Value::Array(items)) = payload.get("content") {
                for it in items {
                    if it.get("type").and_then(Value::as_str) == Some("input_image") {
                        if let Some(u) = it.get("image_url").and_then(Value::as_str) {
                            if !u.is_empty() {
                                images.push(u.to_string());
                            }
                        }
                    }
                }
            }
            let injected = role == "user" && injected_user_text(&text);
            if !text.trim().is_empty() && !injected {
                push_text_dedup(messages, role, ts.clone(), text);
            }
            for u in images {
                push_block(
                    messages,
                    role,
                    ts.clone(),
                    Block {
                        kind: "image".to_string(),
                        text: u,
                        name: None,
                    },
                );
            }
        }
        Some("reasoning") => {
            let mut parts: Vec<String> = Vec::new();
            for key in ["summary", "content"] {
                if let Some(Value::Array(items)) = payload.get(key) {
                    for item in items {
                        if let Some(t) = item.get("text").and_then(Value::as_str) {
                            if !t.trim().is_empty() {
                                parts.push(t.to_string());
                            }
                        }
                    }
                }
            }
            let text = parts.join("\n");
            if !text.trim().is_empty() {
                push_block(
                    messages,
                    "assistant",
                    ts,
                    Block {
                        kind: "thinking".to_string(),
                        text,
                        name: None,
                    },
                );
            }
        }
        Some("function_call" | "custom_tool_call") => {
            let name = payload
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("tool")
                .to_string();
            // function_call 用 arguments；custom_tool_call 用 input
            let args = payload
                .get("arguments")
                .and_then(Value::as_str)
                .or_else(|| payload.get("input").and_then(Value::as_str))
                .unwrap_or("");
            let parsed = serde_json::from_str::<Value>(args).ok();
            // update_plan → plan 块（进度清单）
            if name == "update_plan" {
                if let Some(p) = &parsed {
                    let items = crate::run::plan_items(p.get("plan").or_else(|| p.get("items")));
                    if !items.is_empty() {
                        push_block(
                            messages,
                            "assistant",
                            ts,
                            Block {
                                kind: "plan".to_string(),
                                text: serde_json::to_string(&items).unwrap_or_default(),
                                name: None,
                            },
                        );
                        return;
                    }
                }
            }
            // exec 类参数提取真实 cmd 作摘要（避免整段原始 JSON/JS 刷屏）
            let extracted = parsed
                .as_ref()
                .and_then(|p| p.get("cmd").or_else(|| p.get("command")))
                .and_then(Value::as_str)
                .map(String::from)
                .or_else(|| extract_cmd(args));
            let summary = extracted.as_deref().unwrap_or(args);
            push_block(
                messages,
                "assistant",
                ts.clone(),
                Block {
                    kind: "tool_use".to_string(),
                    text: truncate_chars(summary, SUMMARY_MAX),
                    name: Some(name),
                },
            );
            // 参数里的补丁文件标记 → file_edit 块（apply_patch / exec 内嵌补丁均适用）
            for p in crate::run::patch_file_paths(args) {
                push_block(
                    messages,
                    "assistant",
                    ts.clone(),
                    Block {
                        kind: "file_edit".to_string(),
                        text: p,
                        name: None,
                    },
                );
            }
        }
        Some("function_call_output" | "custom_tool_call_output") => {
            let out = match payload.get("output") {
                Some(Value::String(s)) => s.clone(),
                Some(v @ Value::Array(_)) => content_text(Some(v)),
                _ => String::new(),
            };
            push_block(
                messages,
                "user",
                ts,
                Block {
                    kind: "tool_result".to_string(),
                    text: truncate_chars(&out, SUMMARY_MAX),
                    name: None,
                },
            );
        }
        _ => {}
    }
}

fn handle_event_msg(messages: &mut Vec<ChatMessage>, payload: &Value, ts: Option<String>) {
    match payload.get("type").and_then(Value::as_str) {
        Some("user_message") => {
            let text = payload.get("message").and_then(Value::as_str).unwrap_or("");
            if !text.trim().is_empty() && !injected_user_text(text) {
                push_text_dedup(messages, "user", ts.clone(), text.to_string());
            }
            // 旧式事件的图片路径数组（images / local_images）
            for key in ["images", "local_images"] {
                if let Some(Value::Array(items)) = payload.get(key) {
                    for it in items {
                        if let Some(p) = it.as_str().filter(|p| !p.is_empty()) {
                            push_block(
                                messages,
                                "user",
                                ts.clone(),
                                Block {
                                    kind: "image".to_string(),
                                    text: p.to_string(),
                                    name: None,
                                },
                            );
                        }
                    }
                }
            }
        }
        Some("agent_message") => {
            let text = payload.get("message").and_then(Value::as_str).unwrap_or("");
            if text.trim().is_empty() {
                return;
            }
            push_text_dedup(messages, "assistant", ts, text.to_string());
        }
        Some("turn_aborted") => {
            // 明确标出中断点，与正常结束区分
            messages.push(crate::types::ChatMessage {
                role: "system".to_string(),
                ts,
                blocks: vec![Block {
                    kind: "divider".to_string(),
                    text: "⚠ 回合被中止（进程被终止，任务未完成）".to_string(),
                    name: None,
                }],
            });
        }
        // 运行报错：error / turn_failed 事件，或 task_complete 携带错误消息
        //（如额度 429）。不映射的话报错在重开会话后就"消失"了。
        Some(t @ ("error" | "stream_error" | "turn_failed" | "task_complete")) => {
            let msg = payload
                .get("message")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty());
            let is_err = t != "task_complete" || payload.get("codex_error_info").is_some();
            if let (Some(m), true) = (msg, is_err) {
                messages.push(crate::types::ChatMessage {
                    role: "system".to_string(),
                    ts,
                    blocks: vec![Block {
                        kind: "divider".to_string(),
                        text: format!("⚠ 运行报错：{}", truncate_chars(m, SUMMARY_MAX)),
                        name: None,
                    }],
                });
            }
        }
        Some("item_completed") => {
            let Some(item) = payload.get("item") else {
                return;
            };
            // 大小写不敏感（并容忍下划线风格差异）
            let ty: String = item
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("")
                .chars()
                .filter(|c| *c != '_')
                .collect::<String>()
                .to_ascii_lowercase();
            // 文件改动 item → tool_use + file_edit 块
            if ty == "filechange" || ty == "patchapply" {
                let summary = item
                    .get("changes")
                    .map(|c| c.to_string())
                    .unwrap_or_default();
                push_block(
                    messages,
                    "assistant",
                    ts.clone(),
                    Block {
                        kind: "tool_use".to_string(),
                        text: truncate_chars(&summary, SUMMARY_MAX),
                        name: Some("apply_patch".to_string()),
                    },
                );
                for p in crate::run::file_change_paths(item) {
                    push_block(
                        messages,
                        "assistant",
                        ts.clone(),
                        Block {
                            kind: "file_edit".to_string(),
                            text: p,
                            name: None,
                        },
                    );
                }
                return;
            }
            // 待办/计划清单 → plan 块
            if ty == "todolist" {
                let items = crate::run::plan_items(item.get("items").or_else(|| item.get("plan")));
                if !items.is_empty() {
                    push_block(
                        messages,
                        "assistant",
                        ts,
                        Block {
                            kind: "plan".to_string(),
                            text: serde_json::to_string(&items).unwrap_or_default(),
                            name: None,
                        },
                    );
                }
                return;
            }
            let text = item
                .get("text")
                .and_then(Value::as_str)
                .map(String::from)
                .unwrap_or_else(|| content_text(item.get("content")));
            if text.trim().is_empty() {
                return;
            }
            match ty.as_str() {
                "usermessage" => {
                    if injected_user_text(&text) {
                        return;
                    }
                    push_text_dedup(messages, "user", ts, text);
                }
                "agentmessage" => push_text_dedup(messages, "assistant", ts, text),
                "reasoning" => push_block(
                    messages,
                    "assistant",
                    ts,
                    Block {
                        kind: "thinking".to_string(),
                        text,
                        name: None,
                    },
                ),
                _ => {}
            }
        }
        _ => {}
    }
}

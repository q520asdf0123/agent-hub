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
    clean_title, for_each_line, injected_user_text, normalize_path, sage_original_task,
    sage_prompt_metadata, system_time_to_iso, truncate_chars, SUMMARY_MAX,
};
use crate::types::{Block, ChatMessage, SagePromptMeta, SessionSummary, Transcript};

/// 索引时向后扫描找标题的最大行数（不含首行 session_meta）。
const TITLE_SCAN_LINES: usize = 30;

// ---------------------------------------------------------------------------
// (path, mtime, size) 增量索引
// （键含 size：Windows 上被子进程继承句柄的 rollout 文件 mtime 可能冻结在
// 创建时刻——如记忆插件的分离 worker——append-only 文件靠 size 兜底失效）
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct IndexEntry {
    id: String,
    /// 规范化 cwd
    cwd: String,
    /// session_meta.payload.timestamp
    created: Option<String>,
    title: String,
    sage: Option<SagePromptMeta>,
    /// thread_source=="subagent" 或首行不可解析：整个文件跳过（缓存负结果）
    skip: bool,
}

fn index() -> &'static Mutex<HashMap<PathBuf, (SystemTime, u64, IndexEntry)>> {
    static INDEX: OnceLock<Mutex<HashMap<PathBuf, (SystemTime, u64, IndexEntry)>>> =
        OnceLock::new();
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
fn walk(dir: &Path, out: &mut Vec<(PathBuf, SystemTime, u64)>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_dir() {
            walk(&path, out);
        } else if ft.is_file() && is_rollout_file(&path) {
            if let Ok(md) = entry.metadata() {
                if let Ok(mt) = md.modified() {
                    out.push((path, mt, md.len()));
                }
            }
        }
    }
}

/// 当前存在的全部 rollout 文件：(path, mtime, size, archived)。
fn collect_files() -> Vec<(PathBuf, SystemTime, u64, bool)> {
    let mut out = Vec::new();
    let Some(root) = codex_root() else { return out };
    let mut live = Vec::new();
    walk(&root.join("sessions"), &mut live);
    for (p, mt, sz) in live {
        out.push((p, mt, sz, false));
    }
    if let Ok(rd) = fs::read_dir(root.join("archived_sessions")) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_file() && is_rollout_file(&path) {
                if let Ok(md) = entry.metadata() {
                    if let Ok(mt) = md.modified() {
                        out.push((path, mt, md.len(), true));
                    }
                }
            }
        }
    }
    out
}

/// 分叉会话的父历史：读取父 rollout 到分叉点（优先 ordinal，缺则按字节偏移），
/// 只收集消息（用量由分叉会话自己的 token_count 延续，不重复聚合）。
/// 父会话自身也可能是分叉 → 递归拼接，depth 限深防环。
fn parent_history_messages(
    parent_id: &str,
    ord_limit: Option<u64>,
    byte_limit: Option<u64>,
    depth: u8,
) -> Vec<ChatMessage> {
    if depth == 0 {
        return Vec::new();
    }
    let Some(path) = rollout_path_for(parent_id) else {
        return Vec::new();
    };
    let mut messages: Vec<ChatMessage> = Vec::new();
    let mut fork_ref: Option<(String, Option<u64>, Option<u64>)> = None;
    let mut consumed: u64 = 0;
    for_each_line(&path, |line| {
        if let Some(limit) = byte_limit {
            if consumed >= limit {
                return false;
            }
        }
        consumed += line.len() as u64 + 1; // 行 + 换行
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            return true;
        };
        if let (Some(limit), Some(ord)) = (ord_limit, v.get("ordinal").and_then(Value::as_u64)) {
            if ord >= limit {
                return false;
            }
        }
        let ts = v.get("timestamp").and_then(Value::as_str).map(String::from);
        let Some(payload) = v.get("payload") else {
            return true;
        };
        match v.get("type").and_then(Value::as_str) {
            Some("session_meta") => {
                if let Some(fid) = payload.get("forked_from_id").and_then(Value::as_str) {
                    fork_ref = Some((
                        fid.to_string(),
                        payload
                            .pointer("/history_base/end_ordinal_exclusive")
                            .and_then(Value::as_u64),
                        payload
                            .pointer("/history_base/end_byte_offset")
                            .and_then(Value::as_u64),
                    ));
                }
            }
            Some("response_item") => {
                let before = messages.len();
                handle_response_item(&mut messages, payload, ts);
                backfill_pos(&mut messages, before, &v); // 继承区也可作分叉点
            }
            Some("event_msg") => {
                if payload.get("type").and_then(Value::as_str) != Some("token_count") {
                    let before = messages.len();
                    handle_event_msg(&mut messages, payload, ts);
                    backfill_pos(&mut messages, before, &v);
                }
            }
            _ => {}
        }
        true
    });
    if let Some((fid, ord, bytes)) = fork_ref {
        let mut out = parent_history_messages(&fid, ord, bytes, depth - 1);
        out.append(&mut messages);
        return out;
    }
    messages
}

/// 中点分叉定位回填：本行产出的消息标记来源行 ordinal。
fn backfill_pos(messages: &mut [ChatMessage], from: usize, line: &Value) {
    if let Some(ord) = line.get("ordinal").and_then(Value::as_u64) {
        for m in messages[from..].iter_mut() {
            m.pos = Some(serde_json::json!(ord));
        }
    }
}

/// 中点分叉：合成一个引用父会话到指定 ordinal 的分叉 rollout（codex 原生
/// fork 的同款存储结构：forked_from_id + history_base），零模型调用。
/// cut_ord=None 表示分叉到会话末尾。返回新会话 id。
pub fn fork_at(parent_id: &str, cut_ord: Option<u64>) -> Result<String, String> {
    fork_at_depth(parent_id, cut_ord, 6)
}

fn fork_at_depth(parent_id: &str, cut_ord: Option<u64>, depth: u8) -> Result<String, String> {
    if depth == 0 {
        return Err("分叉链过深".to_string());
    }
    let path = rollout_path_for(parent_id).ok_or("未找到父会话文件")?;
    let raw = fs::read(&path).map_err(|e| format!("读取父会话失败: {e}"))?;
    // 扫描行，定位截断点的 (end_ordinal_exclusive, end_byte_offset) 与父 meta
    let mut meta: Option<Value> = None;
    let mut end_ord: Option<u64> = None;
    let mut end_byte: u64 = 0;
    let mut offset: u64 = 0;
    for seg in raw.split(|b| *b == b'\n') {
        let seg_len = seg.len() as u64 + 1;
        let line = String::from_utf8_lossy(seg);
        let line = line.trim();
        if line.is_empty() {
            offset += seg_len;
            continue;
        }
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            offset += seg_len;
            continue;
        };
        if meta.is_none() && v.get("type").and_then(Value::as_str) == Some("session_meta") {
            meta = v.get("payload").cloned();
            // 截断点落在继承区（早于本文件首行 ordinal）→ 沿分叉链到祖先会话上分叉
            if let (Some(cut), Some(first_ord)) =
                (cut_ord, v.get("ordinal").and_then(Value::as_u64))
            {
                if cut < first_ord {
                    if let Some(fid) = meta
                        .as_ref()
                        .and_then(|m| m.get("forked_from_id"))
                        .and_then(Value::as_str)
                    {
                        return fork_at_depth(fid, cut_ord, depth - 1);
                    }
                    return Err("截断点早于会话起点".to_string());
                }
            }
        }
        let ord = v.get("ordinal").and_then(Value::as_u64);
        match (cut_ord, ord) {
            (Some(cut), Some(o)) if o > cut => break, // 截断点之后不计
            _ => {}
        }
        offset += seg_len;
        end_byte = offset.min(raw.len() as u64);
        if let Some(o) = ord {
            end_ord = Some(o + 1);
        }
    }
    let mut meta = meta.ok_or("父会话缺少 session_meta")?;
    let end_ord = end_ord.ok_or("父会话行缺少 ordinal（版本过旧？）")?;
    let new_id = crate::history::new_uuid(true);
    let (iso, (y, mo, d, h, mi, s)) = crate::history::now_utc_parts();
    if let Some(o) = meta.as_object_mut() {
        o.insert("id".into(), serde_json::json!(new_id));
        o.insert("session_id".into(), serde_json::json!(new_id));
        o.insert("timestamp".into(), serde_json::json!(iso));
        o.insert("forked_from_id".into(), serde_json::json!(parent_id));
        o.insert("history_mode".into(), serde_json::json!("paginated"));
        o.insert(
            "history_base".into(),
            serde_json::json!({
                "thread_id": parent_id,
                "end_ordinal_exclusive": end_ord,
                "end_byte_offset": end_byte,
            }),
        );
    }
    let envelope = serde_json::json!({
        "timestamp": iso,
        "ordinal": end_ord,
        "type": "session_meta",
        "payload": meta,
    });
    let dir = codex_root()
        .ok_or("无 home 目录")?
        .join("sessions")
        .join(format!("{y:04}"))
        .join(format!("{mo:02}"))
        .join(format!("{d:02}"));
    fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {e}"))?;
    let file = dir.join(format!(
        "rollout-{y:04}-{mo:02}-{d:02}T{h:02}-{mi:02}-{s:02}-{new_id}.jsonl"
    ));
    fs::write(&file, envelope.to_string() + "\n").map_err(|e| format!("写入失败: {e}"))?;
    Ok(new_id)
}

/// 按会话 id 定位 rollout 文件（文件名含 id；多处匹配取 mtime 最新）。
pub fn rollout_path_for(session_id: &str) -> Option<PathBuf> {
    if session_id.is_empty() {
        return None;
    }
    collect_files()
        .into_iter()
        .filter(|(p, _, _, _)| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.contains(session_id))
                .unwrap_or(false)
        })
        .max_by_key(|(_, mt, _, _)| *mt)
        .map(|(p, _, _, _)| p)
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
        sage: None,
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
        if entry.sage.is_none() {
            if let Some(payload) = v.get("payload") {
                if let Some(text) = record_user_prompt(v.get("type").and_then(Value::as_str), payload)
                {
                    entry.sage = sage_prompt_metadata(&text);
                }
            }
        }
        if entry.title.is_empty() {
            if let Some(t) = title_candidate(&v) {
                entry.title = clean_title(&t);
            }
        }
        if !entry.title.is_empty() && (entry.sage.is_some() || n >= TITLE_SCAN_LINES) {
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
    let sage_task = sage_original_task(s);
    let s = sage_task.as_deref().unwrap_or(s);
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
/// -i 图片附件会插入 `<image name=... path=...>` 标记文本段——剔除，
/// 否则整段文本以 `<` 开头会被注入过滤误杀（标题与正文都取不到）。
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
        .map(|t| t.trim())
        .filter(|t| !t.is_empty() && !t.starts_with("<image ") && *t != "</image>")
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
    for (path, mtime, size, archived) in files {
        let entry = match idx.get(&path) {
            Some((cached_mtime, cached_size, e))
                if *cached_mtime == mtime && *cached_size == size =>
            {
                e.clone()
            }
            _ => {
                let e = index_file(&path);
                idx.insert(path.clone(), (mtime, size, e.clone()));
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
            sage: e.sage,
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
    let mut last_model: Option<String> = None;
    let mut fork_ref: Option<(String, Option<u64>, Option<u64>)> = None;
    let mut spawn_calls: Vec<String> = Vec::new();
    let mut spawned: HashMap<String, String> = HashMap::new();
    let mut sage: Vec<SagePromptMeta> = Vec::new();
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
        if let Some(text) = record_user_prompt(v.get("type").and_then(Value::as_str), payload) {
            if let Some(meta) = sage_prompt_metadata(&text) {
                if !sage.contains(&meta) {
                    sage.push(meta);
                }
            }
        }
        if let Some(m) = record_model(v.get("type").and_then(Value::as_str), payload) {
            last_model = Some(m);
        }
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
            Some("session_meta") => {
                // 分叉会话不复制历史，只引用父会话（forked_from_id + history_base）
                if let Some(fid) = payload.get("forked_from_id").and_then(Value::as_str) {
                    fork_ref = Some((
                        fid.to_string(),
                        payload
                            .pointer("/history_base/end_ordinal_exclusive")
                            .and_then(Value::as_u64),
                        payload
                            .pointer("/history_base/end_byte_offset")
                            .and_then(Value::as_u64),
                    ));
                }
            }
            Some("response_item") => {
                // spawn_agent 调用按出现顺序记 call_id：与它产出的 tool_use 块同序，
                // 据此把 sub_agent_activity 里的子会话精确对到具体那一次派生。
                if payload.get("type").and_then(Value::as_str) == Some("function_call")
                    && payload.get("name").and_then(Value::as_str) == Some("spawn_agent")
                {
                    spawn_calls.push(
                        payload
                            .get("call_id")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                    );
                }
                let before = messages.len();
                handle_response_item(&mut messages, payload, ts);
                backfill_pos(&mut messages, before, &v);
            }
            Some("event_msg") => {
                // 子 agent 派生：记下被拉起的子会话 id，稍后按顺序挂到
                // 对应的 spawn_agent 工具块下（kind=="interacted" 是与已有
                // 子 agent 通信，不是新派生，跳过）。
                if payload.get("type").and_then(Value::as_str) == Some("sub_agent_activity")
                    && payload.get("kind").and_then(Value::as_str) == Some("started")
                {
                    if let (Some(call), Some(id)) = (
                        payload.get("event_id").and_then(Value::as_str),
                        payload.get("agent_thread_id").and_then(Value::as_str),
                    ) {
                        spawned.insert(call.to_string(), id.to_string());
                    }
                }
                let before = messages.len();
                handle_event_msg(&mut messages, payload, ts);
                backfill_pos(&mut messages, before, &v);
            }
            Some("compacted") => messages.push(ChatMessage {
                role: "system".to_string(),
                ts,
                blocks: vec![Block {
                    kind: "divider".to_string(),
                    text: "上下文已压缩".to_string(),
                    name: None,
                }],
                pos: None,
            }),
            // turn_context / 其他行跳过
            _ => {}
        }
        true
    });

    // 拼接继承的父会话历史（分叉链可多级，限深防环）
    if let Some((fid, ord, bytes)) = fork_ref {
        let mut parent = parent_history_messages(&fid, ord, bytes, 5);
        if !parent.is_empty() {
            parent.push(ChatMessage {
                role: "system".to_string(),
                ts: first_ts.clone(),
                blocks: vec![Block {
                    kind: "divider".to_string(),
                    text: "⑂ 分支点 · 以上为源会话继承的历史".to_string(),
                    name: None,
                }],
                pos: None,
            });
            parent.append(&mut messages);
            messages = parent;
        }
    }

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
            "model": last_model.clone(),
        })
    });
    attach_subagents(&mut messages, &spawn_calls, &spawned);
    Ok(Transcript {
        agent: "codex".to_string(),
        id: entry.id,
        project: entry.cwd,
        title: entry.title,
        messages,
        sage,
        usage,
        model: last_model,
    })
}

/// 该行是否宣告了当前生效的模型。取最后一条即会话真正在跑的模型：
/// resume 会带 -m 下发，前端不按它回填就等于用界面上次选的模型顶掉这条会话。
/// （claude 侧同样把它放在 usage.model。）
fn record_model(record_type: Option<&str>, payload: &Value) -> Option<String> {
    let raw = match record_type {
        Some("turn_context") => payload.get("model"),
        Some("event_msg")
            if payload.get("type").and_then(Value::as_str) == Some("thread_settings_applied") =>
        {
            payload.pointer("/thread_settings/model")
        }
        _ => None,
    };
    raw.and_then(Value::as_str)
        .map(str::trim)
        .filter(|m| !m.is_empty())
        .map(String::from)
}

fn record_user_prompt(record_type: Option<&str>, payload: &Value) -> Option<String> {
    match record_type {
        Some("response_item")
            if payload.get("type").and_then(Value::as_str) == Some("message")
                && payload.get("role").and_then(Value::as_str) == Some("user") =>
        {
            Some(content_text(payload.get("content")))
        }
        Some("event_msg")
            if payload.get("type").and_then(Value::as_str) == Some("user_message") =>
        {
            payload.get("message").and_then(Value::as_str).map(String::from)
        }
        Some("event_msg")
            if payload.get("type").and_then(Value::as_str) == Some("item_completed") =>
        {
            let item = payload.get("item").unwrap_or(payload);
            let kind = item
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_ascii_lowercase();
            (kind == "usermessage").then(|| {
                item.get("text")
                    .and_then(Value::as_str)
                    .map(String::from)
                    .unwrap_or_else(|| content_text(item.get("content")))
            })
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// 行 → 消息映射
// ---------------------------------------------------------------------------

fn push_block(messages: &mut Vec<ChatMessage>, role: &str, ts: Option<String>, block: Block) {
    messages.push(ChatMessage {
        role: role.to_string(),
        ts,
        blocks: vec![block],
        pos: None,
    });
}

/// 文本消息入列；response_item 与 event_msg 重复表达同一消息时去重
/// （同 role、文本相同、相邻出现，保留先出现的）。
fn push_text_dedup(messages: &mut Vec<ChatMessage>, role: &str, ts: Option<String>, text: String) {
    // 回溯时跳过同角色的纯图片消息：带图输入会先落若干 image 消息，只看 last
    // 会漏判，导致 response_item 与 item_completed 各插一条正文（正文重复两遍）。
    let prev_text = messages
        .iter()
        .rev()
        .take_while(|m| m.role == role)
        .find(|m| !(m.blocks.len() == 1 && m.blocks[0].kind == "image"));
    if let Some(last) = prev_text {
        // 比对前统一剔除图片标记：同一条消息的两种记录（response_item /
        // item_completed）只有一方会被剥离，按原文比对会漏判成两条。
        if last.blocks.len() == 1
            && last.blocks[0].kind == "text"
            && strip_image_refs(&last.blocks[0].text) == strip_image_refs(&text)
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

/// 把 SAGE 编排 prompt 还原成一次真实用户任务；后续节点/汇总的同任务副本不再展示。
/// 返回 false 表示该内部消息已过滤，调用方也应丢弃它附带的重复图片块。
fn push_visible_user_text(
    messages: &mut Vec<ChatMessage>,
    ts: Option<String>,
    text: String,
) -> bool {
    if let Some(original) = sage_original_task(&text) {
        let already_visible = messages.iter().any(|message| {
            message.role == "user"
                && message.blocks.iter().any(|block| {
                    block.kind == "text"
                        && strip_image_refs(&block.text) == strip_image_refs(&original)
                })
        });
        if already_visible {
            return false;
        }
        push_text_dedup(messages, "user", ts, original);
        return true;
    }
    if injected_user_text(&text) {
        return false;
    }
    push_text_dedup(messages, "user", ts, text);
    true
}

/// 子 agent 会话 → sub_* 块（只取正文与工具调用；与 claude 侧同口径）。
/// 上限防止超长子任务把响应撑爆——截断后补一条提示块。
const SUBAGENT_BLOCK_MAX: usize = 120;

fn subagent_blocks(path: &Path) -> Vec<Block> {
    subagent_tail(path, 0).0
}

/// 增量读取子 agent 会话：跳过前 skip 行，返回 (新块, 已读总行数)。
/// 供运行中旁路跟随；skip=0 即整份读取。
pub fn subagent_tail(path: &Path, skip: usize) -> (Vec<Block>, usize) {
    let mut out: Vec<Block> = Vec::new();
    let mut truncated = false;
    let mut seen: usize = 0;
    for_each_line(path, |line| {
        seen += 1;
        if seen <= skip {
            return true;
        }
        if out.len() >= SUBAGENT_BLOCK_MAX {
            truncated = true;
            return false;
        }
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            return true;
        };
        if v.get("type").and_then(Value::as_str) != Some("response_item") {
            return true;
        }
        let Some(p) = v.get("payload") else { return true };
        match p.get("type").and_then(Value::as_str) {
            Some("message") if p.get("role").and_then(Value::as_str) == Some("assistant") => {
                let text = content_text(p.get("content"));
                if !text.trim().is_empty() {
                    out.push(Block {
                        kind: "sub_text".to_string(),
                        text: truncate_chars(&text, SUMMARY_MAX),
                        name: None,
                    });
                }
            }
            Some("function_call" | "custom_tool_call") => {
                let args = p
                    .get("arguments")
                    .and_then(Value::as_str)
                    .or_else(|| p.get("input").and_then(Value::as_str))
                    .unwrap_or("");
                let summary = serde_json::from_str::<Value>(args)
                    .ok()
                    .and_then(|j| {
                        j.get("cmd")
                            .or_else(|| j.get("command"))
                            .and_then(Value::as_str)
                            .map(String::from)
                    })
                    .or_else(|| extract_cmd(args))
                    .unwrap_or_else(|| args.to_string());
                out.push(Block {
                    kind: "sub_tool".to_string(),
                    text: truncate_chars(&summary, SUMMARY_MAX),
                    name: p.get("name").and_then(Value::as_str).map(String::from),
                });
            }
            _ => {}
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
    (out, seen)
}

/// 本会话派生出的子 agent 会话：(子会话 id, 文件, 展示名)。
/// 只看 since 之后修改过的文件——全量扫首行对 2000+ 会话太贵，而运行中
/// 新派生的子会话必然是新写入的。
pub fn child_sessions(parent_id: &str, since: SystemTime) -> Vec<(String, PathBuf, String)> {
    let mut out = Vec::new();
    for (path, mtime, _size, _arch) in collect_files() {
        if mtime < since {
            continue;
        }
        let mut first = String::new();
        for_each_line(&path, |l| {
            first = l.to_string();
            false
        });
        let Ok(v) = serde_json::from_str::<Value>(&first) else {
            continue;
        };
        let Some(p) = v.get("payload") else { continue };
        if p.get("thread_source").and_then(Value::as_str) != Some("subagent") {
            continue;
        }
        let Some(spawn) = p.pointer("/source/subagent/thread_spawn") else {
            continue;
        };
        if spawn.get("parent_thread_id").and_then(Value::as_str) != Some(parent_id) {
            continue;
        }
        let Some(id) = p.get("id").and_then(Value::as_str) else {
            continue;
        };
        let label = spawn
            .get("agent_path")
            .and_then(Value::as_str)
            .or_else(|| spawn.get("agent_nickname").and_then(Value::as_str))
            .unwrap_or("subagent")
            .to_string();
        out.push((id.to_string(), path, label));
    }
    out
}

/// 把子 agent 过程插到派生它的 spawn_agent 工具块之后。
/// spawn_calls 是按出现顺序的 call_id，与 spawn_agent 工具块同序；spawned 由
/// sub_agent_activity 提供 call_id → 子会话 id。没有对应活动记录的派生（如启动
/// 失败）自然落空，不会错配到别的块上。
fn attach_subagents(
    messages: &mut [ChatMessage],
    spawn_calls: &[String],
    spawned: &HashMap<String, String>,
) {
    if spawned.is_empty() {
        return;
    }
    let anchors: Vec<(usize, usize)> = messages
        .iter()
        .enumerate()
        .flat_map(|(mi, m)| {
            m.blocks.iter().enumerate().filter_map(move |(bi, b)| {
                (b.kind == "tool_use" && b.name.as_deref() == Some("spawn_agent"))
                    .then_some((mi, bi))
            })
        })
        .collect();
    // 倒序插入，避免前面的插入让后面的下标漂移
    for (idx, &(mi, bi)) in anchors.iter().enumerate().rev() {
        let Some(child) = spawn_calls.get(idx).and_then(|c| spawned.get(c)) else {
            continue;
        };
        let Some(p) = rollout_path_for(child) else {
            continue;
        };
        let blocks = subagent_blocks(&p);
        if blocks.is_empty() {
            continue;
        }
        let at = bi + 1;
        messages[mi].blocks.splice(at..at, blocks);
    }
}

/// 剔除「请查看图片文件: <路径>」标记行（发图的文本形式，整行成立）。
fn strip_image_refs(text: &str) -> String {
    text.lines()
        .filter(|l| {
            let t = l.trim_start();
            !(t.starts_with("请查看图片文件:") || t.starts_with("请查看图片文件："))
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
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
            // 「请查看图片文件: <路径>」是本应用发图时写进 prompt 的文本形式，
            // 前端会把它还原成缩略图。同一条消息又带了 input_image 时，图片块
            // 已经把图带上了，正文里的标记若保留会让同一张图渲染两次。
            let text = if images.is_empty() {
                text
            } else {
                strip_image_refs(&text)
            };
            let keep_assets = if text.trim().is_empty() {
                true
            } else if role == "user" {
                push_visible_user_text(messages, ts.clone(), text)
            } else {
                push_text_dedup(messages, role, ts.clone(), text);
                true
            };
            for u in images.into_iter().filter(|_| keep_assets) {
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
            let keep_assets = text.trim().is_empty()
                || push_visible_user_text(messages, ts.clone(), text.to_string());
            // 旧式事件的图片路径数组（images / local_images）
            for key in ["images", "local_images"] {
                if let Some(Value::Array(items)) = payload.get(key) {
                    if !keep_assets {
                        continue;
                    }
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
                pos: None,
            });
        }
        // 运行报错：error / turn_failed 事件，或 task_complete 携带错误
        //（如额度 429）。两种形态：顶层 message + codex_error_info，或嵌套
        // error:{message, codex_error_info}。不映射的话报错重开会话后就"消失"。
        Some(t @ ("error" | "stream_error" | "turn_failed" | "task_complete")) => {
            let err_obj = payload.get("error").filter(|e| !e.is_null());
            let msg = payload
                .get("message")
                .and_then(Value::as_str)
                .or_else(|| {
                    err_obj
                        .and_then(|e| e.get("message"))
                        .and_then(Value::as_str)
                })
                .filter(|s| !s.trim().is_empty());
            let is_err = (t != "task_complete"
                || err_obj.is_some()
                || payload.get("codex_error_info").is_some())
                // 记忆开关注入的 hooks 信任豁免告警：每次运行必报的纯提示
                && !msg
                    .map(|m| m.contains("dangerously-bypass-hook-trust"))
                    .unwrap_or(false);
            if let (Some(m), true) = (msg, is_err) {
                messages.push(crate::types::ChatMessage {
                    role: "system".to_string(),
                    ts,
                    blocks: vec![Block {
                        kind: "divider".to_string(),
                        text: format!("⚠ 运行报错：{}", truncate_chars(m, SUMMARY_MAX)),
                        name: None,
                    }],
                    pos: None,
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
                    push_visible_user_text(messages, ts, text);
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ORIGINAL: &str = "检查这个活动玩法，时间是不是有问题";

    fn collaborate_prompt(node: &str) -> String {
        format!(
            "【SAGE COLLABORATE · {node}】\n任务所有者：Codex\n当前执行者：Codex\n完整分工：debugging → Codex\n\n原始任务：\n{ORIGINAL}\n\n请完成本节点并给出可供下游节点直接使用的明确产出。"
        )
    }

    #[test]
    fn sage_prompt_title_and_transcript_use_original_task_once() {
        let prompt = collaborate_prompt("debugging");
        let envelope = json!({
            "type": "event_msg",
            "payload": {"type": "user_message", "message": prompt}
        });
        assert_eq!(title_candidate(&envelope).as_deref(), Some(ORIGINAL));

        let mut messages = Vec::new();
        for node in ["debugging", "review"] {
            let payload = json!({
                "type": "message",
                "role": "user",
                "content": [{"type": "input_text", "text": collaborate_prompt(node)}]
            });
            handle_response_item(&mut messages, &payload, None);
        }
        let user_texts: Vec<&str> = messages
            .iter()
            .filter(|message| message.role == "user")
            .flat_map(|message| message.blocks.iter())
            .filter(|block| block.kind == "text")
            .map(|block| block.text.as_str())
            .collect();
        assert_eq!(user_texts, vec![ORIGINAL]);

        let continuation = json!({
            "type": "message",
            "role": "user",
            "content": [{
                "type": "input_text",
                "text": "This session is being continued from a previous conversation that ran out of context. The summary below covers the earlier portion of the conversation.\n\nSummary:\ninternal"
            }]
        });
        let mut injected_messages = Vec::new();
        handle_response_item(&mut injected_messages, &continuation, None);
        assert!(injected_messages.is_empty());
    }

    /// 会话模型要能从 rollout 还原：codex resume 带 -m，取不到就会用界面
    /// 当前选择顶掉老会话原本在跑的模型。
    #[test]
    fn session_model_is_recovered_from_rollout() {
        assert_eq!(
            record_model(Some("turn_context"), &json!({"model": "gpt-5.6-sol"})).as_deref(),
            Some("gpt-5.6-sol")
        );
        assert_eq!(
            record_model(
                Some("event_msg"),
                &json!({
                    "type": "thread_settings_applied",
                    "thread_settings": {"model": "gpt-5.2", "service_tier": "priority"}
                })
            )
            .as_deref(),
            Some("gpt-5.2")
        );
        // 无关行不参与，空串不算数（否则会把最后生效的模型抹掉）
        assert_eq!(record_model(Some("session_meta"), &json!({"model": "x"})), None);
        assert_eq!(
            record_model(Some("event_msg"), &json!({"type": "token_count"})),
            None
        );
        assert_eq!(record_model(Some("turn_context"), &json!({"model": "  "})), None);
        assert_eq!(record_model(Some("turn_context"), &json!({})), None);
    }
}

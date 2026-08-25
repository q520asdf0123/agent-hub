//! POST /api/chat 的流式实现（CONTRACT §3.2 / §3.3）。
//!
//! spawn 本地 CLI 真实 .exe（`kill_on_drop`，客户端断开即杀死子进程），prompt 走 stdin，
//! stdout/stderr 各起一个读行 task，经 mpsc 汇到同一 channel，`async_stream` 消费后
//! 逐行映射为统一 NDJSON 事件（`claude_map_line` / `codex_map_line`）产出 Body 流。

use std::collections::HashMap;
use std::convert::Infallible;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::header;
use axum::response::Response;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, Notify};

use crate::types::ChatReq;

/// 工具输入/输出摘要截断上限（按字符计，UTF-8 安全）。
const SUMMARY_MAX: usize = 400;

/// 子进程输出行（stdout / stderr 双任务汇入同一 channel）。
enum Line {
    Out(String),
    Err(String),
}

/// 跨行映射状态（claude 的 delta 去重、codex 的 init 去重、最终 done 所需信息）。
#[derive(Default)]
struct MapState {
    /// claude：发过 delta 后，assistant 事件的整块 text/thinking 不再重复发。
    sent_delta: bool,
    /// codex：init 事件只发一次。
    sent_init: bool,
    session_id: Option<String>,
    /// claude result 事件的 is_error。
    is_error: bool,
    /// CLI 报告的错误文本（进最终 done 事件）。
    error: Option<String>,
}

// ---------------------------------------------------------------------------
// 后台运行注册表：运行与 HTTP 连接解耦。刷新/断开只是断开「查看」，
// 子进程继续执行；只有 /api/stop 才会杀进程。
// ---------------------------------------------------------------------------

/// 一次后台运行：事件缓冲（可回放）+ 订阅通知。
pub struct RunState {
    pub agent: String,
    pub project: String,
    /// 侧栏展示用（截断）
    pub prompt: String,
    /// SAGE 内部 prompt 的结构化元数据；供前端展示 executor/节点，不回显内部正文。
    pub sage: Option<crate::types::SagePromptMeta>,
    /// 本轮完整 prompt：重连时回显，补上 CLI 尚未落盘的那条用户消息
    prompt_full: String,
    pub session_id: Mutex<Option<String>>,
    /// 已序列化的 NDJSON 行（不含换行）
    events: Mutex<Vec<String>>,
    notify: Notify,
    done: AtomicBool,
    kill: Notify,
    started: Instant,
    /// 结束后的 (ok, error)，供侧栏状态标识
    outcome: Mutex<Option<(bool, Option<String>)>>,
}

impl RunState {
    fn push(&self, v: &Value) {
        if let Some(sid) = v.get("session_id").and_then(Value::as_str) {
            *self.session_id.lock().unwrap() = Some(sid.to_string());
        }
        self.events.lock().unwrap().push(v.to_string());
        self.notify.notify_waiters();
    }
    fn finish_with(&self, ok: bool, error: Option<String>) {
        *self.outcome.lock().unwrap() = Some((ok, error));
        self.done.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }
    pub fn is_done(&self) -> bool {
        self.done.load(Ordering::SeqCst)
    }
    pub fn outcome(&self) -> Option<(bool, Option<String>)> {
        self.outcome.lock().unwrap().clone()
    }
}

#[derive(Default)]
pub struct RunRegistry(Mutex<HashMap<String, Arc<RunState>>>);

impl RunRegistry {
    pub fn get(&self, id: &str) -> Option<Arc<RunState>> {
        self.0.lock().unwrap().get(id).cloned()
    }
    fn insert(&self, id: String, rs: Arc<RunState>) {
        let mut map = self.0.lock().unwrap();
        // 顺带清理：已结束且超过 10 分钟的运行
        map.retain(|_, r| !(r.is_done() && r.started.elapsed().as_secs() > 600));
        map.insert(id, rs);
    }
    /// 全部未清理的运行（含 10 分钟内结束的，供状态标识）
    pub fn list_all(&self) -> Vec<(String, Arc<RunState>)> {
        let mut map = self.0.lock().unwrap();
        map.retain(|_, r| !(r.is_done() && r.started.elapsed().as_secs() > 600));
        map.iter().map(|(k, r)| (k.clone(), r.clone())).collect()
    }
    /// 发送停止信号（notify_one 带 permit，无竞态丢失）
    pub fn stop(&self, id: &str) -> bool {
        match self.get(id) {
            Some(r) if !r.is_done() => {
                r.kill.notify_one();
                true
            }
            _ => false,
        }
    }
}

fn new_run_id() -> String {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("run-{ms}-{}", SEQ.fetch_add(1, Ordering::Relaxed))
}

async fn terminate_child_tree(child: &mut tokio::process::Child) {
    #[cfg(windows)]
    if let Some(pid) = child.id() {
        let pid = pid.to_string();
        let mut killer = Command::new("taskkill.exe");
        killer
            .args(["/PID", &pid, "/T", "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let _ = tokio::time::timeout(Duration::from_secs(4), killer.status()).await;
    }
    let _ = child.start_kill();
}

fn visible_run_prompt(prompt: &str) -> Option<String> {
    if crate::history::claude::sage_prompt_metadata(prompt)
        .is_some_and(|meta| meta.kind == "summary")
    {
        return None;
    }
    if let Some(original) = crate::history::claude::sage_original_task(prompt) {
        return Some(original);
    }
    let prompt = prompt.trim();
    if crate::history::claude::injected_user_text(prompt)
        || [
            "【协作分工】",
            "【协作汇总】",
            "【协作复查回注】",
            "【协作复查】",
            "【协作追问】",
        ]
        .iter()
        .any(|prefix| prompt.starts_with(prefix))
    {
        return None;
    }
    (!prompt.is_empty()).then(|| prompt.to_string())
}

/// 订阅运行事件流。`announce` 为 Some 时首行发 {"t":"run","run_id"}；
/// `from` 是回放起点（0=全量回放，usize::MAX=只看新事件）。
pub fn attach(rs: Arc<RunState>, announce: Option<String>, from: usize) -> Response {
    let stream = async_stream::stream! {
        if let Some(id) = announce {
            yield Ok::<Vec<u8>, Infallible>(nl(&json!({"t": "run", "run_id": id})));
        }
        // 重连（只跟新事件，历史由转录重载补齐）：先回显本轮 prompt。
        // CLI 把用户消息写进会话文件需要时间（大会话尤其慢），这段空窗里
        // 刷新会让刚发出的消息「消失」，且直到切走再切回才补上。
        if from == usize::MAX && !rs.prompt_full.is_empty() {
            yield Ok::<Vec<u8>, Infallible>(
                nl(&json!({"t": "user_echo", "text": rs.prompt_full})),
            );
        }
        let mut cursor = {
            let ev = rs.events.lock().unwrap();
            from.min(ev.len())
        };
        loop {
            // 先注册通知再快照，避免漏唤醒
            let notified = rs.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let (batch, done) = {
                let ev = rs.events.lock().unwrap();
                (ev[cursor..].to_vec(), rs.is_done())
            };
            if !batch.is_empty() {
                cursor += batch.len();
                for l in batch {
                    let mut b = l.into_bytes();
                    b.push(b'\n');
                    yield Ok(b);
                }
                continue;
            }
            if done {
                break;
            }
            notified.await;
        }
    };
    ndjson_response(Body::from_stream(stream))
}

pub async fn stream_chat(registry: &RunRegistry, req: ChatReq) -> Response {
    if req.agent != "claude" && req.agent != "codex" {
        return done_only(&req, format!("未知 agent: {}", req.agent));
    }
    let project = req.project.clone();
    if !std::path::Path::new(&project).is_dir() {
        return done_only(&req, format!("项目目录不存在: {project}"));
    }
    // 本地命令（应用自己执行，不调用模型）
    let head = req.prompt.trim_start();
    if head == "/diff" || head.starts_with("/diff ") {
        return local_diff(&req).await;
    }
    if head == "/status" || head.starts_with("/status ") {
        return local_status(&req).await;
    }
    // /fork 前置校验（实际参数映射在 build_args）
    if head == "/fork" || head.starts_with("/fork ") {
        let has_session = req
            .session_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_some();
        if !has_session {
            return done_only(&req, "/fork 只能在已有会话中使用（先打开或创建一个会话）".to_string());
        }
        if head.strip_prefix("/fork").unwrap_or("").trim().is_empty() {
            return done_only(&req, "用法：/fork <分叉后要继续执行的指令>".to_string());
        }
    }
    let Some(resolved) = crate::cli::resolve(&req.agent).await else {
        return done_only(&req, format!("未找到 {} 可执行文件", req.agent));
    };

    let (args, stdin_payload) = build_args(&req);
    let mut cmd = Command::new(&resolved.exe);
    cmd.args(&args)
        .current_dir(&project)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    // 记忆开关显式两态：插件默认「配置文件存在即启用」，未开时必须显式置 0
    // 才能保证本次不召回不沉淀（claude/codex 的 hooks 子进程继承该变量）。
    cmd.env(
        "OPENVIKING_MEMORY_ENABLED",
        if req.memory.unwrap_or(false) { "1" } else { "0" },
    );
    // 记忆（OpenViking 插件）：hooks 读进程环境变量按次开关（下方 env 注入）；
    // codex 另需 hooks 信任豁免 flag（build_args 的 push_memory_bypass）。
    let mem_note: Option<&str> = if req.memory.unwrap_or(false) && !openviking_configured() {
        Some("OpenViking 未配置（~/.openviking/ovcli.conf），本次无记忆")
    } else {
        None
    };

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return done_only(&req, format!("启动 {} 失败: {e}", req.agent)),
    };

    // stdin 写入 prompt 后立即关闭（drop），否则 CLI 会一直等 EOF 不开始执行。
    if let Some(mut stdin) = child.stdin.take() {
        tokio::spawn(async move {
            let _ = stdin.write_all(stdin_payload.as_bytes()).await;
            let _ = stdin.shutdown().await;
            drop(stdin);
        });
    }

    let (tx, mut rx) = mpsc::channel::<Line>(256);
    if let Some(stdout) = child.stdout.take() {
        spawn_line_reader(stdout, tx.clone(), Line::Out);
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_line_reader(stderr, tx.clone(), Line::Err);
    }
    // 原始 tx 丢弃：两个读 task 结束后 rx 收到 None。
    drop(tx);

    let run_id = new_run_id();
    // 子 agent 旁路跟随的时间基线：只认本次运行之后写入的子会话文件
    let run_started = std::time::SystemTime::now() - std::time::Duration::from_secs(5);
    let visible_prompt = visible_run_prompt(&req.prompt).unwrap_or_default();
    let rs = Arc::new(RunState {
        agent: req.agent.clone(),
        project: req.project.clone(),
        prompt: visible_prompt.chars().take(80).collect(),
        sage: crate::history::claude::sage_prompt_metadata(&req.prompt),
        prompt_full: visible_prompt,
        session_id: Mutex::new(req.session_id.clone()),
        events: Mutex::new(Vec::new()),
        notify: Notify::new(),
        done: AtomicBool::new(false),
        kill: Notify::new(),
        started: Instant::now(),
        outcome: Mutex::new(None),
    });
    registry.insert(run_id.clone(), rs.clone());
    if let Some(note) = mem_note {
        rs.push(&json!({"t": "status", "text": note}));
    }

    // 泵任务：拥有子进程，独立于任何 HTTP 连接存活；只有 kill 信号才杀进程。
    let agent = req.agent.clone();
    let fallback_session = req.session_id.clone();
    let pump = rs.clone();
    tokio::spawn(async move {
        let mut st = MapState::default();
        let mut killed = false;
        let mut tailer_spawned = false;
        loop {
            tokio::select! {
                _ = pump.kill.notified(), if !killed => {
                    killed = true;
                    terminate_child_tree(&mut child).await;
                    break; // 不再等待可能被后代继承而无法 EOF 的 stdout/stderr 管道
                }
                line = rx.recv() => match line {
                    Some(Line::Out(l)) => {
                        let events = if agent == "claude" {
                            claude_map_line(&l, &mut st)
                        } else {
                            codex_map_line(&l, &mut st)
                        };
                        for ev in &events {
                            pump.push(ev);
                        }
                        // codex 的 exec 流只在回合结束报一次用量 → 拿到会话 id 后
                        // 起旁路 task 持续从回放文件读实时 token_count
                        if !tailer_spawned && agent == "codex" && st.session_id.is_some() {
                            tailer_spawned = true;
                            tokio::spawn(codex_usage_tailer(pump.clone()));
                            // 子 agent 旁路：只认本次运行开始后写入的子会话文件
                            tokio::spawn(codex_subagent_tailer(pump.clone(), run_started));
                        }
                    }
                    Some(Line::Err(l)) => {
                        // hooks 信任豁免告警：开记忆的 codex 每次运行必报一次，
                        // 纯提示不影响执行——按错误样式展示只会造成误判。
                        if !l.trim().is_empty() && !l.contains("dangerously-bypass-hook-trust") {
                            pump.push(&json!({"t": "stderr", "text": l}));
                        }
                    }
                    None => break,
                },
            }
        }
        let exit_ok = if killed {
            tokio::time::timeout(Duration::from_secs(5), child.wait())
                .await
                .ok()
                .and_then(Result::ok)
                .map(|status| status.success())
                .unwrap_or(false)
        } else {
            child.wait().await.map(|status| status.success()).unwrap_or(false)
        };
        // codex 短任务可能在 tailer 首次唤醒前就结束 → done 前同步补读最终用量
        if agent == "codex" {
            if let Some(sid) = st.session_id.clone() {
                let info = tokio::task::spawn_blocking(move || {
                    crate::history::codex::rollout_path_for(&sid)
                        .and_then(|p| crate::history::codex::latest_token_count(&p))
                })
                .await
                .ok()
                .flatten();
                if let Some(e) = info.as_ref().and_then(codex_usage_from_info) {
                    pump.push(&e);
                }
            }
        }
        let ok = exit_ok && !st.is_error && st.error.is_none() && !killed;
        let error: Option<String> = if killed {
            Some("已停止".to_string())
        } else if ok {
            None
        } else {
            Some(st.error.clone().unwrap_or_else(|| {
                if st.is_error {
                    "CLI 返回错误".to_string()
                } else {
                    "进程异常退出".to_string()
                }
            }))
        };
        let session_id = st.session_id.clone().or(fallback_session);
        pump.push(&json!({
            "t": "done",
            "ok": ok,
            "session_id": session_id,
            "error": error,
        }));
        pump.finish_with(ok, error);
        // 会话文件刚落盘，失效扫描缓存让侧栏立即看到最新状态
        crate::api::invalidate_sessions_cache();
    });

    // 发起方连接：全量回放 + 实时跟随（断开不影响泵任务）
    attach(rs, Some(run_id), 0)
}

/// codex 实时用量旁路：`exec --json` 的 stdout 流只在回合结束给一次 usage，
/// 而回放文件在整个回合期间持续写 token_count（含 last_token_usage 上下文与
/// model_context_window）。运行期间每 2 秒读一次文件尾部，把最新一条转成
/// scope=session 的 usage 事件推给订阅方，run 结束自动退出。
/// codex 子 agent 旁路跟随：子 agent 跑在独立会话文件里，主流的 --json 不含
/// 它的过程（连 spawn_agent 调用都不出现），只能轮询文件。首次发现某个子会话时
/// 合成一张工具卡片，之后把新增内容以 sub_* 事件挂进去。
async fn codex_subagent_tailer(rs: Arc<RunState>, started: std::time::SystemTime) {
    // 子会话 id → 已读行数
    let mut seen: HashMap<String, usize> = HashMap::new();
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        let done = rs.is_done();
        let Some(sid) = rs.session_id.lock().unwrap().clone() else {
            if done {
                return;
            }
            continue;
        };
        let children =
            tokio::task::spawn_blocking(move || crate::history::codex::child_sessions(&sid, started))
                .await
                .unwrap_or_default();
        for (cid, path, label) in children {
            let skip = seen.get(&cid).copied().unwrap_or(0);
            if skip == 0 {
                // 首次发现：先出卡片，后续内容挂在它下面
                rs.push(&json!({
                    "t": "tool_use", "name": "spawn_agent", "id": cid, "text": label,
                }));
            }
            let p = path.clone();
            let (blocks, total) =
                tokio::task::spawn_blocking(move || crate::history::codex::subagent_tail(&p, skip))
                    .await
                    .unwrap_or_default();
            for b in blocks {
                rs.push(&json!({
                    "t": if b.kind == "sub_tool" { "sub_tool" } else { "sub_text" },
                    "sub": cid,
                    "name": b.name,
                    "text": b.text,
                }));
            }
            seen.insert(cid, total);
        }
        if done {
            return; // 收尾再补读一轮后退出
        }
    }
}

async fn codex_usage_tailer(rs: Arc<RunState>) {
    let mut path: Option<std::path::PathBuf> = None;
    let mut last_sig = String::new();
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        if rs.is_done() {
            return;
        }
        let Some(sid) = rs.session_id.lock().unwrap().clone() else {
            continue;
        };
        if path.is_none() {
            path = crate::history::codex::rollout_path_for(&sid);
        }
        let Some(p) = path.clone() else { continue };
        let info = tokio::task::spawn_blocking(move || {
            crate::history::codex::latest_token_count(&p)
        })
        .await
        .ok()
        .flatten();
        let Some(info) = info else { continue };
        let sig = info.to_string();
        if sig == last_sig {
            continue; // 没有新数据，不重复推送
        }
        last_sig = sig;
        if let Some(e) = codex_usage_from_info(&info) {
            rs.push(&e);
        }
    }
}

/// 回放文件 token_count 的 info → scope=session 的 usage 事件。
fn codex_usage_from_info(info: &Value) -> Option<Value> {
    let tot = info.get("total_token_usage").unwrap_or(info);
    let mut e = usage_event("set", tot)?;
    e["scope"] = json!("session");
    if let Some(last) = info.get("last_token_usage") {
        let g = |k: &str| last.get(k).and_then(Value::as_i64).unwrap_or(0);
        let ctx = g("input_tokens"); // OpenAI 语义已含缓存 → 即当前上下文占用
        if ctx > 0 {
            e["context"] = json!(ctx);
        } else if let Some(o) = e.as_object_mut() {
            o.remove("context");
        }
    } else if let Some(o) = e.as_object_mut() {
        o.remove("context");
    }
    if let Some(w) = info.get("model_context_window").and_then(Value::as_i64) {
        e["window"] = json!(w);
    }
    Some(e)
}

/// 读管道任务：split(b'\n') 逐段读取 + from_utf8_lossy（与 history/claude.rs for_each_line
/// 同一手法），任何字节序列下都持续消费到 EOF。不能用 lines()/next_line()：遇非 UTF-8 字节
/// 返回 Err 即退出，此后子进程写满管道缓冲（Windows 约 64KB）会永久阻塞，done 事件永不发出。
fn spawn_line_reader<R>(reader: R, tx: mpsc::Sender<Line>, wrap: fn(String) -> Line)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut segments = BufReader::new(reader).split(b'\n');
        while let Ok(Some(bytes)) = segments.next_segment().await {
            let mut l = String::from_utf8_lossy(&bytes).into_owned();
            // 保持与 lines() 一致：去掉行尾 \r。
            if l.ends_with('\r') {
                l.pop();
            }
            if tx.send(wrap(l)).await.is_err() {
                break;
            }
        }
    });
}

/// codex 的 /init 是 prompt 展开（TUI 同理），无原生 exec 子命令。
const CODEX_INIT_PROMPT: &str = "请分析当前代码库，创建（或更新）仓库根目录的 AGENTS.md 文件：\
包含项目概述、构建/测试/运行命令、目录结构、代码风格约定和其他对编码 agent 有用的注意事项。\
内容要精炼、可执行，基于仓库实际情况，不要编造。";

/// OpenViking 是否已配置（与插件「配置文件存在即启用」的判定一致）。
fn openviking_configured() -> bool {
    dirs::home_dir()
        .map(|h| {
            h.join(".openviking").join("ovcli.conf").exists()
                || h.join(".openviking").join("ov.conf").exists()
        })
        .unwrap_or(false)
}

/// codex 记忆：插件 hooks 未经 TUI /hooks 审批会被静默跳过，且插件升级换哈希
/// 后要重新审批；无界面场景走官方自动化豁免 flag（插件源为官方仓库，已审源）。
fn push_memory_bypass(args: &mut Vec<String>, req: &ChatReq) {
    if req.agent == "codex" && req.memory.unwrap_or(false) {
        args.push("--dangerously-bypass-hook-trust".to_string());
    }
}

/// codex 图片附件：从 prompt 提取「请查看图片文件: <路径>」约定的本地图片，
/// 以官方 -i 参数直接附进消息——不依赖模型运行中调用读图工具，
/// 规避会话级读图偶发失效。prompt 文本行保留（转录展示仍可还原缩略图）。
fn push_codex_images(args: &mut Vec<String>, req: &ChatReq) {
    if req.agent != "codex" {
        return;
    }
    for line in req.prompt.lines() {
        if let Some(rest) = line.trim().strip_prefix("请查看图片文件:") {
            let p = rest.trim();
            if !p.is_empty() && std::path::Path::new(p).exists() {
                args.push("-i".to_string());
                args.push(p.to_string());
            }
        }
    }
}

/// Codex 所有模型固定使用官方请求级 Fast 模式；即使旧客户端传 false/None，
/// 也统一注入 service_tier="fast"，保证新建、续聊、fork 与 review 路径一致。
fn push_service_tier(args: &mut Vec<String>, req: &ChatReq) {
    if req.agent != "codex" {
        return;
    }
    args.push("-c".to_string());
    args.push("service_tier=\"fast\"".to_string());
}

/// 按 agent 与新建/resume 组装 argv（CONTRACT §3.2；prompt 一律走 stdin，不进 argv）。
/// 返回 (argv, stdin 内容)。codex 的 /review、/init 内置命令在此展开。
fn build_args(req: &ChatReq) -> (Vec<String>, String) {
    let mut args: Vec<String> = Vec::new();
    let mut payload = req.prompt.clone();
    let resume = req
        .session_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    // codex 新会话的内置命令映射（resume 时按普通文本发给会话）
    if req.agent == "codex" && resume.is_none() {
        let trimmed = req.prompt.trim_start();
        if trimmed == "/review" || trimmed.starts_with("/review ") {
            let instructions = trimmed.strip_prefix("/review").unwrap_or("").trim();
            args.push("exec".to_string());
            args.push("review".to_string());
            args.push("--json".to_string());
            args.push("--skip-git-repo-check".to_string());
            if let Some(e) = req.effort.as_deref().filter(|e| !e.trim().is_empty()) {
                args.push("-c".to_string());
                args.push(format!("model_reasoning_effort=\"{}\"", e.trim()));
            }
            push_service_tier(&mut args, req);
            push_memory_bypass(&mut args, req);
            if let Some(m) = req.model.as_deref().filter(|m| !m.trim().is_empty()) {
                args.push("-c".to_string());
                args.push(format!("model=\"{}\"", m.trim()));
            }
            args.push("--uncommitted".to_string());
            if instructions.is_empty() {
                // 无自定义说明：不传 "-"（review 仅在收到 "-" 时读 stdin），stdin 写空即关。
                return (args, String::new());
            }
            args.push("-".to_string());
            return (args, instructions.to_string());
        }
        if trimmed == "/init" || trimmed.starts_with("/init ") {
            let extra = trimmed.strip_prefix("/init").unwrap_or("").trim();
            payload = if extra.is_empty() {
                CODEX_INIT_PROMPT.to_string()
            } else {
                format!("{CODEX_INIT_PROMPT}\n补充要求：{extra}")
            };
        }
    }

    if req.agent == "claude" {
        args.extend(
            [
                "-p",
                "--output-format",
                "stream-json",
                "--verbose",
                "--include-partial-messages",
            ]
            .map(String::from),
        );
        if let Some(m) = req.model.as_deref().filter(|m| !m.trim().is_empty()) {
            args.push("--model".to_string());
            args.push(m.to_string());
        }
        match req.permission.as_deref() {
            Some("bypass") => {
                args.push("--permission-mode".to_string());
                args.push("bypassPermissions".to_string());
            }
            Some("accept-edits") => {
                args.push("--permission-mode".to_string());
                args.push("acceptEdits".to_string());
            }
            Some("plan") => {
                args.push("--permission-mode".to_string());
                args.push("plan".to_string());
            }
            // "read-only" / "default" / None：省略（claude 默认权限）。
            _ => {}
        }
        // 快速模式 / 思考等级都是设置键（-p 模式无专用 flag），合并进一次 --settings 注入
        let mut settings = serde_json::Map::new();
        if req.fast == Some(true) {
            settings.insert("fastMode".to_string(), serde_json::Value::Bool(true));
        }
        if let Some(e) = req.effort.as_deref().filter(|e| !e.trim().is_empty()) {
            settings.insert(
                "effortLevel".to_string(),
                serde_json::Value::String(e.trim().to_string()),
            );
        }
        if !settings.is_empty() {
            args.push("--settings".to_string());
            args.push(serde_json::Value::Object(settings).to_string());
        }
        if let Some(sid) = resume {
            args.push("--resume".to_string());
            args.push(sid.to_string());
            // /fork：以原会话为基础分叉出新 session id 继续（原会话不受影响）
            let trimmed = req.prompt.trim_start();
            if trimmed == "/fork" || trimmed.starts_with("/fork ") {
                args.push("--fork-session".to_string());
                payload = trimmed.strip_prefix("/fork").unwrap_or("").trim().to_string();
            }
        }
    } else {
        // codex
        match resume {
            Some(sid) => {
                let trimmed = req.prompt.trim_start();
                if trimmed == "/fork" || trimmed.starts_with("/fork ") {
                    // 原生 exec fork：分叉出新会话继续
                    args.push("exec".to_string());
                    args.push("fork".to_string());
                    args.push(sid.to_string());
                    args.push("--json".to_string());
                    args.push("--skip-git-repo-check".to_string());
                    if let Some(e) = req.effort.as_deref().filter(|e| !e.trim().is_empty()) {
                        args.push("-c".to_string());
                        args.push(format!("model_reasoning_effort=\"{}\"", e.trim()));
                    }
                    push_service_tier(&mut args, req);
                    push_memory_bypass(&mut args, req);
                    push_codex_images(&mut args, req);
                    if let Some(m) = req.model.as_deref().filter(|m| !m.trim().is_empty()) {
                        args.push("-m".to_string());
                        args.push(m.to_string());
                    }
                    if req.permission.as_deref() == Some("bypass") {
                        args.push("--dangerously-bypass-approvals-and-sandbox".to_string());
                    }
                    args.push("-".to_string());
                    payload = trimmed.strip_prefix("/fork").unwrap_or("").trim().to_string();
                    return (args, payload);
                }
                // resume：无 -C / -s（复用录制的 cwd；current_dir 仍兜底）。
                args.push("exec".to_string());
                args.push("resume".to_string());
                args.push(sid.to_string());
                args.push("--json".to_string());
                args.push("--skip-git-repo-check".to_string());
                if let Some(e) = req.effort.as_deref().filter(|e| !e.trim().is_empty()) {
                    args.push("-c".to_string());
                    args.push(format!("model_reasoning_effort=\"{}\"", e.trim()));
                }
                push_service_tier(&mut args, req);
                push_memory_bypass(&mut args, req);
                push_codex_images(&mut args, req);
                if req.permission.as_deref() == Some("bypass") {
                    args.push("--dangerously-bypass-approvals-and-sandbox".to_string());
                }
                args.push("-".to_string());
            }
            None => {
                args.push("exec".to_string());
                args.push("--json".to_string());
                args.push("--skip-git-repo-check".to_string());
                if let Some(e) = req.effort.as_deref().filter(|e| !e.trim().is_empty()) {
                    args.push("-c".to_string());
                    args.push(format!("model_reasoning_effort=\"{}\"", e.trim()));
                }
                push_service_tier(&mut args, req);
                push_memory_bypass(&mut args, req);
                push_codex_images(&mut args, req);
                args.push("-C".to_string());
                args.push(req.project.clone());
                if let Some(m) = req.model.as_deref().filter(|m| !m.trim().is_empty()) {
                    args.push("-m".to_string());
                    args.push(m.to_string());
                }
                match req.permission.as_deref() {
                    Some("bypass") => {
                        args.push("--dangerously-bypass-approvals-and-sandbox".to_string());
                    }
                    Some("read-only") => {
                        args.push("-s".to_string());
                        args.push("read-only".to_string());
                    }
                    _ => {
                        args.push("-s".to_string());
                        args.push("workspace-write".to_string());
                    }
                }
                args.push("-".to_string());
            }
        }
    }
    (args, payload)
}

// ---------- claude stdout 行 → 统一事件 ----------

/// 子代理行 → sub_* 事件（sub=父 tool_use id）。只取成块的文本与工具调用：
/// 子代理的 delta 与主流共用一条流，若参与 sent_delta 去重会污染主助手输出；
/// 成块内容足以还原「它做了什么」，思考过程略去以免嵌套区过长。
fn claude_map_subagent(v: &Value, parent: &str) -> Vec<Value> {
    let mut out = Vec::new();
    if v.get("type").and_then(Value::as_str) != Some("assistant") {
        return out;
    }
    let Some(content) = v.pointer("/message/content").and_then(Value::as_array) else {
        return out;
    };
    for block in content {
        match block.get("type").and_then(Value::as_str).unwrap_or("") {
            "text" => {
                if let Some(t) = block.get("text").and_then(Value::as_str) {
                    if !t.trim().is_empty() {
                        out.push(json!({"t": "sub_text", "sub": parent, "text": t}));
                    }
                }
            }
            "tool_use" => {
                let name = block.get("name").and_then(Value::as_str).unwrap_or("tool");
                let input = block
                    .get("input")
                    .map(|i| i.to_string())
                    .unwrap_or_default();
                out.push(json!({
                    "t": "sub_tool",
                    "sub": parent,
                    "name": name,
                    "text": truncate_chars(&input, SUMMARY_MAX),
                }));
            }
            _ => {}
        }
    }
    out
}

/// claude `--output-format stream-json` 一行 → 0..n 个统一事件。解析失败/未知 type 忽略。
fn claude_map_line(line: &str, st: &mut MapState) -> Vec<Value> {
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return Vec::new();
    };
    // 子代理（Task 工具）行带非 null 的 parent_tool_use_id：不进主流（避免以主助手
    // 口吻插入），改为带父 id 的 sub_* 事件，由前端嵌进对应工具卡片。
    if let Some(parent) = v.get("parent_tool_use_id").and_then(Value::as_str) {
        return claude_map_subagent(&v, parent);
    }
    let mut out: Vec<Value> = Vec::new();
    match v.get("type").and_then(Value::as_str).unwrap_or("") {
        "system" => {
            if v.get("subtype").and_then(Value::as_str) == Some("init") {
                if let Some(sid) = v.get("session_id").and_then(Value::as_str) {
                    st.session_id = Some(sid.to_string());
                    out.push(json!({"t": "init", "agent": "claude", "session_id": sid}));
                }
            }
        }
        "stream_event" => {
            let ev = v.get("event");
            let is_delta = ev.and_then(|e| e.get("type")).and_then(Value::as_str)
                == Some("content_block_delta");
            if is_delta {
                if let Some(delta) = ev.and_then(|e| e.get("delta")) {
                    match delta.get("type").and_then(Value::as_str).unwrap_or("") {
                        "text_delta" => {
                            if let Some(t) = delta.get("text").and_then(Value::as_str) {
                                st.sent_delta = true;
                                out.push(json!({"t": "delta", "channel": "text", "text": t}));
                            }
                        }
                        "thinking_delta" => {
                            if let Some(t) = delta.get("thinking").and_then(Value::as_str) {
                                st.sent_delta = true;
                                out.push(json!({"t": "delta", "channel": "thinking", "text": t}));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        "assistant" => {
            if let Some(content) = v.pointer("/message/content").and_then(Value::as_array) {
                for block in content {
                    match block.get("type").and_then(Value::as_str).unwrap_or("") {
                        "tool_use" => {
                            let name = block.get("name").and_then(Value::as_str).unwrap_or("tool");
                            // TodoWrite = 任务计划 → 专用 plan 事件（进度清单卡片）
                            if name == "TodoWrite" {
                                let items = plan_items(block.pointer("/input/todos"));
                                if !items.is_empty() {
                                    out.push(json!({"t": "plan", "items": items}));
                                    continue;
                                }
                            }
                            let input = block
                                .get("input")
                                .map(|i| i.to_string())
                                .unwrap_or_default();
                            out.push(json!({
                                "t": "tool_use",
                                "name": name,
                                // 调用 id：子代理事件按它归入对应卡片
                                "id": block.get("id").and_then(Value::as_str),
                                "text": truncate_chars(&input, SUMMARY_MAX),
                            }));
                            // 文件编辑类工具 → file_edit 事件（前端聚合成「已编辑 N 个文件」卡片）
                            if matches!(name, "Edit" | "Write" | "MultiEdit" | "NotebookEdit") {
                                if let Some(fp) =
                                    block.pointer("/input/file_path").and_then(Value::as_str)
                                {
                                    out.push(json!({"t": "file_edit", "path": fp}));
                                }
                            }
                        }
                        "text" => {
                            if !st.sent_delta {
                                if let Some(t) = block.get("text").and_then(Value::as_str) {
                                    if !t.is_empty() {
                                        out.push(json!({"t": "text", "text": t}));
                                    }
                                }
                            }
                        }
                        "thinking" => {
                            if !st.sent_delta {
                                if let Some(t) = block.get("thinking").and_then(Value::as_str) {
                                    if !t.is_empty() {
                                        out.push(json!({"t": "thinking", "text": t}));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            // 每条 assistant 消息的用量增量（result 事件会以权威值覆盖）
            if let Some(u) = v.pointer("/message/usage") {
                if let Some(ev) = usage_event("add", u) {
                    out.push(ev);
                }
            }
        }
        "user" => {
            if let Some(content) = v.pointer("/message/content").and_then(Value::as_array) {
                for block in content {
                    if block.get("type").and_then(Value::as_str) == Some("tool_result") {
                        let text = tool_result_text(block.get("content"));
                        out.push(json!({
                            "t": "tool_result",
                            "text": truncate_chars(&text, SUMMARY_MAX),
                        }));
                    }
                }
            }
        }
        "result" => {
            if let Some(sid) = v.get("session_id").and_then(Value::as_str) {
                st.session_id = Some(sid.to_string());
            }
            if let Some(u) = v.get("usage") {
                if let Some(ev) = usage_event("set", u) {
                    out.push(ev);
                }
            }
            if v.get("is_error").and_then(Value::as_bool).unwrap_or(false) {
                st.is_error = true;
                if st.error.is_none() {
                    st.error = v
                        .get("result")
                        .and_then(Value::as_str)
                        .filter(|s| !s.is_empty())
                        .or_else(|| v.get("subtype").and_then(Value::as_str))
                        .map(|s| truncate_chars(s, SUMMARY_MAX));
                }
            }
        }
        _ => {}
    }
    out
}

/// claude tool_result 的 content：字符串直接用；数组取其中 text 块拼接。
fn tool_result_text(content: Option<&Value>) -> String {
    match content {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|b| {
                if b.get("type").and_then(Value::as_str) == Some("text") {
                    b.get("text").and_then(Value::as_str)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

// ---------- codex stdout 行 → 统一事件（防御式） ----------

/// codex `exec --json` 一行 → 0..n 个统一事件。兼容新式 `{"type":"item.completed","item":..}`
/// 与旧式 `{"id":..,"msg":{"type":"agent_message",..}}` 包装；未知行忽略。
fn codex_map_line(line: &str, st: &mut MapState) -> Vec<Value> {
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return Vec::new();
    };
    let mut out: Vec<Value> = Vec::new();

    // 旧式包装：{"id":..,"msg":{"type":...}}，取 msg 作为事件本体。
    let ev: &Value = match v.get("msg") {
        Some(m) if m.is_object() && m.get("type").is_some() => m,
        _ => &v,
    };

    // 任意行发现 thread_id / session_id（顶层或 payload/msg 内）且未发过 init → init。
    if let Some(sid) = find_session_id(&v).or_else(|| find_session_id(ev)) {
        if st.session_id.is_none() {
            st.session_id = Some(sid.clone());
        }
        if !st.sent_init {
            st.sent_init = true;
            out.push(json!({"t": "init", "agent": "codex", "session_id": sid}));
        }
    }

    let etype = ev
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();

    // thread.started：若上面没拿到 id 也补发一次 init（session_id 可能为 null）。
    if etype.contains("thread") && etype.contains("started") {
        if !st.sent_init {
            st.sent_init = true;
            out.push(json!({
                "t": "init",
                "agent": "codex",
                "session_id": st.session_id.clone(),
            }));
        }
        return out;
    }
    if (etype.contains("turn") && etype.contains("started")) || etype == "task_started" {
        out.push(json!({"t": "status", "text": "运行中"}));
        return out;
    }
    // codex 用量：token_count（info.total_token_usage）或 turn.completed 的 usage
    if etype == "token_count" {
        let u = ev
            .pointer("/info/total_token_usage")
            .or_else(|| ev.get("info"))
            .unwrap_or(ev);
        if let Some(mut e) = usage_event("set", u) {
            e["scope"] = json!("session"); // total_token_usage 是整场累计（含 resume 之前）
            // 上下文占用 = 最后一次请求的 input_tokens（OpenAI 语义已含缓存部分）
            if let Some(last) = ev.pointer("/info/last_token_usage") {
                let g = |k: &str| last.get(k).and_then(Value::as_i64).unwrap_or(0);
                let ctx = g("input_tokens");
                if ctx > 0 {
                    e["context"] = json!(ctx);
                }
            } else if let Some(o) = e.as_object_mut() {
                o.remove("context"); // 全程累计不代表上下文，缺 last 时不给
            }
            if let Some(w) = ev
                .pointer("/info/model_context_window")
                .and_then(Value::as_i64)
            {
                e["window"] = json!(w);
            }
            out.push(e);
        }
        return out;
    }
    // turn.completed 的 usage 不再采用：它是该回合所有请求的 input 累加（含缓存、
    // 可达数百万），既不是上下文也与整场口径冲突；实时用量由 codex_usage_tailer
    // 从回放文件的 token_count 持续补发（含真实 last_token_usage 与窗口）。
    if etype.contains("turn") && etype.contains("completed") {
        return out;
    }
    // item 类：item.started / item.updated / item.completed（及 item_completed 变体）。
    // 约定只在 completed 时产出一次，避免 updated 重复渲染。
    if etype.starts_with("item") {
        if etype.ends_with("completed") {
            if let Some(item) = ev.get("item").or_else(|| ev.pointer("/payload/item")) {
                out.extend(codex_map_item(item, st));
            }
        }
        return out;
    }
    // error / turn.failed：记录 error 文本，进最终 done。
    if etype == "error" || etype.contains("failed") {
        let msg = ev
            .get("message")
            .and_then(Value::as_str)
            .or_else(|| ev.pointer("/error/message").and_then(Value::as_str))
            .or_else(|| ev.get("error").and_then(Value::as_str))
            .unwrap_or("CLI 报告错误");
        if st.error.is_none() && !codex_benign_error(msg) {
            st.error = Some(truncate_chars(msg, SUMMARY_MAX));
        }
        return out;
    }
    // 计划更新（plan_update / update_plan 事件形状）
    if etype.contains("plan") {
        let items = plan_items(ev.get("plan").or_else(|| ev.get("items")));
        if !items.is_empty() {
            out.push(json!({"t": "plan", "items": items}));
        }
        return out;
    }
    // 旧式 event_msg 直接形状。
    match etype.as_str() {
        "agent_message" => {
            if let Some(m) = ev.get("message").and_then(Value::as_str) {
                if !m.is_empty() {
                    out.push(json!({"t": "text", "text": m}));
                }
            }
        }
        "agent_reasoning" => {
            let m = ev
                .get("text")
                .and_then(Value::as_str)
                .or_else(|| ev.get("message").and_then(Value::as_str))
                .unwrap_or("");
            if !m.is_empty() {
                out.push(json!({"t": "thinking", "text": m}));
            }
        }
        // turn.completed / task_complete / token_count / user_message 等：无事件。
        _ => {}
    }
    out
}

/// codex item 对象 → 统一事件（item.type 大小写/下划线不敏感）。
fn codex_map_item(item: &Value, st: &mut MapState) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    let raw = item
        .get("type")
        .and_then(Value::as_str)
        .or_else(|| item.get("item_type").and_then(Value::as_str))
        .unwrap_or("");
    let t: String = raw
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    match t.as_str() {
        "agentmessage" => {
            let text = item_text(item);
            if !text.is_empty() {
                out.push(json!({"t": "text", "text": text}));
            }
        }
        "reasoning" => {
            let text = item_text(item);
            if !text.is_empty() {
                out.push(json!({"t": "thinking", "text": text}));
            }
        }
        "commandexecution" => {
            let cmd = match item.get("command") {
                Some(Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
                None => String::new(),
            };
            out.push(json!({
                "t": "tool_use",
                "name": "shell",
                "text": truncate_chars(&cmd, SUMMARY_MAX),
            }));
        }
        "filechange" | "patchapply" => {
            let summary = item
                .get("changes")
                .map(|c| c.to_string())
                .unwrap_or_default();
            out.push(json!({
                "t": "tool_use",
                "name": "apply_patch",
                "text": truncate_chars(&summary, SUMMARY_MAX),
            }));
            for p in file_change_paths(item) {
                out.push(json!({"t": "file_edit", "path": p}));
            }
        }
        "mcptoolcall" => {
            let server = item.get("server").and_then(Value::as_str).unwrap_or("");
            let tool = item
                .get("tool")
                .and_then(Value::as_str)
                .or_else(|| item.get("name").and_then(Value::as_str))
                .unwrap_or("mcp");
            let name = if server.is_empty() {
                tool.to_string()
            } else {
                format!("{server}.{tool}")
            };
            let args = item
                .get("arguments")
                .map(|a| match a {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_default();
            out.push(json!({
                "t": "tool_use",
                "name": name,
                "text": truncate_chars(&args, SUMMARY_MAX),
            }));
        }
        "websearch" => {
            let q = item.get("query").and_then(Value::as_str).unwrap_or("");
            out.push(json!({
                "t": "tool_use",
                "name": "web_search",
                "text": truncate_chars(q, SUMMARY_MAX),
            }));
        }
        "todolist" | "planupdate" | "updateplan" => {
            let items = plan_items(item.get("items").or_else(|| item.get("plan")));
            if !items.is_empty() {
                out.push(json!({"t": "plan", "items": items}));
            }
        }
        "error" => {
            let msg = item
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("CLI 报告错误");
            // codex 把「配置项不适用、已自动省略」类告警也走 error 事件，任务本身
            // 照常执行——这类不算失败。
            // 记忆开关注入的 hooks 信任豁免同理：每次运行必报，纯提示。
            let benign = codex_benign_error(msg);
            if st.error.is_none() && !benign {
                st.error = Some(truncate_chars(msg, SUMMARY_MAX));
            }
        }
        _ => {}
    }
    out
}

fn codex_benign_error(message: &str) -> bool {
    message.contains("will be omitted")
        || message.contains("dangerously-bypass-hook-trust")
        || message.contains("Skill descriptions were shortened to fit the skills context budget")
}

/// 统一 usage 事件：兼容 claude（cache_read/creation_input_tokens）与
/// codex（cached_input_tokens）字段名。mode=add 增量累计 / set 权威覆盖。
fn usage_event(mode: &str, u: &Value) -> Option<Value> {
    let g = |k: &str| u.get(k).and_then(Value::as_i64).unwrap_or(0);
    // codex（OpenAI 语义）的 input_tokens 已包含 cached_input_tokens，拆出未命中部分；
    // claude（Anthropic 语义）的 input 本就不含缓存，该键不存在，减 0 无影响。
    let codex_cached = g("cached_input_tokens");
    let input = (g("input_tokens") - codex_cached).max(0);
    let output = g("output_tokens");
    let cache_read = g("cache_read_input_tokens") + codex_cached;
    let cache_write = g("cache_creation_input_tokens") + g("cache_write_input_tokens");
    if input == 0 && output == 0 && cache_read == 0 && cache_write == 0 {
        return None;
    }
    Some(json!({
        "t": "usage",
        "mode": mode,
        "input": input,
        "output": output,
        "cache_read": cache_read,
        "cache_write": cache_write,
        // 本次调用的完整 prompt 规模 ≈ 当前上下文占用
        "context": input + cache_read + cache_write,
    }))
}

/// codex file_change item 的 changes（数组或对象）里提取文件路径。
pub fn file_change_paths(item: &Value) -> Vec<String> {
    let mut out = Vec::new();
    match item.get("changes") {
        Some(Value::Array(arr)) => {
            for c in arr {
                if let Some(p) = c.get("path").and_then(Value::as_str) {
                    out.push(p.to_string());
                } else if let Some(s) = c.as_str() {
                    out.push(s.to_string());
                }
            }
        }
        Some(Value::Object(obj)) => out.extend(obj.keys().cloned()),
        _ => {}
    }
    out
}

/// 从 apply_patch / exec 参数文本里扫描 "*** Update File: x" / "*** Add File: x"。
/// 路径可能含 Windows 反斜杠（如 D:\project\new_xs，其中 \n 是路径一部分），
/// 因此只在真实换行/引号处结束，另在「转义换行 + 补丁行标记」（\n 后跟 +/-/@/*）处截断。
pub fn patch_file_paths(s: &str) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for marker in ["*** Update File: ", "*** Add File: "] {
        let mut rest = s;
        while let Some(i) = rest.find(marker) {
            let after = &rest[i + marker.len()..];
            let end = after
                .find(|c| c == '\n' || c == '\r' || c == '"')
                .unwrap_or(after.len());
            let mut seg = &after[..end];
            let bytes = seg.as_bytes();
            let mut k = 0;
            while k + 2 < bytes.len() {
                if bytes[k] == b'\\'
                    && bytes[k + 1] == b'n'
                    && matches!(bytes[k + 2], b'+' | b'-' | b'@' | b'*')
                {
                    seg = &seg[..k];
                    break;
                }
                k += 1;
            }
            // JSON 转义形态归一：\\ → \，避免同一文件出现两种写法
            let p = seg
                .trim()
                .trim_end_matches('\\')
                .replace("\\\\", "\\");
            if p.len() > 2 && !v.iter().any(|x| *x == p) {
                v.push(p);
            }
            rest = &after[end..];
        }
    }
    v
}

/// 计划/待办清单归一化：条目文本取 content/step/text/title，状态取 status 或
/// completed 布尔（true→completed），输出 [{"text","status"}]，status ∈
/// completed | in_progress | pending。最多 50 条。
pub fn plan_items(v: Option<&Value>) -> Vec<Value> {
    let Some(Value::Array(arr)) = v else {
        return Vec::new();
    };
    arr.iter()
        .take(50)
        .filter_map(|it| {
            let text = it
                .get("content")
                .or_else(|| it.get("step"))
                .or_else(|| it.get("text"))
                .or_else(|| it.get("title"))
                .and_then(Value::as_str)
                .unwrap_or("");
            if text.trim().is_empty() {
                return None;
            }
            let status = match it.get("status").and_then(Value::as_str) {
                Some(s @ ("completed" | "in_progress" | "pending")) => s.to_string(),
                Some(other) if other.contains("progress") || other.contains("doing") => {
                    "in_progress".to_string()
                }
                Some(other) if other.contains("done") || other.contains("complete") => {
                    "completed".to_string()
                }
                _ => {
                    if it.get("completed").and_then(Value::as_bool) == Some(true) {
                        "completed".to_string()
                    } else {
                        "pending".to_string()
                    }
                }
            };
            Some(json!({"text": truncate_chars(text, 120), "status": status}))
        })
        .collect()
}

/// item 的文本：优先 `text` 字段，其次 `content[]` / `summary[]` 里的 text 拼接。
fn item_text(item: &Value) -> String {
    if let Some(t) = item.get("text").and_then(Value::as_str) {
        return t.to_string();
    }
    let mut parts: Vec<String> = Vec::new();
    for key in ["content", "summary"] {
        if let Some(arr) = item.get(key).and_then(Value::as_array) {
            for b in arr {
                if let Some(t) = b.as_str() {
                    parts.push(t.to_string());
                } else if let Some(t) = b.get("text").and_then(Value::as_str) {
                    parts.push(t.to_string());
                }
            }
        }
    }
    parts.join("\n")
}

/// 顶层或 payload 内的 thread_id / session_id。
fn find_session_id(v: &Value) -> Option<String> {
    for key in ["thread_id", "session_id"] {
        if let Some(s) = v.get(key).and_then(Value::as_str) {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
        if let Some(s) = v
            .pointer(&format!("/payload/{key}"))
            .and_then(Value::as_str)
        {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

// ---------- 工具 ----------

/// UTF-8 字符安全截断（不按字节切，避免多字节字符 panic）；超长时末尾加省略号，总长 ≤ max。
fn truncate_chars(s: &str, max: usize) -> String {
    let mut it = s.chars();
    let taken: String = it.by_ref().take(max).collect();
    if it.next().is_some() {
        let mut t: String = taken.chars().take(max.saturating_sub(1)).collect();
        t.push('…');
        t
    } else {
        taken
    }
}

/// 一行事件 JSON + '\n' 的字节。
fn nl(v: &Value) -> Vec<u8> {
    let mut b = v.to_string().into_bytes();
    b.push(b'\n');
    b
}

/// 本地命令：把若干事件一次性拼成 NDJSON 响应。
fn local_events(events: Vec<Value>) -> Response {
    let mut buf = Vec::new();
    for ev in &events {
        buf.extend(nl(ev));
    }
    ndjson_response(Body::from(buf))
}

async fn run_git(project: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(project)
        .output()
        .await
        .map_err(|e| format!("git 启动失败: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(if err.trim().is_empty() {
            format!("git {} 失败", args.join(" "))
        } else {
            err.trim().to_string()
        });
    }
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    // 超大 diff 截断（字符安全）
    if s.chars().count() > 60_000 {
        let mut t: String = s.chars().take(60_000).collect();
        t.push_str("\n…（输出过长已截断）");
        Ok(t)
    } else {
        Ok(s)
    }
}

/// /diff：本地 git 状态 + 未暂存/已暂存改动（TUI 同款能力），不调用模型。
async fn local_diff(req: &ChatReq) -> Response {
    let sections: [(&str, &[&str]); 3] = [
        ("状态（含未跟踪文件）", &["status", "--short", "--branch"]),
        ("未暂存改动", &["diff"]),
        ("已暂存改动", &["diff", "--cached"]),
    ];
    let mut text = String::new();
    for (label, args) in sections {
        match run_git(&req.project, args).await {
            Ok(out) => {
                let out = out.trim_end();
                if !out.is_empty() {
                    text.push_str("「");
                    text.push_str(label);
                    text.push_str("」\n```diff\n");
                    text.push_str(out);
                    text.push_str("\n```\n\n");
                }
            }
            Err(e) => {
                return local_events(vec![json!({
                    "t": "done", "ok": false,
                    "session_id": req.session_id,
                    "error": format!("/diff 失败：{e}"),
                })]);
            }
        }
    }
    if text.trim().is_empty() {
        text = "工作区干净，没有改动。".to_string();
    }
    local_events(vec![
        json!({"t":"status","text":"本地命令 /diff（未调用模型）"}),
        json!({"t":"text","text":text}),
        json!({"t":"done","ok":true,"session_id":req.session_id,"error":null}),
    ])
}

/// /status：应用与 CLI 的本地状态信息，不调用模型。
async fn local_status(req: &ChatReq) -> Response {
    let st = crate::cli::status().await;
    let fmt = |c: &crate::types::CliStatus| -> String {
        if c.installed {
            format!(
                "已安装 {}\n  {}",
                c.version.clone().unwrap_or_default(),
                c.path.clone().unwrap_or_default()
            )
        } else {
            format!(
                "未安装{}",
                c.error.clone().map(|e| format!("：{e}")).unwrap_or_default()
            )
        }
    };
    let text = format!(
        "「当前状态」\n- Agent：{}\n- 项目：{}\n- 会话：{}\n- 模型：{}\n- 思考等级：{}\n- 权限：{}\n\n「CLI」\n- Claude Code：{}\n- Codex：{}",
        req.agent,
        req.project,
        req.session_id.clone().unwrap_or_else(|| "（新会话，尚未创建）".to_string()),
        req.model.clone().unwrap_or_else(|| "默认".to_string()),
        req.effort.clone().unwrap_or_else(|| "默认".to_string()),
        req.permission.clone().unwrap_or_else(|| "默认".to_string()),
        fmt(&st.claude),
        fmt(&st.codex),
    );
    local_events(vec![
        json!({"t":"status","text":"本地命令 /status（未调用模型）"}),
        json!({"t":"text","text":text}),
        json!({"t":"done","ok":true,"session_id":req.session_id,"error":null}),
    ])
}

/// 只有一行 done 的错误响应（校验失败 / resolve 失败 / spawn 失败）。
fn done_only(req: &ChatReq, error: String) -> Response {
    let line = format!(
        "{}\n",
        json!({
            "t": "done",
            "ok": false,
            "session_id": req.session_id.clone(),
            "error": error,
        })
    );
    ndjson_response(Body::from(line))
}

fn ndjson_response(body: Body) -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .header("x-accel-buffering", "no")
        .body(body)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_always_uses_fast_service_tier() {
        for fast in [None, Some(false), Some(true)] {
            let req = ChatReq {
                agent: "codex".to_string(),
                project: ".".to_string(),
                prompt: "test".to_string(),
                session_id: None,
                model: Some("gpt-5.6-sol".to_string()),
                permission: Some("default".to_string()),
                fast,
                memory: Some(false),
                effort: Some("xhigh".to_string()),
            };
            let (args, _) = build_args(&req);
            assert!(
                args.windows(2).any(|pair| {
                    pair[0] == "-c" && pair[1] == "service_tier=\"fast\""
                }),
                "Codex fast={fast:?} 时仍须强制 Fast：{args:?}"
            );
            assert!(!args.iter().any(|arg| arg.contains("service_tier=\"standard\"")));
        }
    }

    #[test]
    fn internal_prompts_echo_only_the_original_user_task() {
        let prompt = "【SAGE COLLABORATE · coding】\n任务所有者：Claude\n当前执行者：Codex\n\n原始任务：\nCMS也要加上啊\n\n请完成本节点并给出可供下游节点直接使用的明确产出。";
        assert_eq!(visible_run_prompt(prompt).as_deref(), Some("CMS也要加上啊"));
        assert_eq!(visible_run_prompt("【协作汇总】内部回注"), None);
        let summary = "【SAGE COLLABORATE · 所有者汇总】\n任务所有者：Claude\n\n原始任务：\nCMS也要加上啊\n\n节点产出：\n...";
        assert_eq!(visible_run_prompt(summary), None);
        assert_eq!(visible_run_prompt("真正的用户输入").as_deref(), Some("真正的用户输入"));
    }

    #[test]
    fn codex_skill_budget_notice_is_not_a_run_failure() {
        let line = r#"{"type":"error","message":"Skill descriptions were shortened to fit the skills context budget. Codex can still see every skill."}"#;
        let mut st = MapState::default();
        codex_map_line(line, &mut st);
        assert!(st.error.is_none());
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn terminate_child_tree_stops_descendant_process() {
        let mut child = Command::new("cmd.exe")
            .args(["/D", "/S", "/C", "ping -t 127.0.0.1"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("应启动测试进程树");
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        terminate_child_tree(&mut child).await;
        let status = tokio::time::timeout(std::time::Duration::from_secs(5), child.wait()).await;
        assert!(status.is_ok(), "停止后子进程必须在 5 秒内退出");
    }

    /// 子代理行（parent_tool_use_id 非 null）：不进主流，转为带父 id 的 sub_* 事件。
    #[test]
    fn subagent_line_maps_to_sub_events() {
        let line = r#"{"type":"assistant","parent_tool_use_id":"toolu_PARENT","message":{"content":[
            {"type":"text","text":"先看目录"},
            {"type":"tool_use","name":"Bash","id":"toolu_INNER","input":{"command":"ls"}}]}}"#;
        let mut st = MapState::default();
        let evs = claude_map_line(line, &mut st);
        assert_eq!(evs.len(), 2, "应产出文本与工具两个事件: {evs:?}");
        assert_eq!(evs[0]["t"], "sub_text");
        assert_eq!(evs[0]["sub"], "toolu_PARENT");
        assert_eq!(evs[0]["text"], "先看目录");
        assert_eq!(evs[1]["t"], "sub_tool");
        assert_eq!(evs[1]["sub"], "toolu_PARENT");
        assert_eq!(evs[1]["name"], "Bash");
        // 主流状态不受子代理影响（否则主助手的整块文本会被误去重）
        assert!(!st.sent_delta);
    }

    /// 子代理的 delta 与用量不进主流：只有成块内容被采纳。
    #[test]
    fn subagent_delta_ignored() {
        let line = r#"{"type":"stream_event","parent_tool_use_id":"toolu_PARENT",
            "event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"x"}}}"#;
        let mut st = MapState::default();
        assert!(claude_map_line(line, &mut st).is_empty());
        assert!(!st.sent_delta);
    }

    /// 主链 tool_use 带调用 id，供前端把子代理事件归入对应卡片。
    #[test]
    fn main_tool_use_carries_id() {
        let line = r#"{"type":"assistant","parent_tool_use_id":null,"message":{"content":[
            {"type":"tool_use","name":"Task","id":"toolu_PARENT","input":{"description":"查一下"}}]}}"#;
        let mut st = MapState::default();
        let evs = claude_map_line(line, &mut st);
        let tu = evs.iter().find(|e| e["t"] == "tool_use").expect("应有 tool_use 事件");
        assert_eq!(tu["id"], "toolu_PARENT");
        assert_eq!(tu["name"], "Task");
    }
}

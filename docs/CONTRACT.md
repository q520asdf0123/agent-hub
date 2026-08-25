# CONTRACT — 模块契约与实现事实（并行实现的唯一协调文档）

> 所有并行实现者必读。**不得修改** `src/main.rs`、`src/types.rs`、本文件；发现契约问题在最终报告中说明，由集成者裁决。
> 文件所有权（互斥，不得越界写）：
> - **history 实现者**：`src/history/claude.rs`、`src/history/codex.rs`（`mod.rs` 已固定）
> - **backend 实现者**：`src/api.rs`、`src/cli.rs`、`src/run.rs`、`src/config.rs`
> - **frontend 实现者**：`static/index.html`、`static/style.css`、`static/app.js`
> - 集成者拥有全部文件的修复权。

## 0. 环境事实（2026-08-21 本机实测）

- OS：Windows 11，git-bash 可用；home = `C:\Users\your-name`。
- Claude Code 2.1.237：真实 exe `C:\nvm4w\nodejs\node_modules\@anthropic-ai\claude-code\bin\claude.exe`（PATH 上的 `claude` 是 .cmd shim，另有 `C:\Users\your-name\.local\bin\claude.cmd` 指向 2.1.220 旧版 — **解析时优先 npm 全局目录的新版**）。
- Codex 0.148.0：真实 exe `C:\nvm4w\nodejs\node_modules\@openai\codex\node_modules\@openai\codex-win32-x64\vendor\x86_64-pc-windows-msvc\bin\codex.exe`。
- Rust CreateProcessW **不能执行 .cmd**；即使 `Command::new("claude.cmd")`（Rust 走 cmd /C）也会拒绝含元字符的 argv。**规则：spawn 真实 .exe，prompt 走 stdin。**

## 1. 类型契约

全部 API 序列化类型在 `src/types.rs`（已固定，见源码）。要点：

- `SessionSummary { agent, id, title, project, created, updated, archived, sage? }` — `project` 是**真实规范化路径**（如 `D:\project\demo_app`），`updated` 用文件 mtime 转 ISO 8601 UTC 字符串；`sage` 携带列表级不可见 lineage 摘要，使侧栏首次加载即可识别 child/handoff target。
- `Transcript { agent, id, project, title, messages: Vec<ChatMessage>, sage: Vec<SagePromptMeta>, usage? }`；`ChatMessage { role, ts, blocks }`；`Block { kind, text, name }`，`kind ∈ text|thinking|tool_use|tool_result|image|divider`。`sage` 是从原生历史内部 prompt 提取的不可见 workflow/source-agent/source-session/owner/executor/requirement/original-task 元数据，只用于恢复协作或移交关系，不直接渲染。
- `ChatReq { agent, project, prompt, session_id?, model?, permission?, fast?, memory?, effort? }`，`permission ∈ "bypass"|"accept-edits"|"plan"|"read-only"|"default"`。Claude 按 `fast=true` 注入 `fastMode`；Codex 忽略该字段的关闭值，所有模型和所有执行路径都强制注入请求级 `service_tier="fast"`。
- `SageReq { prompt, agent?, failed?, state?, constraints? }`：`state` 映射官方 `ExecutionState`；`constraints` 映射 Task 约束，服务端以真实 CLI 探测覆盖 `available_agents` 并注入当前 load。
- SAGE 候选 executor id 为 `runtime::model`；服务端从本机模型目录构建能力、相对成本/延迟、权限、load、Fast 状态与支持的 effort 画像。决策返回 `mode/primary/partners/agents/executors/team_size/team_limit/complexity/assignments/efforts/primary_effort/summary_effort/dependencies/topology/switch_recommended` 及审计字段。整体复杂度 effort 进入路由成本/延迟先验，每个 requirement 再选择实际 effort；GPT-5.6 Sol 的自动 effort 硬性封顶为 `xhigh`，Luna/mini/Spark 等低成本模型不设额外 `high` 上限，支持时可到 `max`；`COLLABORATE` 依 requirement DAG 分波执行，同一 executor 的节点由 `executeWave()` 严格串行，不同 executor 的无依赖节点并行，最后由 incumbent 按 summary effort 汇总。不得增加“review 必须换 agent/session”等 assignments 之外的自定义规则。
- 后台运行表中的可见 prompt 与 CLI 实际 prompt 分离：`GET /api/runs` 返回真实用户任务及可选 SAGE metadata；`GET /api/run?id=` 的 `user_echo` 不回显 summary/回注/内部提示。Codex skill-context-budget 等通知不计为失败，SAGE 成功节点的纯 WARN stderr 不渲染。
- `GET /api/instance` 返回进程启动级 `instance_id`。前端每 3 秒检测实例变化；部署重启时保存当前 session/未发送草稿到 sessionStorage，自动 reload 后恢复，不调用 stop、不终止后台任务。

路径规范化函数（history 侧实现并导出 `pub fn normalize_path(p: &str) -> String`，放 `history/claude.rs` 或经 `mod.rs` re-export 均可，backend 可调用）：剥 `\\?\` 前缀、`/`→`\`、盘符大写、去尾部 `\`。

## 2. history 模块契约（history 实现者）

`src/history/mod.rs` 已固定为 `pub mod claude; pub mod codex;`（外加文档注释）。必须实现并保持以下**精确签名**（同步 fn，调用方负责 spawn_blocking）：

```rust
// history/claude.rs
pub fn normalize_path(p: &str) -> String;
pub fn all_sessions() -> Vec<crate::types::SessionSummary>;   // 全部项目全部会话
pub fn transcript(project: &str, session_id: &str) -> Result<crate::types::Transcript, String>;

// history/codex.rs
pub fn all_sessions() -> Vec<crate::types::SessionSummary>;   // 含 archived（archived=true），过滤 thread_source=="subagent"
pub fn transcript(session_id: &str) -> Result<crate::types::Transcript, String>;
```

### 2.1 Claude 历史（实测格式）
- 根：`~/.claude/projects/`，项目目录名 = 真实路径 `[^A-Za-z0-9]`→`-`（有损）。会话 = 目录**顶层** `*.jsonl`，文件名 stem 即 session uuid；忽略一切子目录（`memory/`、`<uuid>/`）。
- 真实路径恢复：读该目录任一 jsonl 前若干行的 `cwd` 字段（user/assistant 行都有）；读不到（如只有 memory/ 的空目录）则跳过该目录。
- 标题：`~/.claude/history.jsonl` 每行 `{"display":"...","timestamp":epoch_ms,"project":"D:\\project","sessionId":"uuid"}`，取该 sessionId 最早的、`display` 不以 `/` 开头的行；fallback：会话文件里第一条 `type=="user" && !isSidechain && !isMeta` 的文本（content 为字符串直接用；为数组取首个 `text` 块），截 80 字符。若首条是 `【SAGE HANDOFF】/【SAGE COLLABORATE】` 内部 prompt，标题只取其中原始任务的首个非空行。再无则 "(无标题)"。
- 转录重建：逐行 JSON；只取 `type ∈ {user, assistant}` 且 `isSidechain!=true && isMeta!=true` 的行。envelope: `{type, message, uuid, parentUuid, timestamp, cwd, sessionId}`。
  - user：`message.content` 字符串→一个 text 块；数组→`text`→text 块、`tool_result`→tool_result 块（content 字符串或嵌套数组的 text，截 400 字符）、`image`→image 块（text 填 "[图片]"）。content 以 `<command-name>`/`<local-command` 开头的行跳过（斜杠命令记录）；以 `<...>` 开头的系统通知、环境上下文，以及 Claude 的上下文压缩续接摘要也跳过。SAGE 内部 prompt 只提取其中原始任务，同一任务在节点/汇总中的后续副本跳过；内部字段不得作为 user 消息返回，但结构化信息写入 `transcript.sage`。**整条消息只有 tool_result 块时 role 仍是 user，前端会并入上一条助手消息展示**，照常返回。
  - assistant：`message.content` 数组：`text`→text、`thinking`→thinking（`thinking` 字段）、`tool_use`→tool_use 块（`name` 填工具名，text 填 `input` 的 JSON 序列化截 400 字符）。`message.model=="<synthetic>"` 的错误行跳过。
  - 大 JSON 行（可能 >1MB）正常 serde_json 解析即可；单行解析失败跳过该行不中断。
- 会话枚举性能：标题需要时才读文件首行；`all_sessions()` 每次调用重扫目录可接受（文件数少），但 history.jsonl 只读一次缓存于 `OnceLock`（mtime 变了重读）。

### 2.2 Codex 历史（实测格式）
- 根：`~/.codex/sessions/YYYY/MM/DD/rollout-<ts>-<uuid>.jsonl` + `~/.codex/archived_sessions/rollout-*.jsonl`（扁平，archived=true）。本机 ~2500 文件。
- 每文件首行 `session_meta`：`{timestamp, type:"session_meta", payload:{id, timestamp, cwd, originator, cli_version, thread_source, ...}}`。`thread_source=="subagent"` 的整个文件跳过。会话 id = `payload.id`（与文件名 uuid 一致）。
- **索引缓存**：`static INDEX: Mutex<HashMap<PathBuf, (SystemTime, IndexEntry)>>`；每次 `all_sessions()` 遍历目录 stat，新文件/变更 mtime 才重读首行。标题惰性：列表阶段可先用首行拿不到的话延迟——为简单起见，索引时顺带向后最多扫 30 行找第一条用户消息作标题（见下），存入 IndexEntry。
- 标题 = 第一条用户输入，来源按优先级：`response_item` 且 `payload.type=="message" && payload.role=="user"` 的 `content[].text`（type `input_text`）；或 `event_msg` 且 `payload.type=="user_message"` 的 `payload.message`。跳过以 `<`、`==` 开头的系统注入文本（如 environment_context）及上下文压缩续接摘要；SAGE 内部 prompt 改取其中原始任务首行。截 80 字符；找不到则 "(无标题)"。
- 转录重建（防御式，逐行；行 `{timestamp, type, payload}`）：
  - `session_meta`→跳过；`turn_context`→跳过（可用其 `model` 更新元信息）。
  - `response_item`：`payload.type=="message"`：role `user`/`assistant` 才收（`developer`/`system` 跳过；user 内容中的 environment/context、系统通知和上下文压缩续接摘要跳过）；SAGE 内部 prompt 只还原一次原始任务，重复节点/汇总正文及重复图片跳过，同时去重记录 `transcript.sage`；`content[]` 中 `input_text`/`output_text`/`text` 的 `text` 拼接为 text 块。`payload.type=="reasoning"`：`summary[]`/`content[]` 里的 text 合为 thinking 块。`function_call`/`custom_tool_call`：tool_use 块（name=`name` 字段，text=arguments 截 400）。`function_call_output`/`custom_tool_call_output`：tool_result 块（output 截 400）。其余 payload.type 跳过。
  - `event_msg`：`user_message`→user text 块；`agent_message`→assistant text 块（`payload.message`）；`item_completed`→按 `item.type`（大小写不敏感 `usermessage`/`agentmessage`/`reasoning`）映射；其余跳过。**注意 response_item 与 event_msg 可能重复表达同一消息**：同 role 且文本完全相同、相邻出现时去重（保留先出现的）。
  - `compacted`→divider 块（text="上下文已压缩"，role "system"）。
- `transcript(session_id)`：从索引反查文件路径（索引未建则先建）。

## 3. backend 契约（backend 实现者）

### 3.1 `src/cli.rs`
```rust
pub struct ResolvedCli { pub exe: std::path::PathBuf, pub version: Option<String> }
pub async fn resolve(agent: &str) -> Option<ResolvedCli>;   // agent: "claude"|"codex"
pub async fn status() -> crate::types::StatusResp;           // 结果缓存 OnceCell，首次调用探测
```
解析算法：`where claude` / `where codex`（Windows；用 `Command::new("where")`）逐行：
1. 行以 `.exe` 结尾 → 直接候选。
2. 行以 `.cmd` 结尾 → 取其目录 D：claude 查 `D/node_modules/@anthropic-ai/claude-code/bin/claude.exe`；codex 依次查 `D/node_modules/@openai/codex/node_modules/@openai/codex-win32-x64/vendor/x86_64-pc-windows-msvc/bin/codex.exe`、`D/node_modules/@openai/codex/bin/*.exe`（glob 任意 codex*.exe）。
3. 多候选取第一个存在的；对候选跑 `<exe> --version`（3s 超时）成功者胜出并记录版本。
4. 全失败 → `installed:false`，error 写明「未找到可执行文件」。

### 3.2 `src/run.rs` — POST /api/chat 的流式实现
- `pub async fn stream_chat(req: ChatReq) -> axum::response::Response`：校验 agent/project（目录须存在），resolve CLI 失败→ 一行 `{"t":"done","ok":false,"error":"..."}`。
- spawn：`kill_on_drop(true)`、`current_dir(project)`、stdin=piped 写入 prompt 后 drop、stdout/stderr piped。
- **claude 新会话** argv：`-p --output-format stream-json --verbose --include-partial-messages` + `--model <m>`（有值时）+ 权限映射（`bypass`→`--permission-mode bypassPermissions`、`accept-edits`→`acceptEdits`、`plan`→`plan`、其余省略）。**resume**：追加 `--resume <session_id>`。
- **codex 新会话** argv：`exec --json --skip-git-repo-check -C <project>` + `-m <m>`（有值）+ 权限（`bypass`→`--dangerously-bypass-approvals-and-sandbox`、`read-only`→`-s read-only`、其余→`-s workspace-write`）+ 位置参数 `-`（stdin 读 prompt）。**resume**：`exec resume <session_id> --json --skip-git-repo-check [--dangerously-bypass-approvals-and-sandbox] -`（无 `-C`/`-s`）。
- 响应：`Content-Type: application/x-ndjson`，Body 为 async_stream 产生的行流。子进程 stdout 逐行读，映射为统一事件（§3.3）；stderr 逐行→`{"t":"stderr","text":...}`（tokio::select 合流或双任务 + mpsc）。进程退出→`{"t":"done","ok":exit_ok,"session_id":...,"error":...}`。
- 客户端断开 = body stream drop = 子进程被杀（kill_on_drop）。

### 3.3 统一 NDJSON 事件（后端→前端，每行一个 JSON）
```
{"t":"init","agent":"claude","session_id":"..."}        // 拿到会话 id 即发
{"t":"delta","channel":"text"|"thinking","text":"..."}  // 流式增量
{"t":"text","text":"..."}                                // 整块助手文本（无增量路径时）
{"t":"thinking","text":"..."}                            // 整块思考
{"t":"tool_use","name":"Bash","text":"<输入摘要≤400字>"}
{"t":"tool_result","text":"<输出摘要≤400字>"}
{"t":"status","text":"..."}                              // 如 codex task_started
{"t":"stderr","text":"..."}
{"t":"done","ok":true,"session_id":"...","error":null}
```
**claude stdout 映射**（每行一个 JSON 事件，字段 `type`）：
- `system`+`subtype=="init"`：`session_id` 字段 → init 事件。
- `stream_event`：`event.type=="content_block_delta"` 时 `event.delta.type` `text_delta`→delta(text)、`thinking_delta`→delta(thinking)；其余 stream_event 忽略。
- `assistant`：`message.content[]` 中 `tool_use`→tool_use 事件（input JSON 序列化截 400）；`text`/`thinking` 块**仅当本会话尚未发过任何 delta** 时才发 text/thinking 事件（防重复渲染）。
- `user`：content 里 `tool_result`→tool_result 事件（摘要截 400）。
- `result`：`is_error`、`session_id` 记录 → 最终 done 事件里带上。
- 解析失败/未知 type：忽略该行。
**codex stdout 映射**（防御式；0.148 `--json` 事件为 `{"type":...}`，可能嵌 `thread_id`/`item`/`msg`）：
- 任意行发现 `thread_id` 或 `session_id`（顶层或 payload/msg 内）且未发过 init → init 事件。
- `type` 含 `thread.started`→init；`turn.started`/`task_started`→status("运行中")。
- item 类（`item.completed`/`item.updated`/`item.started`，item 对象在 `item` 字段）：`item.type`（大小写不敏感）`agent_message`→text（字段 `text` 或 `content[].text`；`item.updated` 可作 delta 发，需前端幂等：**约定 codex 的 agent_message 只在 completed 时发 text 一次**，updated 忽略）、`reasoning`→thinking、`command_execution`→tool_use(name="shell", text=command 字段)、`file_change`/`patch_apply`→tool_use(name="apply_patch")、`mcp_tool_call`→tool_use、`web_search`→tool_use(name="web_search")。
- 旧式 `event_msg` 形状（直接 `{"type":"agent_message","message":...}`）同样映射。
- `turn.completed`→无事件；`error`/`turn.failed`→记录 error 文本进 done。
- 未知行忽略。
- **注**：真实事件名以集成期抓到的样本（docs/samples/）为准，集成者会校准；实现时把「事件名→处理」写成小函数便于调整。

### 3.4 `src/api.rs` + `src/config.rs`
- `pub struct AppState { ... }`（含 config 的 RwLock）；`AppState::new() -> Self`。main.rs 以 `Arc<AppState>` 注入。
- `config.rs`：`~/.agenthub/config.json`（dirs::home_dir）`{ "projects": ["D:\\path", ...] }`，读失败视为空，写入前建目录。
- handlers（签名与 main.rs 路由匹配，State 提取器）：
  - `status`：转发 `cli::status()`。
  - `projects`：`spawn_blocking` 收集 `history::claude::all_sessions()` + `history::codex::all_sessions()`，按 `project`（已规范化）分组统计成 `Vec<ProjectInfo>`（name=末级目录名，exists=目录存在，last_active=两侧最大 updated），并集手动 pinned 项目（pinned=true），按 last_active 降序。
  - `add_project`：body `{"path": "..."}`；目录不存在→400 JSON `{"error":...}`；成功写 config 返回新列表。
  - `pick_folder`：POST /api/pick-folder；Windows 上起独立 STA 线程弹系统「选择文件夹」对话框（`dialog::pick_folder`，IFileOpenDialog 裸 COM，无第三方依赖），返回 `{"path":"D:\\..."}`，用户取消返回 `{"path":null}`；已有窗口未关时→409；非 Windows→501。
  - `sessions`：query `project`（可选，规范化后精确匹配）、`q`（可选，标题子串不区分大小写）、`limit`（默认 200）；两侧合并按 updated 降序。
  - `session_detail`：query `agent`,`id`,`project`（claude 必传 project）；调对应 transcript；Err→404 JSON。
  - `chat`：`Json<ChatReq>` → `run::stream_chat`。
  - 静态：`index_html`/`app_js`/`style_css` 用 `include_str!("../static/…")`，正确 Content-Type（html/js/css 均 `; charset=utf-8`）。
- 所有 handler 内部错误不 panic，返回 JSON 错误体。

## 4. frontend 契约（frontend 实现者）

纯原生三文件：`static/index.html`、`static/style.css`、`static/app.js`（无 CDN、无框架、无外部字体，中文 UI）。参考截图为 Tutti 风格暗色界面。

### 4.1 布局
- 左侧栏 300px（`#1e1e1e`，右边框 `#2a2a2a`）：
  - 顶部：搜索输入框「搜索会话」（输入即过滤，调 `/api/sessions?q=`）+ 主按钮「新建会话」。
  - 「项目」分组：`/api/projects` 渲染；每行文件夹图标 + 项目名 + 灰色小字会话总数；点击展开该项目会话列表（`/api/sessions?project=`），会话行 = agent 圆点（claude 橙 `#e8734a` ✳ / codex 蓝 `#4a90e8`）+ 标题（单行省略）+ 相对时间（如 "3 天前"）。分组标题右侧「＋」按钮：prompt() 输入路径 POST /api/projects。
  - 「对话」分组：`/api/sessions?limit=30` 全局最近会话。
  - 底部状态条：`/api/status` → 两行「● Claude Code 2.1.237」「● Codex 0.148.0」，绿点=已安装，红点+「未安装」。
- 主区两个互斥视图：
  - **Hero（新会话）**：垂直居中。标题「需要 <em class=serif>Claude Code</em> ⌄ 帮你做些什么？」，em 部分衬线斜体（Georgia, serif），点击弹出下拉切 Claude Code / Codex（标题文本随之变）。下方输入卡片（max-width 760px，圆角 14px，`#262626`，边框 `#333`）：textarea（占位「输入你的任务…」，Enter 发送 / Shift+Enter 换行，自动增高）+ 底部工具条：左侧 agent 徽标；右侧 权限下拉（默认「绕过权限」，橙色文字 `#e8964a`；选项：绕过权限/接受编辑/默认/计划[claude]、绕过权限/工作区可写/只读[codex]）、模型下拉（默认「默认模型」；claude: fable/opus/sonnet/haiku；codex: 默认/自定义→prompt()）、圆形发送按钮（橙）。卡片下一行：项目下拉（当前项目名，来自 /api/projects，含「浏览…」=prompt 输入路径）+「🖥 本地」灰徽标；右侧灰色 Tips 文字。
  - **对话视图**：头部（会话标题、agent 徽标、项目路径灰字、「新建会话」快捷钮）；消息滚动区（max-width 820px 居中）：用户消息右对齐深色气泡 `#2b2b2b` 圆角 12px；助手消息左对齐无气泡正文；thinking 块=折叠卡片（默认收起，标题「💭 思考过程」，dim 斜体正文）；tool_use=折叠行（⚙ 工具名 + 摘要单行省略，点击展开 `<pre>`）；tool_result 并入上一个 tool_use 的展开区；divider=居中灰色分隔说明。底部输入卡片同 Hero（复用组件），发送即**继续当前会话**（带 session_id + agent + project）。
- 加载态骨架 / 空态文案「暂无对话」。

### 4.2 交互协议
- 发送：`fetch('/api/chat', {method:'POST', body: JSON.stringify(ChatReq)})`，`resp.body.getReader()` 按行拆 NDJSON（注意 chunk 跨行缓冲）。事件处理：`init`→记录 session_id（新会话首次响应后把会话加入侧栏）；`delta`→向当前流式块追加（channel 区分 text/thinking）；`text`/`thinking`/`tool_use`/`tool_result`/`status` 各自建块；`done`→结束态。流式期间「停止」按钮先从 `/api/runs` 按 current/partner/workflow 解析全部目标，再逐个 POST `/api/stop`，随后 abort viewer 并等待运行表确认；Windows 后端以精确 PID 的 `taskkill /T /F` 终止 CLI 进程树，不依赖继承管道 EOF。UI 必须显示“正在停止→■ 已停止/失败”，停止不归类为运行错误。
- 历史打开：侧栏点会话 → `/api/session` → 渲染转录 → 底部输入框 placeholder「继续这个会话…」。连续 tool_result-only 的 user 消息并入前一条助手消息区。会话列表先依据 `SessionSummary.sage` 建立 exact/受限 legacy links 并过滤 target，不要求用户先打开来源会话；同 session 最新非空 run prompt 覆盖临时“无标题”。协作面板再依据 `transcript.sage` 补扫缺失 links，显示当前任务子会话数、全局运行数、executor 与复用节点。现有主会话触发 HANDOFF 时，target 保留完整所有权但以“移交接管”嵌套到来源会话、从普通侧栏隐藏、提供“来源会话”入口且不自动回注。
- **安全**：一切动态文本经 `textContent` / 自建 escapeHtml 插入，绝不 innerHTML 拼接未转义内容。助手文本做极简 markdown：``` 围栏 → `<pre><code>`，行内 `` ` `` → `<code>`，其余纯文本保留换行（white-space: pre-wrap）。
- 无路由库：hash 记忆当前视图可选，不强制。

### 4.3 视觉
- 字体：`system-ui, "Segoe UI", "Microsoft YaHei", sans-serif`；正文 14px，行高 1.6；背景 `#161616`，正文 `#d4d4d4`，次要 `#8a8a8a`。
- 按钮/下拉均深色扁平（hover `#2e2e2e`），主按钮橙底 `#d97848` 白字圆角 8px。
- 滚动条细样式（`::-webkit-scrollbar` 6px 深色）。
- 顶部窗口标题栏区域（截图中的 macOS 圆点）不做。

## 5. 端口与启动

`127.0.0.1:8721`（env `AGENT_HUB_PORT` 覆盖）。`cargo run` 即可；main.rs 已固定。

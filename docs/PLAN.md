# Agent Hub — 本地 CLI Agent Web 界面 技术方案

## 1. 目标

在浏览器中提供一个类似截图参考（Tutti 风格）的暗色 Web 界面，驱动**本地已安装的 CLI**（Claude Code 与 Codex）完成对话式编码任务。核心能力：

1. **CLI 检测**：启动时检测本机是否安装 Claude Code / Codex CLI，展示版本与可执行路径；未安装时在界面明确提示。
2. **模型能力完全来自本地 CLI**：后端不直连任何模型 API，只 spawn 本地 `claude.exe` / `codex.exe` 子进程，解析其流式 JSON 输出。
3. **多项目**：自动从两个 CLI 的历史存储中发现历史项目（真实 cwd），也支持手动添加任意目录；新会话在所选项目目录下运行。
4. **历史会话**：按项目列出 Claude Code（`~/.claude/projects/**`）与 Codex（`~/.codex/sessions/**`）的历史会话，可查看完整转录（用户/助手/思考/工具调用）。
5. **继续历史对话**：在历史转录底部继续输入，后端用 `claude -p --resume <id>` / `codex exec resume <id>` 恢复该会话并流式返回。

## 2. 技术选型

| 层 | 选型 | 理由 |
|---|---|---|
| 后端 | Rust + axum 0.8 + tokio | 用户指定 Rust；axum 生态成熟，天然支持流式响应 |
| 进程管理 | tokio::process，`kill_on_drop` | 客户端断开即杀死子进程，取消即停 |
| 流式协议 | POST `/api/chat` 返回 NDJSON 流（chunked） | 比 WebSocket 简单，fetch ReadableStream 直接消费 |
| 前端 | 原生 HTML/CSS/JS（无框架），`include_str!` 编译期内嵌 | 单二进制交付，零 Node 依赖 |
| 存储 | 无数据库；历史直接读两家 CLI 的 JSONL；自身配置存 `~/.agenthub/config.json` | 历史的唯一权威数据源就是 CLI 自己的文件 |

已探明的环境（2026-08-21 实测）：

- Claude Code 2.1.237，真实二进制 `C:\nvm4w\nodejs\node_modules\@anthropic-ai\claude-code\bin\claude.exe`
- Codex CLI 0.148.0，真实二进制 `C:\nvm4w\nodejs\node_modules\@openai\codex\node_modules\@openai\codex-win32-x64\vendor\x86_64-pc-windows-msvc\bin\codex.exe`
- Rust 1.98.0（MSVC 工具链，VS Build Tools 18 已装）

## 3. 架构

```
浏览器 (静态 SPA, 内嵌)
   │  REST + NDJSON 流
   ▼
axum 后端 (127.0.0.1:8721)
   ├─ cli.rs      CLI 探测与可执行解析（where + npm shim → 真实 .exe）
   ├─ run.rs      spawn 子进程，stdout JSONL → 统一事件流
   │    ├─ claude 适配：claude -p --output-format stream-json --verbose
   │    │            --include-partial-messages [--resume id]，prompt 走 stdin
   │    └─ codex 适配：codex exec --json --skip-git-repo-check [-C dir]
   │                  / codex exec resume <id> --json，prompt 走 stdin("-")
   ├─ history/    历史读取
   │    ├─ claude.rs  ~/.claude/projects/<编码目录>/*.jsonl + ~/.claude/history.jsonl(标题)
   │    └─ codex.rs   ~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl（首行 session_meta 索引，mtime 增量缓存）
   └─ api.rs      REST：/api/status /api/projects /api/sessions /api/session /api/chat
```

## 4. 关键实现事实（探查结论，实现必须遵守）

### 4.1 Windows 进程启动
- `claude`/`codex` 在 PATH 上是 `.cmd` npm shim；Rust `CreateProcessW` 不能直接执行 `.cmd`，且经 `cmd /C` 转发用户 prompt 有元字符注入风险。
- **方案**：解析出真实 `.exe` 直接 spawn；prompt 一律通过 **stdin** 传入（claude `-p` 无位置参数时读 stdin；codex 位置参数传 `-` 读 stdin），argv 中只有固定 flag。
- 工作目录：claude 无 `--cwd` 参数，会话按进程 cwd 归档，因此 `Command::current_dir(project)`；resume 必须用同一 cwd。codex 新会话用 `-C <dir>`，resume 无 `-C`（复用录制的 cwd），但仍设置 current_dir 兜底。

### 4.2 Claude Code 历史
- 目录编码：真实路径中 `[^A-Za-z0-9]` 全部替换为 `-`（有损，不可逆），如 `D:\project` → `D--project`。
- 真实路径恢复：读目录内任一 jsonl 行的 `cwd` 字段，或用 `~/.claude/history.jsonl`（`{display, project, sessionId, timestamp}`）反查。
- 会话 = 项目目录下**顶层** `*.jsonl`（文件名即 session uuid）；忽略 `memory/`、`<uuid>/` 等子目录。
- 转录行类型：`user`/`assistant`/`system`/`attachment` + 记账行（`file-history-snapshot` 等，跳过）；过滤 `isSidechain=true`、`isMeta=true`。`message.content` 为字符串或块数组（`text`/`thinking`/`tool_use`/`tool_result`/`image`）。
- 标题：history.jsonl 中该 sessionId 最早的非 `/` 开头 `display`；否则第一条用户文本。

### 4.3 Codex 历史
- `~/.codex/sessions/YYYY/MM/DD/rollout-<本地时间>-<uuid>.jsonl` + `~/.codex/archived_sessions/`（扁平）。本机约 2500 个文件。
- 首行 `session_meta`：`payload.id`（会话 id）、`payload.cwd`（项目）、`timestamp`、`thread_source`（`subagent` 需过滤）。
- cwd 规范化：剥 `\\?\` 前缀、盘符大小写、`\`/`/` 统一（还有 `/mnt/d/...` WSL 变体，按需归并）。
- 性能：只读每文件首行建索引，`(path, mtime)` 增量缓存，避免每次全量扫描。
- 转录：`response_item`（`payload.type=message`，`role`+`content[].text`）为主，兼容旧式 `event_msg`（`user_message`/`agent_message`）与新式 `item_completed`；`compacted` 行渲染为「上下文已压缩」分隔。

### 4.4 流式对话
- claude：`stream_event`(text/thinking delta) → 增量；`assistant` 事件中 `tool_use` → 工具卡片；`system/init` → session_id；`result` → 结束。若无 delta 事件自动退化为整块文本。
- codex：`thread.started` → thread_id；`item.*`（agent_message/reasoning/command_execution/file_change）→ 文本/思考/工具事件；`turn.completed`/`error` → 结束。**解析必须防御式**（版本间事件命名有差异），集成阶段用一次真实小 prompt 校准解析器。
- 权限映射：UI「绕过权限/接受编辑/默认/计划」→ claude `--permission-mode`；「绕过权限/工作区可写/只读」→ codex `--dangerously-bypass-approvals-and-sandbox` / `-s workspace-write` / `-s read-only`。

## 5. API 设计（详见 docs/CONTRACT.md 与 src/types.rs）

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/status` | 双 CLI 安装状态、版本、路径 |
| GET | `/api/projects` | 历史发现 + 手动固定的项目列表（含两侧会话数、最近活跃） |
| POST | `/api/projects` | 手动添加项目目录 |
| GET | `/api/sessions?project=&q=` | 某项目（或全部）的会话列表，支持搜索 |
| GET | `/api/session?agent=&id=&project=` | 单会话完整转录（统一块模型） |
| POST | `/api/chat` | 发起/继续对话，返回 NDJSON 事件流 |

## 6. 前端界面（对照参考截图）

- **暗色主题**：背景 `#161616`，侧栏 `#1e1e1e`，卡片 `#262626`，主色橙（Claude ✳）/蓝（Codex）。
- **左侧栏**：顶部「搜索会话」+「新建会话」；「项目」分组 — 可展开的项目行（文件夹图标 + 名称 + 会话数），展开显示该项目会话（按 agent 着色图标）；「对话」分组 — 跨项目最近会话；底部 CLI 状态（两个圆点 + 版本号）。
- **主区 Hero**（新会话态）：居中标题「需要 *Claude Code* ⌄ 帮你做些什么？」（下拉切换 Claude Code / Codex，衬线斜体），下方输入卡片：多行输入框 + 工具条（agent、权限[橙色]、模型下拉、发送按钮），再下方项目选择器 +「本地」徽标。
- **对话视图**：头部（标题 + agent 徽标 + 项目路径），消息流（用户右侧深色气泡、助手左侧正文、思考/工具调用折叠卡片、流式光标），底部固定输入框可**继续该会话**。
- 所有历史内容 HTML 转义，防 XSS；代码围栏渲染为 `<pre>`。

## 7. 目录结构与分工

```
agent-hub/
├─ Cargo.toml
├─ docs/{PLAN.md, CONTRACT.md, samples/}
├─ src/
│  ├─ main.rs        # 路由与启动（脚手架固定）
│  ├─ types.rs       # 全部 serde 契约类型（脚手架固定）
│  ├─ api.rs cli.rs run.rs config.rs   # 后端实现
│  └─ history/{mod.rs, claude.rs, codex.rs}  # 历史读取实现
└─ static/{index.html, style.css, app.js}    # 前端实现
```

实现采用多智能体并行：历史模块 / 后端核心 / 前端三路并行（文件所有权互斥），随后集成构建 + 真实 CLI 小成本校准 + 双维度审查（正确性、安全/XSS）+ 修复。

## 8. 风险与缓解

| 风险 | 缓解 |
|---|---|
| `.cmd` shim 无法被 CreateProcessW 执行 / argv 注入 | 解析真实 .exe + prompt 走 stdin（已定为硬性规则） |
| 流式 JSON 事件格式随 CLI 版本漂移 | 防御式解析 + 未识别事件降级为原始行忽略 + 集成期真实样本校准（样本存 docs/samples/） |
| Codex 2500+ 会话文件扫描慢 | 首行索引 + (path, mtime) 增量缓存 |
| 历史转录含恶意 HTML | 前端全量转义，只允许自建 DOM 结构 |
| `-p` 模式默认权限会拒绝工具调用 | UI 默认「绕过权限」（与截图一致），可切换 |
| 端口冲突 | 默认 8721，`AGENT_HUB_PORT` 环境变量可改 |

## 9. 验证计划

1. `cargo build` 零错误；
2. 启动后 `/api/status` 返回两 CLI 已安装 + 正确版本；
3. `/api/projects` 能列出 `D:\project`、`D:\project\demo_app` 等真实历史项目；
4. `/api/session` 能正确渲染一条真实 Claude 会话与一条真实 Codex 会话转录;
5. `/api/chat` 各发一条最小 prompt（claude 用 haiku，成本极低）验证流式与 session_id 捕获；
6. 浏览器打开 `http://127.0.0.1:8721` 核对界面与截图风格一致、历史可浏览、可继续对话。

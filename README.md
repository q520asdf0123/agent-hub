# Agent Hub

本地 CLI Agent（**Claude Code** / **Codex**）的 Web 界面。单个 Rust 二进制，零 Node 依赖，模型能力完全来自你本机已登录的 CLI —— 本项目不接任何模型 API，只是给两个 CLI 一个现代化的图形界面。

## 功能

- **双 CLI 检测与流式对话**：自动探测本机 `claude` / `codex` 安装（解析 npm shim 背后的真实 `.exe`），以无头模式驱动，NDJSON 流式渲染文本 / 思考 / 工具调用
- **历史会话浏览与续聊**：直接读取两家 CLI 的原生存储（`~/.claude/projects/`、`~/.codex/sessions/`），可查看任意历史转录并 `resume` 继续对话；与终端里的会话完全互通
- **多项目管理**：从历史自动发现项目，自选导入侧栏；按 agent 过滤（全部 / Claude / Codex）
- **后台任务**：刷新、关页面不中断运行中的任务，重开自动重连续看输出；只有停止按钮才会终止；侧栏实时显示 运行中 ● / 完成 ✓ / 出错 ✕
- **SAGE 智能路由**：集成 [sprix-sage-router](https://github.com/wang2122/sprix-sage-router)，按任务需求自动在两个 CLI 间选择执行者，决策卡展示理由
- **技能 / 命令面板**：输入 `/` 唤起，聚合 Claude 技能（用户 / 项目 / 插件）、Codex 技能与自定义 prompt、内置命令（`/review`、`/init`、`/diff`、`/status`、`/fork`…），支持键盘导航，不同技能自动配色
- **图片全链路**：输入框粘贴截图发给 CLI，历史图片内联展示，点击灯箱缩放
- **已编辑文件卡片**：每轮任务汇总改动文件（默认折叠），点击进入 GitHub 风格差异审查（行号 / 整行着色 / 支持嵌套 git 仓库与已提交改动回溯），右键可在 VS Code / 资源管理器中打开
- **文件引用可点击**：转录里的路径 / markdown 链接直接打开，代码文件带行号跳转（`file.rs:100` → VS Code 定位到行）
- **完整 Markdown 渲染 + HTML 沙箱预览**：标题 / 列表 / 表格 / 引用实时渲染；`html` 代码块一键在沙箱 iframe 中预览
- **明亮 / 黑暗双主题**，全部选择（模型、思考等级、权限、项目等）本地记忆
- **模型与思考等级自动发现**：从 CLI 配置与历史中读取实际可用的模型清单和推理档位，非硬编码

## 环境要求

| 依赖 | 说明 |
|---|---|
| Windows 10/11 | 当前实现面向 Windows（进程解析 / 路径处理） |
| Rust stable | 构建用，`rustup` 安装即可 |
| [Claude Code CLI](https://claude.com/claude-code) | ≥ 2.x，已登录 |
| [Codex CLI](https://github.com/openai/codex) | ≥ 0.148，已登录 |
| Python 3.10+（可选） | SAGE 智能路由需要，缺失时该功能自动禁用 |
| git（可选） | 差异审查 / 改动统计需要 |
| VS Code（可选） | 文件点击跳转编辑器；未安装时退回系统默认程序 |

两个 CLI 至少装一个即可用，界面会显示各自的安装状态。

## 快速开始

```bash
git clone https://github.com/q520asdf0123/agent-hub.git
cd agent-hub
cargo run
```

打开 http://127.0.0.1:8721 （端口可用环境变量 `AGENT_HUB_PORT` 修改）。

## 工作原理

```
浏览器（内嵌静态 SPA）
   │ REST + NDJSON 流
   ▼
axum 后端（127.0.0.1:8721）
   ├─ 运行注册表：任务与 HTTP 连接解耦，断开不杀进程，可重连/显式停止
   ├─ claude.exe -p --output-format stream-json …（prompt 走 stdin，resume 用 --resume）
   ├─ codex.exe exec --json …（resume 用 exec resume / fork 用 exec fork）
   └─ 历史读取：解析两家 CLI 的 JSONL 会话存储（带 mtime 增量索引与 TTL 缓存）
```

关键设计：

- **spawn 真实 `.exe` 而非 `.cmd` shim**（Windows `CreateProcessW` 无法执行 .cmd，且经 cmd 转发用户输入有注入风险）；prompt 一律通过 stdin 传递
- 会话按 CLI 原生格式落盘，因此**与终端使用完全互通**：网页里开的会话可在终端 `claude --resume` 继续，反之亦然
- 对 `~/.claude`、`~/.codex` **只读**，不写入不迁移

## 目录结构

```
src/
├─ main.rs        # 路由与启动
├─ api.rs         # REST handlers（会话/项目/技能/模型/差异/打开文件…）
├─ run.rs         # 后台运行注册表 + CLI 流式事件映射
├─ cli.rs         # CLI 探测（shim → 真实 exe 解析）
├─ history/       # claude / codex 原生会话存储解析
├─ models.rs      # 模型与思考等级自动发现
├─ skills.rs      # 技能 / 命令扫描
└─ sage.rs        # SAGE 路由桥接（Python 子进程）
static/           # 前端（原生 JS，编译期内嵌进二进制）
vendor/sprix-sage-router/   # SAGE 算法库（MIT，含 LICENSE）
docs/             # 技术方案（PLAN.md）与模块契约（CONTRACT.md）
```

## 配置

- `~/.agenthub/config.json`：已导入的项目列表（界面操作自动维护）
- 技能来源：`~/.claude/skills/`、项目 `.claude/skills/`、已安装插件、`~/.codex/skills/`、`~/.codex/prompts/`
- SAGE 能力画像：`vendor/sprix-sage-router/sage_bridge.py` 的 `DEFAULT_PROFILES`，可按偏好调整

## 安全说明

- 服务只绑定 `127.0.0.1`，不对外网开放
- 所有转录内容以纯文本渲染（防 XSS）；HTML 预览运行在无同源权限的沙箱 iframe 中
- 无遥测、无外部请求；模型调用、凭证、配额全部走本机 CLI 自身

## 致谢

- 路由算法来自 [Sprix AI 的 sprix-sage-router](https://github.com/wang2122/sprix-sage-router)（MIT，已 vendor 并附原始 LICENSE）
- 界面风格参考 Codex Desktop / Tutti

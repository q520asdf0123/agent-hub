# Agent Hub 的 SAGE 实现合规审查与实施后复核

> 审查日期：2026-08-25  
> 审查对象：当前工作区中的 Agent Hub 集成，以及 Sprix SAGE Router 上游 `main`  
> 上游锁定版本：[`aed97852abc0bbd1dfccd8851b31290bc1b3f507`](https://github.com/wang2122/sprix-sage-router/commit/aed97852abc0bbd1dfccd8851b31290bc1b3f507)，官方版本元数据为 `0.2.0`（[`CITATION.cff`](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/CITATION.cff#L1-L11)、[`pyproject.toml`](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/pyproject.toml#L1-L12)）

## 结论

**整改前结论：部分符合。实施后结论：本地双 Agent 的官方 v0.2 参考语义已基本符合；生产级 A2A roadmap 仍未宣称完成。**

需要分成两层看：

1. **算法核心符合。** 本地 [`sprix_sage.py`](../vendor/sprix-sage-router/sprix_sage.py) 与审查时上游 `main` 的同名文件逐字节一致：两者均为 `31,895` bytes，SHA-256 均为 `AF67236F08A7081F0AE5B55EFD50E191E6B14FE61630B7E8F6CAF6FAC82E8276`。因此 `SELF / COLLABORATE / HANDOFF` 候选比较、受约束效用、需求分配、DAG 调度、beam search 和在线更新算法本身就是官方 v0.2 实现。
2. **Agent Hub 的本地双 Agent 适配已按审查结果整改。** `COLLABORATE` 保持 incumbent 所有权；执行器消费 assignments/dependencies/topology，按 requirement DAG 分波次运行；跨 Agent 同波并行、同 Agent 串行；结果只归因给实际执行的 Agent/需求。API 也接受完整 ExecutionState 和 Task constraints，并由服务端过滤未安装 CLI。

所以，当前功能可以准确描述为：**“基于官方 SAGE v0.2 核心、忠实执行本地 Claude/Codex assignments 与 requirement DAG 的 state-aware 双 Agent 路由器”**。它仍不是开放网络上的生产级 A2A 市场实现。

## 实施后复核（2026-08-25）

已完成：

- `COLLABORATE` 的 `primary` 固定为 incumbent；peer 不再夺取所有权。
- `/api/sage` 接受 `state`/`constraints`，服务端根据真实 CLI 探测覆盖 `available_agents`；桥接支持 active agents/mode、completed requirements、progress、transferable context、失败状态、权限、预算与 deadline。
- 桥接返回 assignments、dependencies、topology、switch recommendation 及 cost/latency/risk/utility 审计字段。
- 前端按 requirement DAG 分波次执行：跨 Agent 同波并行、同 Agent 串行、依赖失败阻断下游；每个 Agent 只收到自己名下需求。
- 删除 `<0.25 → 只读复查` 和旧固定串行流水线；最终结果由 incumbent 会话汇总。
- 新任务和现有会话追问都可执行 `SELF/COLLABORATE/HANDOFF`；HANDOFF 创建 peer 所有权会话。
- outcome 只对实际执行的 Agent/需求评分；团队关键路径 latency 不再污染成员下次 Bid，未知真实 provider cost 时不再用 Token 伪造。
- 旧学习状态先备份为 `sage_state.json.pre-official-v2`，再按 schema v2 重建，避免旧语义污染新决策。
- 决策卡展示分工、拓扑和约束效用指标；手动团队模式继续与 SAGE 互斥。
- 候选池已从固定 `claude/codex` 升级为本机发现的 `runtime::model` executor；任务复杂度只决定 beam-search 团队上限，实际人数继续由官方效用函数选择。
- executor 决策携带 runtime/model/role；同一 CLI 的不同模型使用独立会话并可并行，outcome 按模型 executor 维度学习。简单基准保持 1 人，复杂基准可自动组成多模型团队。
- reasoning effort 已自动化：整体复杂度先调整 executor 的能力/成本/延迟先验，路由后再按 requirement 和模型支持范围返回 efforts；简单节点从 low 起步，很复杂核心节点可提高一档，GPT-5.6 Sol 的自动 effort 固定封顶为 `xhigh`，Luna/mini/Spark 等低成本模型在支持时可到 `max`，Bid 统计按模型与 effort 分桶。
- 所有 Codex executor 都携带 `fast=true`，前端每条 SAGE stage 请求与 Rust 后端所有 Codex CLI 分支均强制使用 `service_tier="fast"`；这属于 Agent Hub 的执行策略，不是 SAGE 上游路由算法规则。
- Claude/Codex 原生历史会把 SAGE stage prompt、系统通知或上下文压缩续接摘要记录为 user；历史适配层现会统一识别这些注入，只恢复一次 SAGE 真实原始任务，过滤内部所有权、分工、拓扑、指令、通知和续接摘要，同时用原始任务生成标题。
- 历史适配层同时保留不可见的 SAGE workflow metadata；前端不依赖标题或 user 正文恢复关联，可在无 localStorage 时找回全部 executor 子会话及节点。运行态只回显可见用户 prompt、隐藏成功任务的无害 WARN，并把其他主任务运行 session 与当前任务子会话分区展示。
- 执行调度集中到可单测的 `SageScheduler.executeWave()`：同一 executor 的 requirement 串行，不同 executor 的独立 requirement 并行；review 与其他 requirement 一样完全服从上游 assignments，不强制创建额外 agent 或 session。
- 停止操作按 SAGE workflow metadata 扩展到所有 owner/partner runs；Windows 精确终止每个 CLI 进程树并在运行表确认完成，前端区分用户停止与仅断开查看，停止状态不冒充失败。
- HANDOFF target 继续拥有完整任务所有权，但 adapter 会记录来源 agent/session 谱系：target 从来源会话的右侧面板进入、普通侧栏不重复平铺，并显示“来源会话”返回入口；该导航关系不改变 SAGE ownership，也不执行 COLLABORATE 回注。
- `SessionSummary.sage` 把 lineage 提升到列表层：首次加载即可隐藏 exact/受限 legacy target 并建立入口，不依赖先打开来源 transcript；运行索引另保留同 session 最近非空用户 prompt，避免 owner 初始显示“无标题”。

仍属明确边界：

- 签名 Agent Cards、开放网络 discovery、真实 A2A transport 与 provider live bids 是上游 roadmap，本地双 CLI 应用不伪装为已实现。
- 当前没有独立质量 evaluator；执行节点的成功证据仍以 CLI 正常完成为基础，但已提供正确的按 Agent/需求归因接口。
- UI 暂无用户预算/deadline 输入；未提供时桥接使用“无该约束”，而非伪造固定 SLA。

下方逐项审查表保留为整改前基线，便于追溯本轮修复来源。

## 判定口径

- **符合**：当前实际行为实现了上游对应规则。
- **部分符合**：调用了对应机制，但输入、适用范围或执行语义不完整。
- **不符合/缺失**：上游要求的关键语义没有进入实际决策或没有按决策执行。

上游把 SAGE 定义为发现与执行之间的策略层，并明确说参考实现只返回决策、不负责传输任务；因此“是否忠实执行决策”应审查 Agent Hub 的 client/编排层，而不能只看 vendored 核心。[官方定位与三模式](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/README.md#L19-L27)；[官方 A2A 集成边界](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/README.md#L112-L123)

## 整改前逐项审查（历史基线）

| 项目 | 判定 | 当前实现与差距 |
|---|---|---|
| 上游版本与路由核心 | **符合** | vendored `sprix_sage.py` 与上游当前 commit 逐字节一致。Rust 层只是把脚本内嵌并通过 JSON/stdin 调用，不重写算法。[`src/sage.rs`](../src/sage.rs#L1-L88) |
| 三模式共同评分 | **符合（核心）** | `SAGERouter.route()` 实际比较 `SELF`、可行的 `HANDOFF` 与 beam-search `COLLABORATE`，不是前端人工 `if/else` 伪造模式。[官方源码](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/sprix_sage.py#L315-L353)；[桥接调用](../vendor/sprix-sage-router/sage_bridge.py#L180-L210) |
| `SELF` | **符合（新任务）** | 新会话由当前 incumbent 独立执行，符合 `SELF`。但现有普通会话不会持续进行完整的 SAGE 重规划，因此不是官方所说的 mid-execution `SELF` 判断。[前端首轮路由](../static/app.js#L4210-L4256) |
| `HANDOFF` | **部分符合** | 新任务返回 `HANDOFF` 时会在创建会话前切换到 peer，基本符合“peer 接管全部所有权”。但普通进行中会话没有带真实 live state 的通用 handoff；协作会话的追问仅按 `primary` 是否等于搭档来分派，没有按返回 mode 完整执行。[首轮切换](../static/app.js#L4225-L4250)；[追问分派](../static/app.js#L4258-L4293) |
| `COLLABORATE` 所有权 | **不符合** | 官方规定 incumbent 保留所有权；桥接层却把“最高权重需求的 assignee”设为 `primary`，它可能是 peer，前端随后真的切换到该 peer。也就是一次 `COLLABORATE` 可被执行成“peer 成为主会话，原 incumbent 成为搭档”。[官方规则](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md#L1-L8)；[错误映射位置](../vendor/sprix-sage-router/sage_bridge.py#L201-L210) |
| 需求分配 `assignments` | **不符合（执行层）** | SAGE 确实生成每项需求的 assignee，但主会话先收到并执行**完整原任务**，没有被限制为只做自己名下需求；主会话结束后，搭档才收到自己的需求。因此会重复工作，也没有做到“每个剩余需求由其 assignee 执行”。[主任务发送](../static/app.js#L4337-L4379)；[搭档分工 prompt](../static/app.js#L3886-L3929) |
| 通信拓扑与 DAG 调度 | **不符合/未执行** | `topology` 被放进 `decision_blob`，但 `static/app.js` 完全不读取 `topology`。当前固定为“主会话完整执行 → 搭档串行执行 → 回注主会话”，不能表达官方的依赖边、同 Agent 串行、跨 Agent 独立节点并行或多阶段交接。[桥接保留拓扑](../vendor/sprix-sage-router/sage_bridge.py#L212-L227)；[当前固定流水线](../static/app.js#L3886-L4056)；[官方分配与调度规则](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md#L39-L49) |
| 小权重协作者 | **不符合** | 当搭档名下需求总权重 `< 0.25` 时，前端不执行该需求，而是改成“只读复查”。这是 Agent Hub 自创的第四种执行语义，不是 SAGE 的 `COLLABORATE` 决策。[门槛与分支](../static/app.js#L4413-L4454)；[只读复查](../static/app.js#L3759-L3883) |
| `ExecutionState` 状态感知 | **部分符合（仅失败列表）** | 当前只传 `failed_agents` 和 `failure_count`。`active_agents`、`active_mode`、`completed_requirements`、当前 `progress`、`transferable_context` 均未传；Task 也固定 `progress=0.0`。失败记录只在当前页面内存中、且要求原文本完全相同才复用。[桥接状态构造](../vendor/sprix-sage-router/sage_bridge.py#L180-L199)；[失败列表来源](../static/app.js#L4216-L4225)；[官方 live state](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md#L9-L17) |
| 权限约束 | **不符合/缺失** | `Task.required_permissions` 未设置，两个 `Agent.permissions` 也为空；用户在 UI 选择的 `read-only`、`bypass` 等权限只在路由完成后传给 CLI，不参与候选过滤。因此不是 permission-first routing。[静态 Agent 构造](../vendor/sprix-sage-router/sage_bridge.py#L137-L144)；[固定 Task](../vendor/sprix-sage-router/sage_bridge.py#L187-L199)；[执行时才传权限](../static/app.js#L4347-L4357)；[官方约束规则](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md#L18-L22) |
| 预算与 deadline | **部分符合（占位值）** | 核心会执行预算和 deadline 过滤，但桥接对所有任务固定 `budget=1.0`、`deadline_ms=1,800,000`，没有来自用户、CLI、剩余额度或真实 SLA 的约束；所以“算法路径被调用”，但不是任务真实约束。[固定值](../vendor/sprix-sage-router/sage_bridge.py#L194-L199) |
| Agent discovery / Agent Cards | **不符合/缺失** | 仅注册硬编码的 `claude`、`codex` 与手工能力画像，没有发现流程、Agent Card、输入/输出兼容性、认证、availability 或 load。需要注意：上游参考实现本身也把真实 A2A adapter 和 signed Agent Card ingestion 列为 roadmap；这是生产集成目标，不是 `sprix_sage.py` 单文件承诺已提供的功能。[本地画像](../vendor/sprix-sage-router/sage_bridge.py#L42-L66)；[官方 A2A 映射与 roadmap](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/README.md#L112-L153) |
| Bids | **部分符合** | 确实构造 `Bid(cost, latency, confidence)` 并交给官方核心；但它不是 provider live bid，而是本地 EMA：默认静态值，之后由历史执行成功、输出 token 代理成本和耗时更新。[本地 bid](../vendor/sprix-sage-router/sage_bridge.py#L164-L177)；[EMA 更新](../vendor/sprix-sage-router/sage_bridge.py#L291-L307) |
| 结果回喂与在线学习 | **部分符合** | 信任、按技能可靠性、pair synergy、cost/latency fidelity 与 online success model 会持久化到 `~/.agenthub/sage_state.json`，也支持按 Agent/需求证据。[状态读写](../vendor/sprix-sage-router/sage_bridge.py#L87-L144)；[outcome](../vendor/sprix-sage-router/sage_bridge.py#L248-L308)。但当前 `success` 主要来自 CLI `doneOk`，表示进程是否正常结束，不是经过评价的任务质量；详见下文。 |
| 可审计输出 | **部分符合** | `decision_blob` 保存 assignments、topology、cost、latency、risk、utility、diagnostics 和 explanation，但 UI 卡片只主要展示模式、需求比例、成功率与覆盖率；topology、约束、risk、cost、latency、utility 未面向用户呈现或执行。[决策 blob](../vendor/sprix-sage-router/sage_bridge.py#L212-L240)；[卡片](../static/app.js#L3684-L3756) |
| 与手动团队模式互斥 | **符合当前产品规则** | 启动时若检测到历史双开状态，保留 SAGE、关闭手动团队；任一按钮开启都会关闭另一个。因此 SAGE 返回 `SELF` 后不会再被手动团队 preamble 改成内部组队，避免了此前的双重编排。[启动迁移](../static/app.js#L610-L625)；[双向互斥](../static/app.js#L4586-L4622) |

## 关键不一致的可复现例子

使用空学习状态、incumbent=`claude`、prompt=`debugging error planning design` 调用当前 [`cmd_route`](../vendor/sprix-sage-router/sage_bridge.py#L180-L241)，会得到：

```json
{
  "mode": "collaborate",
  "agents": ["claude", "codex"],
  "primary": "codex",
  "partner": "claude",
  "assignments": {
    "debugging": "codex",
    "planning": "claude"
  },
  "topology": [["claude", "codex"]]
}
```

这一个结果同时暴露两个问题：

- 官方决策里的 incumbent 是 `claude`，`COLLABORATE` 应由它保留所有权；当前桥接却把 `codex` 设为 `primary`。
- 官方拓扑给出 `claude → codex`，且这两个独立需求本可并行；前端实际却先让 `codex` 运行完整任务，再串行启动 `claude`，既没有消费该通信方向，也没有按 DAG 并行执行。

根因不是上游核心：核心返回的 `agents` 第一项仍是 incumbent，并生成了正确的 topology；偏差产生于桥接层的 `primary` 二次映射和前端固定流水线。

## 结果回喂为什么只能判为“部分符合”

官方 `ExecutionOutcome` 把 `success` 描述为 overall quality，并优先使用按 Agent、按需求的强证据；只有团队总结果时才作为低权重模糊证据。[官方在线更新规则](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md#L90-L102)

当前 Agent Hub 有真实闭环，但证据语义有以下偏差：

1. `SELF` / `HANDOFF` 以 CLI 是否正常结束作为 `success=1/0`，没有质量 evaluator；`actual_cost` 是 `output_tokens / 100000` 的代理值，不是 provider quote 同口径的真实费用。[首轮 outcome](../static/app.js#L4420-L4441)
2. 分工流水线把主会话名下全部需求直接记为 `1`，搭档名下需求只看搭档进程是否成功；没有检查需求产出质量。[流水线评分](../static/app.js#L4011-L4036)
3. 小权重“只读复查”分支会在复查开始前，就用主会话成功结果回喂整个原始 `COLLABORATE` decision；复查自己的成败没有再次进入 outcome。这样会给尚未执行分工的搭档模糊正向信用。[先回喂再分支](../static/app.js#L4420-L4454)；[`runCollabReview` 无 outcome](../static/app.js#L3759-L3883)
4. 主会话失败时搭档根本不会运行，但仍以只有团队整体失败、没有细粒度分数的方式回喂原始 team decision，搭档也会收到低权重负面证据。

因此，**持久化和学习代码是真的，当前观测标签却主要衡量“执行入口是否顺利完成”，还不能可靠解释为 Claude/Codex 的任务质量或协作质量。**

## 输入建模的边界

上游 API 假设调用方已经提供加权 requirement DAG、minimum、permissions、budget、deadline 等结构化 Task。Agent Hub 用关键词计数把 prompt 转成 `planning/coding/debugging/...`，并用静态 `DEPS` 生成 DAG；这是允许的适配策略，但不是上游定义的语义解析器。[本地关键词与 DAG](../vendor/sprix-sage-router/sage_bridge.py#L42-L80)；[需求推断](../vendor/sprix-sage-router/sage_bridge.py#L151-L199)

当前还有一个实际盲点：SAGE 在附件展开之前只接收输入框 `text`。长文本附件和图片附件不会参与 requirement 推断；纯附件新任务甚至不会触发 SAGE。视觉任务若只靠截图体现，`vision` 能力就不会被正确纳入路由。[路由输入](../static/app.js#L4210-L4225)；[路由之后才展开附件](../static/app.js#L4337-L4345)

## 修正优先级

如果目标是“符合当前上游文档”，建议按以下顺序修正：

1. **先修 `COLLABORATE` 所有权：** `primary` 必须保持 incumbent，不应改成最高权重需求的 assignee。
2. **让执行器消费 `assignments + topology`：** 只把各自名下需求交给对应 Agent，按 requirement DAG/通信边安排并行与串行，不再先让主会话做完整任务；删除 `< 0.25 → 只读复查` 这一语义替换，或在路由前把“复查”建模为真实 requirement。
3. **接入完整 live state：** 现有会话重规划时传 active route、已完成需求、progress、transferable context、失败次数，并按 `switch_recommended`/mode 执行所有权变化。
4. **把真实约束放进路由：** 至少把 UI 权限模式映射为 `required_permissions/Agent.permissions`，并定义与实际资源一致的预算、deadline 和成本单位。
5. **提高证据质量：** 将 CLI 正常退出与任务质量分开；协作结果按实际执行的 Agent/需求评价，未运行的搭档不记分；让 quote 与 actual cost 使用同一单位。
6. **最后再扩展 discovery：** 从已安装/可用 Agent、Agent Cards 或能力探测生成候选、availability、load、兼容模式和 live bids。上游也明确标注这是生产化 roadmap，而非 v0.2 单文件核心已经解决的问题。[官方项目状态](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/README.md#L184-L190)

## 核验记录与来源

- `git ls-remote https://github.com/wang2122/sprix-sage-router.git refs/heads/main`：`aed97852abc0bbd1dfccd8851b31290bc1b3f507`。
- 上游官方 README：[Sprix SAGE Router](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/README.md)
- 上游官方算法文档：[SAGE v0.2 algorithm design](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md)
- 上游官方核心源码：[`sprix_sage.py`](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/sprix_sage.py)
- 本地 vendored 核心：[`vendor/sprix-sage-router/sprix_sage.py`](../vendor/sprix-sage-router/sprix_sage.py)
- Agent Hub 自定义桥接：[`vendor/sprix-sage-router/sage_bridge.py`](../vendor/sprix-sage-router/sage_bridge.py)
- Rust 进程桥：[`src/sage.rs`](../src/sage.rs)
- 前端路由与编排：[`static/app.js`](../static/app.js)

本报告只使用上述上游官方仓库文件、版本元数据和当前本地源码，没有用第三方解读作为判定依据。

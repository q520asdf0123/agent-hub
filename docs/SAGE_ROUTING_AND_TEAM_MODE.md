# SAGE 智能路由与团队模式：原始职责、当前集成及冲突边界

> 调研日期：2026-08-25  
> 结论范围：Agent Hub 当前工作区、仓库内 vendored SAGE 源码，以及 Sprix AI 上游官方仓库。本文只使用一手来源。

## 结论

SAGE 智能路由和 Agent Hub 的“团队模式”**没有代码层面的硬冲突，但存在决策语义冲突和重复编排**。

- SAGE 的原始职责不是单纯“在 Claude Code 与 Codex 之间选一个”，而是在统一目标函数中选择 `SELF`、`COLLABORATE` 或 `HANDOFF`，并给协作团队分配需求 DAG、估算成本与关键路径延迟、结合执行证据在线更新。也就是说，“是否组队”本来就是 SAGE 路由决策的一部分。[上游 README：三种路由及所有权语义](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/README.md#L19-L27)；[上游算法文档：决策变量](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md#L1-L8)
- Agent Hub 的“团队模式”不参与 SAGE 评分。它在路由完成、主执行者已经选定后，向实际 prompt 前置一段“再派生 N 个子 agent”的指令。[当前团队前置指令](../static/app.js#L2007-L2018)；[当前发送顺序：先 SAGE，后团队前置指令](../static/app.js#L4208-L4217)
- 因此两个开关同时开启时，SAGE 评估的是“Claude / Codex 单独、二者协作或移交”，实际执行却可能变成“被选中的主 Agent + N 个内部子 Agent”，若 SAGE 又选了 `COLLABORATE`，还会再启动另一家顶层 Agent。SAGE 没有看到这层新增拓扑，无法在决策时计入其协调开销、预算、延迟或子 Agent 能力。

最明显的语义冲突是：SAGE 返回 `SELF`，含义是当前 Agent 独立继续、无需组队；团队模式随后仍会要求它尝试派生子 Agent。当前 prompt 中保留了“若不适合拆分，直接自己做”的退让条件，所以不是必然真的派生，但最终是否组队已从 SAGE 的统一决策退化为执行 Agent 的二次判断。[上游 `SELF` 定义](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/README.md#L21-L27)；[Agent Hub 团队前置指令](../static/app.js#L2007-L2018)

## 1. 原始 SAGE 的职责

### 1.1 它是“发现之后、执行之前”的策略层

官方定义 SAGE（State-Aware Graph Exchange）为 A2A 发现与任务执行之间的决策层：A2A 提供 Agent Card、消息、任务、产物、认证和传输；SAGE 决定哪个可行的 Agent 配置以何种模式执行，以及为什么。[上游 README：定位](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/README.md#L19-L27)

三种原始路由是：

| 路由 | 所有权 | 原始含义 |
|---|---|---|
| `SELF` | 当前 Agent | 已有能力与积累上下文足够，独立继续 |
| `COLLABORATE` | 当前 Agent 保留所有权 | 招募小型互补团队覆盖缺失需求 |
| `HANDOFF` | 同级 Agent 接管 | 专家优势大于上下文转移损失 |

来源：[上游 README 路由表](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/README.md#L21-L27)。

### 1.2 它决定的不只是“主 Agent 是谁”

原始算法一次选择模式 `m`、执行团队 `S`、需求到 Agent 的分配 `z` 和通信拓扑 `E`。它先按可用性、失败状态、权限、预算和 deadline 过滤 Agent，再对团队总成本和需求 DAG 的关键路径延迟做第二次可行性检查。[上游算法：决策问题](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md#L1-L8)；[上游算法：permission-first 可行性](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md#L18-L22)

对于 `COLLABORATE`，SAGE 按校准后的能力为每项需求选择执行者；跨 Agent 的依赖形成通信边，不同 Agent 的独立需求可以并行，同一 Agent 的需求串行。团队搜索使用 bounded beam search，而不是无条件拉满人数。[上游算法：覆盖、分配与拓扑](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md#L39-L49)；[上游算法：团队搜索](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md#L77-L89)

### 1.3 它原本只是路由器，不负责真的启动 Agent

官方 README 明确说明，当前原型只返回 routing decision，**有意不传输或执行任务**；真正的发送、流式处理、轮询和取消由 A2A client 完成。[上游 README：A2A 集成边界](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/README.md#L112-L123)

因此，Agent Hub 负责根据 SAGE 决策启动 Claude Code / Codex、执行分工、建立关联会话和回注结果是合理的集成层职责；但这些执行机制必须忠实实现 SAGE 选中的拓扑，或者把新增拓扑作为输入重新交给 SAGE 评分。

### 1.4 它从结果学习，但需要与实际执行配置一致的证据

`ExecutionOutcome` 支持整体、按 Agent、按需求的评分，以及实际成本和延迟。更新顺序优先使用最细粒度证据；只有团队整体结果时会按低权重的模糊证据处理。[上游算法：证据更新](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md#L90-L102)

上游也明确把它标记为早期 research preview，而非 peer-reviewed result。官方 `CITATION.cff` 将其登记为 `software` v0.2.0；未发现上游单独发布的 SAGE 论文。[上游 README：项目状态](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/README.md#L184-L190)；[官方引用元数据](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/CITATION.cff#L1-L11)

## 2. Agent Hub 当前实际做了什么

### 2.1 Vendored 核心与上游一致，桥接层是 Agent Hub 自己的策略

截至调研时，上游 `main` 为 [`aed97852`](https://github.com/wang2122/sprix-sage-router/commit/aed97852abc0bbd1dfccd8851b31290bc1b3f507)。本地 [`sprix_sage.py`](../vendor/sprix-sage-router/sprix_sage.py) 与该版本上游文件逐字符一致；其 MIT 来源也保留在 [`LICENSE`](../vendor/sprix-sage-router/LICENSE)。

[`sage_bridge.py`](../vendor/sprix-sage-router/sage_bridge.py) 则是 Agent Hub 的适配层，并非上游 SAGE 原始文件。它：

- 只注册 `claude` 和 `codex` 两个顶层 Agent，并使用 Agent Hub 自定的能力画像；[桥接层能力画像](../vendor/sprix-sage-router/sage_bridge.py#L53-L66)
- 通过关键词从 prompt 推断 `planning`、`coding`、`debugging` 等需求及 DAG；[桥接层需求推断](../vendor/sprix-sage-router/sage_bridge.py#L151-L193)
- 为每次新任务使用固定 `budget=1.0`、`deadline_ms=1_800_000`、`progress=0.0`，`ExecutionState` 只注入失败 Agent；[桥接层任务与状态构造](../vendor/sprix-sage-router/sage_bridge.py#L194-L199)
- 把 SAGE 的选择映射成一个 `primary` 和最多一个 `partner`；[桥接层决策映射](../vendor/sprix-sage-router/sage_bridge.py#L201-L240)
- 持久化信任、技能可靠性、协同、报价校准和在线模型，并接收结果回喂。[桥接层状态持久化](../vendor/sprix-sage-router/sage_bridge.py#L87-L144)；[桥接层 outcome](../vendor/sprix-sage-router/sage_bridge.py#L248-L308)

所以当前 Agent Hub 使用了 SAGE 的核心三模式与在线学习，但没有使用其完整的“进行中状态感知”能力：没有把 active executors、已完成需求、当前进度和可转移上下文传入路由。上游原本支持这些状态。[上游算法：live state](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md#L9-L17)；[当前桥接构造](../vendor/sprix-sage-router/sage_bridge.py#L194-L199)

### 2.2 当前“智能路由”既选主执行者，也可能自行组队

新会话发送前，前端把用户原始文本提交给 `/api/sage`。如果 SAGE 改选主 Agent，前端先切换到该 Agent；如果给出 `partner`，主任务完成后还会启动跨 Claude / Codex 的协作流程。[新会话 SAGE 调用与主执行者切换](../static/app.js#L4208-L4247)；[协作条件](../static/app.js#L4355-L4364)

对于搭档分工权重不低于 `0.25` 的决策，Agent Hub 执行“主 Agent → 搭档完成名下需求 → 结果回注主会话”的流水线；权重较小时改成搭档只读复查。这一 `0.25` 门槛与“只读复查”是 Agent Hub 的执行策略，不是上游算法文档定义的第四种模式。[当前协作门控](../static/app.js#L4405-L4446)；[分工流水线](../static/app.js#L3878-L3918)；[只读复查](../static/app.js#L3751-L3787)

### 2.3 当前“团队模式”是路由后的 prompt 指令

团队开关把以下要求加到主 Agent 收到的 prompt 前：

- Codex：使用 `spawn_agent` 派生 N 个子 Agent，拆分独立任务，`wait` 后汇总；
- Claude Code：使用 `Agent` 启动 N 个 `general-purpose` 子 Agent，可用 `SendMessage` 互通，最后汇总；
- 两者均允许在不适合拆分时不组队。

来源：[团队前置指令实现](../static/app.js#L2007-L2018)。

执行顺序是：先用原始用户文本完成 SAGE 路由和 Agent 切换，再把团队要求加到发往被选中主 Agent 的 prompt 中。因此团队前置文字不会污染 SAGE 的关键词需求推断；问题发生在**路由之后的实际执行配置改变**。[SAGE 路由阶段](../static/app.js#L4208-L4247)；[团队前置阶段](../static/app.js#L4329-L4348)

## 3. 哪些情况冲突，哪些只是叠加

| SAGE 结果 | 团队关闭 | 团队开启后的当前行为 | 判断 |
|---|---|---|---|
| `SELF` | 当前 Agent 独立执行 | 当前 Agent 仍收到“尝试派生 N 个子 Agent” | **直接语义冲突**：SAGE 选择了不组队，执行层又提出组队 |
| `HANDOFF` | 另一家顶层 Agent 独立接管 | 被移交的 Agent 再尝试派生内部子 Agent | **可以解释为叠加**：所有权仍正确；但 SAGE 对单 Agent 的成本、延迟和能力估计不再对应实际配置 |
| `COLLABORATE` | Claude 与 Codex 按 SAGE 分工协作或复查 | 主 Agent 内部再尝试派生 N 个子 Agent，随后还启动 SAGE 搭档 | **重复编排最强**：出现顶层跨 Agent 协作与内部子团队两层拓扑，新增开销未被 SAGE 评分 |
| SAGE 调用失败 | 当前 Agent 回退执行 | 当前 Agent 内部尝试组队 | **单纯团队模式**：没有 SAGE 决策可违反 |

两者职责可以按层次区分：

- SAGE 在 Agent Hub 当前画像中做**顶层跨运行时路由**：Claude Code、Codex，及二者是否协作；
- 团队模式做**选定运行时内部的执行战术**：要求主 Agent 再派生同一工具生态中的子 Agent。

如果明确把团队视为被选中 Agent 的内部实现细节，`HANDOFF + 团队` 可以正常叠加；但是 SAGE 的官方目标明确把团队成本、关键路径、协调开销和互补性纳入统一评分，所以在未把内部子团队信息传给 SAGE 前，这种叠加并不符合原始算法的完整假设。[上游 README：统一效用与可审计输出](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/README.md#L31-L57)；[上游算法：团队可行性与搜索](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md#L18-L22)

## 4. 为什么当前叠加会影响学习闭环

SAGE 的决策 blob 只包含路由时已知的 `claude` / `codex` Agent 与任务分配；团队模式派生的子 Agent 不会加入该 blob。[桥接层决策 blob](../vendor/sprix-sage-router/sage_bridge.py#L201-L240)

执行后，Agent Hub 只按顶层主 Agent / 搭档和顶层需求回喂结果。内部子 Agent 的数量、角色、各自成功与否没有单独传回 SAGE。[当前流水线 outcome 回喂](../static/app.js#L4003-L4028)；[桥接层 outcome 过滤与更新](../vendor/sprix-sage-router/sage_bridge.py#L268-L307)

这意味着：当团队模式实际参与执行时，SAGE 学到的是“某个顶层 Agent 在该任务上成功/失败”，但无法区分成功来自其单体能力还是内部子团队。该证据仍可用于“这个顶层执行入口整体是否可靠”的产品级统计，却不再是上游算法所设想的完整团队成员级信用分配。

## 5. 建议的产品语义

若希望忠实于 SAGE 原始设计，建议把“智能路由”设为组队决策的唯一权威：

1. 有 SAGE 决策时，不再无条件附加团队前置指令；`SELF` 按单 Agent 执行，`COLLABORATE` 执行 SAGE 分工，`HANDOFF` 由新所有者接管。
2. 团队开关单独开启时，继续作为“强制允许内部并行”的手动模式。
3. 如果产品确实需要两个开关同时工作，应把“是否允许内部子团队、人数、预估成本/延迟、子 Agent 能力”建模成 SAGE 候选配置或任务约束，再让 SAGE 比较“单体 / 内部团队 / 跨 Claude-Codex 团队”，而不是路由后悄悄改执行拓扑。

在不改算法的最小 UI 方案中，可以明确提示：**智能路由已包含跨 Agent 的组队判断；同时开启团队模式会额外增加内部子 Agent，只建议用户有意需要嵌套并行时使用。**

## 6. 来源与版本说明

- 上游官方仓库：[Sprix SAGE Router](https://github.com/wang2122/sprix-sage-router)
- 调研锁定的上游 revision：[`aed97852abc0bbd1dfccd8851b31290bc1b3f507`](https://github.com/wang2122/sprix-sage-router/commit/aed97852abc0bbd1dfccd8851b31290bc1b3f507)
- 官方算法设计：[SAGE v0.2 algorithm design](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/ALGORITHM.md)
- 官方引用元数据：[CITATION.cff](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/CITATION.cff)
- 本仓库 vendored 核心：[`vendor/sprix-sage-router/sprix_sage.py`](../vendor/sprix-sage-router/sprix_sage.py)
- 本仓库 Agent Hub 桥接：[`vendor/sprix-sage-router/sage_bridge.py`](../vendor/sprix-sage-router/sage_bridge.py)
- 本仓库前端编排：[`static/app.js`](../static/app.js)
- 许可：[上游 MIT LICENSE](https://github.com/wang2122/sprix-sage-router/blob/aed97852abc0bbd1dfccd8851b31290bc1b3f507/LICENSE)；[vendored LICENSE](../vendor/sprix-sage-router/LICENSE)

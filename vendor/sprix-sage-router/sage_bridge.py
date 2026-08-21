# -*- coding: utf-8 -*-
"""agent-hub 与 sprix-sage-router 的桥接（完整版）。

按库的设计本意实现：
- 有状态 router：信任度（BetaBelief）、技能可靠性、协同、bid 校准、在线成功率模型
  全部持久化到 ~/.agenthub/sage_state.json，跨请求复用同一套学习状态；
- record_outcome 证据回喂：执行结束后回传 成功与否/实际耗时/实际成本，驱动学习；
- COLLABORATE 由算法自主产生（RouterWeights 按本地双 agent 场景调参，无人工补充规则）；
- Bid 报价来自各 agent 的历史统计（成功率/平均耗时/平均成本），供 bid 校准闭环；
- 需求 DAG（planning→coding→review 等依赖）；失败重路由经 ExecutionState.failed_agents。

命令（stdin JSON）：
  {"cmd":"route","prompt":...,"incumbent":"claude|codex","failed":["codex"]?}
  {"cmd":"outcome","decision_blob":{...},"success":0..1,
   "actual_cost":x?,"actual_latency_ms":y?}

上游库: https://github.com/wang2122/sprix-sage-router (MIT)
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from sprix_sage import (  # noqa: E402
    Agent,
    BetaBelief,
    Bid,
    ExecutionOutcome,
    ExecutionState,
    Mode,
    Requirement,
    RouteDecision,
    RouterWeights,
    SAGERouter,
    Task,
)

STATE_PATH = os.path.join(os.path.expanduser("~"), ".agenthub", "sage_state.json")
TASK_ID = "chat-task"

KEYWORDS = {
    "debugging": ["修复", "报错", "bug", "为什么", "排查", "错误", "fix", "error", "崩溃", "失败", "异常", "不行", "问题"],
    "coding": ["实现", "开发", "写", "增加", "新增", "功能", "集成", "implement", "add", "build", "创建", "改成", "支持"],
    "review": ["审查", "review", "检查代码", "代码质量", "安全审查", "audit"],
    "analysis": ["分析", "检查", "查看", "看看", "确认", "对比", "统计", "explain", "解释", "理解", "是不是", "是否"],
    "planning": ["方案", "计划", "设计", "架构", "规划", "plan", "design", "怎么做", "思路"],
    "refactor": ["重构", "优化", "整理", "简化", "refactor", "clean"],
    "vision": ["图片", "截图", "界面", "样式", "ui", "image", "screenshot", "视觉"],
    "docs": ["文档", "readme", "注释", "说明", "总结", "报告", "document"],
}

# 画像刻意拉开差距（参考上游 README 示例的形态）：两边都是 0.8+ 的全才时
# 算法永远单干；只有互补且有真实短板，COLLABORATE 才会由效用比较自然胜出。
DEFAULT_PROFILES = {
    "claude": {
        "planning": 0.93, "analysis": 0.92, "review": 0.90, "docs": 0.94,
        "coding": 0.80, "debugging": 0.72, "refactor": 0.85, "vision": 0.90,
        "cost": 0.10, "latency_ms": 1500.0,
    },
    "codex": {
        "planning": 0.65, "analysis": 0.80, "review": 0.85, "docs": 0.60,
        "coding": 0.94, "debugging": 0.94, "refactor": 0.90, "vision": 0.75,
        "cost": 0.10, "latency_ms": 1500.0,
    },
}

# 需求 DAG：仅当依赖类别同样被推断出时生效
DEPS = {
    "coding": ["planning"],
    "review": ["coding", "debugging"],
    "docs": ["coding", "debugging", "analysis"],
    "refactor": ["analysis"],
}

# 本地双 agent：协调/移交开销远低于开放网络，调低对应惩罚使 COLLABORATE 可由算法自然胜出
WEIGHTS = RouterWeights(
    cost=0.12, latency=0.06, risk=0.10,
    handoff=0.10, coordination=0.02, uncertainty=0.04, exploration=0.06,
)


# ---------------------------------------------------------------------------
# 状态持久化
# ---------------------------------------------------------------------------

def load_state():
    try:
        with open(STATE_PATH, encoding="utf-8") as fh:
            return json.load(fh)
    except Exception:
        return {}


def save_state(st):
    os.makedirs(os.path.dirname(STATE_PATH), exist_ok=True)
    tmp = STATE_PATH + ".tmp"
    with open(tmp, "w", encoding="utf-8") as fh:
        json.dump(st, fh, ensure_ascii=False)
    os.replace(tmp, STATE_PATH)


def restore_router(router, st):
    for aid, ab in (st.get("reliability") or {}).items():
        if aid in router.reliability:
            router.reliability[aid] = BetaBelief(*ab)
    for key, ab in (st.get("skill") or {}).items():
        aid, req = key.split("|", 1)
        router.skill_reliability[(aid, req)] = BetaBelief(*ab)
    for key, ab in (st.get("synergy") or {}).items():
        left, right = key.split("|", 1)
        router.synergy[(left, right)] = BetaBelief(*ab)
    for name in ("cost_fidelity", "latency_fidelity"):
        for aid, ab in (st.get(name) or {}).items():
            if aid in getattr(router, name):
                getattr(router, name)[aid] = BetaBelief(*ab)
    m = st.get("model")
    if m:
        sm = router.success_model
        sm.bias = m.get("bias", sm.bias)
        sm.updates = int(m.get("updates", 0))
        for k, v in (m.get("weights") or {}).items():
            if k in sm.weights:
                sm.weights[k] = v


def dump_router(router, st):
    st["reliability"] = {a: [b.alpha, b.beta] for a, b in router.reliability.items()}
    st["skill"] = {f"{a}|{r}": [b.alpha, b.beta] for (a, r), b in router.skill_reliability.items()}
    st["synergy"] = {f"{l}|{r}": [b.alpha, b.beta] for (l, r), b in router.synergy.items()}
    st["cost_fidelity"] = {a: [b.alpha, b.beta] for a, b in router.cost_fidelity.items()}
    st["latency_fidelity"] = {a: [b.alpha, b.beta] for a, b in router.latency_fidelity.items()}
    sm = router.success_model
    st["model"] = {"bias": sm.bias, "updates": sm.updates, "weights": dict(sm.weights)}


def make_router(incumbent, st):
    agents = []
    for aid, p in DEFAULT_PROFILES.items():
        caps = {k: v for k, v in p.items() if k not in ("cost", "latency_ms")}
        agents.append(Agent(aid, caps, cost=float(p["cost"]), latency_ms=float(p["latency_ms"])))
    router = SAGERouter(agents, incumbent_id=incumbent, weights=WEIGHTS)
    restore_router(router, st)
    return router


# ---------------------------------------------------------------------------
# route
# ---------------------------------------------------------------------------

def infer_requirements(prompt):
    low = prompt.lower()
    hits = {}
    for cat, words in KEYWORDS.items():
        n = sum(low.count(w) for w in words)
        if n > 0:
            hits[cat] = n
    if not hits:
        hits = {"coding": 1, "analysis": 1}
    total = sum(hits.values())
    return {cat: max(0.15, n / total) for cat, n in hits.items()}


def build_bids(st):
    """各 agent 的报价来自历史统计（无历史用画像默认值），供 bid 校准闭环。"""
    stats = st.get("stats") or {}
    bids = []
    for aid, p in DEFAULT_PROFILES.items():
        s = stats.get(aid) or {}
        bids.append(Bid(
            agent_id=aid,
            task_id=TASK_ID,
            quoted_cost=float(s.get("cost", p["cost"])),
            promised_latency_ms=float(s.get("latency_ms", p["latency_ms"])),
            confidence=float(s.get("success", 0.7)),
        ))
    return bids


def cmd_route(req, st):
    prompt = req.get("prompt") or ""
    incumbent = req.get("incumbent") or "claude"
    if incumbent not in DEFAULT_PROFILES:
        incumbent = "claude"
    failed = frozenset(a for a in (req.get("failed") or []) if a in DEFAULT_PROFILES)

    weights = infer_requirements(prompt)
    ordered = sorted(weights.items(), key=lambda kv: -kv[1])
    present = {cat for cat, _ in ordered}
    requirements = tuple(
        Requirement(cat, w, depends_on=tuple(d for d in DEPS.get(cat, []) if d in present))
        for cat, w in ordered
    )
    task = Task(TASK_ID, requirements=requirements, value=1.0,
                budget=1.0, deadline_ms=1_800_000.0, progress=0.0)

    router = make_router(incumbent, st)
    state = ExecutionState(failed_agents=failed, failure_count=len(failed))
    d = router.route(task, bids=build_bids(st), state=state)

    mode = str(d.mode.value if hasattr(d.mode, "value") else d.mode).lower()
    if mode == "self":
        primary = incumbent
    elif mode == "handoff":
        primary = d.agents[0]
    else:
        top_req = requirements[0].name if requirements else None
        primary = dict(d.assignments).get(top_req) or (d.agents[0] if d.agents else incumbent)
    # 搭档：仅由算法的组队决策产生（COLLABORATE 团队里的另一位）
    partner = next((a for a in d.agents if a != primary), None)

    blob = {
        "mode": mode,
        "agents": list(d.agents),
        "utility": d.utility,
        "success_probability": d.success_probability,
        "coverage": d.coverage,
        "cost": d.cost,
        "latency_ms": d.latency_ms,
        "risk": d.risk,
        "explanation": d.explanation,
        "assignments": dict(d.assignments),
        "topology": [list(e) for e in d.topology],
        "switch_recommended": d.switch_recommended,
        "diagnostics": dict(d.diagnostics),
        "model_features": dict(d.model_features),
    }
    return {
        "mode": mode,
        "primary": primary,
        "partner": partner,
        "agents": list(d.agents),
        "assignments": dict(d.assignments),
        "utility": round(d.utility, 4),
        "success_probability": round(d.success_probability, 4),
        "coverage": round(d.coverage, 4),
        "requirements": {r.name: round(r.weight, 3) for r in requirements},
        "explanation": d.explanation,
        "decision_blob": blob,
        "learned_updates": router.success_model.updates,
    }


# ---------------------------------------------------------------------------
# outcome（证据回喂）
# ---------------------------------------------------------------------------

def cmd_outcome(req, st):
    blob = req.get("decision_blob") or {}
    decision = RouteDecision(
        mode=Mode(blob.get("mode", "self")),
        agents=tuple(blob.get("agents") or []),
        utility=float(blob.get("utility", 0.0)),
        success_probability=float(blob.get("success_probability", 0.5)),
        coverage=float(blob.get("coverage", 0.5)),
        cost=float(blob.get("cost", 0.0)),
        latency_ms=float(blob.get("latency_ms", 0.0)),
        risk=float(blob.get("risk", 0.0)),
        explanation=str(blob.get("explanation", "")),
        assignments=dict(blob.get("assignments") or {}),
        topology=tuple(tuple(e) for e in (blob.get("topology") or [])),
        switch_recommended=bool(blob.get("switch_recommended", False)),
        diagnostics=dict(blob.get("diagnostics") or {}),
        model_features=dict(blob.get("model_features") or {}),
    )
    if not decision.agents:
        return {"ok": False, "error": "decision_blob 缺少 agents"}
    success = max(0.0, min(1.0, float(req.get("success", 0.0))))
    outcome = ExecutionOutcome(
        success=success,
        actual_cost=req.get("actual_cost"),
        actual_latency_ms=req.get("actual_latency_ms"),
    )
    incumbent = decision.agents[0]
    router = make_router(incumbent, st)
    router.record_outcome(decision, outcome)
    dump_router(router, st)

    # 滚动统计（EMA）→ 下次 route 的 Bid 报价
    stats = st.setdefault("stats", {})
    for aid in decision.agents:
        s = stats.setdefault(aid, {})
        n = int(s.get("n", 0))
        alpha = 0.3
        s["n"] = n + 1
        s["success"] = round((1 - alpha) * float(s.get("success", 0.7)) + alpha * success, 4)
        if req.get("actual_latency_ms") is not None:
            s["latency_ms"] = round(
                (1 - alpha) * float(s.get("latency_ms", DEFAULT_PROFILES[aid]["latency_ms"]))
                + alpha * float(req["actual_latency_ms"]), 1)
        if req.get("actual_cost") is not None:
            s["cost"] = round(
                (1 - alpha) * float(s.get("cost", DEFAULT_PROFILES[aid]["cost"]))
                + alpha * float(req["actual_cost"]), 5)
    save_state(st)
    return {"ok": True, "learned_updates": router.success_model.updates}


def main():
    raw = sys.stdin.buffer.read().decode("utf-8")
    req = json.loads(raw)
    st = load_state()
    if req.get("cmd") == "outcome":
        out = cmd_outcome(req, st)
    else:
        out = cmd_route(req, st)
    print(json.dumps(out, ensure_ascii=False))


if __name__ == "__main__":
    main()

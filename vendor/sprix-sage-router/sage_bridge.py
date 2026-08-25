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
import math
import os
import shutil
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
STATE_SCHEMA = 4
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
        "permissions": ["read", "workspace_write"],
    },
    "codex": {
        "planning": 0.65, "analysis": 0.80, "review": 0.85, "docs": 0.60,
        "coding": 0.94, "debugging": 0.94, "refactor": 0.90, "vision": 0.75,
        "cost": 0.10, "latency_ms": 1500.0,
        "permissions": ["read", "workspace_write"],
    },
}

ROLE_NAMES = {
    "planning": "Planner",
    "analysis": "Analyst",
    "review": "Reviewer",
    "docs": "Documenter",
    "coding": "Coder",
    "debugging": "Debugger",
    "refactor": "Refactorer",
    "vision": "Vision",
}


def _legacy_profiles():
    return {
        aid: {
            "runtime": aid,
            "model": None,
            "label": "Claude Code" if aid == "claude" else "Codex",
            "role": "Generalist",
            "skills": {
                key: value for key, value in profile.items()
                if key not in ("cost", "latency_ms", "permissions")
            },
            "cost": profile["cost"],
            "latency_ms": profile["latency_ms"],
            "permissions": list(profile["permissions"]),
            "fast": aid == "codex",
            "supported_efforts": ["low", "medium", "high", "xhigh"],
            "default_effort": "medium",
        }
        for aid, profile in DEFAULT_PROFILES.items()
    }


def _model_profile(runtime, model, context_window=None):
    base = _legacy_profiles()[runtime]
    skills = dict(base["skills"])
    low = model.lower()
    cost = float(base["cost"])
    latency = float(base["latency_ms"])

    if runtime == "codex":
        if "5.6-sol" in low:
            cost, latency = 0.20, 1800.0
            for key, bonus in {"planning": .10, "analysis": .08, "coding": .04,
                               "debugging": .04, "review": .06, "vision": .04}.items():
                skills[key] = min(0.99, skills[key] + bonus)
        elif "5.6-terra" in low:
            cost, latency = 0.11, 1200.0
            for key in ("planning", "analysis", "coding", "debugging", "review"):
                skills[key] = min(0.97, skills[key] + 0.02)
        elif "luna" in low or "mini" in low or "spark" in low:
            cost, latency = 0.035, 550.0
            for key in ("planning", "analysis", "review", "vision"):
                skills[key] = max(0.35, skills[key] - 0.12)
        elif "auto-review" in low:
            cost, latency = 0.05, 650.0
            skills.update({"review": 0.97, "analysis": 0.90, "docs": 0.82,
                           "coding": 0.45, "debugging": 0.65, "refactor": 0.60})
            base["permissions"] = ["read"]
        elif "5.5" in low or "5.4" in low:
            cost, latency = 0.13, 1350.0
    else:
        if "opus" in low or "fable" in low:
            cost, latency = 0.20, 1800.0
            for key, bonus in {"planning": .05, "analysis": .05, "review": .05,
                               "docs": .04, "vision": .04}.items():
                skills[key] = min(0.99, skills[key] + bonus)
        elif "sonnet" in low:
            cost, latency = 0.10, 1000.0
            skills["coding"] = min(0.90, skills["coding"] + 0.05)
            skills["debugging"] = min(0.84, skills["debugging"] + 0.08)
        elif "haiku" in low:
            cost, latency = 0.035, 500.0
            for key in ("planning", "analysis", "review", "vision"):
                skills[key] = max(0.40, skills[key] - 0.10)

    if context_window and context_window >= 1_000_000:
        skills["analysis"] = min(0.99, skills["analysis"] + 0.02)
        skills["docs"] = min(0.99, skills["docs"] + 0.02)
    role_key = max(skills, key=skills.get)
    return {
        "runtime": runtime,
        "model": model,
        "label": ("Claude Code" if runtime == "claude" else "Codex") + " · " + model,
        "role": ROLE_NAMES.get(role_key, role_key),
        "skills": skills,
        "cost": cost,
        "latency_ms": latency,
        "permissions": list(base["permissions"]),
        "fast": runtime == "codex",
    }


def build_executor_profiles(constraints, incumbent_runtime):
    catalog = constraints.get("model_catalog") or {}
    if not catalog:
        profiles = _legacy_profiles()
        return profiles, incumbent_runtime
    profiles = {}
    for runtime in ("claude", "codex"):
        info = catalog.get(runtime) or {}
        models = []
        for model in [info.get("default"), *(info.get("models") or [])]:
            if model and model not in models:
                models.append(model)
        windows = info.get("windows") or {}
        supported_efforts = info.get("efforts") or ["low", "medium", "high", "xhigh"]
        for model in models:
            executor_id = runtime + "::" + model
            profile = _model_profile(runtime, model, windows.get(model))
            profile["supported_efforts"] = list(supported_efforts)
            profile["default_effort"] = info.get("default_effort") or "medium"
            profiles[executor_id] = profile
    incumbent_model = constraints.get("incumbent_model")
    runtime_info = catalog.get(incumbent_runtime) or {}
    incumbent_model = incumbent_model or runtime_info.get("default")
    incumbent_id = incumbent_runtime + "::" + incumbent_model if incumbent_model else incumbent_runtime
    if incumbent_id not in profiles:
        fallback = next(
            (executor_id for executor_id, profile in profiles.items()
             if profile["runtime"] == incumbent_runtime),
            None,
        )
        incumbent_id = fallback or incumbent_runtime
    return profiles or _legacy_profiles(), incumbent_id


def task_complexity(requirements):
    count = len(requirements)
    edges = sum(len(requirement.depends_on) for requirement in requirements)
    total = sum(requirement.weight for requirement in requirements) or 1.0
    concentration = max(requirement.weight for requirement in requirements) / total
    score = min(1.0, 0.14 * count + 0.12 * edges + 0.35 * (1.0 - concentration))
    if score < 0.28:
        collaborators = 0
        label = "simple"
    elif score < 0.52:
        collaborators = 1
        label = "moderate"
    elif score < 0.76:
        collaborators = 2
        label = "complex"
    else:
        collaborators = 4
        label = "very_complex"
    return {"score": round(score, 4), "label": label, "max_collaborators": collaborators}


EFFORT_ORDER = ["none", "minimal", "low", "medium", "high", "xhigh", "max", "ultra"]
EFFORT_FACTORS = {
    "none": (0.60, 0.60, 0.96),
    "minimal": (0.65, 0.65, 0.97),
    "low": (0.75, 0.75, 0.98),
    "medium": (1.00, 1.00, 1.00),
    "high": (1.20, 1.30, 1.01),
    "xhigh": (1.45, 1.65, 1.025),
    "max": (1.75, 2.05, 1.04),
    "ultra": (2.10, 2.45, 1.05),
}
COMPLEXITY_EFFORT = {
    "simple": "low",
    "moderate": "medium",
    "complex": "high",
    "very_complex": "xhigh",
}
COMPLEXITY_VALUE = {
    "simple": 1.0,
    "moderate": 1.1,
    "complex": 1.3,
    "very_complex": 1.5,
}


def select_supported_effort(profile, desired):
    supported = [value for value in profile.get("supported_efforts") or [] if value in EFFORT_ORDER]
    if not supported:
        return profile.get("default_effort") or "medium"
    desired_index = EFFORT_ORDER.index(desired) if desired in EFFORT_ORDER else EFFORT_ORDER.index("medium")
    # 产品规则：Sol 自动思考最高只到 xhigh，即使模型目录声明支持 max。
    if "5.6-sol" in (profile.get("model") or "").lower():
        cap_index = EFFORT_ORDER.index("xhigh")
        desired_index = min(desired_index, cap_index)
        supported = [value for value in supported if EFFORT_ORDER.index(value) <= cap_index]
        if not supported:
            return "xhigh"
    return min(supported, key=lambda value: abs(EFFORT_ORDER.index(value) - desired_index))


def apply_effort_priors(profiles, complexity):
    desired = COMPLEXITY_EFFORT[complexity["label"]]
    for profile in profiles.values():
        effort = select_supported_effort(profile, desired)
        cost_factor, latency_factor, skill_factor = EFFORT_FACTORS[effort]
        profile["effort"] = effort
        profile["cost"] = round(float(profile["cost"]) * cost_factor, 6)
        profile["latency_ms"] = round(float(profile["latency_ms"]) * latency_factor, 1)
        profile["skills"] = {
            key: min(0.995, value * skill_factor) for key, value in profile["skills"].items()
        }


def requirement_effort(profile, complexity, requirement):
    desired = COMPLEXITY_EFFORT[complexity["label"]]
    index = EFFORT_ORDER.index(desired)
    if requirement == "docs":
        index -= 1
    elif complexity["label"] == "very_complex" and requirement in {
        "planning", "coding", "debugging", "analysis", "review"
    }:
        index += 1
    desired = EFFORT_ORDER[max(0, min(index, EFFORT_ORDER.index("max")))]
    return select_supported_effort(profile, desired)

# 需求 DAG：仅当依赖类别同样被推断出时生效
DEPS = {
    "coding": ["planning"],
    "review": ["coding", "debugging"],
    "docs": ["coding", "debugging", "analysis"],
    "refactor": ["analysis"],
}

# 本地双 agent：协调/移交开销远低于开放网络，调低对应惩罚使 COLLABORATE 可由算法自然胜出
WEIGHTS = RouterWeights(
    # 模型池使用归一化相对成本；降低其权重，避免高复杂度任务永远被单模型低价吞掉。
    # 实际人数仍由 SAGE 覆盖率、冗余、延迟与协调开销共同决定。
    cost=0.04, latency=0.04, risk=0.08,
    handoff=0.10, coordination=0.005, uncertainty=0.04, exploration=0.06,
)


# ---------------------------------------------------------------------------
# 状态持久化
# ---------------------------------------------------------------------------

def load_state():
    try:
        with open(STATE_PATH, encoding="utf-8") as fh:
            st = json.load(fh)
    except Exception:
        return {}
    if st.get("_schema") != STATE_SCHEMA:
        backup = STATE_PATH + f".pre-schema-v{STATE_SCHEMA}"
        if not os.path.exists(backup):
            shutil.copy2(STATE_PATH, backup)
        st = {"_schema": STATE_SCHEMA}
        save_state(st)
    return st


def save_state(st):
    st["_schema"] = STATE_SCHEMA
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


def make_router(incumbent, st, constraints=None, profiles=None, max_collaborators=2):
    constraints = constraints or {}
    profiles = profiles or _legacy_profiles()
    raw_available = constraints.get("available_agents")
    available = {profile["runtime"] for profile in profiles.values()}
    if raw_available is not None:
        available &= set(raw_available)
    loads = constraints.get("loads") or {}
    agents = []
    for aid, profile in profiles.items():
        runtime = profile.get("runtime", aid)
        agents.append(Agent(
            aid,
            profile["skills"],
            cost=float(profile["cost"]),
            latency_ms=float(profile["latency_ms"]),
            permissions=frozenset(profile["permissions"]),
            availability=1.0 if runtime in available else 0.0,
            load=max(0.0, min(1.0, float(loads.get(runtime, 0.0)))),
        ))
    router = SAGERouter(
        agents,
        incumbent_id=incumbent,
        weights=WEIGHTS,
        max_collaborators=max_collaborators,
    )
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


def build_bids(st, profiles):
    """各 agent 的报价来自历史统计（无历史用画像默认值），供 bid 校准闭环。"""
    stats = st.get("stats") or {}
    bids = []
    for aid, profile in profiles.items():
        root = stats.get(aid) or {}
        effort = profile.get("effort") or profile.get("default_effort") or "medium"
        s = (root.get("efforts") or {}).get(effort) or root
        bids.append(Bid(
            agent_id=aid,
            task_id=TASK_ID,
            quoted_cost=float(s.get("cost", profile["cost"])),
            promised_latency_ms=float(s.get("latency_ms", profile["latency_ms"])),
            confidence=float(s.get("success", 0.7)),
        ))
    return bids


def _expand_executor_refs(values, profiles, incumbent_id):
    out = []
    for value in values or []:
        if value in profiles and value not in out:
            out.append(value)
            continue
        matches = [
            executor_id for executor_id, profile in profiles.items()
            if profile["runtime"] == value
        ]
        if incumbent_id in matches:
            matches.remove(incumbent_id)
            matches.insert(0, incumbent_id)
        if matches and matches[0] not in out:
            out.append(matches[0])
    return out


def route_without_idle_peers(router, task, bids, state, incumbent):
    """执行适配约束：peer 必须至少承接一个 requirement，避免虚假团队人数。"""
    current = state
    for _ in range(len(router.agents)):
        decision = router.route(task, bids=bids, state=current)
        assigned = set(decision.assignments.values())
        idle = {
            agent for agent in decision.agents
            if agent != incumbent and agent not in assigned
        }
        if decision.mode is not Mode.COLLABORATE or not idle:
            return decision
        current = ExecutionState(
            active_agents=current.active_agents,
            active_mode=current.active_mode,
            completed_requirements=current.completed_requirements,
            progress=current.progress,
            transferable_context=current.transferable_context,
            failed_agents=current.failed_agents | frozenset(idle),
            failure_count=current.failure_count,
        )
    return decision


def cmd_route(req, st):
    prompt = req.get("prompt") or ""
    incumbent_runtime = req.get("incumbent") or "claude"
    if incumbent_runtime not in DEFAULT_PROFILES:
        incumbent_runtime = "claude"
    constraints = req.get("constraints") or {}
    profiles, incumbent = build_executor_profiles(constraints, incumbent_runtime)
    raw_state = req.get("state") or {}
    raw_failed = set(req.get("failed") or []) | set(raw_state.get("failed_agents") or [])
    failed = set(_expand_executor_refs(raw_failed, profiles, incumbent))
    for value in raw_failed:
        if value in DEFAULT_PROFILES:
            failed.update(
                executor_id for executor_id, profile in profiles.items()
                if profile["runtime"] == value
            )
    failed = frozenset(failed)

    weights = infer_requirements(prompt)
    ordered = sorted(weights.items(), key=lambda kv: -kv[1])
    present = {cat for cat, _ in ordered}
    requirements = tuple(
        Requirement(cat, w, depends_on=tuple(d for d in DEPS.get(cat, []) if d in present))
        for cat, w in ordered
    )
    complexity = task_complexity(requirements)
    requested_max_team = max(1, int(constraints.get("max_team_size", 5)))
    max_collaborators = min(
        complexity["max_collaborators"],
        requested_max_team - 1,
        max(0, len(profiles) - 1),
    )
    apply_effort_priors(profiles, complexity)
    required_permissions = constraints.get("required_permissions")
    if required_permissions is None:
        required_permissions = ["read"]
        write_requirements = {"coding", "debugging", "refactor"}
        if constraints.get("permission_mode") != "read-only" and present & write_requirements:
            required_permissions.append("workspace_write")
    progress = raw_state.get("progress")
    task_progress = progress if progress is not None else constraints.get("progress", 0.0)
    task = Task(
        TASK_ID,
        requirements=requirements,
        value=float(
            constraints["value"] if constraints.get("value") is not None
            else COMPLEXITY_VALUE[complexity["label"]]
        ),
        budget=float(constraints.get("budget", math.inf)),
        deadline_ms=float(constraints.get("deadline_ms", math.inf)),
        required_permissions=frozenset(required_permissions),
        risk_tolerance=float(constraints.get("risk_tolerance", 0.5)),
        progress=max(0.0, min(1.0, float(task_progress))),
        handoff_friction=float(constraints.get("handoff_friction", 0.25)),
        coordination_overhead=float(constraints.get("coordination_overhead", 0.06)),
        context_transferability=float(constraints.get("context_transferability", 0.70)),
        replan_friction=float(constraints.get("replan_friction", 0.03)),
    )

    active_mode = str(raw_state.get("active_mode") or "self").lower()
    if active_mode not in {mode.value for mode in Mode}:
        active_mode = "self"
    completed = frozenset(
        name for name in (raw_state.get("completed_requirements") or []) if name in present
    )
    if len(completed) == len(requirements):
        completed = frozenset()
    state = ExecutionState(
        active_agents=tuple(
            _expand_executor_refs(raw_state.get("active_agents") or [], profiles, incumbent)
        ),
        active_mode=Mode(active_mode),
        completed_requirements=completed,
        progress=None if progress is None else max(0.0, min(1.0, float(progress))),
        transferable_context=(
            None if raw_state.get("transferable_context") is None
            else max(0.0, min(1.0, float(raw_state["transferable_context"])))
        ),
        failed_agents=failed,
        failure_count=max(int(raw_state.get("failure_count", 0)), len(failed)),
    )
    router = make_router(
        incumbent,
        st,
        constraints,
        profiles=profiles,
        max_collaborators=max_collaborators,
    )
    d = route_without_idle_peers(
        router,
        task,
        build_bids(st, profiles),
        state,
        incumbent,
    )

    mode = str(d.mode.value if hasattr(d.mode, "value") else d.mode).lower()
    if mode == "self":
        primary = incumbent
    elif mode == "handoff":
        primary = d.agents[0]
    else:
        # 官方 COLLABORATE 语义：incumbent 始终保留任务所有权。
        primary = incumbent
    partners = [a for a in d.agents if a != primary]
    partner = partners[0] if partners else None
    selected_profiles = {agent: profiles[agent] for agent in d.agents}
    profile_snapshot = dict(selected_profiles)
    profile_snapshot.setdefault(incumbent, profiles[incumbent])
    efforts = {
        requirement: requirement_effort(profiles[executor], complexity, requirement)
        for requirement, executor in d.assignments.items()
    }
    primary_owned_efforts = [
        effort for requirement, effort in efforts.items()
        if d.assignments.get(requirement) == primary
    ]
    primary_effort = max(
        primary_owned_efforts or [profiles[primary]["effort"]],
        key=lambda value: EFFORT_ORDER.index(value),
    )
    summary_effort = select_supported_effort(
        profiles[primary], COMPLEXITY_EFFORT[complexity["label"]]
    )

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
        "requirements": {r.name: r.weight for r in requirements},
        "dependencies": {r.name: list(r.depends_on) for r in requirements},
        "incumbent": incumbent,
        "incumbent_runtime": incumbent_runtime,
        "profile_snapshot": profile_snapshot,
        "complexity": complexity,
        "team_limit": max_collaborators + 1,
        "efforts": efforts,
        "primary_effort": primary_effort,
        "summary_effort": summary_effort,
        "switch_recommended": d.switch_recommended,
        "diagnostics": dict(d.diagnostics),
        "model_features": dict(d.model_features),
    }
    return {
        "mode": mode,
        "primary": primary,
        "partner": partner,
        "partners": partners,
        "agents": list(d.agents),
        "executors": selected_profiles,
        "primary_runtime": profiles[primary]["runtime"],
        "primary_model": profiles[primary]["model"],
        "team_size": len(d.agents),
        "team_limit": max_collaborators + 1,
        "complexity": complexity,
        "efforts": efforts,
        "primary_effort": primary_effort,
        "summary_effort": summary_effort,
        "assignments": dict(d.assignments),
        "topology": [list(e) for e in d.topology],
        "dependencies": {r.name: list(r.depends_on) for r in requirements},
        "switch_recommended": d.switch_recommended,
        "utility": round(d.utility, 4),
        "success_probability": round(d.success_probability, 4),
        "coverage": round(d.coverage, 4),
        "cost": round(d.cost, 5),
        "latency_ms": round(d.latency_ms, 1),
        "risk": round(d.risk, 4),
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
    clip = lambda v: max(0.0, min(1.0, float(v)))  # noqa: E731
    # 分工模式（COLLABORATE 流水线）：按 agent / 按需求打分，过滤到决策范围内
    agent_scores = {
        a: clip(v) for a, v in (req.get("agent_scores") or {}).items()
        if a in decision.agents
    }
    requirement_scores = {
        r: clip(v) for r, v in (req.get("requirement_scores") or {}).items()
        if r in decision.assignments
    }
    outcome = ExecutionOutcome(
        success=success,
        agent_scores=agent_scores,
        requirement_scores=requirement_scores,
        actual_cost=req.get("actual_cost"),
        actual_latency_ms=req.get("actual_latency_ms"),
    )
    profiles = blob.get("profile_snapshot") or _legacy_profiles()
    incumbent = blob.get("incumbent") or decision.agents[0]
    if incumbent not in profiles:
        incumbent = decision.agents[0]
    router = make_router(incumbent, st, profiles=profiles)
    router.record_outcome(decision, outcome)
    dump_router(router, st)

    # 滚动统计（EMA）→ 下次 route 的 Bid 报价
    stats = st.setdefault("stats", {})
    for aid in decision.agents:
        root = stats.setdefault(aid, {})
        effort = profiles[aid].get("effort") or profiles[aid].get("default_effort") or "medium"
        s = root.setdefault("efforts", {}).setdefault(effort, {})
        n = int(s.get("n", 0))
        alpha = 0.3
        observed_success = agent_scores.get(aid, success)
        s["n"] = n + 1
        s["success"] = round(
            (1 - alpha) * float(s.get("success", 0.7)) + alpha * observed_success, 4
        )
        # 团队 outcome 的 latency/cost 是整条关键路径，不归因到任一成员的下一次 Bid。
        if len(decision.agents) == 1 and req.get("actual_latency_ms") is not None:
            s["latency_ms"] = round(
                (1 - alpha) * float(s.get("latency_ms", profiles[aid]["latency_ms"]))
                + alpha * float(req["actual_latency_ms"]), 1)
        if len(decision.agents) == 1 and req.get("actual_cost") is not None:
            s["cost"] = round(
                (1 - alpha) * float(s.get("cost", profiles[aid]["cost"]))
                + alpha * float(req["actual_cost"]), 5)
    if req.get("persist", True):
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

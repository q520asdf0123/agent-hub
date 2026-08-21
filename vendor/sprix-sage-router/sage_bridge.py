# -*- coding: utf-8 -*-
"""agent-hub 与 sprix-sage-router 的桥接：
stdin 读入 {"prompt": "...", "incumbent": "claude"|"codex", "profiles": {...}?}，
从 prompt 推断任务需求，让 SAGE 在 Claude Code 与 Codex 之间做
SELF / COLLABORATE / HANDOFF 路由决策，stdout 输出 JSON。

上游库: https://github.com/wang2122/sprix-sage-router (MIT)
"""
import json
import sys

sys.path.insert(0, __file__.rsplit("\\", 1)[0].rsplit("/", 1)[0] if ("\\" in __file__ or "/" in __file__) else ".")
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from sprix_sage import Agent, Requirement, SAGERouter, Task  # noqa: E402

# 需求类别 → 触发关键词（中英）。命中次数决定权重。
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

# 两个本地 CLI 的默认能力画像（可被请求里的 profiles 覆盖；纯启发式，供 SAGE 打分用）
DEFAULT_PROFILES = {
    "claude": {
        "planning": 0.93, "analysis": 0.92, "review": 0.91, "docs": 0.93,
        "coding": 0.88, "debugging": 0.88, "refactor": 0.90, "vision": 0.90,
        "cost": 0.10, "latency_ms": 1500.0,
    },
    "codex": {
        "planning": 0.82, "analysis": 0.86, "review": 0.88, "docs": 0.80,
        "coding": 0.94, "debugging": 0.93, "refactor": 0.90, "vision": 0.84,
        "cost": 0.10, "latency_ms": 1500.0,
    },
}


def infer_requirements(prompt: str):
    low = prompt.lower()
    hits = {}
    for cat, words in KEYWORDS.items():
        n = sum(low.count(w) for w in words)
        if n > 0:
            hits[cat] = n
    if not hits:
        hits = {"coding": 1, "analysis": 1}
    total = sum(hits.values())
    # 权重归一化，保底 0.15 避免单一需求过度主导
    return {cat: max(0.15, n / total) for cat, n in hits.items()}


def main():
    raw = sys.stdin.buffer.read().decode("utf-8")
    req = json.loads(raw)
    prompt = req.get("prompt") or ""
    incumbent = req.get("incumbent") or "claude"
    if incumbent not in ("claude", "codex"):
        incumbent = "claude"
    profiles = dict(DEFAULT_PROFILES)
    for aid, over in (req.get("profiles") or {}).items():
        if aid in profiles and isinstance(over, dict):
            profiles[aid] = {**profiles[aid], **over}

    weights = infer_requirements(prompt)
    agents = []
    for aid, p in profiles.items():
        caps = {k: v for k, v in p.items() if k not in ("cost", "latency_ms")}
        agents.append(Agent(aid, caps, cost=float(p.get("cost", 0.1)),
                            latency_ms=float(p.get("latency_ms", 1500.0))))

    requirements = tuple(Requirement(cat, w) for cat, w in
                         sorted(weights.items(), key=lambda kv: -kv[1]))
    task = Task("chat-task", requirements=requirements, value=1.0,
                budget=1.0, deadline_ms=600000.0, progress=0.0)

    router = SAGERouter(agents, incumbent_id=incumbent)
    d = router.route(task)

    mode = str(d.mode.value if hasattr(d.mode, "value") else d.mode).lower()
    # 主执行者：SELF=现任；HANDOFF=目标；COLLABORATE=承担最重需求的成员
    if mode == "self":
        primary = incumbent
    elif mode == "handoff":
        primary = d.agents[0]
    else:
        top_req = requirements[0].name if requirements else None
        primary = dict(d.assignments).get(top_req) or (d.agents[0] if d.agents else incumbent)
    partner = next((a for a in d.agents if a != primary), None)
    # 搭档补充规则：算法未组队时，若某个次要需求（权重≥0.25）明显是另一方强项，
    # 指定其为复查搭档（触发前端的自动协作复查）。
    if partner is None:
        other = "codex" if primary == "claude" else "claude"
        po, pp = profiles[other], profiles[primary]
        for cat, w in weights.items():
            if w >= 0.25 and po.get(cat, 0) >= pp.get(cat, 0) + 0.03:
                partner = other
                break

    print(json.dumps({
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
    }, ensure_ascii=False))


if __name__ == "__main__":
    main()

import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "vendor" / "sprix-sage-router"))

import sage_bridge  # noqa: E402


class SageBridgeTests(unittest.TestCase):
    MODEL_CATALOG = {
        "claude": {
            "default": "opus[1m]",
            "models": ["opus[1m]", "claude-sonnet-5"],
            "windows": {"opus[1m]": 1_000_000, "claude-sonnet-5": 1_000_000},
            "efforts": ["low", "medium", "high", "xhigh"],
            "default_effort": "xhigh",
        },
        "codex": {
            "default": "gpt-5.6-sol",
            "models": ["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.6-luna", "codex-auto-review"],
            "windows": {"gpt-5.6-sol": 1_050_000},
            "efforts": ["low", "medium", "high", "xhigh", "max"],
            "default_effort": "xhigh",
        },
    }

    def route(self, **overrides):
        request = {
            "prompt": "debugging error planning design",
            "incumbent": "claude",
            "constraints": {
                "available_agents": ["claude", "codex"],
                "required_permissions": ["read", "workspace_write"],
            },
            "state": {},
        }
        request.update(overrides)
        return sage_bridge.cmd_route(request, {})

    def test_collaborate_keeps_incumbent_as_primary(self):
        decision = self.route()

        self.assertEqual(decision["mode"], "collaborate")
        self.assertEqual(decision["primary"], "claude")
        self.assertEqual(decision["partner"], "codex")
        self.assertEqual(decision["agents"][0], "claude")

    def test_route_returns_assignments_dependencies_and_topology(self):
        decision = self.route()

        self.assertEqual(decision["assignments"]["debugging"], "codex")
        self.assertEqual(decision["assignments"]["planning"], "claude")
        self.assertEqual(decision["dependencies"], {"debugging": [], "planning": []})
        self.assertEqual(decision["topology"], [["claude", "codex"]])

    def test_unavailable_agent_is_not_selected(self):
        decision = self.route(
            constraints={
                "available_agents": ["claude"],
                "required_permissions": ["read", "workspace_write"],
            }
        )

        self.assertEqual(decision["mode"], "self")
        self.assertEqual(decision["agents"], ["claude"])

    def test_permission_budget_and_deadline_are_hard_constraints(self):
        for constraints in (
            {"available_agents": ["claude", "codex"], "required_permissions": ["admin"]},
            {"available_agents": ["claude", "codex"], "budget": 0.001},
            {"available_agents": ["claude", "codex"], "deadline_ms": 1},
        ):
            with self.subTest(constraints=constraints):
                with self.assertRaises(RuntimeError):
                    self.route(constraints=constraints)

    def test_simple_task_keeps_a_single_model_executor(self):
        decision = self.route(
            prompt="解释这个概念",
            constraints={
                "available_agents": ["claude", "codex"],
                "model_catalog": self.MODEL_CATALOG,
                "incumbent_model": "opus[1m]",
            },
        )

        self.assertEqual(decision["complexity"]["label"], "simple")
        self.assertEqual(decision["team_limit"], 1)
        self.assertEqual(decision["team_size"], 1)
        self.assertIn(decision["primary"], decision["executors"])
        self.assertEqual(decision["primary_effort"], "low")
        self.assertTrue(all(effort == "low" for effort in decision["efforts"].values()))

    def test_effort_selection_respects_model_support_and_product_caps(self):
        very_complex = {"label": "very_complex"}
        sol = sage_bridge._model_profile("codex", "gpt-5.6-sol", 1_050_000)
        sol["supported_efforts"] = ["low", "medium", "high", "xhigh", "max"]
        opus = sage_bridge._model_profile("claude", "opus[1m]", 1_000_000)
        opus["supported_efforts"] = ["low", "medium", "high", "xhigh"]

        for model in ("gpt-5.6-luna", "gpt-5.4-mini", "gpt-5.3-codex-spark"):
            with self.subTest(model=model):
                profile = sage_bridge._model_profile("codex", model, 1_050_000)
                profile["supported_efforts"] = ["low", "medium", "high", "xhigh", "max"]
                self.assertEqual(
                    sage_bridge.requirement_effort(profile, very_complex, "coding"), "max"
                )

        limited_mini = sage_bridge._model_profile("codex", "limited-mini", 400_000)
        limited_mini["supported_efforts"] = ["low", "medium", "high", "xhigh"]
        self.assertEqual(
            sage_bridge.requirement_effort(limited_mini, very_complex, "coding"), "xhigh"
        )
        self.assertEqual(sage_bridge.select_supported_effort(sol, "max"), "xhigh")
        self.assertEqual(sage_bridge.requirement_effort(sol, very_complex, "coding"), "xhigh")
        self.assertEqual(sage_bridge.requirement_effort(opus, very_complex, "planning"), "xhigh")

    def test_every_codex_executor_is_marked_fast(self):
        profiles, _ = sage_bridge.build_executor_profiles(
            {"model_catalog": self.MODEL_CATALOG, "incumbent_model": "opus[1m]"},
            "claude",
        )

        self.assertTrue(all(
            profile["fast"] for profile in profiles.values() if profile["runtime"] == "codex"
        ))
        self.assertTrue(all(
            not profile["fast"] for profile in profiles.values() if profile["runtime"] == "claude"
        ))

    def test_complex_task_routes_to_model_executor_pool(self):
        decision = self.route(
            prompt="规划设计并实现功能，调试错误，重构优化，安全审查，分析图片并写文档",
            constraints={
                "available_agents": ["claude", "codex"],
                "model_catalog": self.MODEL_CATALOG,
                "incumbent_model": "opus[1m]",
                "max_team_size": 5,
            },
        )

        self.assertEqual(decision["complexity"]["label"], "very_complex")
        self.assertGreaterEqual(decision["team_limit"], 3)
        self.assertGreater(decision["team_size"], 1)
        self.assertLessEqual(decision["team_size"], decision["team_limit"])
        self.assertTrue(all(agent in decision["executors"] for agent in decision["agents"]))
        self.assertTrue(
            all(decision["executors"][agent]["model"] for agent in decision["agents"])
        )
        self.assertTrue(
            all(assignee in decision["executors"] for assignee in decision["assignments"].values())
        )
        self.assertTrue(set(decision["partners"]).issubset(set(decision["assignments"].values())))
        for requirement, effort in decision["efforts"].items():
            executor = decision["assignments"][requirement]
            self.assertIn(effort, decision["executors"][executor]["supported_efforts"])
        self.assertIn(decision["summary_effort"], decision["executors"][decision["primary"]]["supported_efforts"])
        self.assertEqual(decision["primary_effort"], "xhigh")
        self.assertEqual(decision["efforts"]["coding"], "xhigh")
        self.assertEqual(decision["efforts"]["debugging"], "xhigh")
        self.assertEqual(decision["efforts"]["docs"], "high")
        if decision["mode"] == "collaborate":
            self.assertEqual(decision["primary"], "claude::opus[1m]")

    def test_model_executor_outcome_updates_selected_profiles(self):
        decision = self.route(
            prompt="规划设计并实现功能，调试错误，重构优化，安全审查，分析图片并写文档",
            constraints={
                "available_agents": ["claude", "codex"],
                "model_catalog": self.MODEL_CATALOG,
                "incumbent_model": "opus[1m]",
                "max_team_size": 5,
            },
        )
        state = {}
        scores = {agent: 1 for agent in decision["agents"]}

        result = sage_bridge.cmd_outcome(
            {
                "decision_blob": decision["decision_blob"],
                "success": 1,
                "agent_scores": scores,
                "requirement_scores": {name: 1 for name in decision["assignments"]},
                "persist": False,
            },
            state,
        )

        self.assertTrue(result["ok"])
        self.assertEqual(set(state["stats"]), set(decision["agents"]))

    def test_live_state_is_forwarded_to_the_router(self):
        decision = self.route(
            state={
                "active_agents": ["claude", "codex"],
                "active_mode": "collaborate",
                "completed_requirements": [],
                "progress": 0.4,
                "transferable_context": 0.9,
                "failed_agents": ["codex"],
                "failure_count": 1,
            }
        )

        self.assertNotIn("codex", decision["agents"])
        self.assertTrue(decision["switch_recommended"])

    def test_team_outcome_uses_agent_scores_without_polluting_member_latency(self):
        decision = self.route()
        state = {}

        result = sage_bridge.cmd_outcome(
            {
                "decision_blob": decision["decision_blob"],
                "success": 0.5,
                "agent_scores": {"claude": 1, "codex": 0},
                "requirement_scores": {"planning": 1, "debugging": 0},
                "actual_latency_ms": 999999,
                "persist": False,
            },
            state,
        )

        self.assertTrue(result["ok"])
        claude_effort = decision["executors"]["claude"]["effort"]
        codex_effort = decision["executors"]["codex"]["effort"]
        claude_stat = state["stats"]["claude"]["efforts"][claude_effort]
        codex_stat = state["stats"]["codex"]["efforts"][codex_effort]
        self.assertGreater(claude_stat["success"], codex_stat["success"])
        self.assertNotIn("latency_ms", claude_stat)
        self.assertNotIn("latency_ms", codex_stat)

    def test_legacy_learning_state_is_backed_up_and_reset(self):
        with tempfile.TemporaryDirectory() as tmp:
            original = sage_bridge.STATE_PATH
            try:
                sage_bridge.STATE_PATH = str(Path(tmp) / "sage_state.json")
                Path(sage_bridge.STATE_PATH).write_text(
                    '{"stats":{"claude":{"success":1}}}', encoding="utf-8"
                )

                state = sage_bridge.load_state()

                self.assertEqual(state, {"_schema": sage_bridge.STATE_SCHEMA})
                self.assertTrue(
                    Path(sage_bridge.STATE_PATH + f".pre-schema-v{sage_bridge.STATE_SCHEMA}").is_file()
                )
            finally:
                sage_bridge.STATE_PATH = original

    def test_keywords_do_not_misread_plain_questions(self):
        """「写」曾是单字关键词，把纯提问判成写代码任务；复查/单元测试则整个漏掉。"""
        asking, matched = sage_bridge.infer_requirements("这段代码是怎么写的")
        self.assertNotEqual(asking, {"coding": 1.0})
        writing, _ = sage_bridge.infer_requirements("帮我写个导出脚本")
        self.assertIn("coding", writing)
        review, _ = sage_bridge.infer_requirements("做一轮安全复查")
        self.assertIn("review", review)
        tests, _ = sage_bridge.infer_requirements("补齐单元测试")
        self.assertIn("coding", tests)

    def test_unrecognised_prompt_falls_back_to_incumbent_strengths(self):
        """认不出任务类型时不该替用户换人：需求取在任执行者的强项，让效用比较落回 SELF。

        旧实现兜底成 {coding, analysis}，而 analysis 上 claude 天然占优，
        导致「这个字段是干嘛的」这种没有技术动词的随口一问也会被移交出去。
        """
        profiles = sage_bridge._legacy_profiles()
        for runtime in ("codex", "claude"):
            weights, matched = sage_bridge.infer_requirements(
                "这个字段是干嘛的", profiles[runtime]["skills"]
            )
            self.assertFalse(matched, "该 prompt 本就不该命中任何关键词")
            best = max(profiles[runtime]["skills"], key=profiles[runtime]["skills"].get)
            self.assertIn(best, weights, f"{runtime} 的兜底需求应包含它自己的强项")

    def test_live_session_state_damps_drift(self):
        """上游 ALGORITHM.md：积累了不可转移的工作之后，重新规划应该变难。

        app.js 曾对没有路由记录的会话传 active_agents=[]（命中上游
        「无 active route」分支，移交成本归零）外加 progress=0 /
        transferable_context=1，三者叠加把切换阻尼整个关掉——这才是
        每轮追问都换模型的根因。
        """
        def route(prompt, active_agents, progress, transferable):
            return self.route(
                prompt=prompt,
                incumbent="codex",
                constraints={
                    "available_agents": ["claude", "codex"],
                    "model_catalog": self.MODEL_CATALOG,
                    "incumbent_model": "gpt-5.6-sol",
                    "max_team_size": 5,
                },
                state={
                    "active_agents": active_agents,
                    "active_mode": "self",
                    "progress": progress,
                    "transferable_context": transferable,
                    "failed_agents": [],
                    "failure_count": 0,
                },
            )

        # 旧行为：会话在跑却宣称无人在任 + 毫无积累 + 上下文可无损转移 → 拱手让人
        drifted = route("整理并输出一份架构设计方案", [], 0, 1)
        self.assertEqual(drifted["mode"], "handoff")
        self.assertFalse(
            drifted["primary"].startswith("codex"), "旧参数下应当复现漂移：所有权被交出去"
        )

        # 修复后：在任执行者如实上报，进度与可转移度交给上游默认值 → 保住所有权
        damped = route("整理并输出一份架构设计方案", ["codex"], 0.5, None)
        self.assertTrue(
            damped["primary"].startswith("codex"), "如实上报会话状态后，在任执行者应当保住所有权"
        )

        # 但阻尼不是一刀切：短板悬殊、移交收益明确时仍然要移交，
        # 否则就从「乱换人」矫枉过正成「永远不换人」。
        docs = route("写一份项目说明文档", ["codex"], 0.5, None)
        self.assertEqual(docs["mode"], "handoff")
        self.assertTrue(docs["primary"].startswith("claude"))


if __name__ == "__main__":
    unittest.main()

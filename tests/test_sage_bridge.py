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


if __name__ == "__main__":
    unittest.main()

//! SAGE 智能路由：调用 vendor/sprix-sage-router（MIT，Python 零依赖）在
//! Claude Code 与 Codex 之间做 SELF / COLLABORATE / HANDOFF 决策。
//! 脚本编译期内嵌，首次调用落到临时目录执行，保持单二进制交付。

use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::OnceLock;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const SPRIX_SAGE: &str = include_str!("../vendor/sprix-sage-router/sprix_sage.py");
const BRIDGE: &str = include_str!("../vendor/sprix-sage-router/sage_bridge.py");

fn ensure_scripts() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("agent-hub-sage");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 SAGE 目录失败: {e}"))?;
    for (name, content) in [("sprix_sage.py", SPRIX_SAGE), ("sage_bridge.py", BRIDGE)] {
        let p = dir.join(name);
        let stale = std::fs::read_to_string(&p)
            .map(|c| c != content)
            .unwrap_or(true);
        if stale {
            std::fs::write(&p, content).map_err(|e| format!("写入 {name} 失败: {e}"))?;
        }
    }
    Ok(dir)
}

fn find_python() -> Option<String> {
    for cand in ["python", "python3", "py"] {
        if let Ok(out) = std::process::Command::new("where").arg(cand).output() {
            if out.status.success() {
                return Some(cand.to_string());
            }
        }
    }
    None
}

pub async fn route(
    prompt: &str,
    incumbent: &str,
    failed: &[String],
    unavailable: &[String],
    state: Value,
    constraints: Value,
) -> Result<Value, String> {
    call(json!({
        "cmd": "route",
        "prompt": prompt,
        "incumbent": incumbent,
        "failed": failed,
        "unavailable": unavailable,
        "state": state,
        "constraints": constraints,
    }))
    .await
}

/// 执行结束后回喂证据（record_outcome），驱动 SAGE 的在线学习闭环。
/// 分工模式可附带按 agent / 按需求的细粒度评分。
pub async fn outcome(
    decision_blob: Value,
    success: f64,
    actual_cost: Option<f64>,
    actual_latency_ms: Option<f64>,
    agent_scores: Option<Value>,
    requirement_scores: Option<Value>,
) -> Result<Value, String> {
    call(json!({
        "cmd": "outcome",
        "decision_blob": decision_blob,
        "success": success,
        "actual_cost": actual_cost,
        "actual_latency_ms": actual_latency_ms,
        "agent_scores": agent_scores,
        "requirement_scores": requirement_scores,
    }))
    .await
}

async fn call(payload: Value) -> Result<Value, String> {
    let dir = ensure_scripts()?;
    static PY: OnceLock<Option<String>> = OnceLock::new();
    let py = PY
        .get_or_init(find_python)
        .clone()
        .ok_or_else(|| "未找到 python（SAGE 路由需要本机 Python 3.10+）".to_string())?;

    let mut child = Command::new(&py)
        .arg(dir.join("sage_bridge.py"))
        .env("PYTHONIOENCODING", "utf-8")
        .env("PYTHONUTF8", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("启动 python 失败: {e}"))?;

    let payload = payload.to_string();
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(payload.as_bytes()).await;
        let _ = stdin.shutdown().await;
    }
    let out = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        child.wait_with_output(),
    )
    .await
    .map_err(|_| "SAGE 路由超时".to_string())?
    .map_err(|e| format!("python 运行失败: {e}"))?;

    if !out.status.success() {
        let err: String = String::from_utf8_lossy(&out.stderr).chars().take(300).collect();
        return Err(format!("SAGE 桥接失败: {err}"));
    }
    serde_json::from_slice::<Value>(&out.stdout).map_err(|e| format!("解析路由决策失败: {e}"))
}

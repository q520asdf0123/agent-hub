//! CLI 探测与真实可执行解析（CONTRACT §3.1）。
//!
//! PATH 上的 `claude`/`codex` 是 npm `.cmd` shim，Rust CreateProcessW 不能执行 .cmd，
//! 因此用 `where` 逐行解析：`.exe` 行直接作候选；`.cmd` 行按 npm 全局目录布局推导真实 .exe。

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;
use tokio::sync::OnceCell;

use crate::types::{CliStatus, StatusResp};

#[derive(Clone, Debug)]
pub struct ResolvedCli {
    pub exe: PathBuf,
    pub version: Option<String>,
}

static CLAUDE_CELL: OnceCell<Option<ResolvedCli>> = OnceCell::const_new();
static CODEX_CELL: OnceCell<Option<ResolvedCli>> = OnceCell::const_new();
static STATUS_CELL: OnceCell<StatusResp> = OnceCell::const_new();

/// 解析 agent（"claude" | "codex"）的真实可执行文件；结果缓存，首次调用探测。
pub async fn resolve(agent: &str) -> Option<ResolvedCli> {
    let cell = match agent {
        "claude" => &CLAUDE_CELL,
        "codex" => &CODEX_CELL,
        _ => return None,
    };
    let agent = agent.to_string();
    cell.get_or_init(|| async move { probe(&agent).await })
        .await
        .clone()
}

/// 双 CLI 安装状态；结果缓存 OnceCell，首次调用探测。
pub async fn status() -> StatusResp {
    STATUS_CELL
        .get_or_init(|| async {
            let (claude, codex) = tokio::join!(resolve("claude"), resolve("codex"));
            StatusResp {
                claude: to_status(claude),
                codex: to_status(codex),
            }
        })
        .await
        .clone()
}

fn to_status(r: Option<ResolvedCli>) -> CliStatus {
    match r {
        Some(c) => CliStatus {
            installed: true,
            version: c.version.clone(),
            path: Some(c.exe.to_string_lossy().to_string()),
            error: None,
        },
        None => CliStatus {
            installed: false,
            version: None,
            path: None,
            error: Some("未找到可执行文件".to_string()),
        },
    }
}

async fn probe(agent: &str) -> Option<ResolvedCli> {
    let candidates = find_candidates(agent).await;
    let mut first_existing: Option<PathBuf> = None;
    for c in candidates {
        if !c.is_file() {
            continue;
        }
        if first_existing.is_none() {
            first_existing = Some(c.clone());
        }
        // 对存在的候选跑 --version（3 秒超时），成功者胜出并记录版本。
        if let Some(v) = try_version(&c).await {
            return Some(ResolvedCli {
                exe: c,
                version: Some(v),
            });
        }
    }
    // 存在但 --version 全失败：仍返回第一个存在的候选（版本未知）。
    first_existing.map(|exe| ResolvedCli { exe, version: None })
}

/// `where <agent>` 输出逐行推导候选 .exe（保持 PATH 顺序）。
async fn find_candidates(agent: &str) -> Vec<PathBuf> {
    let output = match Command::new("where")
        .arg(agent)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    if !output.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let mut out: Vec<PathBuf> = Vec::new();
    for raw in text.lines() {
        // where 输出为 CRLF 行尾，trim 去掉 \r 与空白。
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let lower = line.to_ascii_lowercase();
        if lower.ends_with(".exe") {
            out.push(PathBuf::from(line));
        } else if lower.ends_with(".cmd") {
            let Some(dir) = Path::new(line).parent() else {
                continue;
            };
            match agent {
                "claude" => {
                    out.push(dir.join(r"node_modules\@anthropic-ai\claude-code\bin\claude.exe"));
                }
                "codex" => {
                    out.push(dir.join(
                        r"node_modules\@openai\codex\node_modules\@openai\codex-win32-x64\vendor\x86_64-pc-windows-msvc\bin\codex.exe",
                    ));
                    // 其次：D/node_modules/@openai/codex/bin/ 下任意 codex*.exe
                    let bin = dir.join(r"node_modules\@openai\codex\bin");
                    if let Ok(rd) = std::fs::read_dir(&bin) {
                        let mut exes: Vec<PathBuf> = rd
                            .filter_map(|e| e.ok())
                            .map(|e| e.path())
                            .filter(|p| {
                                p.file_name()
                                    .map(|n| {
                                        let n = n.to_string_lossy().to_ascii_lowercase();
                                        n.starts_with("codex") && n.ends_with(".exe")
                                    })
                                    .unwrap_or(false)
                            })
                            .collect();
                        exes.sort();
                        out.extend(exes);
                    }
                }
                _ => {}
            }
        }
    }
    out
}

/// 跑 `<exe> --version`（3 秒超时）；成功返回首行输出。
async fn try_version(exe: &Path) -> Option<String> {
    let fut = Command::new(exe)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output();
    let out = tokio::time::timeout(Duration::from_secs(3), fut)
        .await
        .ok()?
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().next().unwrap_or("").trim();
    if line.is_empty() {
        None
    } else {
        Some(line.to_string())
    }
}

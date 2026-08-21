//! 模型自动发现：从本地 CLI 配置与最近会话历史收集可用模型名。
//! - claude：~/.claude/settings.json 的 model（全局默认）+ 最近会话 assistant 行的 model + 官方别名
//! - codex：~/.codex/config.toml 的 model（默认）与 model_catalog_json 目录 + 最近会话 turn_context 的 model

use crate::types::{AgentModels, ModelsResp};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const MAX_MODELS: usize = 15;
const RECENT_FILES: usize = 15;

pub fn discover() -> ModelsResp {
    ModelsResp {
        claude: claude_models(),
        codex: codex_models(),
    }
}

fn claude_models() -> AgentModels {
    let mut default = None;
    let mut models: Vec<String> = Vec::new();
    if let Some(h) = dirs::home_dir() {
        if let Ok(raw) = fs::read_to_string(h.join(".claude").join("settings.json")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(m) = v.get("model").and_then(|m| m.as_str()) {
                    default = Some(m.to_string());
                    push_unique(&mut models, m);
                }
            }
        }
        // 最近会话文件里 assistant 行的 "model" 字段（按 mtime 取最近若干个）
        let mut files: Vec<(PathBuf, SystemTime)> = Vec::new();
        if let Ok(rd) = fs::read_dir(h.join(".claude").join("projects")) {
            for proj in rd.flatten() {
                let Ok(fd) = fs::read_dir(proj.path()) else { continue };
                for f in fd.flatten() {
                    let p = f.path();
                    if p.extension().map(|e| e == "jsonl").unwrap_or(false) {
                        if let Some(mt) = f.metadata().ok().and_then(|m| m.modified().ok()) {
                            files.push((p, mt));
                        }
                    }
                }
            }
        }
        files.sort_by(|a, b| b.1.cmp(&a.1));
        for (p, _) in files.into_iter().take(RECENT_FILES) {
            for m in scan_model_fields(&p, 150) {
                push_unique(&mut models, &m);
            }
        }
    }
    // 官方别名兜底
    for alias in ["fable", "opus", "sonnet", "haiku"] {
        push_unique(&mut models, alias);
    }
    models.truncate(MAX_MODELS + 4);
    // effortLevel 合法值来自 CLI 内置校验（zod enum）；默认值读全局 settings.json
    let default_effort = dirs::home_dir()
        .and_then(|h| fs::read_to_string(h.join(".claude").join("settings.json")).ok())
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|v| v.get("effortLevel").and_then(|e| e.as_str()).map(String::from));
    AgentModels {
        default,
        models,
        efforts: ["low", "medium", "high", "xhigh"]
            .map(String::from)
            .to_vec(),
        default_effort,
        context_window: None, // 前端按模型名推断（[1m] → 1M，其余 200k）
    }
}

/// canonical 思考等级顺序（codex 目录里出现的等级按此排序）
const EFFORT_ORDER: [&str; 7] = ["minimal", "low", "medium", "high", "xhigh", "max", "ultra"];

fn codex_models() -> AgentModels {
    let mut default = None;
    let mut default_effort = None;
    let mut context_window: Option<i64> = None;
    let mut models: Vec<String> = Vec::new();
    let mut effort_set: Vec<String> = Vec::new();
    let Some(h) = dirs::home_dir() else {
        return AgentModels {
            default,
            models,
            efforts: Vec::new(),
            default_effort,
            context_window: None,
        };
    };
    let codex = h.join(".codex");
    let mut catalog_path: Option<PathBuf> = None;
    if let Ok(raw) = fs::read_to_string(codex.join("config.toml")) {
        for line in raw.lines() {
            let t = line.trim();
            if let Some(v) = toml_str_value(t, "model") {
                if default.is_none() {
                    default = Some(v.clone());
                }
                push_unique(&mut models, &v);
            } else if let Some(v) = toml_str_value(t, "model_catalog_json") {
                catalog_path = Some(PathBuf::from(v));
            } else if let Some(v) = toml_str_value(t, "model_reasoning_effort") {
                default_effort = Some(v);
            } else if let Some(rest) = t.strip_prefix("model_context_window") {
                if let Some(v) = rest.trim_start().strip_prefix('=') {
                    context_window = v.trim().parse::<i64>().ok();
                }
            }
        }
    }
    // 模型目录（自定义 provider 的完整模型清单 + 各模型支持的思考等级）
    if let Some(cp) = catalog_path {
        if let Ok(raw) = fs::read_to_string(&cp) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(arr) = v.get("models").and_then(|m| m.as_array()) {
                    for m in arr {
                        if let Some(slug) = m.get("slug").and_then(|s| s.as_str()) {
                            push_unique(&mut models, slug);
                        }
                        if let Some(levels) =
                            m.get("supported_reasoning_levels").and_then(|l| l.as_array())
                        {
                            for lv in levels {
                                if let Some(e) = lv.get("effort").and_then(|e| e.as_str()) {
                                    push_unique(&mut effort_set, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // 最近 rollout 的 turn_context.model
    let mut files: Vec<(PathBuf, SystemTime)> = Vec::new();
    collect_rollouts(&codex.join("sessions"), 0, &mut files);
    files.sort_by(|a, b| b.1.cmp(&a.1));
    for (p, _) in files.into_iter().take(RECENT_FILES) {
        for m in scan_model_fields(&p, 60) {
            push_unique(&mut models, &m);
        }
    }
    models.truncate(MAX_MODELS);
    // 目录未提供等级时用官方标准集兜底；统一按 canonical 顺序输出
    let efforts: Vec<String> = if effort_set.is_empty() {
        ["minimal", "low", "medium", "high", "xhigh"]
            .map(String::from)
            .to_vec()
    } else {
        EFFORT_ORDER
            .iter()
            .filter(|e| effort_set.iter().any(|x| x == *e))
            .map(|e| e.to_string())
            .collect()
    };
    AgentModels {
        default,
        models,
        efforts,
        default_effort,
        context_window,
    }
}

/// 解析一行 `key = "value"` / `key = 'value'`；键必须精确匹配（防 model_provider 误配）。
fn toml_str_value(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?;
    let v = rest.trim().trim_matches(|c| c == '"' || c == '\'').trim();
    if v.is_empty() {
        None
    } else {
        Some(v.to_string())
    }
}

fn collect_rollouts(dir: &Path, depth: u32, out: &mut Vec<(PathBuf, SystemTime)>) {
    if depth > 3 {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else { return };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            collect_rollouts(&p, depth + 1, out);
        } else if p
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("rollout-") && n.ends_with(".jsonl"))
            .unwrap_or(false)
        {
            if let Some(mt) = ent.metadata().ok().and_then(|m| m.modified().ok()) {
                out.push((p, mt));
            }
        }
    }
}

/// 逐行（字节安全）扫描前 max_lines 行里的 "model":"xxx" / "model": "xxx"。
fn scan_model_fields(path: &Path, max_lines: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let Ok(f) = fs::File::open(path) else {
        return out;
    };
    let reader = BufReader::new(f);
    let mut n = 0usize;
    for seg in reader.split(b'\n') {
        let Ok(bytes) = seg else { break };
        n += 1;
        if n > max_lines {
            break;
        }
        let s = String::from_utf8_lossy(&bytes);
        let mut rest: &str = &s;
        while let Some(i) = rest.find("\"model\":") {
            rest = &rest[i + 8..];
            let after = rest.trim_start();
            let Some(q) = after.strip_prefix('"') else { continue };
            let Some(j) = q.find('"') else { break };
            let m = &q[..j];
            if !m.is_empty() && m != "<synthetic>" {
                push_unique(&mut out, m);
            }
            rest = &q[j..];
        }
    }
    out
}

fn push_unique(v: &mut Vec<String>, m: &str) {
    if !v.iter().any(|x| x == m) {
        v.push(m.to_string());
    }
}

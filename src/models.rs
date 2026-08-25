//! 模型自动发现：从本地 CLI 配置与最近会话历史收集可用模型名。
//! - claude：~/.claude/settings.json 的 model（全局默认）+ 最近会话 assistant 行的 model + 官方别名
//! - codex：~/.codex/config.toml 的 model（默认）与 model_catalog_json 目录 + 最近会话 turn_context 的 model
//!
//! codex 侧还会用 provider 网关的 `/models` 清单做一次交叉过滤：model_catalog_json 是
//! 官方全量目录，而自建 / 中转 provider 通常只代理其中一部分，不过滤会把网关根本不认识
//! 的型号交给 SAGE 分配，直到运行期才 400 model_not_found。

use crate::types::{AgentModels, ModelsResp};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

const MAX_MODELS: usize = 15;
const RECENT_FILES: usize = 15;
/// 网关清单缓存时长：模型上下线很少，避免每次路由都打一次网关。
const GATEWAY_TTL: Duration = Duration::from_secs(600);
/// 网关探测超时（秒）；超时即降级为「不过滤」，不阻塞模型发现。
const GATEWAY_TIMEOUT_SECS: u64 = 4;

/// 用户手动排除的模型清单：provider 提供、但不希望被 SAGE 分配或在前端选中的型号
/// （通常是被新版本取代的旧档位）。与网关过滤是两回事——那个滤「不可用」，这个滤「不想用」。
const EXCLUDE_FILE: &str = "model-exclude.json";

pub fn discover() -> ModelsResp {
    ModelsResp {
        claude: claude_models(),
        codex: codex_models(),
    }
}

/// 读 ~/.agenthub/model-exclude.json。支持两种写法：
/// `["gpt-5.5"]`（对所有 runtime 生效）或 `{"codex":["gpt-5.5"],"claude":[]}`（按 runtime）。
/// 每次调用都重读，改完不用重启。文件缺失或格式不对即视为空清单。
fn excluded_models(runtime: &str) -> HashSet<String> {
    let Some(path) = dirs::home_dir().map(|h| h.join(".agenthub").join(EXCLUDE_FILE)) else {
        return HashSet::new();
    };
    let Ok(raw) = fs::read_to_string(path) else {
        return HashSet::new();
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return HashSet::new();
    };
    let list = match v.get(runtime) {
        Some(per_runtime) => per_runtime.as_array().cloned().unwrap_or_default(),
        None => v.as_array().cloned().unwrap_or_default(),
    };
    list.iter()
        .filter_map(|m| m.as_str())
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .collect()
}

/// 应用排除名单。整份列表被排空说明名单写过了头，退回原列表——
/// 让某个 runtime 一个模型都选不了不是这个开关该干的事（那是 available_agents 的职责）。
fn apply_exclusions(models: &[String], excluded: &HashSet<String>) -> Vec<String> {
    if excluded.is_empty() {
        return models.to_vec();
    }
    let kept: Vec<String> = models
        .iter()
        .filter(|m| !excluded.contains(m.as_str()))
        .cloned()
        .collect();
    if kept.is_empty() {
        models.to_vec()
    } else {
        kept
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
    // 官方在售完整模型名兜底（本地未用过也可选）
    push_unique(&mut models, "claude-sonnet-5");
    models = apply_exclusions(&models, &excluded_models("claude"));
    // 默认模型被排除时改指列表首位，避免前端默认值落在候选之外
    if let Some(d) = default.clone() {
        if !models.iter().any(|m| *m == d) {
            default = models.first().cloned();
        }
    }
    models.truncate(MAX_MODELS + 4);
    // effortLevel 合法值来自 CLI 内置校验（zod enum）；默认值读全局 settings.json
    let default_effort = dirs::home_dir()
        .and_then(|h| fs::read_to_string(h.join(".claude").join("settings.json")).ok())
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|v| v.get("effortLevel").and_then(|e| e.as_str()).map(String::from));
    // 窗口按官方口径（2026-03 起 1M GA）：Fable/Opus/Sonnet 家族 1M，Haiku 200k
    let windows = models
        .iter()
        .map(|m| {
            let w = if m.to_lowercase().contains("haiku") {
                200_000_i64
            } else {
                1_000_000_i64
            };
            (m.clone(), w)
        })
        .collect();
    AgentModels {
        default,
        models,
        efforts: ["low", "medium", "high", "xhigh"]
            .map(String::from)
            .to_vec(),
        default_effort,
        context_window: None, // 前端按模型名推断（[1m] → 1M，其余 200k）
        windows,
        service_tier: None,
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
    let mut windows = std::collections::HashMap::new();
    let mut service_tier = None;
    let Some(h) = dirs::home_dir() else {
        return AgentModels {
            default,
            models,
            efforts: Vec::new(),
            default_effort,
            context_window: None,
            windows,
            service_tier,
        };
    };
    let codex = h.join(".codex");
    let mut catalog_path: Option<PathBuf> = None;
    let mut provider_name: Option<String> = None;
    let mut providers: HashMap<String, ProviderCfg> = HashMap::new();
    if let Ok(raw) = fs::read_to_string(codex.join("config.toml")) {
        // 顶层键与 [model_providers.x] 段要分开读：config.toml 里 [projects.'…']、
        // [tui.*] 等段落同名键不少，逐行全扫会误配。
        let mut section = String::new();
        for line in raw.lines() {
            let t = line.trim();
            if let Some(name) = t.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                section = name.trim().to_string();
                continue;
            }
            if let Some(name) = section.strip_prefix("model_providers.") {
                let cfg = providers
                    .entry(name.trim_matches(|c| c == '"' || c == '\'').to_string())
                    .or_default();
                if let Some(v) = toml_str_value(t, "base_url") {
                    cfg.base_url = Some(v);
                } else if let Some(v) = toml_str_value(t, "env_key") {
                    cfg.env_key = Some(v);
                }
                continue;
            }
            if !section.is_empty() {
                continue;
            }
            if let Some(v) = toml_str_value(t, "model") {
                if default.is_none() {
                    default = Some(v.clone());
                }
                push_unique(&mut models, &v);
            } else if let Some(v) = toml_str_value(t, "model_provider") {
                provider_name = Some(v);
            } else if let Some(v) = toml_str_value(t, "model_catalog_json") {
                catalog_path = Some(PathBuf::from(v));
            } else if let Some(v) = toml_str_value(t, "model_reasoning_effort") {
                default_effort = Some(v);
            } else if let Some(v) = toml_str_value(t, "service_tier") {
                // TUI /fast 持久化的全局速度档，作为前端快速开关的初始态
                service_tier = Some(v);
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
                            if let Some(w) = m.get("context_window").and_then(|w| w.as_i64()) {
                                windows.insert(slug.to_string(), w);
                            }
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
    // provider 网关交叉过滤：目录与历史里的型号未必被当前 provider 代理。
    if let Some(kept) = provider_name
        .as_deref()
        .and_then(|name| providers.get(name))
        .and_then(|cfg| gateway_models(&codex, cfg))
        .and_then(|serviceable| filter_by_gateway(&models, default.as_deref(), &serviceable))
    {
        windows.retain(|slug, _| kept.iter().any(|m| m == slug));
        models = kept;
    }
    // 用户排除名单：网关提供、但不想再被分配到的旧档位
    let kept = apply_exclusions(&models, &excluded_models("codex"));
    if kept.len() != models.len() {
        windows.retain(|slug, _| kept.iter().any(|m| m == slug));
        models = kept;
    }
    if let Some(d) = default.clone() {
        if !models.iter().any(|m| *m == d) {
            default = models.first().cloned();
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
    // config.toml 的全局 model_context_window 补给默认模型（目录值优先）
    if let (Some(w), Some(d)) = (context_window, default.as_ref()) {
        windows.entry(d.clone()).or_insert(w);
    }
    AgentModels {
        default,
        models,
        efforts,
        default_effort,
        context_window,
        windows,
        service_tier,
    }
}

/// 用网关清单过滤候选模型。默认模型是用户显式配置的，即使网关未列出也保留。
/// 全部被滤掉说明网关清单与本机口径对不上（换了鉴权、网关只暴露别名等），返回 None
/// 让调用方保持原列表——宁可留下几个不可用型号，也不能把可选项清空。
fn filter_by_gateway(
    models: &[String],
    default: Option<&str>,
    serviceable: &HashSet<String>,
) -> Option<Vec<String>> {
    let kept: Vec<String> = models
        .iter()
        .filter(|m| serviceable.contains(m.as_str()) || default == Some(m.as_str()))
        .cloned()
        .collect();
    if kept.is_empty() {
        None
    } else {
        Some(kept)
    }
}

/// config.toml 里 `[model_providers.x]` 段中与网关探测相关的字段。
#[derive(Default)]
struct ProviderCfg {
    base_url: Option<String>,
    env_key: Option<String>,
}

/// provider 凭据：优先 env_key 指定的环境变量，其次 ~/.codex/auth.json（apikey 与 OAuth 两种模式）。
fn provider_key(codex: &Path, cfg: &ProviderCfg) -> Option<String> {
    if let Some(env_key) = cfg.env_key.as_deref() {
        if let Ok(v) = std::env::var(env_key) {
            if !v.trim().is_empty() {
                return Some(v.trim().to_string());
            }
        }
    }
    let raw = fs::read_to_string(codex.join("auth.json")).ok()?;
    let v = serde_json::from_str::<serde_json::Value>(&raw).ok()?;
    v.get("OPENAI_API_KEY")
        .and_then(|k| k.as_str())
        .or_else(|| v.pointer("/tokens/access_token").and_then(|k| k.as_str()))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// 网关实际代理的模型清单（OpenAI 兼容 `GET {base_url}/models`）。
/// 按 base_url 进程内缓存 GATEWAY_TTL；失败结果同样缓存，避免每次发现都吃一次超时。
/// 任何一步失败都返回 None，调用方据此降级为不过滤。
fn gateway_models(codex: &Path, cfg: &ProviderCfg) -> Option<HashSet<String>> {
    let base = cfg.base_url.as_deref()?.trim_end_matches('/').to_string();
    static CACHE: OnceLock<Mutex<HashMap<String, (Instant, Option<HashSet<String>>)>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(map) = cache.lock() {
        if let Some((at, hit)) = map.get(&base) {
            if at.elapsed() < GATEWAY_TTL {
                return hit.clone();
            }
        }
    }
    let fresh = provider_key(codex, cfg).and_then(|key| fetch_gateway_models(&base, &key));
    if let Ok(mut map) = cache.lock() {
        map.insert(base, (Instant::now(), fresh.clone()));
    }
    fresh
}

/// 用 curl 拉一次网关模型清单。凭据经 stdin 的 curl 配置传入，不进 argv（避免出现在进程列表）。
fn fetch_gateway_models(base: &str, key: &str) -> Option<HashSet<String>> {
    // curl 配置里是带引号的字符串字面量，含引号/反斜杠/换行的值无法安全转义，直接放弃探测
    let unquotable = |s: &str| s.contains(|c| matches!(c, '"' | '\\' | '\n' | '\r'));
    if unquotable(base) || unquotable(key) {
        return None;
    }
    let mut child = Command::new("curl")
        .args(["-s", "-K", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    if let Some(mut stdin) = child.stdin.take() {
        let cfg = format!(
            "url = \"{base}/models\"\nheader = \"Authorization: Bearer {key}\"\nmax-time = {GATEWAY_TIMEOUT_SECS}\n"
        );
        let _ = stdin.write_all(cfg.as_bytes());
    }
    let out = child.wait_with_output().ok()?;
    if !out.status.success() {
        return None;
    }
    let v = serde_json::from_slice::<serde_json::Value>(&out.stdout).ok()?;
    let arr = v.get("data").or_else(|| v.get("models"))?.as_array()?;
    let ids: HashSet<String> = arr
        .iter()
        .filter_map(|m| m.get("id").or_else(|| m.get("slug")).and_then(|s| s.as_str()))
        .map(String::from)
        .collect();
    if ids.is_empty() {
        None
    } else {
        Some(ids)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn vs(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn hs(items: &[&str]) -> HashSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn gateway_filter_drops_models_the_provider_does_not_serve() {
        let models = vs(&["gpt-5.6-sol", "gpt-5.2", "gpt-5.5"]);
        let kept = filter_by_gateway(&models, Some("gpt-5.6-sol"), &hs(&["gpt-5.6-sol", "gpt-5.5"]));
        assert_eq!(kept, Some(vs(&["gpt-5.6-sol", "gpt-5.5"])));
    }

    #[test]
    fn gateway_filter_keeps_the_configured_default_even_if_unlisted() {
        let models = vs(&["my-alias", "gpt-5.2"]);
        let kept = filter_by_gateway(&models, Some("my-alias"), &hs(&["gpt-5.6-sol"]));
        assert_eq!(kept, Some(vs(&["my-alias"])));
    }

    /// 网关清单与本机口径完全对不上时必须降级为「不过滤」，否则前端会一个模型都选不了。
    #[test]
    fn gateway_filter_degrades_instead_of_emptying_the_list() {
        let models = vs(&["gpt-5.6-sol", "gpt-5.2"]);
        assert_eq!(filter_by_gateway(&models, None, &hs(&["something-else"])), None);
        assert_eq!(filter_by_gateway(&models, None, &hs(&[])), None);
    }

    #[test]
    fn exclusions_drop_only_the_listed_models() {
        let models = vs(&["gpt-5.6-sol", "gpt-5.5", "gpt-5.4", "gpt-5.4-mini", "gpt-5.6-luna"]);
        let kept = apply_exclusions(&models, &hs(&["gpt-5.5", "gpt-5.4", "gpt-5.4-mini"]));
        assert_eq!(kept, vs(&["gpt-5.6-sol", "gpt-5.6-luna"]));
    }

    #[test]
    fn empty_exclusion_list_is_a_no_op() {
        let models = vs(&["gpt-5.6-sol", "gpt-5.5"]);
        assert_eq!(apply_exclusions(&models, &hs(&[])), models);
    }

    /// 名单写过头把整个 runtime 排空时退回原列表——禁用某个 agent 不是这个开关的职责。
    #[test]
    fn exclusions_degrade_instead_of_emptying_the_list() {
        let models = vs(&["gpt-5.6-sol", "gpt-5.5"]);
        assert_eq!(apply_exclusions(&models, &hs(&["gpt-5.6-sol", "gpt-5.5"])), models);
    }
}

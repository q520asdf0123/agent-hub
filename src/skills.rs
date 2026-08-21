//! 技能发现：扫描 Claude Code 的项目/用户/插件技能与自定义命令，以及
//! Codex 的自定义 prompts。技能通过在 prompt 开头写 `/名称` 调用，由 CLI 解析。

use crate::types::SkillInfo;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

const SOURCE_BUILTIN: &str = "内置命令";

pub fn list_skills(project: Option<&str>) -> Vec<SkillInfo> {
    let mut out: Vec<SkillInfo> = Vec::new();
    builtin_commands(&mut out);

    // 项目级优先（与 CLI 解析优先级一致：更具体的先注册，去重时保留）
    if let Some(p) = project.filter(|p| !p.trim().is_empty()) {
        let root = Path::new(p);
        scan_skill_dir(&root.join(".claude").join("skills"), "项目", None, &mut out);
        scan_command_dir(&root.join(".claude").join("commands"), "项目命令", &mut out);
    }
    if let Some(h) = dirs::home_dir() {
        scan_skill_dir(&h.join(".claude").join("skills"), "用户", None, &mut out);
        scan_command_dir(&h.join(".claude").join("commands"), "用户命令", &mut out);
        scan_plugins(&h.join(".claude").join("plugins"), &mut out);
        scan_codex_prompts(&h.join(".codex").join("prompts"), &mut out);
        scan_codex_skills(&h.join(".codex").join("skills"), &mut out);
    }

    let mut seen: HashSet<(String, String)> = HashSet::new();
    out.retain(|s| seen.insert((s.agent.clone(), s.invoke.clone())));
    // 内置命令置顶，其余按调用名排序
    out.sort_by(|a, b| {
        (a.source != SOURCE_BUILTIN, &a.invoke).cmp(&(b.source != SOURCE_BUILTIN, &b.invoke))
    });
    out
}

/// 无头模式下可用的 CLI 内置命令。
/// - claude：-p 模式会解析 prompt 开头的斜杠命令（init/review/security-review/compact 为 prompt 型命令）
/// - codex：exec 不解析斜杠命令；/review 由本应用映射为原生 `exec review` 子命令，/init 做 prompt 展开
fn builtin_commands(out: &mut Vec<SkillInfo>) {
    let claude: [(&str, &str); 4] = [
        ("/init", "分析项目并生成 / 更新 CLAUDE.md 指导文件"),
        ("/review", "审查代码改动 / PR"),
        ("/security-review", "对当前分支的改动做安全审查"),
        ("/compact", "压缩会话上下文（继续长会话前使用）"),
    ];
    for (inv, d) in claude {
        out.push(SkillInfo {
            agent: "claude".to_string(),
            invoke: inv.to_string(),
            name: inv.trim_start_matches('/').to_string(),
            description: d.to_string(),
            source: SOURCE_BUILTIN.to_string(),
        });
    }
    let codex: [(&str, &str); 2] = [
        (
            "/review",
            "代码审查（原生 exec review，默认审查未提交改动；可在命令后附自定义审查说明）",
        ),
        ("/init", "分析项目并生成 / 更新 AGENTS.md 指导文件"),
    ];
    for (inv, d) in codex {
        out.push(SkillInfo {
            agent: "codex".to_string(),
            invoke: inv.to_string(),
            name: inv.trim_start_matches('/').to_string(),
            description: d.to_string(),
            source: SOURCE_BUILTIN.to_string(),
        });
    }
    // 应用本地执行/映射的通用命令，两个 agent 都可用
    let local: [(&str, &str); 3] = [
        ("/diff", "查看项目 git 改动（状态 + 未暂存 + 已暂存，本地执行不调用模型）"),
        ("/status", "查看当前 Agent / 项目 / 会话 / CLI 状态（本地执行不调用模型）"),
        (
            "/fork",
            "把当前会话分叉成新会话继续（原会话不受影响；仅在已打开会话中可用，需附继续指令）",
        ),
    ];
    for agent in ["claude", "codex"] {
        for (inv, d) in local {
            out.push(SkillInfo {
                agent: agent.to_string(),
                invoke: inv.to_string(),
                name: inv.trim_start_matches('/').to_string(),
                description: d.to_string(),
                source: SOURCE_BUILTIN.to_string(),
            });
        }
    }
}

/// <dir>/<name>/SKILL.md 结构；plugin 前缀存在时调用形式为 /插件名:技能名。
fn scan_skill_dir(dir: &Path, source: &str, plugin: Option<&str>, out: &mut Vec<SkillInfo>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for ent in rd.flatten() {
        let p = ent.path();
        if !p.is_dir() {
            continue;
        }
        let dname = ent.file_name().to_string_lossy().to_string();
        let md = p.join("SKILL.md");
        if !md.is_file() {
            continue;
        }
        let (name, desc) = parse_frontmatter(&md, &dname);
        let invoke = match plugin {
            Some(pl) => format!("/{pl}:{name}"),
            None => format!("/{name}"),
        };
        out.push(SkillInfo {
            agent: "claude".to_string(),
            invoke,
            name,
            description: desc,
            source: source.to_string(),
        });
    }
}

/// <dir>/*.md 的自定义斜杠命令（文件名即命令名）。
fn scan_command_dir(dir: &Path, source: &str, out: &mut Vec<SkillInfo>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for ent in rd.flatten() {
        let p = ent.path();
        if !p.is_file() || p.extension().map(|e| e != "md").unwrap_or(true) {
            continue;
        }
        let stem = match p.file_stem() {
            Some(s) => s.to_string_lossy().to_string(),
            None => continue,
        };
        let (_, desc) = parse_frontmatter(&p, &stem);
        out.push(SkillInfo {
            agent: "claude".to_string(),
            invoke: format!("/{stem}"),
            name: stem,
            description: desc,
            source: source.to_string(),
        });
    }
}

/// installed_plugins.json → 各插件 installPath/skills/**/SKILL.md（最多递归 3 层，
/// 兼容 skills/engineering/tdd/SKILL.md 之类的分组目录；跳过 deprecated）。
fn scan_plugins(plugins_root: &Path, out: &mut Vec<SkillInfo>) {
    let Ok(raw) = fs::read_to_string(plugins_root.join("installed_plugins.json")) else {
        return;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return;
    };
    let Some(plugins) = v.get("plugins").and_then(|p| p.as_object()) else {
        return;
    };
    for (key, entries) in plugins {
        let plugin_name = key.split('@').next().unwrap_or(key).to_string();
        let Some(install) = entries
            .as_array()
            .and_then(|a| a.first())
            .and_then(|e| e.get("installPath"))
            .and_then(|p| p.as_str())
        else {
            continue;
        };
        walk_plugin_skills(&Path::new(install).join("skills"), &plugin_name, 0, out);
    }
}

fn walk_plugin_skills(dir: &Path, plugin: &str, depth: u32, out: &mut Vec<SkillInfo>) {
    if depth > 3 {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else { return };
    for ent in rd.flatten() {
        let p = ent.path();
        if !p.is_dir() {
            continue;
        }
        let dname = ent.file_name().to_string_lossy().to_string();
        if dname.eq_ignore_ascii_case("deprecated") {
            continue;
        }
        if p.join("SKILL.md").is_file() {
            let (name, desc) = parse_frontmatter(&p.join("SKILL.md"), &dname);
            out.push(SkillInfo {
                agent: "claude".to_string(),
                invoke: format!("/{plugin}:{name}"),
                name,
                description: desc,
                source: format!("插件 {plugin}"),
            });
        } else {
            walk_plugin_skills(&p, plugin, depth + 1, out);
        }
    }
}

/// ~/.codex/skills/<name>/SKILL.md：Codex 技能，在 prompt 里以 $名称 触发
/// （或由描述自动触发）。
fn scan_codex_skills(dir: &Path, out: &mut Vec<SkillInfo>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for ent in rd.flatten() {
        let p = ent.path();
        if !p.is_dir() {
            continue;
        }
        let dname = ent.file_name().to_string_lossy().to_string();
        let md = p.join("SKILL.md");
        if !md.is_file() {
            continue;
        }
        let (name, desc) = parse_frontmatter(&md, &dname);
        out.push(SkillInfo {
            agent: "codex".to_string(),
            invoke: format!("${name}"),
            name,
            description: desc,
            source: "codex 技能".to_string(),
        });
    }
}

/// ~/.codex/prompts/*.md：Codex 的自定义 prompt，同样以 /名称 调用。
fn scan_codex_prompts(dir: &Path, out: &mut Vec<SkillInfo>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for ent in rd.flatten() {
        let p = ent.path();
        if !p.is_file() || p.extension().map(|e| e != "md").unwrap_or(true) {
            continue;
        }
        let stem = match p.file_stem() {
            Some(s) => s.to_string_lossy().to_string(),
            None => continue,
        };
        let (_, desc) = parse_frontmatter(&p, &stem);
        out.push(SkillInfo {
            agent: "codex".to_string(),
            invoke: format!("/{stem}"),
            name: stem,
            description: desc,
            source: "codex prompt".to_string(),
        });
    }
}

/// 解析 YAML frontmatter 的 name / description（description 支持缩进续行）。
/// 无 frontmatter 时 name 用 fallback，description 为空。
fn parse_frontmatter(path: &Path, fallback_name: &str) -> (String, String) {
    let mut name = fallback_name.to_string();
    let mut desc = String::new();
    let Ok(raw) = fs::read_to_string(path) else {
        return (name, desc);
    };
    let mut lines = raw.lines();
    if lines.next().map(str::trim) != Some("---") {
        return (name, desc);
    }
    let mut in_desc = false;
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        if let Some(v) = line.strip_prefix("name:") {
            let v = v.trim();
            if !v.is_empty() {
                name = v.to_string();
            }
            in_desc = false;
        } else if let Some(v) = line.strip_prefix("description:") {
            desc = v.trim().to_string();
            in_desc = true;
        } else if in_desc && (line.starts_with(' ') || line.starts_with('\t')) {
            if !desc.is_empty() {
                desc.push(' ');
            }
            desc.push_str(line.trim());
        } else {
            in_desc = false;
        }
    }
    (name, truncate_chars(&desc, 240))
}

fn truncate_chars(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(n).collect();
        t.push('…');
        t
    }
}

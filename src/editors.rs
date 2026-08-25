//! 本机编辑器 / IDE 检测。结果只用于生成项目目录的“打开方式”菜单。

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

#[derive(Clone, Debug, Serialize)]
pub struct EditorInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub path: String,
}

struct EditorSpec {
    id: &'static str,
    name: &'static str,
    commands: &'static [&'static str],
    executable_names: &'static [&'static str],
}

const EDITORS: &[EditorSpec] = &[
    EditorSpec {
        id: "vscode",
        name: "Visual Studio Code",
        commands: &["code"],
        executable_names: &["Code.exe"],
    },
    EditorSpec {
        id: "cursor",
        name: "Cursor",
        commands: &["cursor"],
        executable_names: &["Cursor.exe"],
    },
    EditorSpec {
        id: "windsurf",
        name: "Windsurf",
        commands: &["windsurf"],
        executable_names: &["Windsurf.exe"],
    },
    EditorSpec {
        id: "idea",
        name: "IntelliJ IDEA",
        commands: &["idea64", "idea"],
        executable_names: &["idea64.exe"],
    },
    EditorSpec {
        id: "android-studio",
        name: "Android Studio",
        commands: &["studio64", "studio"],
        executable_names: &["studio64.exe"],
    },
];

static DETECTED: OnceLock<Vec<(EditorInfo, PathBuf)>> = OnceLock::new();

/// 已安装编辑器列表。检测只执行一次，安装新软件后重启 Agent Hub 即可刷新。
pub fn list() -> Vec<EditorInfo> {
    detected().iter().map(|(info, _)| info.clone()).collect()
}

/// 按受控 ID 获取已检测到的可执行文件，调用方不能传入任意程序路径。
pub fn executable(id: &str) -> Option<PathBuf> {
    detected()
        .iter()
        .find(|(info, _)| info.id == id)
        .map(|(_, path)| path.clone())
}

fn detected() -> &'static Vec<(EditorInfo, PathBuf)> {
    DETECTED.get_or_init(|| {
        EDITORS
            .iter()
            .filter_map(|spec| {
                detect(spec).map(|path| {
                    let info = EditorInfo {
                        id: spec.id,
                        name: spec.name,
                        path: path.display().to_string(),
                    };
                    (info, path)
                })
            })
            .collect()
    })
}

fn detect(spec: &EditorSpec) -> Option<PathBuf> {
    for command in spec.commands {
        if let Some(path) = find_on_path(command, spec.executable_names) {
            return Some(path);
        }
    }

    known_candidates(spec)
        .into_iter()
        .filter(|path| path.is_file())
        .max_by_key(last_modified)
}

fn find_on_path(command: &str, executable_names: &[&str]) -> Option<PathBuf> {
    let output = Command::new("where.exe").arg(command).output().ok()?;
    if !output.status.success() {
        return None;
    }

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let shim = PathBuf::from(line.trim());
        if shim.is_file()
            && shim
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
        {
            return Some(shim);
        }

        // npm shim 和 JetBrains launcher 通常位于安装目录下的 bin 子目录。
        for ancestor in shim.ancestors().take(6) {
            for executable in executable_names {
                let candidate = ancestor.join(executable);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn known_candidates(spec: &EditorSpec) -> Vec<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let program_files = std::env::var_os("ProgramFiles").map(PathBuf::from);
    let program_files_x86 = std::env::var_os("ProgramFiles(x86)").map(PathBuf::from);
    let mut candidates = Vec::new();

    match spec.id {
        "vscode" => {
            push_joined(
                &mut candidates,
                local.as_deref(),
                r"Programs\Microsoft VS Code\Code.exe",
            );
            push_joined(
                &mut candidates,
                program_files.as_deref(),
                r"Microsoft VS Code\Code.exe",
            );
            push_joined(
                &mut candidates,
                program_files_x86.as_deref(),
                r"Microsoft VS Code\Code.exe",
            );
        }
        "cursor" => {
            push_joined(
                &mut candidates,
                local.as_deref(),
                r"Programs\cursor\Cursor.exe",
            );
            push_joined(
                &mut candidates,
                program_files.as_deref(),
                r"cursor\Cursor.exe",
            );
        }
        "windsurf" => {
            push_joined(
                &mut candidates,
                local.as_deref(),
                r"Programs\Windsurf\Windsurf.exe",
            );
            push_joined(
                &mut candidates,
                program_files.as_deref(),
                r"Windsurf\Windsurf.exe",
            );
        }
        "idea" => {
            if let Some(root) = local.as_deref() {
                collect_named(&root.join("Programs"), "idea64.exe", 3, &mut candidates);
                collect_named(
                    &root.join(r"JetBrains\Toolbox\apps"),
                    "idea64.exe",
                    8,
                    &mut candidates,
                );
            }
            if let Some(root) = program_files.as_deref() {
                collect_named(&root.join("JetBrains"), "idea64.exe", 4, &mut candidates);
            }
        }
        "android-studio" => {
            push_joined(
                &mut candidates,
                program_files.as_deref(),
                r"Android\Android Studio\bin\studio64.exe",
            );
            push_joined(
                &mut candidates,
                local.as_deref(),
                r"Programs\Android Studio\bin\studio64.exe",
            );
        }
        _ => {}
    }

    candidates
}

fn push_joined(out: &mut Vec<PathBuf>, root: Option<&Path>, child: &str) {
    if let Some(root) = root {
        out.push(root.join(child));
    }
}

fn collect_named(root: &Path, file_name: &str, depth: usize, out: &mut Vec<PathBuf>) {
    if depth == 0 || !root.is_dir() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case(file_name))
        {
            out.push(path);
        } else if file_type.is_dir() && !file_type.is_symlink() {
            collect_named(&path, file_name, depth - 1, out);
        }
    }
}

fn last_modified(path: &PathBuf) -> std::time::SystemTime {
    path.metadata()
        .and_then(|meta| meta.modified())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn editor_ids_are_unique() {
        let ids: HashSet<_> = EDITORS.iter().map(|spec| spec.id).collect();
        assert_eq!(ids.len(), EDITORS.len());
    }

    #[test]
    fn detected_paths_are_files() {
        for (_, path) in detected() {
            assert!(path.is_file(), "{} 不是文件", path.display());
        }
    }
}

//! 应用自身配置：`~/.agenthub/config.json`（CONTRACT §3.4）。读失败容错为空配置。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    /// 手动固定的项目目录（真实路径）。
    #[serde(default)]
    pub projects: Vec<String>,
}

fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".agenthub").join("config.json"))
}

/// 读配置；任何失败（无 home、无文件、JSON 损坏）都容错为默认空配置。
pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

/// 写配置；写入前确保 `~/.agenthub` 目录存在。
pub fn save(cfg: &Config) -> Result<(), String> {
    let Some(path) = config_path() else {
        return Err("无法定位用户主目录".to_string());
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("创建 {} 失败: {e}", dir.display()))?;
    }
    let text = serde_json::to_string_pretty(cfg).map_err(|e| format!("序列化配置失败: {e}"))?;
    std::fs::write(&path, text).map_err(|e| format!("写入 {} 失败: {e}", path.display()))
}

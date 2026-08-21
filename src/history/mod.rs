//! 历史会话读取。子模块契约（精确签名）见 docs/CONTRACT.md §2：
//! - claude: normalize_path / all_sessions / transcript(project, session_id)
//! - codex:  all_sessions / transcript(session_id)

pub mod claude;
pub mod codex;

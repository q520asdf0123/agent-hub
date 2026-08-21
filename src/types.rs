//! 全部 API 序列化契约类型。此文件为并行实现的固定契约，勿改字段名。

use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Debug)]
pub struct CliStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct StatusResp {
    pub claude: CliStatus,
    pub codex: CliStatus,
}

#[derive(Serialize, Clone, Debug)]
pub struct ProjectInfo {
    /// 真实规范化路径，如 D:\project\demo_app
    pub path: String,
    /// 末级目录名
    pub name: String,
    /// 目录当前是否存在
    pub exists: bool,
    pub claude_sessions: usize,
    pub codex_sessions: usize,
    /// ISO 8601 UTC
    pub last_active: Option<String>,
    /// 用户手动添加
    pub pinned: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct SessionSummary {
    /// "claude" | "codex"
    pub agent: String,
    /// 会话 uuid
    pub id: String,
    pub title: String,
    /// 真实规范化 cwd
    pub project: String,
    /// ISO 8601
    pub created: Option<String>,
    /// ISO 8601（文件 mtime）
    pub updated: Option<String>,
    /// codex archived_sessions
    pub archived: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct Transcript {
    pub agent: String,
    pub id: String,
    pub project: String,
    pub title: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ChatMessage {
    /// "user" | "assistant" | "system"
    pub role: String,
    pub ts: Option<String>,
    pub blocks: Vec<Block>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Block {
    /// text | thinking | tool_use | tool_result | image | divider
    pub kind: String,
    pub text: String,
    /// tool_use 的工具名
    pub name: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ChatReq {
    /// "claude" | "codex"
    pub agent: String,
    /// 工作目录（新会话与 resume 都传）
    pub project: String,
    pub prompt: String,
    /// Some => 继续历史会话
    pub session_id: Option<String>,
    pub model: Option<String>,
    /// "bypass" | "accept-edits" | "plan" | "read-only" | "default"
    pub permission: Option<String>,
    /// 快速模式（仅 claude：注入 --settings {"fastMode":true}）
    pub fast: Option<bool>,
    /// 思考等级：claude 经 --settings effortLevel；codex 经 -c model_reasoning_effort
    pub effort: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct AgentModels {
    /// 配置文件里的全局默认模型（省缺 model 参数时实际使用）
    pub default: Option<String>,
    /// 发现的可用模型名，按相关性排序（默认 → 目录/历史 → 别名）
    pub models: Vec<String>,
    /// 支持的思考等级（canonical 顺序）
    pub efforts: Vec<String>,
    /// 配置文件里的全局默认思考等级
    pub default_effort: Option<String>,
    /// 模型上下文窗口（token 数，能探明时提供）
    pub context_window: Option<i64>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ModelsResp {
    pub claude: AgentModels,
    pub codex: AgentModels,
}

#[derive(Serialize, Clone, Debug)]
pub struct SkillInfo {
    /// 该技能适用的 agent："claude" | "codex"
    pub agent: String,
    /// 插入 prompt 的调用形式，如 "/tdd"、"/superpowers:brainstorming"
    pub invoke: String,
    pub name: String,
    pub description: String,
    /// 来源："用户" | "项目" | "插件 xxx" | "命令" | "codex prompt"
    pub source: String,
}

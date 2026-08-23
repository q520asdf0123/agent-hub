//! Agent Hub — 本地 CLI Agent（Claude Code / Codex）的 Web 界面。
//! 路由与启动固定于此；各模块契约见 docs/CONTRACT.md。

mod api;
mod cli;
mod config;
mod dialog;
mod history;
mod models;
mod run;
mod sage;
mod skills;
mod types;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let state = Arc::new(api::AppState::new());
    // 会话索引预热（后台进行，不阻塞启动）
    tokio::spawn(api::warm_sessions());
    let app = Router::new()
        .route("/", get(api::index_html))
        .route("/app.js", get(api::app_js))
        .route("/style.css", get(api::style_css))
        .route("/favicon.svg", get(api::favicon_svg))
        .route("/api/status", get(api::status))
        .route("/api/projects", get(api::projects).post(api::add_project))
        .route("/api/projects/discover", get(api::discover_projects))
        .route("/api/pick-folder", post(api::pick_folder))
        .route("/api/projects/remove", post(api::remove_project))
        .route("/api/sessions", get(api::sessions))
        .route("/api/session", get(api::session_detail))
        .route("/api/skills", get(api::skills))
        .route("/api/models", get(api::models))
        .route("/api/chat", post(api::chat))
        .route("/api/run", get(api::run_attach))
        .route("/api/runs", get(api::runs_list))
        .route("/api/stop", post(api::stop_run))
        .route("/api/session/delete", post(api::delete_session))
        .route("/api/fork-at", post(api::fork_at))
        .route("/api/sage", post(api::sage_route))
        .route("/api/sage/outcome", post(api::sage_outcome))
        .route("/api/file", get(api::file))
        .route("/api/open", post(api::open_path))
        .route("/api/filestat", post(api::filestat))
        .route("/api/diff", get(api::diff))
        .route("/api/filetext", get(api::filetext))
        .route("/api/upload", post(api::upload))
        .layer(DefaultBodyLimit::max(35 * 1024 * 1024))
        .with_state(state);

    let port: u16 = std::env::var("AGENT_HUB_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8721);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .unwrap_or_else(|e| panic!("端口 {port} 绑定失败: {e}"));
    println!("Agent Hub: http://127.0.0.1:{port}");
    axum::serve(listener, app).await.unwrap();
}

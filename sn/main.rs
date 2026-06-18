use axum::{
    routing::{get, post},
    Router,
    extract::ws::{WebSocket, Message},
    response::Html,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

mod terminal;
mod commands;
mod kernel_link;
mod fastfetch;

use terminal::TerminalSession;

#[tokio::main]
async fn main() {
    println!("🐧 Linuxab OS Terminal Shell v0.1.0");
    println!("🔧 Initializing kernel link...");
    
    // Initialize kernel interface
    kernel_link::init().await;
    
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_handler))
        .route("/api/sysinfo", get(api_sysinfo))
        .route("/api/kernel", get(api_kernel_info))
        .nest_service("/assets", ServeDir::new("assets"));
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Terminal server running at http://localhost:8080");
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index_handler() -> Html<String> {
    Html(include_str!("../assets/index.html").to_string())
}

async fn ws_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let session = Arc::new(Mutex::new(TerminalSession::new()));
    
    // Send welcome banner
    let banner = fastfetch::generate_banner().await;
    socket.send(Message::Text(banner)).await.ok();
    
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(cmd) = msg {
            let mut term = session.lock().await;
            let output = term.execute(&cmd).await;
            socket.send(Message::Text(output)).await.ok();
        }
    }
}

async fn api_sysinfo() -> String {
    fastfetch::get_sysinfo_json().await
}

async fn api_kernel_info() -> String {
    kernel_link::get_kernel_info().await
}

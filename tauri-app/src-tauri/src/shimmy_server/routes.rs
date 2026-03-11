use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::shimmy_server::structs::{MCPRequest, MCPResponse};

#[derive(Clone)]
struct ProxyState {
    tauri_app: AppHandle,
}

async fn mcp_initialize(
    State(state): State<ProxyState>,
    Json(payload): Json<MCPRequest>,
) -> (StatusCode, String) {
    if !("initialize" == payload.method.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            format!(
                "Method should be `initialize` but got {} instead",
                payload.method,
            ),
        );
    }

    println!("initialize request: {:?}", payload);

    (StatusCode::OK, "efjieiege".to_string())
}

async fn client_mcp_request(
    State(state): State<ProxyState>,
    Json(payload): Json<MCPRequest>,
) -> (StatusCode, ()) {
    println!("client request: {:?}", payload);

    match payload.method.as_str() {
        "initialize" => {}
        "tools/call" => {}
        _ => {}
    }

    if let Err(e) = state.tauri_app.emit("mcp-traffic", &payload) {
        eprintln!("Failed to emit to frontend: {}", e);
    }

    (StatusCode::OK, ())
}

async fn server_mcp_request(
    State(state): State<ProxyState>,
    Json(payload): Json<MCPRequest>,
) -> (StatusCode, ()) {
    println!("server request: {:?}", payload);

    match payload.method.as_str() {
        "initialize" => {}
        "tools/call" => {}
        _ => {}
    }

    if let Err(e) = state.tauri_app.emit("mcp-traffic", &payload) {
        eprintln!("Failed to emit to frontend: {}", e);
    }

    (StatusCode::OK, ())
}

async fn mcp_response(
    State(state): State<ProxyState>,
    Json(payload): Json<MCPResponse>,
) -> (StatusCode, ()) {
    println!("response: {:?}", payload);

    if let Err(e) = state.tauri_app.emit("mcp-traffic", &payload) {
        eprintln!("Failed to emit to frontend: {}", e);
    }

    (StatusCode::OK, ())
}

pub async fn spawn_server(app_handle: AppHandle) {
    let proxy_state = ProxyState {
        tauri_app: app_handle,
    };

    let client_route = Router::new().route("/request", post(client_mcp_request));
    let server_route = Router::new().route("/request", post(server_mcp_request));

    let router = Router::new()
        .route("/initialize", post(mcp_initialize))
        .route("/response", post(mcp_response))
        .nest("/client", client_route)
        .nest("/server", server_route)
        .with_state(proxy_state);

    let listenner = tokio::net::TcpListener::bind("127.0.0.1:13579")
        .await
        .unwrap();
    axum::serve(listenner, router).await.unwrap();
}

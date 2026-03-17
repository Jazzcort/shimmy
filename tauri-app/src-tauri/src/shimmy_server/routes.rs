use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use jiff::{tz::TimeZone, Timestamp};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::shimmy_server::structs::{
    Id, MCPNotification, MCPRequest, MCPResponse, StampedMcpNotification, StampedMcpRequest,
    StampedMcpResponse,
};

#[derive(Clone)]
pub(crate) struct ProxyState {
    pub tauri_app: AppHandle,
    pub mcp_client_request_store: Arc<Mutex<HashMap<(String, Id), StampedMcpRequest>>>,
    pub mcp_server_request_store: Arc<Mutex<HashMap<(String, Id), StampedMcpRequest>>>,
    pub mcp_server_response_store: Arc<Mutex<HashMap<(String, Id), StampedMcpResponse>>>,
    pub mcp_client_response_store: Arc<Mutex<HashMap<(String, Id), StampedMcpResponse>>>,
    pub mcp_client_notification_store: Arc<Mutex<HashMap<(String, Id), StampedMcpNotification>>>,
    pub mcp_server_notification_store: Arc<Mutex<HashMap<(String, Id), StampedMcpNotification>>>,
}

async fn mcp_initialize_start(
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

    let id = Uuid::new_v4();
    let time = Timestamp::now();

    state.mcp_client_request_store.lock().await.insert(
        (id.to_string(), payload.id.clone()),
        StampedMcpRequest {
            request: payload.clone(),
            timestamp: time,
        },
    );

    if let Err(e) = state.tauri_app.emit("mcp-initialize-start", id.to_string()) {
        eprintln!("Failed to emit mcp-initialize: {}", e);

        // TODO: Notify the frontend to refresh the entire list for this mcp server
    }

    (StatusCode::OK, id.to_string())
}

async fn mcp_initialize_finish(
    Path(id): Path<String>,
    State(state): State<ProxyState>,
    Json(payload): Json<MCPResponse>,
) -> (StatusCode, ()) {
    eprintln!("initialize finish: {:?}", payload);
    eprintln!("id: {}", id);

    match &payload {
        MCPResponse::Success {
            jsonrpc,
            id: request_id,
            result,
        } => {
            let time = Timestamp::now();
            state.mcp_server_response_store.lock().await.insert(
                (id.clone(), request_id.clone()),
                StampedMcpResponse {
                    response: payload.clone(),
                    timestamp: time,
                },
            );

            if let Err(e) = state.tauri_app.emit(
                "mcp-initialize-finish",
                json!({
                    "serverId": id,
                    "requestId": request_id
                }),
            ) {
                eprintln!("Failed to emit mcp-initialize: {}", e);

                // TODO: Notify the frontend to refresh the entire list for this mcp server
            }

            (StatusCode::OK, ())
        }
        MCPResponse::Fail { jsonrpc, id, error } => (StatusCode::BAD_REQUEST, ()),
    }
}

async fn client_mcp_request(
    Path(id): Path<String>,
    State(state): State<ProxyState>,
    Json(payload): Json<MCPRequest>,
) -> (StatusCode, ()) {
    println!("client request: {:?}", payload);
    println!("client request id: {}", id);

    let time = Timestamp::now();
    state.mcp_client_request_store.lock().await.insert(
        (id.clone(), payload.id.clone()),
        StampedMcpRequest {
            request: payload.clone(),
            timestamp: time,
        },
    );

    if let Err(e) = state.tauri_app.emit(
        "mcp-client-request",
        json!({
            "serverId": id,
            "requestId": payload.id,
        }),
    ) {
        eprintln!("Failed to emit to frontend: {}", e);

        // TODO: Notify the frontend to refresh the entire list for this mcp server
    }

    (StatusCode::OK, ())
}

async fn server_mcp_request(
    Path(id): Path<String>,
    State(state): State<ProxyState>,
    Json(payload): Json<MCPRequest>,
) -> (StatusCode, ()) {
    println!("server request: {:?}", payload);
    println!("server request id: {}", id);
    let time = Timestamp::now();

    state.mcp_server_request_store.lock().await.insert(
        (id.clone(), payload.id.clone()),
        StampedMcpRequest {
            request: payload.clone(),
            timestamp: time,
        },
    );

    if let Err(e) = state.tauri_app.emit(
        "mcp-server-request",
        json!({
            "serverId": id,
            "requestId": payload.id,
        }),
    ) {
        eprintln!("Failed to emit to frontend: {}", e);

        // TODO: Notify the frontend to refresh the entire list for this mcp server
    }

    (StatusCode::OK, ())
}

async fn client_mcp_response(
    Path(id): Path<String>,
    State(state): State<ProxyState>,
    Json(payload): Json<MCPResponse>,
) -> (StatusCode, ()) {
    println!("response: {:?}", payload);
    println!("response id: {}", id);

    let response_id = match &payload {
        MCPResponse::Success {
            jsonrpc,
            id,
            result,
        } => id.clone(),
        MCPResponse::Fail { jsonrpc, id, error } => id.clone(),
    };

    let time = Timestamp::now();
    state.mcp_client_response_store.lock().await.insert(
        (id.clone(), response_id.clone()),
        StampedMcpResponse {
            response: payload,
            timestamp: time,
        },
    );

    if let Err(e) = state.tauri_app.emit(
        "mcp-client-response",
        json!({
            "serverId": id,
            "responseId": response_id,
        }),
    ) {
        eprintln!("Failed to emit to frontend: {}", e);
    }

    (StatusCode::OK, ())
}

async fn server_mcp_response(
    Path(id): Path<String>,
    State(state): State<ProxyState>,
    Json(payload): Json<MCPResponse>,
) -> (StatusCode, ()) {
    println!("response: {:?}", payload);
    println!("response id: {}", id);

    let response_id = match &payload {
        MCPResponse::Success {
            jsonrpc,
            id,
            result,
        } => id.clone(),
        MCPResponse::Fail { jsonrpc, id, error } => id.clone(),
    };

    let time = Timestamp::now();
    state.mcp_server_response_store.lock().await.insert(
        (id.clone(), response_id.clone()),
        StampedMcpResponse {
            response: payload,
            timestamp: time,
        },
    );

    if let Err(e) = state.tauri_app.emit(
        "mcp-server-response",
        json!({
            "serverId": id,
            "responseId": response_id,
        }),
    ) {
        eprintln!("Failed to emit to frontend: {}", e);
    }

    (StatusCode::OK, ())
}

async fn client_mcp_notification(
    Path(id): Path<String>,
    State(state): State<ProxyState>,
    Json(payload): Json<MCPNotification>,
) -> (StatusCode, ()) {
    let notification_id = Id::StringId(Uuid::new_v4().to_string());
    let time = Timestamp::now();

    state.mcp_client_notification_store.lock().await.insert(
        (id.clone(), notification_id.clone()),
        StampedMcpNotification {
            notification: payload,
            timestamp: time,
        },
    );

    if let Err(e) = state.tauri_app.emit(
        "mcp-client-notification",
        json!({
            "serverId": id,
            "notificationId": notification_id,
        }),
    ) {
        eprintln!("Failed to emit to frontend: {}", e);
    }

    (StatusCode::OK, ())
}

async fn server_mcp_notification(
    Path(id): Path<String>,
    State(state): State<ProxyState>,
    Json(payload): Json<MCPNotification>,
) -> (StatusCode, ()) {
    let notification_id = Id::StringId(Uuid::new_v4().to_string());
    let time = Timestamp::now();

    state.mcp_server_notification_store.lock().await.insert(
        (id.clone(), notification_id.clone()),
        StampedMcpNotification {
            notification: payload,
            timestamp: time,
        },
    );

    if let Err(e) = state.tauri_app.emit(
        "mcp-server-notification",
        json!({
            "serverId": id,
            "notificationId": notification_id,
        }),
    ) {
        eprintln!("Failed to emit to frontend: {}", e);
    }

    (StatusCode::OK, ())
}

pub async fn spawn_server(proxy_state: ProxyState) {
    let client_route = Router::new()
        .route("/request/{id}", post(client_mcp_request))
        .route("/response/{id}", post(client_mcp_response))
        .route("/notification/{id}", post(client_mcp_notification));
    let server_route = Router::new()
        .route("/request/{id}", post(server_mcp_request))
        .route("/response/{id}", post(server_mcp_response))
        .route("/notification/{id}", post(server_mcp_notification));
    let initialize_route = Router::new()
        .route("/start", post(mcp_initialize_start))
        .route("/finish/{id}", post(mcp_initialize_finish));

    let router = Router::new()
        .nest("/initialize", initialize_route)
        .nest("/client", client_route)
        .nest("/server", server_route)
        .with_state(proxy_state);

    let listenner = tokio::net::TcpListener::bind("127.0.0.1:13579")
        .await
        .unwrap();
    axum::serve(listenner, router).await.unwrap();
}

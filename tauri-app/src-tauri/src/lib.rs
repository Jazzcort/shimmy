mod commands;
mod errors;
mod shimmy_server;
use std::{collections::HashMap, sync::Arc};

use tauri::Manager;
use tokio::sync::Mutex;

use crate::shimmy_server::{
    routes::{spawn_server, ProxyState},
    structs::{Id, StampedMcpRequest, StampedMcpResponse},
};

use crate::commands::{get_mcp_logs, get_mcp_request, get_mcp_response};

#[derive(Clone)]
struct AppData {
    mcp_request_store: Arc<Mutex<HashMap<(String, Id), StampedMcpRequest>>>,
    mcp_response_store: Arc<Mutex<HashMap<(String, Id), StampedMcpResponse>>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.app_handle().clone();
            let app_data = AppData {
                mcp_request_store: Arc::new(Mutex::new(HashMap::new())),
                mcp_response_store: Arc::new(Mutex::new(HashMap::new())),
            };
            app.manage(app_data.clone());

            let proxy_state = ProxyState {
                tauri_app: app_handle,
                mcp_request_store: app_data.mcp_request_store,
                mcp_response_store: app_data.mcp_response_store,
            };

            tauri::async_runtime::spawn(async move {
                spawn_server(proxy_state).await;
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_mcp_request,
            get_mcp_response,
            get_mcp_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

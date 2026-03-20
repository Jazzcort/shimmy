use jiff::{tz::TimeZone, Timestamp};
use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::{
    errors::ShimmyError,
    shimmy_server::structs::{
        Id, InspectorEntry, LogStatus, MCPRequest, MCPResponse, RequestType,
        StampedMcpNotification, StampedMcpRequest, StampedMcpRequestForSerialize,
        StampedMcpResponse, StampedMcpResponseForSerialize,
    },
    utils::create_legit_svelte_id,
    AppData,
};

#[derive(Serialize)]
pub struct ColorizedLine {
    pub indent: usize,
    pub html: String,
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn colorize_json_string(json: &str) -> Vec<ColorizedLine> {
    let bytes = json.as_bytes();
    let len = bytes.len();

    let mut result = String::with_capacity(json.len() * 2);
    let mut i = 0;

    while i < len {
        match bytes[i] {
            b'"' => {
                // Find end of string (handling escapes)
                let mut j = i + 1;
                while j < len {
                    if bytes[j] == b'\\' {
                        j += 2;
                        continue;
                    }
                    if bytes[j] == b'"' {
                        j += 1;
                        break;
                    }
                    j += 1;
                }
                let escaped = escape_html(&json[i..j]);

                // Check if key (followed by colon)
                let mut k = j;
                while k < len && bytes[k] == b' ' {
                    k += 1;
                }
                if k < len && bytes[k] == b':' {
                    result.push_str(r#"<span class="text-blue-300">"#);
                } else {
                    result.push_str(r#"<span class="text-green-400">"#);
                }
                result.push_str(&escaped);
                result.push_str("</span>");
                i = j;
            }
            b'n' if json[i..].starts_with("null") => {
                result.push_str(r#"<span class="text-orange-400">null</span>"#);
                i += 4;
            }
            b't' if json[i..].starts_with("true") => {
                result.push_str(r#"<span class="text-yellow-400">true</span>"#);
                i += 4;
            }
            b'f' if json[i..].starts_with("false") => {
                result.push_str(r#"<span class="text-yellow-400">false</span>"#);
                i += 5;
            }
            b'-' | b'0'..=b'9' => {
                let mut j = i;
                if bytes[j] == b'-' {
                    j += 1;
                }
                while j < len && matches!(bytes[j], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
                {
                    j += 1;
                }
                result.push_str(r#"<span class="text-purple-400">"#);
                result.push_str(&json[i..j]);
                result.push_str("</span>");
                i = j;
            }
            // Just to be safe
            b'&' => {
                result.push_str("&amp;");
                i += 1;
            }
            b'<' => {
                result.push_str("&lt;");
                i += 1;
            }
            b'>' => {
                result.push_str("&gt;");
                i += 1;
            }
            _ => {
                result.push(bytes[i] as char);
                i += 1;
            }
        }
    }

    // Split into lines and compute indent
    let raw_lines: Vec<&str> = json.split('\n').collect();
    let html_lines: Vec<&str> = result.split('\n').collect();

    html_lines
        .iter()
        .enumerate()
        .map(|(idx, html_line)| {
            let indent = raw_lines
                .get(idx)
                .map(|l| l.len() - l.trim_start_matches(' ').len())
                .unwrap_or(0);
            let trimmed = html_line.trim_start();
            ColorizedLine {
                indent,
                html: trimmed.to_string(),
            }
        })
        .collect()
}

#[tauri::command]
pub fn colorize_json(data: Value) -> Vec<ColorizedLine> {
    let json = serde_json::to_string_pretty(&data).unwrap_or_default();
    colorize_json_string(&json)
}

#[tauri::command]
pub async fn get_mcp_client_request(
    server_id: String,
    request_id: Id,
    state: State<'_, AppData>,
) -> Result<StampedMcpRequestForSerialize, ShimmyError> {
    let request = state
        .mcp_client_request_store
        .lock()
        .await
        .get(&(server_id, request_id))
        .ok_or(ShimmyError::Shimmy(
            "Failed to find mcp request".to_string(),
        ))?
        .clone();

    Ok(request.pack_for_serializing())
}

#[tauri::command]
pub async fn get_mcp_server_request(
    server_id: String,
    request_id: Id,
    state: State<'_, AppData>,
) -> Result<StampedMcpRequestForSerialize, ShimmyError> {
    let request = state
        .mcp_server_request_store
        .lock()
        .await
        .get(&(server_id, request_id))
        .ok_or(ShimmyError::Shimmy(
            "Failed to find mcp request".to_string(),
        ))?
        .clone();

    Ok(request.pack_for_serializing())
}

#[tauri::command]
pub async fn get_mcp_server_response(
    server_id: String,
    response_id: Id,
    state: State<'_, AppData>,
) -> Result<StampedMcpResponseForSerialize, ShimmyError> {
    let response = state
        .mcp_server_response_store
        .lock()
        .await
        .get(&(server_id, response_id))
        .ok_or(ShimmyError::Shimmy(
            "Failed to find mcp response".to_string(),
        ))?
        .clone();

    Ok(response.pack_for_serializing())
}

#[tauri::command]
pub async fn get_mcp_client_response(
    server_id: String,
    response_id: Id,
    state: State<'_, AppData>,
) -> Result<StampedMcpResponseForSerialize, ShimmyError> {
    let response = state
        .mcp_client_response_store
        .lock()
        .await
        .get(&(server_id, response_id))
        .ok_or(ShimmyError::Shimmy(
            "Failed to find mcp response".to_string(),
        ))?
        .clone();

    Ok(response.pack_for_serializing())
}

#[tauri::command]
pub async fn get_mcp_client_notification(
    server_id: String,
    notification_id: Id,
    state: State<'_, AppData>,
) -> Result<StampedMcpNotification, ShimmyError> {
    let notification = state
        .mcp_client_notification_store
        .lock()
        .await
        .get(&(server_id, notification_id))
        .ok_or(ShimmyError::Shimmy(
            "Failed to find mcp notification".to_string(),
        ))?
        .clone();

    Ok(notification)
}

#[tauri::command]
pub async fn get_mcp_server_notification(
    server_id: String,
    notification_id: Id,
    state: State<'_, AppData>,
) -> Result<StampedMcpNotification, ShimmyError> {
    let notification = state
        .mcp_server_notification_store
        .lock()
        .await
        .get(&(server_id, notification_id))
        .ok_or(ShimmyError::Shimmy(
            "Failed to find mcp notification".to_string(),
        ))?
        .clone();

    Ok(notification)
}

#[tauri::command]
pub async fn get_mcp_logs(
    server_id: String,
    state: State<'_, AppData>,
) -> Result<Vec<InspectorEntry>, ShimmyError> {
    // Client requests
    let requests: Vec<StampedMcpRequest> = state
        .mcp_client_request_store
        .lock()
        .await
        .iter()
        .filter_map(|((id, _), request)| {
            if id == &server_id {
                Some(request.clone())
            } else {
                None
            }
        })
        .collect();

    let server_response_store_handle = state.mcp_server_response_store.lock().await;

    let mut entries: Vec<InspectorEntry> = requests
        .into_iter()
        .map(|req| {
            let stamped_response =
                server_response_store_handle.get(&(server_id.clone(), req.request.id.clone()));
            let mut stderr: Option<String> = None;
            let mut response: Option<MCPResponse> = None;
            let status = match stamped_response {
                Some(inner_response) => match &inner_response.response {
                    MCPResponse::Success {
                        jsonrpc,
                        id,
                        result,
                    } => {
                        response = Some(MCPResponse::Success {
                            jsonrpc: jsonrpc.clone(),
                            id: id.clone(),
                            result: result.clone(),
                        });

                        LogStatus::Success
                    }
                    MCPResponse::Fail { jsonrpc, id, error } => {
                        response = Some(MCPResponse::Fail {
                            jsonrpc: jsonrpc.clone(),
                            id: id.clone(),
                            error: error.clone(),
                        });

                        stderr = Some(error.message.clone());

                        LogStatus::Error
                    }
                },
                None => LogStatus::Pending,
            };

            let legit_svelte_id = create_legit_svelte_id(&req.request.id, RequestType::Client);

            InspectorEntry {
                id: legit_svelte_id,
                timestamp: req.timestamp.clone(),
                method: req.request.method.clone(),
                status,
                request: serde_json::to_value(req.request)
                    .expect("This serialization should never fail"),
                request_type: RequestType::Client,
                response,
                stderr,
            }
        })
        .collect();

    // Server requests
    let server_requests: Vec<StampedMcpRequest> = state
        .mcp_server_request_store
        .lock()
        .await
        .iter()
        .filter_map(|((id, _), request)| {
            if id == &server_id {
                Some(request.clone())
            } else {
                None
            }
        })
        .collect();

    let client_response_store_handle = state.mcp_client_response_store.lock().await;

    let server_request_entries: Vec<InspectorEntry> = server_requests
        .into_iter()
        .map(|req| {
            let stamped_response =
                client_response_store_handle.get(&(server_id.clone(), req.request.id.clone()));
            let mut stderr: Option<String> = None;
            let mut response: Option<MCPResponse> = None;
            let status = match &stamped_response {
                Some(inner_response) => match &inner_response.response {
                    MCPResponse::Success {
                        jsonrpc,
                        id,
                        result,
                    } => {
                        response = Some(MCPResponse::Success {
                            jsonrpc: jsonrpc.clone(),
                            id: id.clone(),
                            result: result.clone(),
                        });

                        LogStatus::Success
                    }
                    MCPResponse::Fail { jsonrpc, id, error } => {
                        response = Some(MCPResponse::Fail {
                            jsonrpc: jsonrpc.clone(),
                            id: id.clone(),
                            error: error.clone(),
                        });

                        stderr = Some(error.message.clone());

                        LogStatus::Error
                    }
                },
                None => LogStatus::Pending,
            };

            let legit_svelte_id = create_legit_svelte_id(&req.request.id, RequestType::Server);

            InspectorEntry {
                id: legit_svelte_id,
                timestamp: req.timestamp.clone(),
                method: req.request.method.clone(),
                status,
                request: serde_json::to_value(&req.request)
                    .expect("This serialization should never fail"),
                request_type: RequestType::Server,
                response,
                stderr,
            }
        })
        .collect();

    let client_notifications: Vec<(Id, StampedMcpNotification)> = state
        .mcp_client_notification_store
        .lock()
        .await
        .iter()
        .filter_map(|((id, notification_id), notification)| {
            if id == &server_id {
                Some((notification_id.clone(), notification.clone()))
            } else {
                None
            }
        })
        .collect();

    let client_nofitication_entries: Vec<InspectorEntry> = client_notifications
        .into_iter()
        .map(|(notification_id, notification)| {
            let legit_svelte_id = create_legit_svelte_id(&notification_id, RequestType::Client);

            InspectorEntry {
                id: legit_svelte_id,
                timestamp: notification.timestamp.clone(),
                method: notification.notification.method.clone(),
                status: LogStatus::Notification,
                request: serde_json::to_value(&notification.notification)
                    .expect("This serialization should never fail"),
                request_type: RequestType::Client,
                response: None,
                stderr: None,
            }
        })
        .collect();

    let server_notifications: Vec<(Id, StampedMcpNotification)> = state
        .mcp_server_notification_store
        .lock()
        .await
        .iter()
        .filter_map(|((id, notification_id), notification)| {
            if id == &server_id {
                Some((notification_id.clone(), notification.clone()))
            } else {
                None
            }
        })
        .collect();

    let server_nofitication_entries: Vec<InspectorEntry> = server_notifications
        .into_iter()
        .map(|(notification_id, notification)| {
            let legit_svelte_id = create_legit_svelte_id(&notification_id, RequestType::Server);

            InspectorEntry {
                id: legit_svelte_id,
                timestamp: notification.timestamp.clone(),
                method: notification.notification.method.clone(),
                status: LogStatus::Notification,
                request: serde_json::to_value(&notification.notification)
                    .expect("This serialization should never fail"),
                request_type: RequestType::Server,
                response: None,
                stderr: None,
            }
        })
        .collect();

    entries.extend(server_request_entries);
    entries.extend(client_nofitication_entries);
    entries.extend(server_nofitication_entries);
    entries.sort_by_key(|entry| entry.timestamp);

    Ok(entries)
}

#[tauri::command]
pub async fn delete_connection_data(
    server_id: String,
    state: State<'_, AppData>,
) -> Result<(), ShimmyError> {
    state
        .mcp_client_request_store
        .lock()
        .await
        .retain(|(id, _), _| id != &server_id);

    state
        .mcp_server_request_store
        .lock()
        .await
        .retain(|(id, _), _| id != &server_id);

    state
        .mcp_client_response_store
        .lock()
        .await
        .retain(|(id, _), _| id != &server_id);

    state
        .mcp_server_request_store
        .lock()
        .await
        .retain(|(id, _), _| id != &server_id);

    state
        .mcp_client_notification_store
        .lock()
        .await
        .retain(|(id, _), _| id != &server_id);

    state
        .mcp_server_notification_store
        .lock()
        .await
        .retain(|(id, _), _| id != &server_id);

    Ok(())
}

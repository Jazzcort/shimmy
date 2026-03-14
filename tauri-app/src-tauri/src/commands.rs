use jiff::{tz::TimeZone, Timestamp};
use tauri::State;

use crate::{
    errors::ShimmyError,
    shimmy_server::structs::{
        Id, InspectorEntry, LogStatus, MCPRequest, MCPResponse, StampedMcpRequest,
        StampedMcpRequestForSerialize, StampedMcpResponse, StampedMcpResponseForSerialize,
    },
    AppData,
};

#[tauri::command]
pub async fn get_mcp_request(
    server_id: String,
    request_id: Id,
    state: State<'_, AppData>,
) -> Result<StampedMcpRequestForSerialize, ShimmyError> {
    let request = state
        .mcp_request_store
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
pub async fn get_mcp_response(
    server_id: String,
    response_id: Id,
    state: State<'_, AppData>,
) -> Result<StampedMcpResponseForSerialize, ShimmyError> {
    let response = state
        .mcp_response_store
        .lock()
        .await
        .get(&(server_id, response_id))
        .ok_or(ShimmyError::Shimmy(
            "Failed to find mcp request".to_string(),
        ))?
        .clone();

    Ok(response.pack_for_serializing())
}

#[tauri::command]
pub async fn get_mcp_logs(
    server_id: String,
    state: State<'_, AppData>,
) -> Result<Vec<InspectorEntry>, ShimmyError> {
    let mut requests: Vec<StampedMcpRequest> = state
        .mcp_request_store
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

    requests.sort_by_key(|req| req.timestamp);

    let response_store_handle = state.mcp_response_store.lock().await;

    let entries: Vec<InspectorEntry> = requests
        .into_iter()
        .map(|req| {
            let stamped_response =
                response_store_handle.get(&(server_id.clone(), req.request.id.clone()));
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
                        stderr = Some(error.message.clone());

                        LogStatus::Error
                    }
                },
                None => LogStatus::Pending,
            };

            let legit_svelte_id = match &req.request.id {
                Id::NumberId(number) => Id::NumberId(number + 1),
                rest => rest.clone(),
            };

            return InspectorEntry {
                id: legit_svelte_id,
                timestamp: req.timestamp.to_zoned(TimeZone::system()).to_string(),
                method: req.request.method.clone(),
                status,
                request: req.request,
                response,
                stderr,
            };
        })
        .collect();

    Ok(entries)
}

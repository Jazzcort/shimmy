use rmcp::{
    model::{
        ErrorCode, ErrorData, Extensions, JsonObject, JsonRpcError, JsonRpcRequest,
        JsonRpcResponse, JsonRpcVersion2_0, Request, RequestId,
    },
    service::ServiceError,
};
use serde::Serialize;

pub fn create_mcp_request<S>(method: S, params: JsonObject) -> Request
where
    S: Into<String>,
{
    Request {
        method: method.into(),
        params,
        extensions: Extensions::new(),
    }
}

pub fn create_jsonrpc_request(id: RequestId, request: Request) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: JsonRpcVersion2_0,
        id,
        request,
    }
}

pub fn create_jsonrpc_response(id: RequestId, result: JsonObject) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: JsonRpcVersion2_0,
        id,
        result,
    }
}

pub fn create_jsonrpc_error(id: RequestId, error: ErrorData) -> JsonRpcError {
    JsonRpcError {
        jsonrpc: JsonRpcVersion2_0,
        id,
        error,
    }
}

pub fn convert_to_json_object<S>(obj: S) -> Result<JsonObject, ErrorData>
where
    S: Serialize,
{
    let value = serde_json::to_value(obj).map_err(|err| ErrorData {
        code: ErrorCode::PARSE_ERROR,
        message: err.to_string().into(),
        data: None,
    })?;

    if let serde_json::Value::Object(json_map) = value {
        Ok(json_map)
    } else {
        Err(ErrorData {
            code: ErrorCode::PARSE_ERROR,
            message: "Parsed failed: not a json map format".into(),
            data: None,
        })
    }
}

pub fn convert_service_error_to_error_data(service_error: ServiceError) -> ErrorData {
    match service_error {
        ServiceError::McpError(mcp_err) => mcp_err,
        err => ErrorData {
            code: ErrorCode::INTERNAL_ERROR,
            message: err.to_string().into(),
            data: None,
        },
    }
}

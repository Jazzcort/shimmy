use std::borrow::Cow;

use rmcp::{
    model::{
        ErrorCode, ErrorData, Extensions, JsonObject, JsonRpcError, JsonRpcRequest,
        JsonRpcResponse, JsonRpcVersion2_0, Request, RequestId,
    },
    service::ServiceError,
};
use serde::Serialize;

use crate::error::ShimmyError;

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
    let value = serde_json::to_value(obj)
        .map_err(|err| convert_error_to_error_data(ErrorCode::PARSE_ERROR, err))?;

    if let serde_json::Value::Object(json_map) = value {
        Ok(json_map)
    } else {
        Err(convert_text_to_error_data(
            ErrorCode::PARSE_ERROR,
            "Parsed failed: not a json map format",
        ))
    }
}

pub fn convert_service_error_to_error_data(service_error: ServiceError) -> ErrorData {
    match service_error {
        ServiceError::McpError(mcp_err) => mcp_err,
        err => convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err),
    }
}

pub fn convert_error_to_error_data(
    error_code: ErrorCode,
    error: impl std::error::Error,
) -> ErrorData {
    ErrorData {
        code: error_code,
        message: error.to_string().into(),
        data: None,
    }
}

pub fn convert_text_to_error_data(
    error_code: ErrorCode,
    text: impl Into<Cow<'static, str>>,
) -> ErrorData {
    ErrorData {
        code: error_code,
        message: text.into().into(),
        data: None,
    }
}

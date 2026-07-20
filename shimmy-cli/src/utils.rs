use std::borrow::Cow;

use rmcp::{
    model::{ErrorCode, ErrorData, NumberOrString, RequestId},
    service::ServiceError,
};
use serde::Serialize;
use serde_json::{Value, json};
use tokio::time::Duration;

use shimmy_common::common_structs::{Id, MCPError};

pub fn convert_request_id(id: &RequestId) -> Id {
    match id {
        NumberOrString::String(s) => Id::StringId(s.to_string()),
        NumberOrString::Number(num) => Id::NumberId(*num),
    }
}

pub fn convert_error_data(error_data: ErrorData) -> MCPError {
    MCPError::new(error_data.code.0, error_data.message, error_data.data)
}

pub fn convert_optional_params<S>(params: &Option<S>) -> Result<Option<Value>, ErrorData>
where
    S: Serialize,
{
    match params {
        Some(p) => {
            let value = convert_to_json_value(p)?;
            if value.as_object().map_or(false, |m| m.is_empty()) {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        }
        None => Ok(None),
    }
}

pub fn convert_to_json_value<S>(obj: S) -> Result<Value, ErrorData>
where
    S: Serialize,
{
    let res = serde_json::to_value(obj)
        .map_err(|err| convert_error_to_error_data(ErrorCode::PARSE_ERROR, err));

    // Map unit type () to empty map object that follows MCP protocol as response with empty result
    if let Ok(Value::Null) = &res {
        return Ok(json!({}));
    }

    res
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

pub async fn sleep_for_seconds(seconds: u64) {
    tokio::time::sleep(Duration::from_secs(seconds)).await;
}

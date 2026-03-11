use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) static JSON_RPC: &str = "2.0";

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
#[serde(untagged)]
pub(crate) enum Id {
    NumberId(i64),
    StringId(String),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub(crate) enum MCPDataPacket {
    Request(MCPRequest),
    Response(MCPResponse),
    Notification(MCPNotification),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct MCPRequest {
    pub(crate) jsonrpc: String,
    pub(crate) id: Id,
    pub(crate) method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) params: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct MCPError {
    pub(crate) code: i32,
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub(crate) enum MCPResponse {
    Success {
        jsonrpc: String,
        id: Id,
        result: Value,
    },
    Fail {
        jsonrpc: String,
        id: Id,
        error: MCPError,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EmittedMCPResponse {
    pub(crate) request_id: String,
    pub(crate) response_id: String,

    pub(crate) response: Value,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct MCPNotification {
    pub(crate) jsonrpc: String,
    pub(crate) method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) params: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) enum Role {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct Annotations {
    audience: Option<Vec<Role>>,
    last_modified: Option<String>,
    priority: Option<f64>,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct TextContent {
    #[serde(rename = "_meta")]
    _meta: Option<Value>,
    annotations: Option<Annotations>,
    text: String,
    #[serde(rename = "type")]
    data_type: String,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub(crate) enum ContentBlock {
    Text(TextContent),
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ToolAnnotations {
    destructive_hint: Option<bool>,
    idempotent_hint: Option<bool>,
    open_world_hint: Option<bool>,
    read_only_hint: Option<bool>,
    title: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub(crate) struct FunctionSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) required: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub(crate) data_type: String,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Tool {
    #[serde(rename = "_meta")]
    pub(crate) _meta: Option<Value>,
    pub(crate) annotations: Option<ToolAnnotations>,
    pub(crate) description: Option<String>,
    pub(crate) input_schema: FunctionSchema,
    pub(crate) name: String,
    pub(crate) output_schema: Option<FunctionSchema>,
    pub(crate) title: Option<String>,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListToolsResult {
    #[serde(rename = "_meta")]
    pub(crate) _meta: Option<Value>,
    pub(crate) next_cursor: Option<String>,
    pub(crate) tools: Vec<Tool>,

    #[serde(flatten)]
    pub(crate) extra_fields: Value,
}

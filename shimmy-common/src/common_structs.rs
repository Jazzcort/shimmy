use jiff::{Timestamp, tz::TimeZone};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpServerTransport {
    Stdio,
    Http,
}

pub(crate) static JSON_RPC: &str = "2.0";

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
#[serde(untagged)]
pub enum Id {
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
pub struct MCPRequest {
    pub(crate) jsonrpc: String,
    pub(crate) id: Id,
    pub(crate) method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) params: Option<Value>,
}

impl MCPRequest {
    pub fn new<S>(id: Id, method: S, params: Option<Value>) -> Self
    where
        S: Into<String>,
    {
        MCPRequest {
            jsonrpc: JSON_RPC.into(),
            id,
            method: method.into(),
            params,
        }
    }
}

#[derive(Clone)]
pub(crate) struct StampedMcpRequest {
    pub(crate) request: MCPRequest,
    pub(crate) timestamp: Timestamp,
}

impl StampedMcpRequest {
    pub fn pack_for_serializing(self) -> StampedMcpRequestForSerialize {
        StampedMcpRequestForSerialize {
            request: self.request,
            timestamp: self.timestamp.to_zoned(TimeZone::system()).to_string(),
        }
    }
}

#[derive(Clone, Serialize)]
pub(crate) struct StampedMcpRequestForSerialize {
    pub(crate) request: MCPRequest,
    pub(crate) timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MCPError {
    pub(crate) code: i32,
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<Value>,
}

impl MCPError {
    pub fn new<S>(code: i32, message: S, data: Option<Value>) -> Self
    where
        S: Into<String>,
    {
        MCPError {
            code,
            message: message.into(),
            data,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub enum MCPResponse {
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

impl MCPResponse {
    pub fn succeed(id: Id, result: Value) -> Self {
        MCPResponse::Success {
            jsonrpc: JSON_RPC.into(),
            id,
            result,
        }
    }

    pub fn fail(id: Id, error: MCPError) -> Self {
        MCPResponse::Fail {
            jsonrpc: JSON_RPC.into(),
            id,
            error,
        }
    }
}

#[derive(Clone)]
pub(crate) struct StampedMcpResponse {
    pub(crate) response: MCPResponse,
    pub(crate) timestamp: Timestamp,
}

impl StampedMcpResponse {
    pub fn pack_for_serializing(self) -> StampedMcpResponseForSerialize {
        StampedMcpResponseForSerialize {
            response: self.response,
            timestamp: self.timestamp.to_zoned(TimeZone::system()).to_string(),
        }
    }
}

#[derive(Clone, Serialize)]
pub(crate) struct StampedMcpResponseForSerialize {
    pub(crate) response: MCPResponse,
    pub(crate) timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EmittedMCPResponse {
    pub(crate) request_id: String,
    pub(crate) response_id: String,

    pub(crate) response: Value,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MCPNotification {
    pub(crate) jsonrpc: String,
    pub(crate) method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) params: Option<Value>,
}

impl MCPNotification {
    pub fn new<S>(method: S, params: Option<Value>) -> Self
    where
        S: Into<String>,
    {
        MCPNotification {
            jsonrpc: JSON_RPC.into(),
            method: method.into(),
            params,
        }
    }
}

#[derive(Clone, Serialize)]
pub(crate) struct StampedMcpNotification {
    pub notification: MCPNotification,

    #[serde(serialize_with = "serialize_timestamp_as_string")]
    pub timestamp: Timestamp,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorEntry {
    pub id: String,

    #[serde(serialize_with = "serialize_timestamp_as_string")]
    pub timestamp: Timestamp,

    pub method: String,
    pub status: LogStatus,
    // MCPRequest | MCPNotification
    pub request: Value,
    pub request_type: RequestType,
    pub response: Option<MCPResponse>,
    // Option<String> handles 'string | null'
    pub stderr: Option<String>,
}

fn serialize_timestamp_as_string<S>(timestamp: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = timestamp.to_zoned(TimeZone::system()).to_string();
    serializer.serialize_str(&s)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogStatus {
    Success,
    Error,
    Request,
    Start,
    Notification,
    Pending,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RequestType {
    Client,
    Server,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionInitializeFinishRequest {
    pub response: MCPResponse,
    pub transport: McpServerTransport,
}

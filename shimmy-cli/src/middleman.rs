use std::ffi::OsStr;
use std::fmt::Display;
use std::sync::Arc;

use crate::utils::{
    convert_error_data, convert_error_to_error_data, convert_request_id,
    convert_service_error_to_error_data, convert_text_to_error_data, convert_to_json_value,
    sleep_for_seconds,
};
use crate::error::ShimmyError;
use reqwest::Client;
use rmcp::Peer;
use rmcp::model::{
    ClientRequest, ClientResult, CreateElicitationRequest,
    PingRequest, PingRequestMethod, ServerRequest, ServerResult,
};
use rmcp::{
    ClientHandler, RoleClient, RoleServer, ServerHandler, ServiceExt,
    handler::server::tool::ToolRouter,
    model::{
        CallToolRequestParams, CallToolResult, CancelledNotificationParam, ClientInfo,
        CompleteRequestParams, CompleteResult, CreateMessageRequestParams, CreateMessageResult,
        ErrorCode, ErrorData, Extensions, GetPromptRequestParams, GetPromptResult,
        InitializeRequestParams, InitializeResult,
        ListPromptsResult, ListResourceTemplatesResult, ListResourcesResult, ListRootsResult,
        ListToolsResult, PaginatedRequestParams,
        ProgressNotificationParam, ReadResourceRequestParams,
        ReadResourceResult, Request, ServerInfo,
        SetLevelRequestParams, SubscribeRequestParams, UnsubscribeRequestParams,
    },
    service::{NotificationContext, RequestContext, RunningService},
    tool_router,
    transport::{ConfigureCommandExt, StreamableHttpClientTransport, TokioChildProcess, stdio},
};
use serde::Serialize;
use shimmy_common::common_structs::{
    Id, MCPError, MCPNotification, MCPRequest, MCPResponse, McpServerTransport,
};
use tokio::process::Command;
use tokio::sync::OnceCell;

const SHIMMY_SERVER: &str = "http://127.0.0.1:13579";

pub enum McpClient {
    Stdio(McpStdioClient),
    Http(McpHttpClient),
}

impl McpClient {
    pub fn transport(&self) -> McpServerTransport {
        match self {
            McpClient::Stdio(_) => McpServerTransport::Stdio,
            McpClient::Http(_) => McpServerTransport::Http,
        }
    }
}

pub struct McpStdioClient {
    cmd: String,
    args: Vec<String>,
}

pub struct McpHttpClient {
    url: String,
}

impl McpStdioClient {
    pub fn new<S, I>(cmd: S, args: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        Self {
            cmd: cmd.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }
}

pub struct Middleman {
    tool_router: ToolRouter<Self>,
    shimmy_client: Arc<ShimmyClient>,
    mcp_client: McpClient,
    _server_service: OnceCell<RunningService<RoleClient, McpClientService>>,
    _client_service: Arc<OnceCell<Peer<RoleServer>>>,
}

#[derive(Debug)]
struct ShimmyClient {
    http_client: Client,
    _id: OnceCell<String>,
}

impl ShimmyClient {
    fn get_id(&self) -> Result<&String, ErrorData> {
        self._id.get().ok_or(convert_text_to_error_data(
            ErrorCode::INTERNAL_ERROR,
            "Missing ID from shimmy server",
        ))
    }

    /**
     * This is a helper function that pipe mcp request/response to shimmy app.
     * It does nothing if id is not initialized from shimmy app.
     * **/
    fn send_to_shimmy_app<S, Ser>(&self, path: S, json_data: Ser)
    where
        S: AsRef<str> + Display,
        Ser: Serialize + Send + 'static,
    {
        if let Ok(id) = self.get_id() {
            let client = self.http_client.clone();
            let url = format!("{}/{}/{}", SHIMMY_SERVER, path, id);

            tokio::task::spawn(async move {
                let _ = client.post(url).json(&json_data).send().await;
            });
        }
    }

    fn pipe_mcp_error_if_any<T, S>(
        &self,
        id: Id,
        path: S,
        result: Result<T, ErrorData>,
    ) -> Result<T, ErrorData>
    where
        S: AsRef<str> + Display,
    {
        if let Err(err) = &result {
            let error = MCPError::new(err.code.0, err.message.as_ref(), err.data.clone());
            let jsonrpc_error = MCPResponse::fail(id, error);
            self.send_to_shimmy_app(path, jsonrpc_error);
        }

        result
    }

    async fn disconnect(&self) {
        if let Ok(id) = self.get_id() {
            let _ = self
                .http_client
                .delete(format!("{}/disconnect/{}", SHIMMY_SERVER, id))
                .send()
                .await;
        }
    }
}

#[derive(Serialize)]
struct InitializeFinishRequest {
    response: MCPResponse,
    transport: McpServerTransport,
}

#[tool_router]
impl Middleman {
    fn new(
        mcp_client: McpClient,
        http_client: Client,
        client_service: Arc<OnceCell<Peer<RoleServer>>>,
    ) -> Self {
        return Self {
            tool_router: Self::tool_router(),
            // To share with client service
            shimmy_client: Arc::new(ShimmyClient {
                http_client,
                _id: OnceCell::new(),
            }),
            mcp_client,
            _server_service: OnceCell::new(),
            _client_service: client_service,
        };
    }

    fn get_service(&self) -> Result<&RunningService<RoleClient, McpClientService>, ErrorData> {
        self._server_service.get().ok_or(convert_text_to_error_data(
            ErrorCode::INTERNAL_ERROR,
            "Failed trying to use service before it's initialized",
        ))
    }

    async fn start_initialize_with_shimmy_app(
        &self,
        json_data: MCPRequest,
    ) -> Result<String, ErrorData> {
        self.shimmy_client
            .http_client
            .post(format!("{}/{}", SHIMMY_SERVER, "initialize/start"))
            .json(&json_data)
            .send()
            .await
            .map_err(|err| convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err))?
            .error_for_status()
            .map_err(|err| convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err))?
            .text()
            .await
            .map_err(|err| convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err))
    }

    async fn finish_initialize_with_shimmy_app(
        &self,
        jsonrpc_response: MCPResponse,
    ) -> Result<(), ErrorData> {
        let id = self.shimmy_client.get_id()?;
        let initialize_finish_request = InitializeFinishRequest {
            response: jsonrpc_response,
            transport: self.mcp_client.transport(),
        };

        let _ = self
            .shimmy_client
            .http_client
            .post(format!("{}/{}/{}", SHIMMY_SERVER, "initialize/finish", id))
            .json(&initialize_finish_request)
            .send()
            .await
            .map_err(|err| convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err))?
            .error_for_status()
            .map_err(|err| convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err))?;

        Ok(())
    }
}

impl Drop for Middleman {
    fn drop(&mut self) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                tokio::select! {
                    _ = self.shimmy_client.disconnect() =>  {}
                    _ = sleep_for_seconds(2) => {}
                }
            })
        })
    }
}

impl ServerHandler for Middleman {
    // This does not matter. We will gather these information from the real server during
    // initialize
    fn get_info(&self) -> ServerInfo {
        ServerInfo::default()
    }

    // Requests

    async fn initialize(
        &self,
        request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        let id = convert_request_id(&context.id);
        let final_result: Result<InitializeResult, ErrorData> = async {
            let params = convert_to_json_value(&request)?;
            let jsonrpc_request = MCPRequest::new(id.clone(), "initialize", Some(params));

            // Should not crach the mcp connection if we can not connect to shimmy app
            if let Ok(id) = self.start_initialize_with_shimmy_app(jsonrpc_request).await {
                let _ = self.shimmy_client._id.set(id);
            }

            let mcp_client = McpClientService {
                client_info: request.clone(),
                shimmy_client: self.shimmy_client.clone(),
                _service: self._client_service.clone(),
            };

            let mut initialize_result: Option<InitializeResult> = None;

            match &self.mcp_client {
                McpClient::Stdio(stdio_client) => {
                    let service = mcp_client
                        .serve(
                            TokioChildProcess::new(Command::new(&stdio_client.cmd).configure(
                                |_cmd| {
                                    _cmd.args(&stdio_client.args);
                                },
                            ))
                            .map_err(|err| {
                                convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err)
                            })?,
                        )
                        .await
                        .map_err(|err| {
                            convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err)
                        })?;

                    initialize_result = service.peer_info().cloned();

                    self._server_service.set(service).map_err(|err| {
                        convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err)
                    })?;
                }
                McpClient::Http(http_client) => {
                    let service = mcp_client
                        .serve(StreamableHttpClientTransport::from_uri(
                            http_client.url.clone(),
                        ))
                        .await
                        .map_err(|err| {
                            convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err)
                        })?;

                    initialize_result = service.peer_info().cloned();

                    self._server_service.set(service).map_err(|err| {
                        convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err)
                    })?;
                }
            }

            let initialize_result = initialize_result.ok_or(convert_text_to_error_data(
                ErrorCode::INTERNAL_ERROR,
                "Failed to fetch server information",
            ))?;

            let params = convert_to_json_value(&initialize_result)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), params);

            // Should not crach the mcp connection if we can not connect to shimmy app
            let _ = self
                .finish_initialize_with_shimmy_app(jsonrpc_response)
                .await;

            Ok(initialize_result)
        }
        .await;

        if let Err(error_data) = &final_result {
            let jsonrpc_error = MCPResponse::fail(id, convert_error_data(error_data.clone()));
            let _ = self.finish_initialize_with_shimmy_app(jsonrpc_error).await;
        }

        final_result
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let params = match &request {
                Some(paginate_params) => Some(convert_to_json_value(paginate_params)?),
                None => None,
            };
            let jsonrpc_request = MCPRequest::new(id.clone(), "tools/list", params);

            self.shimmy_client
                .send_to_shimmy_app("client/request", jsonrpc_request);

            let list_tools_result = self
                .get_service()?
                .list_tools(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;

            let result = convert_to_json_value(&list_tools_result)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), result);

            self.shimmy_client
                .send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(list_tools_result)
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "server/response", final_result)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let params = convert_to_json_value(&request)?;
            let jsonrpc_request = MCPRequest::new(id.clone(), "tools/call", Some(params));

            self.shimmy_client
                .send_to_shimmy_app("client/request", jsonrpc_request);

            let call_tool_result = self
                .get_service()?
                .call_tool(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let result = convert_to_json_value(&call_tool_result)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), result);

            self.shimmy_client
                .send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(call_tool_result)
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "server/response", final_result)
    }

    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let params = match &request {
                Some(inner) => Some(convert_to_json_value(inner)?),
                None => None,
            };
            let jsonrpc_request = MCPRequest::new(id.clone(), "resources/list", params);

            self.shimmy_client
                .send_to_shimmy_app("client/request", jsonrpc_request);

            let list_resources_result = self
                .get_service()?
                .list_resources(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let result = convert_to_json_value(&list_resources_result)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), result);

            self.shimmy_client
                .send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(list_resources_result)
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "server/response", final_result)
    }

    async fn list_resource_templates(
        &self,
        request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let params = match &request {
                Some(inner) => Some(convert_to_json_value(inner)?),
                None => None,
            };
            let jsonrpc_request = MCPRequest::new(id.clone(), "resources/templates/list", params);

            self.shimmy_client
                .send_to_shimmy_app("client/request", jsonrpc_request);

            let list_resource_templates_result = self
                .get_service()?
                .list_resource_templates(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let result = convert_to_json_value(&list_resource_templates_result)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), result);

            self.shimmy_client
                .send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(list_resource_templates_result)
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "server/response", final_result)
    }

    async fn subscribe(
        &self,
        request: SubscribeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<(), ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let params = convert_to_json_value(&request)?;
            let jsonrpc_request = MCPRequest::new(id.clone(), "resources/subscribe", Some(params));

            self.shimmy_client
                .send_to_shimmy_app("client/request", jsonrpc_request);

            let subscribe_result = self
                .get_service()?
                .subscribe(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let result = convert_to_json_value(&subscribe_result)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), result);

            self.shimmy_client
                .send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(subscribe_result)
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "server/response", final_result)
    }

    async fn unsubscribe(
        &self,
        request: UnsubscribeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<(), ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let params = convert_to_json_value(&request)?;
            let jsonrpc_request =
                MCPRequest::new(id.clone(), "resources/unsubscribe", Some(params));

            self.shimmy_client
                .send_to_shimmy_app("client/request", jsonrpc_request);

            let unsubscribe_result = self
                .get_service()?
                .unsubscribe(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let result = convert_to_json_value(&unsubscribe_result)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), result);

            self.shimmy_client
                .send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(unsubscribe_result)
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "server/response", final_result)
    }

    async fn complete(
        &self,
        request: CompleteRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CompleteResult, ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let params = convert_to_json_value(&request)?;
            let jsonrpc_request = MCPRequest::new(id.clone(), "completion/complete", Some(params));

            self.shimmy_client
                .send_to_shimmy_app("client/request", jsonrpc_request);

            let complete_result = self
                .get_service()?
                .complete(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let result = convert_to_json_value(&complete_result)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), result);

            self.shimmy_client
                .send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(complete_result)
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "server/response", final_result)
    }

    async fn set_level(
        &self,
        request: SetLevelRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<(), ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let params = convert_to_json_value(&request)?;
            let jsonrpc_request = MCPRequest::new(id.clone(), "logging/setLevel", Some(params));

            self.shimmy_client
                .send_to_shimmy_app("client/request", jsonrpc_request);

            let set_level_result = self
                .get_service()?
                .set_level(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let result = convert_to_json_value(&set_level_result)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), result);

            self.shimmy_client
                .send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(set_level_result)
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "server/response", final_result)
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let params = convert_to_json_value(&request)?;
            let jsonrpc_request = MCPRequest::new(id.clone(), "resources/read", Some(params));

            self.shimmy_client
                .send_to_shimmy_app("client/request", jsonrpc_request);

            let read_resource_result = self
                .get_service()?
                .read_resource(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let result = convert_to_json_value(&read_resource_result)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), result);

            self.shimmy_client
                .send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(read_resource_result)
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "server/response", final_result)
    }

    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let params = match &request {
                Some(req) => Some(convert_to_json_value(req)?),
                None => None,
            };
            let jsonrpc_request = MCPRequest::new(id.clone(), "prompts/list", params);

            self.shimmy_client
                .send_to_shimmy_app("client/request", jsonrpc_request);

            let list_prompts_response = self
                .get_service()?
                .list_prompts(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let result = convert_to_json_value(&list_prompts_response)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), result);

            self.shimmy_client
                .send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(list_prompts_response)
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "server/response", final_result)
    }

    async fn ping(&self, context: RequestContext<RoleServer>) -> Result<(), ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let jsonrpc_request = MCPRequest::new(id.clone(), "ping", None);

            self.shimmy_client
                .send_to_shimmy_app("client/request", jsonrpc_request);

            let internal_ping_request = PingRequest {
                method: PingRequestMethod,
                extensions: Extensions::new(),
            };
            if let ServerResult::EmptyResult(_) = self
                .get_service()?
                .send_request(ClientRequest::PingRequest(internal_ping_request))
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?
            {
                let jsonrpc_response = MCPResponse::succeed(id.clone(), convert_to_json_value(())?);
                self.shimmy_client
                    .send_to_shimmy_app("server/response", jsonrpc_response);
                Ok(())
            } else {
                Err(convert_text_to_error_data(
                    ErrorCode::INTERNAL_ERROR,
                    "Expect ping response, but got something else",
                ))
            }
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "server/response", final_result)
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let params = convert_to_json_value(&request)?;
            let jsonrpc_request = MCPRequest::new(id.clone(), "prompts/get", Some(params));

            self.shimmy_client
                .send_to_shimmy_app("client/request", jsonrpc_request);

            let get_prompt_response = self
                .get_service()?
                .get_prompt(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let result = convert_to_json_value(&get_prompt_response)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), result);

            self.shimmy_client
                .send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(get_prompt_response)
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "server/response", final_result)
    }

    // Notifications

    async fn on_initialized(&self, _context: NotificationContext<RoleServer>) -> () {
        // No need to trigger notify_initialized since it's already handled by rmcp Client

        let jsonrpc_notification = MCPNotification::new("notifications/initialized", None);
        self.shimmy_client
            .send_to_shimmy_app("client/notification", jsonrpc_notification);
    }

    async fn on_cancelled(
        &self,
        notification: CancelledNotificationParam,
        _context: NotificationContext<RoleServer>,
    ) {
        let _: Result<(), ErrorData> = async {
            self.get_service()?
                .notify_cancelled(notification.clone())
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;

            // Only log the notification when it's delivered successfully
            let params = convert_to_json_value(&notification)?;
            let jsonrpc_notification =
                MCPNotification::new("notifications/cancelled", Some(params));
            self.shimmy_client
                .send_to_shimmy_app("client/notification", jsonrpc_notification);
            Ok(())
        }
        .await;
    }

    async fn on_progress(
        &self,
        notification: ProgressNotificationParam,
        _context: NotificationContext<RoleServer>,
    ) {
        let _: Result<(), ErrorData> = async {
            self.get_service()?
                .notify_progress(notification.clone())
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;

            // Only log the notification when it's delivered successfully
            let params = convert_to_json_value(&notification)?;
            let jsonrpc_notification = MCPNotification::new("notifications/progress", Some(params));
            self.shimmy_client
                .send_to_shimmy_app("client/notification", jsonrpc_notification);
            Ok(())
        }
        .await;
    }

    async fn on_roots_list_changed(&self, _context: NotificationContext<RoleServer>) {
        let _: Result<(), ErrorData> = async {
            self.get_service()?
                .notify_roots_list_changed()
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;

            let jsonrpc_notification =
                MCPNotification::new("notifications/roots/list_changed", None);
            self.shimmy_client
                .send_to_shimmy_app("client/notification", jsonrpc_notification);
            Ok(())
        }
        .await;
    }
}

#[derive(Debug, Clone)]
pub struct McpClientService {
    client_info: ClientInfo,
    shimmy_client: Arc<ShimmyClient>,
    _service: Arc<OnceCell<Peer<RoleServer>>>,
}

impl McpClientService {
    fn get_service(&self) -> Result<&Peer<RoleServer>, ErrorData> {
        self._service.get().ok_or(convert_text_to_error_data(
            ErrorCode::INTERNAL_ERROR,
            "Failed trying to use service before it's initialized",
        ))
    }
}

impl ClientHandler for McpClientService {
    fn get_info(&self) -> ClientInfo {
        self.client_info.clone()
    }

    // Requests

    async fn ping(&self, context: RequestContext<RoleClient>) -> Result<(), ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let jsonrpc_request = MCPRequest::new(id.clone(), "ping", None);

            self.shimmy_client
                .send_to_shimmy_app("server/request", jsonrpc_request);

            let internal_ping_request = PingRequest {
                method: PingRequestMethod,
                extensions: Extensions::new(),
            };

            if let ClientResult::EmptyResult(_) = self
                .get_service()?
                .send_request(ServerRequest::PingRequest(internal_ping_request))
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?
            {
                let jsonrpc_response = MCPResponse::succeed(id.clone(), convert_to_json_value(())?);
                self.shimmy_client
                    .send_to_shimmy_app("client/response", jsonrpc_response);

                Ok(())
            } else {
                Err(convert_text_to_error_data(
                    ErrorCode::INTERNAL_ERROR,
                    "Expect ping response, but got something else",
                ))
            }
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "client/response", final_result)
    }

    async fn create_elicitation(
        &self,
        request: rmcp::model::CreateElicitationRequestParams,
        context: RequestContext<RoleClient>,
    ) -> Result<rmcp::model::CreateElicitationResult, ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let params = convert_to_json_value(&request)?;
            let jsonrpc_request = MCPRequest::new(id.clone(), "elicitation/create", Some(params));

            self.shimmy_client
                .send_to_shimmy_app("server/request", jsonrpc_request);

            let create_elicitation_request: CreateElicitationRequest =
                Request::new(request.clone());
            if let ClientResult::CreateElicitationResult(create_elicitation_result) = self
                .get_service()?
                .send_request(ServerRequest::CreateElicitationRequest(
                    create_elicitation_request,
                ))
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?
            {
                let result = convert_to_json_value(&create_elicitation_result)?;
                let jsonrpc_response = MCPResponse::succeed(id.clone(), result);
                self.shimmy_client
                    .send_to_shimmy_app("client/response", jsonrpc_response);

                Ok(create_elicitation_result)
            } else {
                Err(convert_text_to_error_data(
                    ErrorCode::INTERNAL_ERROR,
                    "Expect elicitation result, but got something else",
                ))
            }
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "client/response", final_result)
    }

    async fn create_message(
        &self,
        request: CreateMessageRequestParams,
        context: RequestContext<RoleClient>,
    ) -> Result<CreateMessageResult, ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let params = convert_to_json_value(&request)?;
            let jsonrpc_request =
                MCPRequest::new(id.clone(), "sampling/createMessage", Some(params));

            self.shimmy_client
                .send_to_shimmy_app("server/request", jsonrpc_request);

            let create_message_result = self
                .get_service()?
                .create_message(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;

            let result = convert_to_json_value(&create_message_result)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), result);
            self.shimmy_client
                .send_to_shimmy_app("client/response", jsonrpc_response);

            Ok(create_message_result)
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "client/response", final_result)
    }

    async fn list_roots(
        &self,
        context: RequestContext<RoleClient>,
    ) -> Result<ListRootsResult, ErrorData> {
        let id = convert_request_id(&context.id);

        let final_result = async {
            let jsonrpc_request = MCPRequest::new(id.clone(), "roots/list", None);

            self.shimmy_client
                .send_to_shimmy_app("server/request", jsonrpc_request);

            let list_roots_result = self
                .get_service()?
                .list_roots()
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;

            let result = convert_to_json_value(&list_roots_result)?;
            let jsonrpc_response = MCPResponse::succeed(id.clone(), result);
            self.shimmy_client
                .send_to_shimmy_app("client/response", jsonrpc_response);

            Ok(list_roots_result)
        }
        .await;

        self.shimmy_client
            .pipe_mcp_error_if_any(id, "client/response", final_result)
    }

    // Notifications

    async fn on_tool_list_changed(&self, _context: NotificationContext<RoleClient>) -> () {
        let _: Result<(), ErrorData> = async {
            let _ = self
                .get_service()?
                .notify_tool_list_changed()
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;

            // Only log the notification when it's delivered successfully
            let jsonrpc_notification =
                MCPNotification::new("notifications/tools/list_changed", None);
            self.shimmy_client
                .send_to_shimmy_app("server/notification", jsonrpc_notification);

            Ok(())
        }
        .await;
    }

    async fn on_cancelled(
        &self,
        notification: CancelledNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) -> () {
        let _: Result<(), ErrorData> = async {
            let _ = self
                .get_service()?
                .notify_cancelled(notification.clone())
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;

            // Only log the notification when it's delivered successfully
            let params = convert_to_json_value(&notification)?;
            let jsonrpc_notification =
                MCPNotification::new("notifications/cancelled", Some(params));
            self.shimmy_client
                .send_to_shimmy_app("server/notification", jsonrpc_notification);
            Ok(())
        }
        .await;
    }
}

pub async fn spawn_middleman_with_stdio<S, I>(cmd: S, args: I) -> Result<(), ShimmyError>
where
    S: Into<String> + AsRef<OsStr>,
    I: IntoIterator<Item = S>,
{
    let mcp_client = McpClient::Stdio(McpStdioClient::new(cmd, args));
    let http_client = Client::builder().build()?;
    let client_service = Arc::new(OnceCell::new());

    let middleman = Middleman::new(mcp_client, http_client, client_service.clone());

    let middleman_service = middleman
        .serve(stdio())
        .await
        .inspect_err(|e| println!("Error starting server: {}", e))?;

    let _ = client_service.set(middleman_service.clone());

    middleman_service.waiting().await?;

    Ok(())
}

pub async fn spawn_middleman_with_http(url: String) -> Result<(), ShimmyError> {
    let mcp_client = McpClient::Http(McpHttpClient { url });
    let http_client = Client::builder().build()?;
    let client_service = Arc::new(OnceCell::new());

    let middleman = Middleman::new(mcp_client, http_client, client_service.clone());

    let middleman_service = middleman
        .serve(stdio())
        .await
        .inspect_err(|e| println!("Error starting server: {}", e))?;

    let _ = client_service.set(middleman_service.clone());

    middleman_service.waiting().await?;

    Ok(())
}

use std::ffi::OsStr;
use std::fmt::Display;

use crate::utils::{
    convert_error_to_error_data, convert_service_error_to_error_data, convert_text_to_error_data,
    convert_to_json_object, create_jsonrpc_error, create_jsonrpc_request, create_jsonrpc_response,
};
use crate::{error::ShimmyError, utils::create_mcp_request};
use reqwest::{Client, Response};
use rmcp::{
    ClientHandler, RoleClient, RoleServer, ServerHandler, ServiceExt,
    handler::server::tool::ToolRouter,
    model::{
        Annotated, CallToolRequestParams, CallToolResult, ClientInfo, ErrorCode, ErrorData,
        Extensions, Implementation, InitializeRequestParams, InitializeResult, JsonRpcRequest,
        JsonRpcVersion2_0, ListPromptsResult, ListResourcesResult, ListToolsResult,
        PaginatedRequestParams, Prompt, ProtocolVersion, RawResource, ReadResourceRequestParams,
        ReadResourceResult, Request, RequestId, ServerCapabilities, ServerInfo, Tool,
    },
    service::{RequestContext, RunningService, ServiceError},
    tool_handler, tool_router,
    transport::{ConfigureCommandExt, TokioChildProcess, stdio},
};
use serde::Serialize;
use serde_json::{Value, json};
use tokio::process::Command;
use tokio::sync::OnceCell;

const SHIMMY_SERVER: &str = "http://127.0.0.1:13579";

pub enum McpClient {
    Stdio(McpStdioClient),
}

pub struct McpStdioClient {
    cmd: String,
    args: Vec<String>,
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
    protocol_version: ProtocolVersion,
    server_info: Implementation,
    server_capabilities: ServerCapabilities,
    instruction: Option<String>,

    tool_router: ToolRouter<Self>,
    http_client: Client,
    mcp_client: McpClient,
    _service: OnceCell<RunningService<RoleClient, McpClientService>>,
    _id: OnceCell<String>,
}

#[tool_router]
impl Middleman {
    fn new(
        protocol_version: ProtocolVersion,
        server_info: Implementation,
        server_capabilities: ServerCapabilities,
        instruction: Option<String>,
        mcp_client: McpClient,
        http_client: Client,
    ) -> Self {
        return Self {
            protocol_version,
            server_info,
            server_capabilities,
            instruction,
            tool_router: Self::tool_router(),
            http_client,
            mcp_client,
            _service: OnceCell::new(),
            _id: OnceCell::new(),
        };
    }

    fn get_service(&self) -> Result<&RunningService<RoleClient, McpClientService>, ErrorData> {
        self._service.get().ok_or(convert_text_to_error_data(
            ErrorCode::INTERNAL_ERROR,
            "Failed trying to use service before it's initialized",
        ))
    }

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
        id: RequestId,
        path: S,
        result: Result<T, ErrorData>,
    ) -> Result<T, ErrorData>
    where
        S: AsRef<str> + Display,
    {
        if let Err(err) = &result {
            let jsonrpc_error = create_jsonrpc_error(id, err.clone());
            self.send_to_shimmy_app(path, jsonrpc_error);
        }

        result
    }

    async fn start_initialize_with_shimmy_app<Ser>(
        &self,
        json_data: Ser,
    ) -> Result<String, ErrorData>
    where
        Ser: Serialize,
    {
        self.http_client
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

    async fn finish_initialize_with_shimmy_app<Ser>(&self, json_data: Ser) -> Result<(), ErrorData>
    where
        Ser: Serialize,
    {
        let id = self.get_id()?;
        let _ = self
            .http_client
            .post(format!("{}/{}/{}", SHIMMY_SERVER, "initialize/finish", id))
            .json(&json_data)
            .send()
            .await
            .map_err(|err| convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err))?
            .error_for_status()
            .map_err(|err| convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err))?;

        Ok(())
    }
}

impl ServerHandler for Middleman {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: self.protocol_version.clone(),
            server_info: self.server_info.clone(),
            capabilities: self.server_capabilities.clone(),
            instructions: self.instruction.clone(),
        }
    }

    async fn initialize(
        &self,
        request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        let final_result = async {
            let params = convert_to_json_object(&request)?;
            let initialize_request = create_mcp_request("initialize", params);
            let jsonrpc_request = create_jsonrpc_request(context.id.clone(), initialize_request);

            // Should not crach the mcp connection if we can not connect to shimmy app
            if let Ok(id) = self.start_initialize_with_shimmy_app(jsonrpc_request).await {
                let _ = self._id.set(id);
            }

            let mcp_client = McpClientService {
                client_info: request.clone(),
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

                    self._service.set(service).map_err(|err| {
                        convert_error_to_error_data(ErrorCode::INTERNAL_ERROR, err)
                    })?;
                }
            }

            let initialize_result = initialize_result.ok_or(convert_text_to_error_data(
                ErrorCode::INTERNAL_ERROR,
                "Failed to fetch server information",
            ))?;
            let params = convert_to_json_object(&initialize_result)?;
            let jsonrpc_response = create_jsonrpc_response(context.id.clone(), params);

            // Should not crach the mcp connection if we can not connect to shimmy app
            let _ = self
                .finish_initialize_with_shimmy_app(jsonrpc_response)
                .await;

            Ok(initialize_result)
        }
        .await;

        self.pipe_mcp_error_if_any(context.id, "server/response", final_result)
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let final_result = async {
            let params = match &request {
                Some(paginate_params) => convert_to_json_object(paginate_params)?,
                None => serde_json::Map::new(),
            };
            let list_tools_request = create_mcp_request("tools/list", params);
            let jsonrpc_request = create_jsonrpc_request(context.id.clone(), list_tools_request);

            self.send_to_shimmy_app("client/request", jsonrpc_request);

            let list_tools_result = self
                .get_service()?
                .list_tools(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;

            let params = convert_to_json_object(&list_tools_result)?;
            let jsonrpc_response = create_jsonrpc_response(context.id.clone(), params);
            self.send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(list_tools_result)
        }
        .await;

        self.pipe_mcp_error_if_any(context.id, "server/response", final_result)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let final_result = async {
            let params = convert_to_json_object(&request)?;
            let call_tool_request = create_mcp_request("tools/call", params);
            let jsonrpc_request = create_jsonrpc_request(context.id.clone(), call_tool_request);

            self.send_to_shimmy_app("client/request", jsonrpc_request);

            let call_tool_result = self
                .get_service()?
                .call_tool(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let params = convert_to_json_object(&call_tool_result)?;
            let jsonrpc_response = create_jsonrpc_response(context.id.clone(), params);

            self.send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(call_tool_result)
        }
        .await;

        self.pipe_mcp_error_if_any(context.id, "server/response", final_result)
    }

    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        let final_result = async {
            let params = match &request {
                Some(paginate_params) => convert_to_json_object(paginate_params)?,
                None => serde_json::Map::new(),
            };
            let list_resources_request = create_mcp_request("resources/list", params);
            let jsonrpc_request =
                create_jsonrpc_request(context.id.clone(), list_resources_request);

            self.send_to_shimmy_app("client/request", jsonrpc_request);

            let list_resources_request = self
                .get_service()?
                .list_resources(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let params = convert_to_json_object(&list_resources_request)?;
            let jsonrpc_response = create_jsonrpc_response(context.id.clone(), params);

            self.send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(list_resources_request)
        }
        .await;

        self.pipe_mcp_error_if_any(context.id, "server/response", final_result)
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        let final_result = async {
            let params = convert_to_json_object(&request)?;
            let read_resource_request = create_mcp_request("resources/read", params);
            let jsonrpc_request = create_jsonrpc_request(context.id.clone(), read_resource_request);

            self.send_to_shimmy_app("client/request", jsonrpc_request);

            let read_resource_response = self
                .get_service()?
                .read_resource(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let params = convert_to_json_object(&read_resource_response)?;
            let jsonrpc_response = create_jsonrpc_response(context.id.clone(), params);

            self.send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(read_resource_response)
        }
        .await;

        self.pipe_mcp_error_if_any(context.id, "server/response", final_result)
    }

    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        let final_result = async {
            let params = match &request {
                Some(paginate_params) => convert_to_json_object(paginate_params)?,
                None => serde_json::Map::new(),
            };
            let list_prompts_request = create_mcp_request("prompts/list", params);
            let jsonrpc_request = create_jsonrpc_request(context.id.clone(), list_prompts_request);

            self.send_to_shimmy_app("client/request", jsonrpc_request);

            let list_prompts_response = self
                .get_service()?
                .list_prompts(request)
                .await
                .map_err(|err| convert_service_error_to_error_data(err))?;
            let params = convert_to_json_object(&list_prompts_response)?;
            let jsonrpc_response = create_jsonrpc_response(context.id.clone(), params);

            self.send_to_shimmy_app("server/response", jsonrpc_response);

            Ok(list_prompts_response)
        }
        .await;

        self.pipe_mcp_error_if_any(context.id, "server/response", final_result)
    }
}

#[derive(Debug, Clone)]
pub struct McpClientService {
    client_info: ClientInfo,
}

impl ClientHandler for McpClientService {
    fn get_info(&self) -> ClientInfo {
        self.client_info.clone()
    }
}

pub async fn spawn_middleman_with_stdio<S, I>(cmd: S, args: I) -> Result<(), ShimmyError>
where
    S: Into<String> + AsRef<OsStr> + Clone,
    I: IntoIterator<Item = S> + Clone,
{
    let mcp_client = McpClient::Stdio(McpStdioClient::new(cmd.clone(), args.clone()));
    let http_client = Client::builder().build()?;
    let service = ()
        .serve(TokioChildProcess::new(Command::new(cmd).configure(
            |_cmd| {
                _cmd.args(args);
            },
        ))?)
        .await?;

    let initialize_request = service
        .peer_info()
        .ok_or(ShimmyError::Middleman("Missing server info".to_string()))?
        .clone();

    drop(service);

    let middleman = Middleman::new(
        initialize_request.protocol_version,
        initialize_request.server_info,
        initialize_request.capabilities,
        initialize_request.instructions,
        mcp_client,
        http_client,
    );

    let middleman_service = middleman
        .serve(stdio())
        .await
        .inspect_err(|e| println!("Error starting server: {}", e))?;

    middleman_service.waiting().await?;

    Ok(())
}

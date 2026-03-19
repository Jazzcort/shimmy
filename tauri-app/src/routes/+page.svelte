<script lang="ts">
	import Toolbar from "$lib/components/inspector/Toolbar.svelte";
	import TimelinePanel from "$lib/components/inspector/TimelinePanel.svelte";
	import RequestPanel from "$lib/components/inspector/RequestPanel.svelte";
	import ResponsePanel from "$lib/components/inspector/ResponsePanel.svelte";
	import { listen, type UnlistenFn } from "@tauri-apps/api/event";
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";
	import type {
		IncomingNotification,
		IncomingRequest,
		IncomingResponse,
		McpInitializeFinish,
		StampedMcpNotification,
		StampedMcpRequest,
		StampedMcpResponse,
	} from "$lib/types/emittedMessages";
	import type {
		InspectorEntry,
		McpConnection,
	} from "$lib/types/inspector";
	import {
		isJSONRPCErrorResponse,
		isJSONRPCResultResponse,
		type InitializeRequest,
		type InitializeRequestParams,
		type InitializeResult,
		type JSONRPCRequest,
		type JSONRPCResponse,
		type JSONRPCResultResponse,
	} from "@modelcontextprotocol/sdk/types.js";
	import { createLegitSvelteId } from "$lib/utils";
	import { toast } from "svelte-sonner";

	let connections = $state<McpConnection[]>([]);
	let entries = $state<InspectorEntry[]>([]);
	let pendingConnections = $state<string[]>([]);

	let selectedEntryId = $state<number | string | null>(null);
	let filter = $state("");
	let selectedConnectionId = $state<string | null>(null);
	let paused = $state(false);

	onMount(() => {
		let unlistenInitializeStart: UnlistenFn | undefined = undefined;
		let unlistenInitializeFinish: UnlistenFn | undefined =
			undefined;
		let unlistenInitializeFail: UnlistenFn | undefined = undefined;
		let unlistenClientRequest: UnlistenFn | undefined = undefined;
		let unlistenServerResponse: UnlistenFn | undefined = undefined;

		let unlistenServerRequest: UnlistenFn | undefined = undefined;
		let unlistenClientResponse: UnlistenFn | undefined = undefined;

		let unlistenClientNotification: UnlistenFn | undefined =
			undefined;
		let unlistenServerNotification: UnlistenFn | undefined =
			undefined;

		async function startListenInitializeStart() {
			unlistenInitializeStart = await listen<string>(
				"mcp-initialize-start",
				async (event) => {
					console.log("initializing!!", event);
					pendingConnections.push(event.payload);
					toast.info("Connecting to a shimmy client...");
				},
			);
		}

		async function startListenInitializeFinish() {
			unlistenInitializeFinish =
				await listen<McpInitializeFinish>(
					"mcp-initialize-finish",
					async (event) => {
						console.log(
							"initialized!!",
							event,
						);
						const original_length =
							pendingConnections.length;

						console.log(
							"before",
							$state.snapshot(
								pendingConnections,
							),
						);

						pendingConnections =
							pendingConnections.filter(
								(id) =>
									id !==
									event
										.payload
										.serverId,
							);

						console.log(
							"after",
							$state.snapshot(
								pendingConnections,
							),
						);

						if (
							pendingConnections.length <
							original_length
						) {
							const initialize_request =
								(
									(await invoke(
										"get_mcp_client_request",
										{
											serverId: event
												.payload
												.serverId,
											requestId: event
												.payload
												.requestId,
										},
									)) as StampedMcpRequest
								).request;
							const clientName = (
								initialize_request.params as InitializeRequestParams
							).clientInfo.name;

							const initialize_response =
								(
									(await invoke(
										"get_mcp_server_response",
										{
											serverId: event
												.payload
												.serverId,
											responseId: event
												.payload
												.requestId,
										},
									)) as StampedMcpResponse
								)
									.response as JSONRPCResultResponse;

							const serverName = (
								initialize_response.result as InitializeResult
							).serverInfo.name;

							const connectionName = `${clientName}-${serverName}`;
							connections.push({
								transport: event
									.payload
									.transport,
								id: event
									.payload
									.serverId,
								name: connectionName,
							});

							toast.success(
								`Connection "${connectionName}" is established`,
							);

							if (
								!selectedConnectionId
							) {
								console.log(
									"first connection!!!",
								);
								selectedConnectionId =
									event
										.payload
										.serverId;
							}
						}
					},
				);
		}

		async function startListenInitializeFail() {
			unlistenInitializeFail = await listen<{
				serverId: string;
				requestId: number | string;
			}>(
				"mcp-initialize-fail",
				async (event) => {
					console.log(
						"initialize failed!!",
						event,
					);
					pendingConnections =
						pendingConnections.filter(
							(id) =>
								id !==
								event.payload
									.serverId,
						);
					toast.error(
						"A connection has failed",
					);
				},
			);
		}

		async function startListenClientRequest() {
			unlistenClientRequest = await listen<IncomingRequest>(
				"mcp-client-request",
				async (event) => {
					const legitSvelteId =
						createLegitSvelteId(
							event.payload.requestId,
							"client",
						);

					if (
						event.payload.serverId ===
							selectedConnectionId &&
						entries.every(
							(entry) =>
								entry.id !==
								legitSvelteId,
						)
					) {
						const stampedRequest =
							(await invoke(
								"get_mcp_client_request",
								{
									serverId: event
										.payload
										.serverId,
									requestId: event
										.payload
										.requestId,
								},
							)) as StampedMcpRequest;

						console.log(
							"stamped request",
							stampedRequest,
						);

						const entry: InspectorEntry = {
							id: legitSvelteId,
							timestamp: stampedRequest.timestamp,
							method: stampedRequest
								.request.method,
							status: "pending",
							request: stampedRequest.request,
							requestType: "client",
							response: null,
							stderr: null,
						};

						entries.push(entry);
					}
				},
			);
		}

		async function startListenServerRequest() {
			unlistenServerRequest = await listen<IncomingRequest>(
				"mcp-server-request",
				async (event) => {
					const legitSvelteId =
						createLegitSvelteId(
							event.payload.requestId,
							"server",
						);

					if (
						event.payload.serverId ===
							selectedConnectionId &&
						entries.every(
							(entry) =>
								entry.id !==
								legitSvelteId,
						)
					) {
						const stampedRequest =
							(await invoke(
								"get_mcp_server_request",
								{
									serverId: event
										.payload
										.serverId,
									requestId: event
										.payload
										.requestId,
								},
							)) as StampedMcpRequest;

						console.log(
							"stamped request",
							stampedRequest,
						);

						const entry: InspectorEntry = {
							id: legitSvelteId,
							timestamp: stampedRequest.timestamp,
							method: stampedRequest
								.request.method,
							status: "pending",
							request: stampedRequest.request,
							requestType: "server",
							response: null,
							stderr: null,
						};

						entries.push(entry);
					}
				},
			);
		}

		async function startListenServerResponse() {
			unlistenServerResponse = await listen<IncomingResponse>(
				"mcp-server-response",
				async (event) => {
					if (
						event.payload.serverId ===
						selectedConnectionId
					) {
						const stampedResponse =
							(await invoke(
								"get_mcp_server_response",
								{
									serverId: event
										.payload
										.serverId,
									responseId: event
										.payload
										.responseId,
								},
							)) as StampedMcpResponse;

						console.log(
							"stamped response",
							stampedResponse,
						);

						const legitSvelteId =
							createLegitSvelteId(
								event.payload
									.responseId,
								"client",
							);

						const entry = entries.find(
							(item) =>
								item.id ===
								legitSvelteId,
						);

						if (entry) {
							entry.response =
								stampedResponse.response;

							if (
								isJSONRPCErrorResponse(
									stampedResponse.response,
								)
							) {
								entry.stderr =
									stampedResponse.response.error.message;
								entry.status =
									"error";
							} else {
								entry.status =
									"success";
							}
						}
					}
				},
			);
		}

		async function startListenClientResponse() {
			unlistenClientResponse = await listen<IncomingResponse>(
				"mcp-client-response",
				async (event) => {
					console.log("client response: ", event);
					if (
						event.payload.serverId ===
						selectedConnectionId
					) {
						const stampedResponse =
							(await invoke(
								"get_mcp_client_response",
								{
									serverId: event
										.payload
										.serverId,
									responseId: event
										.payload
										.responseId,
								},
							)) as StampedMcpResponse;

						console.log(
							"stamped response",
							stampedResponse,
						);

						const legitSvelteId =
							createLegitSvelteId(
								event.payload
									.responseId,
								"server",
							);

						const entry = entries.find(
							(item) =>
								item.id ===
								legitSvelteId,
						);

						if (entry) {
							entry.response =
								stampedResponse.response;

							if (
								isJSONRPCErrorResponse(
									stampedResponse.response,
								)
							) {
								entry.stderr =
									stampedResponse.response.error.message;
								entry.status =
									"error";
							} else {
								entry.status =
									"success";
							}
						}
					}
				},
			);
		}

		async function startListenClientNotification() {
			unlistenClientNotification =
				await listen<IncomingNotification>(
					"mcp-client-notification",
					async (event) => {
						console.log(
							"client notification",
							event,
						);
						const legitSvelteId =
							createLegitSvelteId(
								event.payload
									.notificationId,
								"client",
							);

						if (
							event.payload
								.serverId ===
								selectedConnectionId &&
							entries.every(
								(entry) =>
									entry.id !==
									legitSvelteId,
							)
						) {
							const stampedNotification =
								(await invoke(
									"get_mcp_client_notification",
									{
										serverId: event
											.payload
											.serverId,
										notificationId:
											event
												.payload
												.notificationId,
									},
								)) as StampedMcpNotification;

							console.log(
								"stamped notification",
								stampedNotification,
							);

							const entry: InspectorEntry =
								{
									id: legitSvelteId,
									timestamp: stampedNotification.timestamp,
									method: stampedNotification
										.notification
										.method,
									status: "notification",
									request: stampedNotification.notification,
									requestType:
										"client",
									response: null,
									stderr: null,
								};

							entries.push(entry);
						}
					},
				);
		}

		async function startListenServerNotification() {
			unlistenServerNotification =
				await listen<IncomingNotification>(
					"mcp-server-notification",
					async (event) => {
						console.log(
							"server notification",
							event,
						);
						const legitSvelteId =
							createLegitSvelteId(
								event.payload
									.notificationId,
								"server",
							);

						if (
							event.payload
								.serverId ===
								selectedConnectionId &&
							entries.every(
								(entry) =>
									entry.id !==
									legitSvelteId,
							)
						) {
							const stampedNotification =
								(await invoke(
									"get_mcp_server_notification",
									{
										serverId: event
											.payload
											.serverId,
										notificationId:
											event
												.payload
												.notificationId,
									},
								)) as StampedMcpNotification;

							console.log(
								"stamped notification",
								stampedNotification,
							);

							const entry: InspectorEntry =
								{
									id: legitSvelteId,
									timestamp: stampedNotification.timestamp,
									method: stampedNotification
										.notification
										.method,
									status: "notification",
									request: stampedNotification.notification,
									requestType:
										"server",
									response: null,
									stderr: null,
								};

							entries.push(entry);
						}
					},
				);
		}

		// TODO: Can optimize to have events like
		// UpdateEvent {
		//   serverId: ...,
		//   dataId: ...,
		//   origin: "client" | "server",
		//   dataType: "request" | "response" | "notification"
		// }
		startListenInitializeStart();
		startListenInitializeFinish();
		startListenInitializeFail();
		startListenClientRequest();
		startListenServerResponse();
		startListenServerRequest();
		startListenClientResponse();
		startListenClientNotification();
		startListenServerNotification();

		return () => {
			if (unlistenInitializeStart) {
				unlistenInitializeStart();
			}

			if (unlistenInitializeFinish) {
				unlistenInitializeFinish();
			}

			if (unlistenInitializeFail) {
				unlistenInitializeFail();
			}

			if (unlistenClientRequest) {
				unlistenClientRequest();
			}

			if (unlistenServerResponse) {
				unlistenServerResponse();
			}

			if (unlistenServerRequest) {
				unlistenServerRequest();
			}

			if (unlistenClientResponse) {
				unlistenClientResponse();
			}

			if (unlistenClientNotification) {
				unlistenClientNotification();
			}

			if (unlistenServerNotification) {
				unlistenServerNotification();
			}
		};
	});

	$effect(() => {
		if (selectedConnectionId) {
			invoke("get_mcp_logs", {
				serverId: selectedConnectionId,
			}).then((data) => {
				entries = data as InspectorEntry[];
			});
		}
	});

	let filteredEntries = $derived(
		filter
			? entries.filter((e) =>
					e.method
						.toLowerCase()
						.includes(filter.toLowerCase()),
				)
			: entries,
	);

	let selectedEntry = $derived(
		selectedEntryId
			? (entries.find((e) => e.id === selectedEntryId) ??
					null)
			: null,
	);

	let flag_array = $derived(entries.map((e) => e.id === selectedEntryId));

	$effect(() => {
		console.log("----------------------------------");
		console.log($state.snapshot(selectedConnectionId));
		console.log($state.snapshot(connections));
		console.log($state.snapshot(entries));
		console.log($state.snapshot(selectedEntryId));
		console.log($state.snapshot(selectedEntry));
		console.log($state.snapshot(flag_array));
		console.log("----------------------------------");
	});

	// Resizable columns
	let timelineWidth = $state(280);
	let requestFlex = $state(1);
	let responseFlex = $state(1);
	let dragging = $state<"timeline" | "request" | null>(null);
	let containerEl = $state<HTMLDivElement | null>(null);

	function onPointerDown(
		handle: "timeline" | "request",
		e: PointerEvent,
	) {
		dragging = handle;
		(e.target as HTMLElement).setPointerCapture(e.pointerId);
	}

	function onPointerMove(e: PointerEvent) {
		if (!dragging || !containerEl) return;
		const rect = containerEl.getBoundingClientRect();
		const x = e.clientX - rect.left;
		const handleWidth = 8; // 4px handle * 2

		if (dragging === "timeline") {
			const newWidth = Math.max(
				150,
				Math.min(x, rect.width * 0.5),
			);
			timelineWidth = newWidth;
		} else if (dragging === "request") {
			const remaining =
				rect.width - timelineWidth - handleWidth;
			const requestArea = x - timelineWidth - handleWidth / 2;
			const ratio = Math.max(
				0.15,
				Math.min(0.85, requestArea / remaining),
			);
			requestFlex = ratio;
			responseFlex = 1 - ratio;
		}
	}

	function onPointerUp() {
		dragging = null;
	}
</script>

<div class="flex h-screen flex-col bg-background text-foreground">
	{#if connections.length === 0}
		<div class="flex flex-1 items-center justify-center">
			<div class="flex flex-col items-center gap-6 text-center">
				<div class="flex items-center gap-3">
					<div class="relative flex h-14 w-14 items-center justify-center rounded-2xl bg-primary/10">
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
							stroke-linecap="round"
							stroke-linejoin="round"
							class="h-7 w-7 text-primary"
						>
							<path d="M12 2L2 7l10 5 10-5-10-5z" />
							<path d="M2 17l10 5 10-5" />
							<path d="M2 12l10 5 10-5" />
						</svg>
					</div>
				</div>
				<div class="flex flex-col gap-2">
					<h1 class="text-2xl font-bold tracking-tight">Shimmy</h1>
					<p class="max-w-sm text-sm text-muted-foreground">
						MCP protocol inspector. Connect an MCP client through the shimmy proxy to start inspecting requests, responses, and notifications.
					</p>
				</div>

				<div class="flex flex-col gap-3 rounded-lg border border-border bg-card p-4 text-left">
					<p class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
						Waiting for connections
					</p>
					<div class="flex items-center gap-3">
						<div class="relative flex h-2 w-2">
							<span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-primary/75"></span>
							<span class="relative inline-flex h-2 w-2 rounded-full bg-primary"></span>
						</div>
						<span class="text-sm text-muted-foreground">
							Listening on <code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">127.0.0.1:13579</code>
						</span>
					</div>
					{#if pendingConnections.length > 0}
						<div class="flex items-center gap-3">
							<div class="relative flex h-2 w-2">
								<span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-amber-500/75"></span>
								<span class="relative inline-flex h-2 w-2 rounded-full bg-amber-500"></span>
							</div>
							<span class="text-sm text-muted-foreground">
								{pendingConnections.length} connection{pendingConnections.length > 1 ? 's' : ''} initializing...
							</span>
						</div>
					{/if}
				</div>
			</div>
		</div>
	{:else}
		<Toolbar
			{connections}
			bind:selectedConnectionId
			bind:filter
			bind:paused
		/>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="flex min-h-0 flex-1 {dragging
				? 'select-none'
				: ''}"
			bind:this={containerEl}
			onpointermove={onPointerMove}
			onpointerup={onPointerUp}
		>
			<div style="width: {timelineWidth}px; flex-shrink: 0;">
				<TimelinePanel
					entries={filteredEntries}
					selectedId={selectedEntryId}
					onselect={(id) =>
						(selectedEntryId = id)}
					onrefresh={() => {
						if (selectedConnectionId) {
							invoke("get_mcp_logs", {
								serverId: selectedConnectionId,
							}).then((data) => {
								entries = data as InspectorEntry[];
							});
						}
					}}
				/>
			</div>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="w-1 cursor-col-resize bg-border hover:bg-primary/50 transition-colors flex-shrink-0"
				onpointerdown={(e) =>
					onPointerDown("timeline", e)}
			></div>
			<div style="flex: {requestFlex}; min-width: 0;">
				<RequestPanel entry={selectedEntry} />
			</div>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="w-1 cursor-col-resize bg-border hover:bg-primary/50 transition-colors flex-shrink-0"
				onpointerdown={(e) =>
					onPointerDown("request", e)}
			></div>
			<div style="flex: {responseFlex}; min-width: 0;">
				<ResponsePanel entry={selectedEntry} />
			</div>
		</div>
	{/if}
</div>

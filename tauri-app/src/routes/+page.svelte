<script lang="ts">
	import Toolbar from "$lib/components/inspector/Toolbar.svelte";
	import TimelinePanel from "$lib/components/inspector/TimelinePanel.svelte";
	import RequestPanel from "$lib/components/inspector/RequestPanel.svelte";
	import ResponsePanel from "$lib/components/inspector/ResponsePanel.svelte";
	import { listen, type UnlistenFn } from "@tauri-apps/api/event";
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";
	import type {
		IncomingRequest,
		IncomingResponse,
		McpInitializeFinish,
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
		let unlistenClientRequest: UnlistenFn | undefined = undefined;
		let unlistenResponse: UnlistenFn | undefined = undefined;

		async function startListenInitializeStart() {
			unlistenInitializeStart = await listen<string>(
				"mcp-initialize-start",
				async (event) => {
					console.log("initializing!!", event);
					pendingConnections.push(event.payload);
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
										"get_mcp_request",
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
										"get_mcp_response",
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

							connections.push({
								transport: "stdio",
								id: event
									.payload
									.serverId,
								name: `${clientName}-${serverName}`,
							});

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

		async function startListenClientRequest() {
			unlistenClientRequest = await listen<IncomingRequest>(
				"mcp-client-request",
				async (event) => {
					const legitSvelteId =
						createLegitSvelteId(
							event.payload.requestId,
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
								"get_mcp_request",
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
							response: null,
							stderr: null,
						};

						entries.push(entry);
					}
				},
			);
		}

		async function startListenResponse() {
			unlistenResponse = await listen<IncomingResponse>(
				"mcp-response",
				async (event) => {
					if (
						event.payload.serverId ===
						selectedConnectionId
					) {
						const stampedResponse =
							(await invoke(
								"get_mcp_response",
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

						const entry = entries.find(
							(item) =>
								(
									item.request as JSONRPCRequest
								).id ===
								event.payload
									.responseId,
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

		startListenInitializeStart();
		startListenInitializeFinish();
		startListenClientRequest();
		startListenResponse();

		return () => {
			if (unlistenInitializeStart) {
				unlistenInitializeStart();
			}

			if (unlistenInitializeFinish) {
				unlistenInitializeFinish();
			}

			if (unlistenClientRequest) {
				unlistenClientRequest();
			}

			if (unlistenResponse) {
				unlistenResponse();
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

	function onPointerDown(handle: "timeline" | "request", e: PointerEvent) {
		dragging = handle;
		(e.target as HTMLElement).setPointerCapture(e.pointerId);
	}

	function onPointerMove(e: PointerEvent) {
		if (!dragging || !containerEl) return;
		const rect = containerEl.getBoundingClientRect();
		const x = e.clientX - rect.left;
		const handleWidth = 8; // 4px handle * 2

		if (dragging === "timeline") {
			const newWidth = Math.max(150, Math.min(x, rect.width * 0.5));
			timelineWidth = newWidth;
		} else if (dragging === "request") {
			const remaining = rect.width - timelineWidth - handleWidth;
			const requestArea = x - timelineWidth - handleWidth / 2;
			const ratio = Math.max(0.15, Math.min(0.85, requestArea / remaining));
			requestFlex = ratio;
			responseFlex = 1 - ratio;
		}
	}

	function onPointerUp() {
		dragging = null;
	}
</script>

<div class="flex h-screen flex-col bg-background text-foreground">
	{#if selectedConnectionId === null || connections.length === 0 || entries.length === 0}
		<div>Placeholder</div>
	{:else}
		<Toolbar
			{connections}
			bind:selectedConnectionId
			bind:filter
			bind:paused
		/>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="flex min-h-0 flex-1 {dragging ? 'select-none' : ''}"
			bind:this={containerEl}
			onpointermove={onPointerMove}
			onpointerup={onPointerUp}
		>
			<div style="width: {timelineWidth}px; flex-shrink: 0;">
				<TimelinePanel
					entries={filteredEntries}
					selectedId={selectedEntryId}
					onselect={(id) => (selectedEntryId = id)}
				/>
			</div>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="w-1 cursor-col-resize bg-border hover:bg-primary/50 transition-colors flex-shrink-0"
				onpointerdown={(e) => onPointerDown("timeline", e)}
			></div>
			<div style="flex: {requestFlex}; min-width: 0;">
				<RequestPanel entry={selectedEntry} />
			</div>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="w-1 cursor-col-resize bg-border hover:bg-primary/50 transition-colors flex-shrink-0"
				onpointerdown={(e) => onPointerDown("request", e)}
			></div>
			<div style="flex: {responseFlex}; min-width: 0;">
				<ResponsePanel entry={selectedEntry} />
			</div>
		</div>
	{/if}
</div>

<script lang="ts">
	import { entries, connections } from "$lib/data/mock-data";
	import Toolbar from "$lib/components/inspector/Toolbar.svelte";
	import TimelinePanel from "$lib/components/inspector/TimelinePanel.svelte";
	import RequestPanel from "$lib/components/inspector/RequestPanel.svelte";
	import ResponsePanel from "$lib/components/inspector/ResponsePanel.svelte";
	import { listen, type UnlistenFn } from "@tauri-apps/api/event";

	let selectedEntryId = $state<string | null>(null);
	let filter = $state("");
	let selectedConnectionId = $state(connections[0].id);
	let paused = $state(false);

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
</script>

<div class="flex h-screen flex-col bg-background text-foreground">
	<Toolbar
		{connections}
		bind:selectedConnectionId
		bind:filter
		bind:paused
	/>
	<div class="flex min-h-0 flex-1">
		<TimelinePanel
			entries={filteredEntries}
			selectedId={selectedEntryId}
			onselect={(id) => (selectedEntryId = id)}
		/>
		<RequestPanel entry={selectedEntry} />
		<ResponsePanel entry={selectedEntry} />
	</div>
</div>

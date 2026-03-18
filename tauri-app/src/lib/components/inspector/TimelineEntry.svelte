<script lang="ts">
	import type { InspectorEntry } from "$lib/types/inspector";
	import StatusBadge from "./StatusBadge.svelte";
	import { formatTimestampString } from "$lib/utils";

	let {
		entry,
		selected = false,
		onclick,
	}: {
		entry: InspectorEntry;
		selected?: boolean;
		onclick: () => void;
	} = $props();
</script>

<button
	class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm transition-colors hover:bg-accent/50
		{selected ? 'bg-accent text-accent-foreground' : 'text-muted-foreground'}"
	{onclick}
>
	<span
		class="inline-flex h-5 w-5 shrink-0 items-center justify-center rounded text-[10px] font-bold {entry.requestType === 'server'
			? 'bg-blue-500/15 text-blue-500'
			: 'bg-amber-500/15 text-amber-500'}"
		title={entry.requestType === "server" ? "Server" : "Client"}
	>
		{entry.requestType === "server" ? "S" : "C"}
	</span>
	<span class="shrink-0 font-mono text-xs text-muted-foreground"
		>{formatTimestampString(entry.timestamp)}</span
	>
	<span
		class="min-w-0 flex-1 truncate font-medium {selected
			? 'text-foreground'
			: ''}">{entry.method}</span
	>
	<StatusBadge status={entry.status} />
</button>

<script lang="ts">
	import type { InspectorEntry } from "$lib/types/inspector";
	import StatusBadge from "./StatusBadge.svelte";

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
	onclick={onclick}
>
	<span class="shrink-0 font-mono text-xs text-muted-foreground">{entry.timestamp}</span>
	<span class="min-w-0 flex-1 truncate font-medium {selected ? 'text-foreground' : ''}">{entry.method}</span>
	<StatusBadge status={entry.status} />
</button>

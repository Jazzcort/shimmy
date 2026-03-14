<script lang="ts">
	import type { InspectorEntry } from "$lib/types/inspector";
	import TimelineEntry from "./TimelineEntry.svelte";

	let {
		entries,
		selectedId,
		onselect,
	}: {
		entries: InspectorEntry[];
		selectedId: number | string | null;
		onselect: (id: string | number) => void;
	} = $props();
</script>

<div class="flex h-full w-full flex-col">
	<div class="border-b border-border px-3 py-2 h-11 flex items-center">
		<h2
			class="text-xs font-semibold uppercase tracking-wider text-muted-foreground"
		>
			Timeline
		</h2>
	</div>
	<div class="flex-1 overflow-y-auto">
		{#each entries as entry (entry.id)}
			<TimelineEntry
				{entry}
				selected={selectedId === entry.id}
				onclick={() => onselect(entry.id)}
			/>
		{/each}
		{#if entries.length === 0}
			<div
				class="px-3 py-8 text-center text-sm text-muted-foreground"
			>
				No entries match your filter.
			</div>
		{/if}
	</div>
</div>

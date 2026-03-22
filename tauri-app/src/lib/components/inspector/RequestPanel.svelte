<script lang="ts">
	import type { InspectorEntry } from "$lib/types/inspector";
	import JsonViewer from "./JsonViewer.svelte";
	import { Separator } from "$lib/components/ui/separator";

	let { entry }: { entry: InspectorEntry | null } = $props();
</script>

<div class="flex h-full flex-1 flex-col">
	<div class="border-b border-border px-4 py-2 flex items-center h-11">
		<h2
			class="text-xs font-semibold uppercase tracking-wider text-muted-foreground"
		>
			Request
		</h2>
	</div>
	{#if entry}
		<div class="flex-1 overflow-y-auto p-4">
			<JsonViewer data={entry.request} />
			{#if entry.stderr}
				<Separator class="my-4" />
				<div>
					<h3
						class="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground"
					>
						Stderr
					</h3>
					<pre
						class="whitespace-pre-wrap rounded-md bg-red-500/10 p-3 font-mono text-xs text-red-400">{entry.stderr}</pre>
				</div>
			{/if}
		</div>
	{:else}
		<div
			class="flex flex-1 items-center justify-center text-sm text-muted-foreground px-4"
		>
			<p class="truncate">
				Select an entry to view the request.
			</p>
		</div>
	{/if}
</div>

<script lang="ts">
	import type { InspectorEntry } from "$lib/types/inspector";
	import JsonViewer from "./JsonViewer.svelte";
	import Button from "$lib/components/ui/button/button.svelte";
	import CopyIcon from "lucide-svelte/icons/copy";
	import CheckIcon from "lucide-svelte/icons/check";

	let { entry }: { entry: InspectorEntry | null } = $props();

	let copied = $state(false);

	function copyResponse() {
		if (!entry?.response) return;
		navigator.clipboard.writeText(
			JSON.stringify(entry.response, null, 2),
		);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	}
</script>

<div class="flex h-full flex-1 flex-col">
	<div
		class="flex items-center justify-between border-b border-border px-4 py-2 h-11"
	>
		<h2
			class="text-xs font-semibold uppercase tracking-wider text-muted-foreground"
		>
			Response
		</h2>
		{#if entry?.response}
			<Button
				variant="outline"
				size="sm"
				class="h-6 gap-1.5 px-2 text-xs"
				onclick={copyResponse}
			>
				{#if copied}
					<CheckIcon class="size-3.5" />
					Copied
				{:else}
					<CopyIcon class="size-3.5" />
					Copy
				{/if}
			</Button>
		{/if}
	</div>
	{#if entry}
		<div class="flex-1 overflow-y-auto p-4">
			{#if entry.response}
				<div class="font-mono text-sm leading-relaxed">
					<JsonViewer data={entry.response} />
				</div>
			{:else}
				<div
					class="flex items-center justify-center py-8 text-sm text-muted-foreground"
				>
					No response (notification).
				</div>
			{/if}
		</div>
	{:else}
		<div
			class="flex flex-1 items-center justify-center text-sm text-muted-foreground"
		>
			Select an entry to view the response.
		</div>
	{/if}
</div>

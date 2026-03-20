<script lang="ts">
	import type { McpConnection } from "$lib/types/inspector";
	import Badge from "$lib/components/ui/badge/badge.svelte";
	import { Input } from "$lib/components/ui/input";
	import * as Select from "$lib/components/ui/select";
	import Button from "$lib/components/ui/button/button.svelte";
	import CircleIcon from "lucide-svelte/icons/circle";
	import PauseIcon from "lucide-svelte/icons/pause";
	import PlayIcon from "lucide-svelte/icons/play";
	import FilterIcon from "lucide-svelte/icons/filter";
	import Trash2Icon from "lucide-svelte/icons/trash-2";

	let {
		connections,
		selectedConnectionId = $bindable(),
		filter = $bindable(""),
		ondelete,
	}: {
		connections: McpConnection[];
		selectedConnectionId: string;
		filter: string;
		ondelete?: (connectionId: string) => void;
	} = $props();

	let selectedConnection = $derived(
		connections.find((c) => c.id === selectedConnectionId),
	);
</script>

<div class="flex items-center gap-3 border-b border-border bg-card px-4 py-2">
	{#if selectedConnection?.isConnected === false}
		<Badge
			variant="outline"
			class="gap-1.5 border-red-500/30 bg-red-500/10 text-red-400"
		>
			<CircleIcon class="size-2 fill-red-400" />
			Disconnected
		</Badge>
		<Button
			variant="ghost"
			size="sm"
			class="h-7 gap-1.5 px-2 text-red-400 hover:bg-red-500/10 hover:text-red-300"
			onclick={() => ondelete?.(selectedConnectionId)}
		>
			<Trash2Icon class="size-3.5" />
			Delete
		</Button>
	{:else}
		<Badge
			variant="outline"
			class="gap-1.5 border-green-500/30 bg-green-500/10 text-green-400"
		>
			<CircleIcon class="size-2 fill-green-400" />
			Connected
		</Badge>
	{/if}

	<Select.Root type="single" bind:value={selectedConnectionId}>
		<Select.Trigger class="w-[200px] overflow-hidden">
			<span class="truncate"
				>{selectedConnection?.name ??
					"Select connection"}</span
			>
		</Select.Trigger>
		<Select.Content>
			{#each connections as conn (conn.id)}
				<Select.Item
					value={conn.id}
					label={conn.name}
				/>
			{/each}
		</Select.Content>
	</Select.Root>

	{#if selectedConnection}
		<Badge variant="secondary" class="text-xs uppercase">
			{selectedConnection.transport}
		</Badge>
	{/if}

	<div class="relative flex-1">
		<FilterIcon
			class="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground"
		/>
		<Input
			bind:value={filter}
			placeholder="Filter methods..."
			class="h-8 pl-8 text-sm"
		/>
	</div>
</div>

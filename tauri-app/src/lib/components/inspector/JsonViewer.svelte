<script lang="ts">
	import JsonViewer from "./JsonViewer.svelte";

	let {
		data,
		depth = 0,
		lastElement = true,
	}: { data: unknown; depth?: number; lastElement?: boolean } = $props();

	function isObject(val: unknown): val is Record<string, unknown> {
		return (
			val !== null &&
			typeof val === "object" &&
			!Array.isArray(val)
		);
	}

	function isArray(val: unknown): val is unknown[] {
		return Array.isArray(val);
	}
</script>

{#if data === null}
	<span class="text-orange-400">null</span>{#if !lastElement}<span>,</span
		>{/if}
{:else if data === undefined}
	<span class="text-orange-400">undefined</span>{#if !lastElement}<span
			>,</span
		>{/if}
{:else if typeof data === "string"}
	<span class="text-green-400">"{data}"</span>{#if !lastElement}<span
			>,</span
		>{/if}
{:else if typeof data === "number"}
	<span class="text-purple-400">{data}</span>{#if !lastElement}<span
			>,</span
		>{/if}
{:else if typeof data === "boolean"}
	<span class="text-yellow-400">{data}</span>{#if !lastElement}<span
			>,</span
		>{/if}
{:else if isArray(data)}
	{#if data.length === 0}
		<span class="text-muted-foreground">[]</span>
	{:else}
		<span class="text-muted-foreground">[</span>
		{#each data as item, i}
			<div class="pl-6 -indent-6">
				<JsonViewer
					data={item}
					depth={depth + 1}
					lastElement={i === data.length - 1}
				/>
			</div>
		{/each}
		<div>
			<span class="text-muted-foreground">]</span
			>{#if !lastElement}<span>,</span>{/if}
		</div>
	{/if}
{:else if isObject(data)}
	{@const keys = Object.keys(data)}
	{#if keys.length === 0}
		<span class="text-muted-foreground">{"{}"}</span>
	{:else}
		<span class="text-muted-foreground">{"{"}</span>
		{#each keys as key, i}
			<div class={`${depth === 0 ? "pl-6" : ""}`}>
				<div class="pl-6 -indent-6">
					<span class="text-blue-300"
						>"{key}"</span
					><span class="text-muted-foreground"
						>:
					</span><JsonViewer
						data={data[key]}
						depth={depth + 1}
						lastElement={i ===
							keys.length - 1}
					/>
				</div>
			</div>
		{/each}
		<div>
			<span class="text-muted-foreground">{"}"}</span
			>{#if !lastElement}<span>,</span>{/if}
		</div>
	{/if}
{/if}

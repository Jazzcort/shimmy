<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	let {
		data,
	}: { data: unknown } = $props();

	let lines = $state<{ indent: number; html: string }[]>([]);

	$effect(() => {
		invoke("colorize_json", { data }).then((result) => {
			lines = result as { indent: number; html: string }[];
		});
	});
</script>

<div class="font-mono text-sm leading-relaxed text-muted-foreground">
	{#each lines as line}
		<div style="padding-left: {line.indent + 4}ch; text-indent: -4ch;" class="overflow-wrap-anywhere">{@html line.html}</div>
	{/each}
</div>

<style>
	.overflow-wrap-anywhere {
		overflow-wrap: anywhere;
	}
</style>

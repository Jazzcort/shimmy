<script lang="ts">
	import Badge from "$lib/components/ui/badge/badge.svelte";
	import type { LogStatus } from "$lib/types/inspector";

	let { status }: { status: LogStatus } = $props();

	const styles: Record<LogStatus, string> = {
		request: "bg-blue-500/20 text-blue-400 border-blue-500/30",
		success: "bg-green-500/20 text-green-400 border-green-500/30",
		error: "bg-red-500/20 text-red-400 border-red-500/30",
		start: "bg-amber-500/20 text-amber-400 border-amber-500/30",
	};
</script>

<Badge variant="outline" class={styles[status]}>
	{status}
</Badge>

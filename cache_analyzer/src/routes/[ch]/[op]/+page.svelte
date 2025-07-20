<script lang="ts">
	import { type PageProps } from './$types';
	import type { DepsMap } from '$lib/data';
	import { goto } from '$app/navigation';

	let { data }: PageProps = $props();

	let snapshot = $derived(data.cache_state);
	let stack = $derived(data.stack);
	let opLen = $derived(data.chapterDetails.ops.length);
	let currentOp = $derived(data.currentOp);

	let lengths = $derived.by(() => {
		let lengths = [];
		for (let row of snapshot) {
			let len = Math.max(
				row.entry.world_state.length,
				row.entry.direct_world_state.length,
				row.entry.deps_state.length, 1);
			lengths.push(len);
		}
		return lengths;
	});

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'ArrowUp' && currentOp > 0) {
			document.getElementById(`op-${currentOp-1}`)?.scrollIntoView({behavior: "smooth", block: "center"});
			goto(`${currentOp-1}/`)
		} else if (event.key === 'ArrowDown' && currentOp < opLen-1) {
			goto(`${currentOp+1}/`)
			document.getElementById(`op-${currentOp+1}`)?.scrollIntoView({behavior: "smooth", block: "center"});
		}
	}

</script>

<svelte:window on:keydown={handleKeydown} />

<div class="flex-auto flex flex-col gap-2">
	<div class="h-fit border-yellow-500 border-1 rounded-md font-mono p-2">
		:
		{#each stack as elem, n (n)}
			{#if n === 0}
				{elem}&nbsp;
			{:else}
				> {elem}&nbsp;
			{/if}
		{/each}
	</div>
	<div class="bg-white border-1 border-yellow-500 rounded-md h-full font-mono p-4 overflow-scroll">
		<div class="grid w-full h-auto auto-rows-fr grid-cols-6">
			{#each lengths as len, n (n)}
				{#snippet monorow(text: string)}
					<div class="row-span-{len} centered border-yellow-500" class:border-b-2={n !== lengths.length-1}>{text}</div>
				{/snippet}

				{@render monorow(snapshot[n].key)}
				{@render monorow(snapshot[n].entry.value)}
				{@render monorow(snapshot[n].entry.fingerprint)}
				{#each [...Array(len).keys()] as i (i)}
					{#snippet worldPolyrow(elements: DepsMap)}
						{#if i < elements.length}
							<div class="border-yellow-500 p-2" class:border-b-2={n !== lengths.length-1 && i === lengths[n] - 1}>
								<div class="p-2 flex flex-col items-center justify-center rounded-md"
										 class:bg-lime-300={elements[i].freshness === "Fresh"}
										 class:bg-red-300={elements[i].freshness === "Stale"}
								>
									<div class="border-b">{elements[i].key}</div>
									<div>{elements[i].fingerprint}</div>
								</div>
							</div>
						{:else}
							<div class="border-yellow-500" class:border-b-2={n !== lengths.length-1 && i === lengths[n] - 1}></div>
						{/if}
					{/snippet}

					{@render worldPolyrow(snapshot[n].entry.world_state)}
					{@render worldPolyrow(snapshot[n].entry.direct_world_state)}
					{@render worldPolyrow(snapshot[n].entry.deps_state)}
				{/each}
			{/each}
		</div>
	</div>
	<!--	<div class="bg-white border-1 border-yellow-500 rounded-md h-full basis-1/2 overflow-hidden font-mono p-4">-->
	<!--	</div>-->
</div>

<style>
    @reference "tailwindcss";

    .centered {
        @apply flex justify-center items-center;
    }
</style>
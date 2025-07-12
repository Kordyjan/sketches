<script lang="ts">
	import type { LayoutProps } from './$types.js';
	import '../app.css';
	import { page } from '$app/state';
	import OddEvenList from '$lib/OddEvenList.svelte';
	import { type Chapter } from '$lib/data';

	let { children, data }: LayoutProps = $props();

	let current = $derived(parseInt(page.params['ch']));
	let chapters = $derived(data.chapters);
</script>


<div class="min-h-screen p-4 border-1 border-yellow-500 h-screen w-screen flex gap-2">
	<div class="bg-white border-1 border-yellow-500 rounded-md h-full basis-1/8 overflow-hidden">
		<OddEvenList data={chapters} keyFn={it => it.id} {current}>
			{#snippet content(ch: Chapter)}
				<a href="/{ch.id}">
					<div class="w-full p-2 font-mono text-center">
						<div class="w-full text-xl">
							{ch.data}
						</div>
						<div class="w-full">
							{ch.fingerprint}
						</div>
					</div>
				</a>
			{/snippet}
		</OddEvenList>
	</div>
	{@render children()}
</div>

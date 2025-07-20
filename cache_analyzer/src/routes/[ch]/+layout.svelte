<script lang="ts">
	import type { LayoutProps } from './$types';
	import OddEvenList from '$lib/OddEvenList.svelte';
	import { type OpHead } from '$lib/data';
	import { page } from '$app/state';
	import { slide } from 'svelte/transition';

	let { children, data }: LayoutProps = $props();

	let chapter = $derived(data.chapterDetails);
	let current = $derived(parseInt(page.params['op']));

	let filteredList = $derived(chapter.ops.filter(op => !op.is_comment));
	let showComments = $state(false);
	let toShow = $derived(showComments ? data.chapterDetails.ops : filteredList);
</script>

<div class="basis-1/8 flex flex-col gap-2 h-full">
	<div class="bg-white border-1 border-yellow-500 rounded-md overflow-hidden flex-auto">
		<OddEvenList data={toShow} keyFn={it => it.id} {current}>
			{#snippet content(op: OpHead)}
				<a href="{op.id.toString()}">
					<div class="p-2" id="op-{op.id}" class:text-right={!op.is_comment} class:text-mono={!op.is_comment} transition:slide|global>
						{op.desc}
					</div>
				</a>
			{/snippet}
		</OddEvenList>
	</div>
	<div class="flex-none flex items-center gap-2 p-2">
		<label>
			<input type="checkbox" id="showComments" bind:checked={showComments} class="form-checkbox" />
			Show comments
		</label>
	</div>
</div>
{@render children()}

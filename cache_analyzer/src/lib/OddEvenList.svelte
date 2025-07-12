<script lang="ts" generics="T, K">
	import type { Snippet } from 'svelte';

	type Props = { data: Array<T>, current: K | undefined, keyFn: (it: T) => K, content: Snippet<[T]> };

	let { data, current = undefined, keyFn, content }: Props = $props();
</script>

<div class="w-full h-full overflow-scroll">
	{#each data as item (keyFn(item))}
		<div class="item" class:selected={current === keyFn(item)}>
			{@render content(item)}
		</div>
	{/each}
</div>

<style>
	@reference "tailwindcss";

	.item {
			@apply odd:bg-yellow-100;

			&.selected {
					@apply bg-yellow-200;
			}
	}
</style>
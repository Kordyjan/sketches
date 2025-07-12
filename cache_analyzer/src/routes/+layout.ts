import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ fetch }) =>
	({
		chapters: await (await fetch('http://127.0.0.1:8000/chapters')).json()
	});

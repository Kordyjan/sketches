import type { PageLoad } from './$types';
import type { Snapshot } from '$lib/data';

export const load: PageLoad = async ({ params, fetch }) => {
	const cache_state = fetch(
		`http://127.0.0.1:8000/chapters/${params.ch}/${params.op}/snapshot`
	).then((r) => r.json());
	return {
		cache_state: (await cache_state) as Snapshot
	};
};

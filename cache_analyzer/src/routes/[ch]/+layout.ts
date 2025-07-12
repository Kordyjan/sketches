import type { LayoutLoad } from './$types';
import type { ChapterDetail } from '$lib/data';

export const load: LayoutLoad = async ({ params, fetch }) => ({
	chapterDetails: (await (
		await fetch(`http://127.0.0.1:8000/chapters/${params.ch}`)
	).json()) as ChapterDetail
});

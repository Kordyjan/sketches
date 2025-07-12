import type { PageLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: PageLoad = ({params}) => {
	redirect(303, `/${params['ch']}/0`)
}
export type Chapter = {
	id: number;
	data: string;
	fingerprint: string;
};

export type ChapterDetail = {
	head: Chapter;
	ops: Array<OpHead>;
};

export type OpHead = {
	id: number;
	desc: string;
	is_comment: boolean;
};

export type Snapshot = Array<KeyedEntry>;

export type KeyedEntry = {
	key: string;
	entry: CacheEntryDetail;
};

export type CacheEntryDetail = {
	value: string;
	fingerprint: string;
	world_state: DepsMap;
	direct_world_state: DepsMap;
	deps_state: DepsMap;
}

export type DepsMap = Array<DepsEntry>;

export type DepsEntry = {
	freshness: Freshness;
	key: string;
	fingerprint: string;
}

export type Freshness = "Fresh" | "Stale";

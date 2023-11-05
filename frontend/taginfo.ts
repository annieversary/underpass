export type TagKey = {
    key: string;
    values: TagValue[],
    description: string;

    count_all: number;
    count_all_fraction: number;
    count_nodes_fraction: number;
    count_ways_fraction: number;
    count_relations_fraction: number;
};
export type TagValue = {
    value: string;
    description: string;
    fraction: number;
    count: number;
};

async function loadTaginfo() {
    const r = await fetch('/taginfo.json');
    const res = await r.json();

    for (const key of res) {
        taginfo[key.key] = key;
    }
}
loadTaginfo();

export let taginfo: { [key: string]: TagKey } = {};

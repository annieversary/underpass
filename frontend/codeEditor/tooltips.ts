type TagKey = {
    key: string;
    values: TagValue[],
    description: string;

    count_all: number;
    count_all_fraction: number;
    count_nodes_fraction: number;
    count_ways_fraction: number;
    count_relations_fraction: number;
};
type TagValue = {
    value: string;
    description: string;
    fraction: number;
    count: number;
};
let taginfo: { [key: string]: TagKey } = {};
async function loadTaginfo() {
    const r = await fetch('/taginfo.json');
    const res = await r.json();

    for (const key of res) {
        taginfo[key.key] = key;
    }
}
loadTaginfo();


import { hoverTooltip } from "@codemirror/view";

export function tooltip() {
    return hoverTooltip((view, pos, side) => {
        let { from, to, text } = view.state.doc.lineAt(pos);
        let start = pos, end = pos;

        // get the thing inside the current brackets
        // `start` and `end` equals `pos` at the start, but they grow until they hit [], (), or {}
        const regex = /[^\[\]\(\)\{\}]/;
        while (start > from && regex.test(text[start - from - 1])) start--;
        while (end < to && regex.test(text[end - from])) end++;
        if (start == pos && side < 0 || end == pos && side > 0) return null;

        const slice = text.slice(start - from, end - from);

        const matches = slice.match(/^"?(\w+)"?\s*(=\s*"?(\w+)"?)?/);
        if (matches == null) return null;
        const key = matches[1];
        const value = matches[3];

        const info = taginfo[key];

        if (info === undefined) return null;

        const valInfo = info.values.find(v => v.value == value);

        return {
            pos: start,
            end,
            above: true,
            create(_view) {
                const description = nl2br(info.description);

                const vs = info.values.map(v => `<li ${v.value == value ? 'style="font-weight: bold;"' : ''}>${perc(v.fraction)}% - ${v.value} - ${v.description}</li>`).join("");
                const values = info.values.length ? `<div class="values"><ul>${vs}</ul></div>` : "";

                let title = `<a href="https://wiki.openstreetmap.org/wiki/Key:${key}" target="_blank">${key}</a>`;
                if (valInfo) {
                    title += `=<a href="https://wiki.openstreetmap.org/wiki/Tag:${key}=${value}" target="_blank">${value}</a>`;
                } else if (value) {
                    title += `=${value}`;
                }

                const valDesc = valInfo ? nl2br(`<b>${value}</b>: ${valInfo.description}\n`) : '';

                let dom = document.createElement("div");
                dom.innerHTML = `
                <div class="tooltip">
                    <h2>${title}</h2>
                    <i>
                        <p>${info.count_all} objects (${perc(info.count_all_fraction)}%)</p>
                        <p>${perc(info.count_nodes_fraction)}% nodes - ${perc(info.count_ways_fraction)}% ways - ${perc(info.count_relations_fraction)}% relations</p>
                    </i>
                    <p>${description}</p>
                    <p>${valDesc}</p>

                    ${values}
                </div>`;
                return { dom };
            }
        }
    });
}

function perc(v: number) {
    return (v * 100).toFixed(2);
}

function nl2br(str: string, is_xhtml = true) {
    var breakTag = (is_xhtml || typeof is_xhtml === 'undefined') ? '<br />' : '<br>';
    return (str + '').replace(/([^>\r\n]?)(\r\n|\n\r|\r|\n)/g, '$1' + breakTag + breakTag + '$2');
}

import { EditorView, ViewPlugin, keymap, lineNumbers, rectangularSelection, highlightActiveLine, ViewUpdate } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { closeBrackets } from "@codemirror/autocomplete";
import { bracketMatching } from "@codemirror/language";

import './codeEditor.css';

import { processedQueries } from './index';
import { settings } from './settings';

export function addTab(
    id: string,
    name: string,
    selected: boolean,
    onclick: () => void,
    saveGraph: () => void,
    query: string = settings.tagsShouldHaveQuotes() ? 'way["highway"]({{bbox}});' : 'way[highway]({{bbox}});',
): HTMLDivElement {
    // create code editor
    const editor = document.createElement('div');
    editor.classList.add('code-editor')
    editor.dataset.nodeId = id;
    editor.dataset.selected = selected ? 'yes' : 'no';
    codeEditorMap[id] = newEditor(editor, query, saveGraph, id);
    codeContainer.appendChild(editor);

    // create the tab
    const tab = document.createElement('div');
    tab.onclick = () => {
        for (const t of tabsEl.children) {
            (t as HTMLDivElement).dataset.selected = 'no';
        }

        tab.dataset.selected = 'yes';

        for (const e of codeContainer.children) {
            (e as HTMLDivElement).dataset.selected = 'no';
        }

        editor.dataset.selected = 'yes';

        onclick();
    };
    tab.dataset.nodeId = id;
    tab.classList.add('tab');
    tab.innerHTML = `<p>${name}</p>`;
    tabsEl.appendChild(tab)

    if (selected) tab.onclick(new MouseEvent(''));

    return tab;
}

export function removeTab(id: string) {
    codeEditorMap[id]?.destroy();
    const tab = document.querySelector<HTMLDivElement>(`.tab[data-node-id="${id}"]`);
    const wasSelected = tab.dataset.selected === 'yes';
    tab?.remove();

    if (wasSelected) {
        const newSelected = document.querySelector<HTMLDivElement>(`.tab`);
        newSelected?.onclick(new MouseEvent(''));
    }
}

export const codeEditorMap = {};

const tabsEl = document.querySelector<HTMLDivElement>('#code-tabs');
const codeContainer = document.querySelector<HTMLDivElement>("#code-container");

function newEditor(parent: HTMLDivElement, query: string, saveGraph: () => void, id: string): EditorView {
    let editor = new EditorView({
        extensions: [
            history(),
            lineNumbers(),
            rectangularSelection(),
            highlightActiveLine(),
            bracketMatching(),
            closeBrackets(),
            keymap.of([...defaultKeymap, ...historyKeymap]),
            ViewPlugin.fromClass(class {
                update(_update: ViewUpdate) {
                    saveGraph();
                    delete processedQueries[id];
                }
            }),
            tooltip(),
        ],
        parent,
    });

    editor.dispatch({
        changes: { from: 0, to: editor.state.doc.length, insert: query }
    });

    return editor;
}



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

function tooltip() {
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

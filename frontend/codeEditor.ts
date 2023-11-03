import { EditorView, ViewPlugin, keymap, lineNumbers, rectangularSelection, highlightActiveLine, ViewUpdate } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { closeBrackets } from "@codemirror/autocomplete";
import { bracketMatching } from "@codemirror/language";

import './codeEditor.css';

import { processedQueries } from './index';

export function addTab(
    id: string,
    name: string,
    selected: boolean,
    onclick: () => void,
    saveGraph: () => void,
    query: string = 'way[highway]({{bbox}});',
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



import { hoverTooltip } from "@codemirror/view";

function tooltip() {
    return hoverTooltip((view, pos, side) => {
        let { from, to, text } = view.state.doc.lineAt(pos);
        let start = pos, end = pos;
        while (start > from && /\w/.test(text[start - from - 1])) start--;
        while (end < to && /\w/.test(text[end - from])) end++;
        if (start == pos && side < 0 || end == pos && side > 0) return null;
        return {
            pos: start,
            end,
            above: true,
            create(_view) {
                let dom = document.createElement("div");
                const val = text.slice(start - from, end - from);
                console.log(val);
                dom.textContent = val;
                return { dom };
            }
        }
    });
}

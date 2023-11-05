import { EditorView, ViewPlugin, keymap, lineNumbers, rectangularSelection, highlightActiveLine, ViewUpdate } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { closeBrackets } from "@codemirror/autocomplete";
import { bracketMatching } from "@codemirror/language";

import '../codeEditor.css';

import { processedQueries } from '../index';
import { settings } from '../settings';

import { tooltip } from './tooltips';

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




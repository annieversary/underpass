import { EditorView, ViewPlugin, keymap, lineNumbers, rectangularSelection, highlightActiveLine, ViewUpdate } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { closeBrackets } from "@codemirror/autocomplete";
import { bracketMatching } from "@codemirror/language";

import './codeEditor.css';

const codeContainer = document.querySelector<HTMLDivElement>("#code-container");

function newEditor(parent: HTMLDivElement, query: string, saveGraph: () => void): EditorView {
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
                }
            }),
        ],
        parent,
    });

    editor.dispatch({
        changes: { from: 0, to: editor.state.doc.length, insert: query }
    });

    return editor;
}

export const codeEditorMap = {};

const tabsEl = document.querySelector<HTMLDivElement>('#code-tabs');

export function addTab(
    id: string,
    name: string,
    selected: boolean,
    onclick: () => void,
    saveGraph: () => void,
    query: string = '[out:json][timeout:25];\n\nway[highway]({{bbox}});\n\n{{out}}',
): HTMLDivElement {
    // create code editor
    const editor = document.createElement('div');
    editor.classList.add('code-editor')
    editor.dataset.nodeId = id;
    editor.dataset.selected = selected ? 'yes' : 'no';
    codeEditorMap[id] = newEditor(editor, query, saveGraph);
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

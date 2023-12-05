import { EditorView, ViewPlugin, keymap, ViewUpdate } from "@codemirror/view";
import { tags } from "@lezer/highlight";
import { HighlightStyle, syntaxHighlighting, syntaxTree } from "@codemirror/language";
import { acceptCompletion } from "@codemirror/autocomplete";
import { indentWithTab } from "@codemirror/commands";
import { TransactionSpec } from "@codemirror/state";
import { basicSetup } from "codemirror";
import { vim, Vim } from "@replit/codemirror-vim"

import '../codeEditor.css';

import { processedQueries } from '../index';
import { settings, vimCompartment } from '../settings';
import { oql } from '../oql-parser';

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
        for (const t of Array.from(tabsEl.children)) {
            (t as HTMLDivElement).dataset.selected = 'no';
        }

        tab.dataset.selected = 'yes';

        for (const e of Array.from(codeContainer.children)) {
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
            keymap.of([{ key: "Cmd-Enter", run: () => { return true; } }]),
            keymap.of([{ key: "Ctrl-Enter", run: () => { return true; } }]),
            vimCompartment.of(settings.vim() ? vim() : []),
            basicSetup,
            keymap.of({ key: "Tab", run: acceptCompletion } as any),
            keymap.of([indentWithTab]),
            syntaxHighlighting(myHighlightStyle),
            ViewPlugin.fromClass(class {
                update(_update: ViewUpdate) {
                    saveGraph();
                    if (processedQueries) {
                        delete processedQueries[id];
                    }
                }
            }),
            tooltip(),
            oql(),
        ],
        parent,
    });

    editor.dispatch({
        changes: { from: 0, to: editor.state.doc.length, insert: query }
    });

    return editor;
}

/// Dispatch event to all active editors
export function dispatchToAllEditors(...event: TransactionSpec[]) {
    for (const editor of Object.values<EditorView>(codeEditorMap)) {
        editor.dispatch(...event);
    }
}


Vim.defineEx('syntax', 'syn', function() {
    const r = Object.values(codeEditorMap).map(c => syntaxTree((c as any).state).toString());
    console.log(r);
});


const myHighlightStyle = HighlightStyle.define([
    // macro
    {
        tag: tags.meta,
        color: "#4a5"
    },
    {
        tag: tags.link,
        textDecoration: "underline"
    },
    {
        tag: tags.emphasis,
        fontStyle: "italic"
    },
    {
        tag: tags.strong,
        fontWeight: "bold"
    },
    {
        tag: tags.strikethrough,
        textDecoration: "line-through"
    },
    {
        tag: [tags.operatorKeyword, tags.operator],
        color: "#ff1493"
    },
    {
        tag: tags.keyword,
        color: "#708"
    },
    {
        tag: tags.number,
        color: "#0cf"
    },
    {
        tag: tags.variableName,
        color: "#66c"
    },
    {
        tag: [tags.atom, tags.bool, tags.url, tags.contentSeparator, tags.labelName],
        color: "#219"
    },
    {
        tag: [tags.literal, tags.inserted],
        color: "#164"
    },
    {
        tag: [tags.string, tags.deleted],
        color: "#a11"
    },
    {
        tag: [tags.regexp, tags.escape, tags.special(tags.string)],
        color: "#f40"
    },
    {
        tag: tags.docComment,
        textDecoration: "underline",
        fontWeight: "bold",
        color: "#940"
    },
    {
        tag: tags.comment,
        color: "#940"
    },
    {
        tag: tags.invalid,
        color: "#f00"
    }
]);

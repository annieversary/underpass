import { EditorView, ViewPlugin, keymap, lineNumbers, rectangularSelection, highlightActiveLine, ViewUpdate } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { closeBrackets } from "@codemirror/autocomplete";
import { bracketMatching } from "@codemirror/language";

export let editor = new EditorView({
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
                window.localStorage.setItem('query', editor.state.doc.toString());

                const b = document.getElementById('view-query-button');
                if (b) {
                    b.remove();
                }
            }
        }),
    ],
    parent: document.getElementById("code-container"),
});

const query = window.localStorage.getItem('query') || '[out:json][timeout:25];\n\nway["highway"]({{bbox}});\n\nout;>;out skel qt;';
editor.dispatch({
    changes: { from: 0, to: editor.state.doc.length, insert: query }
});

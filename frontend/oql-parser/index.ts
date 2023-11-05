import { LRLanguage, LanguageSupport, syntaxTree } from "@codemirror/language";
import { SyntaxNode } from "@lezer/common";
import { completeFromList, CompletionSource, ifIn, CompletionContext, Completion } from "@codemirror/autocomplete";
import { styleTags, tags as t } from "@lezer/highlight";

import { parser } from "./oql.js";
export { parser } from "./oql.js";

import { taginfo, TagKey, TagValue } from "../taginfo.js";

let parserWithMetadata = parser.configure({
    props: [
        styleTags({
            Tag: t.paren,
            Identifier: t.string,
            Regex: t.regexp,

            LineCommentTitle: t.docComment,
            LineComment: t.lineComment,

            LanguageCode: t.keyword,
            "@": t.operator,
            MacroValue: t.string,
            Macro: t.meta,

            Set: t.variableName,
            "->": t.operator,

            Keyword: t.keyword,
            "( )": t.paren,
            "[ ]": t.squareBracket,
            "{ }": t.brace,
        }),
    ]
})

export const exampleLanguage = LRLanguage.define({
    parser: parserWithMetadata,
    languageData: {
        commentTokens: { line: "//" }
    }
});

export const exampleCompletion = exampleLanguage.data.of({
    autocomplete: concatCompletionSource([
        ifIn(['Key'], completeKeyFromTagInfo),
        ifIn(['Value'], completeValueFromTagInfo),
        ifIn(['Macro'], completeFromList([
            { label: "bbox", type: "keyword" },
            { label: "center", type: "keyword" },
            { label: "geocodeArea", type: "keyword" },
            { label: "aroundSelf", type: "keyword" },
        ])),
        completeFromList([
            { label: "node", type: "keyword" },
            { label: "way", type: "keyword" },
            { label: "relation", type: "keyword" },
            { label: "nw", type: "keyword" },
            { label: "wr", type: "keyword" },
            { label: "nr", type: "keyword" },
            { label: "nwr", type: "keyword" },
        ]),
    ])
});

let taginfoAutocomplete = [];
let validForMatch = null;

function completeKeyFromTagInfo(context: CompletionContext) {
    if (Object.keys(taginfo).length == 0) return;

    if (taginfoAutocomplete.length == 0) {
        taginfoAutocomplete = Object.values(taginfo).map((k: TagKey) => {
            return { label: k.key, type: "keyword", detail: k.description };
        });

        validForMatch = taginfoAutocomplete.every(o => /^\w+$/.test(o.label)) ? [/\w*$/, /\w+$/] : prefixMatch(taginfoAutocomplete);
    }

    let [validFor, match] = validForMatch;
    let token = context.matchBefore(match)
    console.log(token);
    return token || context.explicit ? { from: token ? token.from : context.pos, options: taginfoAutocomplete, validFor } : null
}

function completeValueFromTagInfo(context: CompletionContext) {
    let pos: SyntaxNode | null = syntaxTree(context.state).resolveInner(context.pos, -1);
    let { from, to } = pos.parent.prevSibling.firstChild;

    const key = context.state.sliceDoc(from, to);
    const options = taginfo[key].values.map((v: TagValue) => {
        return { label: v.value, type: "keyword", detail: v.description };
    });

    let [validFor, match] = options.every(o => /^\w+$/.test(o.label)) ? [/\w*$/, /\w+$/] : prefixMatch(options);

    let token = context.matchBefore(match);
    return token || context.explicit ? { from: token ? token.from : context.pos, options, validFor } : null;
}

function toSet(chars: { [ch: string]: true }) {
    let flat = Object.keys(chars).join("")
    let words = /\w/.test(flat)
    if (words) flat = flat.replace(/\w/g, "")
    return `[${words ? "\\w" : ""}${flat.replace(/[^\w\s]/g, "\\$&")}]`
}

function prefixMatch(options: readonly Completion[]) {
    let first = Object.create(null), rest = Object.create(null)
    for (let { label } of options) {
        first[label[0]] = true
        for (let i = 1; i < label.length; i++) rest[label[i]] = true
    }
    let source = toSet(first) + toSet(rest) + "*$"
    return [new RegExp("^" + source), new RegExp(source)]
}

function concatCompletionSource(sources: CompletionSource[]): CompletionSource {
    return async (c) => {
        let r = await sources[0](c);

        for (let i = 1; i < sources.length; i++) {
            if (r == null) {
                r = await sources[i](c);
            } else break;
        }

        return r;
    };
}

export function oql() {
    return new LanguageSupport(exampleLanguage, [exampleCompletion])
}

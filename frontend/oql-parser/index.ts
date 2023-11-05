import { LRLanguage, LanguageSupport } from "@codemirror/language"
import { completeFromList } from "@codemirror/autocomplete"
import { styleTags, tags as t } from "@lezer/highlight"

import { parser } from "./oql.js"

export { parser } from "./oql.js"

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
})
export const exampleCompletion = exampleLanguage.data.of({
    autocomplete: completeFromList([
        { label: "node", type: "keyword" },
        { label: "way", type: "keyword" },
        { label: "relation", type: "keyword" },
        { label: "nw", type: "function" },
        { label: "wr", type: "function" },
        { label: "nr", type: "function" },
        { label: "nwr", type: "function" },
    ])
})

export function oql() {
    return new LanguageSupport(exampleLanguage, [exampleCompletion])
}

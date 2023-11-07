import { test } from '@jest/globals';

import { testTree } from "@lezer/generator/test";
import { parser } from "./oql";

test('parse simple query', () => {
    let tree = parser.parse(`node["highway"="trunk"];`)
    let spec = `Oql(Query(Keyword,SquareFilter("[",Tag(Key(Identifier),Value(Identifier)),"]")))`;
    testTree(tree, spec)
});

test('parse query with bbox', () => {
    let tree = parser.parse(`node["highway"="trunk"]({{bbox}});`)
    let spec = `Oql(Query(Keyword,SquareFilter("[",Tag(Key(Identifier),Value(Identifier)),"]"),RoundFilter("(",Macro,")")))`;
    testTree(tree, spec)
});

test('parse geocodearea', () => {
    let tree = parser.parse(`{{geocodeArea:Japan@ja}}->.japan;`)
    let spec = `Oql(Macro(MacroValue("@",LanguageCode)),Assignment("->",Set(Variable)))`;
    testTree(tree, spec)
});

test('parse geocodearea multiple values', () => {
    let tree = parser.parse(`{{geocodeArea:"Japan"@ja; Italia@es}}->.japan;`)
    let spec = `Oql(Macro(MacroValue("@",LanguageCode),MacroValue("@",LanguageCode)),Assignment("->",Set(Variable)))`;
    testTree(tree, spec)
});

test('parse around', () => {
    let tree = parser.parse(` way["asdllsdf"](around.hey:8); `)
    let spec = `Oql(Query(Keyword,SquareFilter("[",Tag(Key(Identifier)),"]"),RoundFilter("(",Around(Set(Variable),Radius(Number)),")")))`;
    testTree(tree, spec)
});

test('parse union', () => {
    let tree = parser.parse(`(
        way["highway"~"trunk|hey"]({{bbox}});
        way[~"highway"~"trunk|hey"]({{bbox}});
    );`)
    let spec = `Oql(Union("(",Query(Keyword,SquareFilter("[",Tag(Key(Identifier),Regex),"]"),RoundFilter("(",Macro,")")),Query(Keyword,SquareFilter("[",Tag(Regex,Regex),"]"),RoundFilter("(",Macro,")")),")"))`;
    testTree(tree, spec)
});

test('parse difference', () => {
    let tree = parser.parse(`(
        way["highway"~"trunk|hey"]({{bbox}});
        - way[~"highway"~"trunk|hey"]({{bbox}});
    );`)
    let spec = `Oql(Difference("(",Query(Keyword,SquareFilter("[",Tag(Key(Identifier),Regex),"]"),RoundFilter("(",Macro,")")),Query(Keyword,SquareFilter("[",Tag(Regex,Regex),"]"),RoundFilter("(",Macro,")")),")"))`;
    testTree(tree, spec)
});

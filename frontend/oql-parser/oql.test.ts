import { test } from '@jest/globals';

import { testTree } from "@lezer/generator/test";
import { parser } from "./oql";

test('parse simple query', () => {
    let tree = parser.parse(`node["highway"="trunk"];`)
    let spec = `Oql(Query(Keyword,SquareFilter("[",Tag(Key(Identifier),Value(Identifier)),"]")))`;
    testTree(tree, spec)
});

test('parse query with set filter', () => {
    let tree = parser.parse(`node.setname["highway"="trunk"];`)
    let spec = `Oql(Query(Keyword,Set(Variable),SquareFilter("[",Tag(Key(Identifier),Value(Identifier)),"]")))`;
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
    )->._;`)
    let spec = `Oql(Difference("(",Query(Keyword,SquareFilter("[",Tag(Key(Identifier),Regex),"]"),RoundFilter("(",Macro,")")),Query(Keyword,SquareFilter("[",Tag(Regex,Regex),"]"),RoundFilter("(",Macro,")")),")"),Assignment("->",Set(Variable)))`;
    testTree(tree, spec)
});

test('parse settings', () => {
    let tree = parser.parse(`[out:csv(::id,::type,"name")][bbox:50.6,7.0,50.8,7.3];`)
    let spec = `Oql(Settings("[",SettingName,SettingValue,"]","[",SettingName,SettingValue,"]"))`;
    testTree(tree, spec)
});

test('parse foreach', () => {
    let tree = parser.parse(`foreach.a->.b(node.b["hey"];);`)
    let spec = `Oql(Foreach(Set(Variable),Assignment("->",Set(Variable)),"(",Query(Keyword,Set(Variable),SquareFilter("[",Tag(Key(Identifier)),"]")),")"))`;
    testTree(tree, spec)
});

test('parse recurse', () => {
    let tree = parser.parse(`.a < -> .b; <<; >>;`)
    let spec = `Oql(Set(Variable),RecurseUp,Assignment("->",Set(Variable)),RecurseUpRelations,RecurseDownRelations)`;
    testTree(tree, spec)
});

test('parse is_in', () => {
    let tree = parser.parse(`.a is_in(50.7,7.2)->.b;`)
    let spec = `Oql(IsIn(Set(Variable),"(",Number,Number,")"),Assignment("->",Set(Variable)))`;
    testTree(tree, spec)
});

test('parse is_in empty', () => {
    let tree = parser.parse(`is_in;`)
    let spec = `Oql(IsIn)`;
    testTree(tree, spec)
});

test('parse out', () => {
    let tree = parser.parse(`out;out skel qt;`)
    let spec = `Oql(Out,Out)`;
    testTree(tree, spec)
});

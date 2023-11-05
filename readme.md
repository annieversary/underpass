# underpass

underpass is a data mining tool based on [overpass turbo](https://overpass-turbo.eu/),
aiming to implement extra processing/filtering

## improvements over overpass-turbo

first and foremost, node popups include a link to google maps and a link to copy coordinates for the node.
this helps smooth the workflow of reviewing results

### code editor

the code editor implements an overpass ql parser, which enables better syntax highlighting and smarter autocomplete

it also adds some features that are common in more fully featured, such as `Ctrl+F` to find/replace, and `Ctrl+D` to select next find match

### overpass ql extensions

#### geocodeArea

using a `geocodeArea` macro will display the found area at the bottom, so you can ensure you are looking in the correct spot

`geocodeArea` has been extended to support multiple areas, separated by `;`

```
{{geocodeArea:Hokkaido, Japan; Aomori, Japan}}->.japan;
```

`geocodeArea` supports specifying what language to search in by adding `@{lang code}`:

```
{{geocodeArea:Hokkaido, Japón@es; Madrid, España@es; Île-de-France@fr}}->.places;
```

if no language is specified, `en` is used

#### aroundSelf macro
    
it also implements more macros, such as `aroundSelf`, which works like:

```
node["amenity"="bench"]({{bbox}})->.benches;
{{aroundSelf.benches:7}}->.benchesAroundOtherBenches;
```

#### out macro

`{{out}}` simply expands to `out;>;out skel qt;`

### node editor

underpass implements a node editor that allows for powerful filtering that cannot be simply with the overpass API.
it can be used to set up advanced filters, such as filtering roads by their [bearing](https://en.wikipedia.org/wiki/Bearing_(angle)).
not many filters have been implemented yet, but more are comming soon

## building

you will need `rust`, `cargo`, `node`, and `npm` installed

frontend is built by running either `npm run build` (development), `npm run watch` (watch command), or `npm run prod` (production).
typescript can be typechecked by running `npm run typecheck`

rust can be built and run by `cargo run`, and `cargo run --release` for production

remember to build the frontend first, since it gets included into the rust binary during compilation

# underpass

(wip name until we decide on something else)

underpass is a clone of [overpass turbo](https://overpass-turbo.eu/)
which aims to implement extra processing/filtering of nodes.
at the moment it has (mostly) feature parity with overpass, so you can definitely use this instead of the original

## improvements over overpass turbo

first and foremost, node popups include a link to google maps and a link to copy coordinates for the node.
this helps smooth the workflow of reviewing results

### geocodeArea

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

### aroundSelf macro

it also implements more macros, such as `aroundSelf`, which works like:

```
node["amenity"="bench"]({{bbox}})->.benches;
{{aroundSelf.benches:7}}->.benchesAroundOtherBenches;
```

### out macro

`{{out}}` simply expands to `out;>;out skel qt;`

## building

you will need `rust`, `cargo`, `node`, and `npm` installed

frontend is built by running either `npm run build` (development), `npm run watch` (watch command), or `npm run prod` (production).
typescript can be typechecked by running `npm run typecheck`

rust can be built and run by `cargo run`, and `cargo run --release` for production

remember to build the frontend first, since it gets included into the rust binary during compilation

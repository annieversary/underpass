# underpass

(wip name until we decide on something else)

underpass is a clone of [overpass turbo](https://overpass-turbo.eu/)
which aims to implement extra processing/filtering of nodes.
at the moment it has (mostly) feature parity with overpass, so you can definitely use this instead of the original


## improvements over overpass turbo

first and foremost, node popups include a link to google maps and a link to copy coordinates for the node.
this helps smooth the workflow of reviewing results

using a `geocodeArea` macro will display the found area at the bottom, so you can ensure you are looking in the correct spot

it also implements more macros, such as `aroundSelf`, which works like:

```
node["amenity"="bench"]({{bbox}})->.benches;
{{aroundSelf.benches:7}}->.benchesAroundOtherBenches;
```

`geocodeArea` has been extended to support multiple areas, separated by `;`

```
{{geocodeArea:Hokkaido, Japan; Aomori, Japan}}->.japan;
```

# geocodeArea

using a `geocodeArea` macro will display the found area at the bottom, so you can ensure you are looking in the correct spot

`geocodeArea` has been extended to support multiple areas, separated by `;`

```
{{geocodeArea:Hokkaido, Japan; Aomori, Japan}}->.japan;
```

`geocodeArea` also supports specifying what language to search in by adding `@{lang code}`:

```
{{geocodeArea:Hokkaido, Japón@es; Madrid, España@es; Île-de-France@fr}}->.places;
```

if no language is specified, `en` is used

# aroundSelf macro
    
it also implements more macros, such as `aroundSelf`, which works like:

```
node["amenity"="bench"]({{bbox}})->.benches;
{{aroundSelf.benches:7}}->.benchesAroundOtherBenches;
```

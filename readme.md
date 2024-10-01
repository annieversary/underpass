# underpass

underpass is a data mining tool inspired by [overpass turbo](https://overpass-turbo.eu/),
aiming to implement extra processing/filtering

the implementation differs highly from overpass-turbo's, as underpass performs the fetching and filtering steps on a backend,
instead of directly on the user's browser

## building

you will need `rust`, `cargo`, `node`, and `npm` installed

frontend is built by running either `npm run build` (development), `npm run watch` (watch command), or `npm run prod` (production).
typescript can be typechecked by running `npm run typecheck`

rust can be built and run by `cargo run`, and `cargo run --release` for production

remember to build the frontend first, as it gets included into the rust binary during compilation

### datasets

underpass needs two datasets: taginfo and elevation. 
these are not included in this repo, and have to be generated before running underpass. 
read [data/readme.md](./data/readme.md) for more information

## improvements over overpass-turbo

first and foremost, node popups include a link to google maps and a link to copy coordinates for the node.
this helps smooth the workflow of reviewing results

### code editor

the code editor implements an overpass ql parser, which enables better syntax highlighting and smarter autocomplete

it also adds some features that are common in more fully featured, such as `Ctrl+F` to find/replace, and `Ctrl+D` to select next find match

### overpass ql extensions

read [docs/overpass-ql-extensions.md](./docs/overpass-ql-extensions.md)

### node editor

underpass implements a node editor that allows for powerful filtering that cannot be simply with the overpass API.
it can be used to set up advanced filters, such as filtering roads by their [bearing](https://en.wikipedia.org/wiki/Bearing_(angle)).
not many filters have been implemented yet, but more are comming soon

### map

the map is implemented using the [maplibre gl](https://maplibre.org/maplibre-gl-js/docs/) library,
which has a better performance when dealing with larger amounts of nodes


import { GeoJSON, Feature } from 'geojson';

import './style.css';
import 'maplibre-gl/dist/maplibre-gl.css';
import '@maplibre/maplibre-gl-geocoder/dist/maplibre-gl-geocoder.css';

import { SearchError, SearchSuccess, search } from './search';
import { mapBounds, setMapData, getMapData } from './map';
import { serializeGraph } from './graph/save';
import { settings } from './settings';
import { setLoading, isLoading } from './loading';
import './resizer';

export let processedQueries = {};



let resultsDiv = document.querySelector("#results");

async function run() {
    if (isLoading()) return;
    setLoading(true);

    setMapData('OverpassAPI', { type: "FeatureCollection", features: [] });

    try {
        const response = await search(mapBounds(), serializeGraph());

        resultsDiv.innerHTML = '';
        if (response.ok === 'true') {
            handleRunSuccess(response);
        } else {
            handleRunError(response);
        }
    } catch (e) {
        console.error(e);
        if (e instanceof TypeError) {
            alert(e.message);
        }
    }

    setLoading(false);
}

function handleRunSuccess(response: SearchSuccess) {
    const data = response.data;

    if (settings.hideEmptyNodes()) {
        data.features = data.features
            .filter((f: Feature) => !(f.geometry.type == "Point" && Object.keys(f.properties).length == 0));
    }

    setMapData('OverpassAPI', data);

    if (response.geocode_areas.length > 0) {
        const areas = response.geocode_areas.map((a: any) => `${a.original} - <a href="//www.openstreetmap.org/${a.ty}/${a.id}" target="_blank" class="osm-link">${a.name}</a><br/>`).join('');
        resultsDiv.innerHTML = `<h2>Geocode areas found:</h2>${areas}`;
    }

    if (response.processed_queries) {
        processedQueries = response.processed_queries;
    }
}

function handleRunError(response: SearchError) {
    const data = response.data;
    if (data.format === "xml") {
        const dom = new window.DOMParser().parseFromString(
            data.message,
            "text/xml"
        );
        alert(
            Array.from(dom.body.querySelectorAll("p"))
                .slice(1)
                .map((p) => p.textContent)
                .join("\n")
        );
    } else {
        alert(response.error);
    }

    // TODO do something to highlight node if node_id is set
}

document.querySelector<HTMLButtonElement>('#run-button').onclick = run;
document.addEventListener('keydown', (event) => {
    if ((event.ctrlKey || event.metaKey) && event.key == "Enter") {
        event.preventDefault();
        run();
    }
});

document.querySelector<HTMLButtonElement>('#clear-button').onclick = () => {
    setMapData('OverpassAPI', { type: "FeatureCollection", features: [] });
};

document.querySelector<HTMLButtonElement>('#export-button').onclick = () => {
    const out = getMapData('OverpassAPI');
    if (out.data.features.length == 0) {
        alert('No data to export!');
    } else {
        downloadAsJsonFile('export.json', out);
    }
};
/**
 * Download the provided object as a JSON file
 */
function downloadAsJsonFile(filename: string, object: any) {
    var element = document.createElement('a');
    element.setAttribute('href', 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(object)));
    element.setAttribute('download', filename);

    element.style.display = 'none';
    document.body.appendChild(element);

    element.click();

    document.body.removeChild(element);
}

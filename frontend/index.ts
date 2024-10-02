import { MapMouseEvent } from 'maplibre-gl';
import * as turf from '@turf/turf';
import { GeoJSON, Feature } from 'geojson';

import './style.css';
import 'maplibre-gl/dist/maplibre-gl.css';
import '@maplibre/maplibre-gl-geocoder/dist/maplibre-gl-geocoder.css';

import { SearchError, SearchSuccess, search } from './search';
import { map, mapBounds, mapSetData } from './map';
import { serializeGraph } from './graph/save';
import { settings } from './settings';
import { setLoading, isLoading } from './loading';
import './resizer';

export let processedQueries = {};



let resultsDiv = document.querySelector("#results");

async function run() {
    if (isLoading()) return;
    setLoading(true);

    mapSetData('OverpassAPI', { type: "FeatureCollection", features: [] });

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

    mapSetData('OverpassAPI', data);

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
    mapSetData('OverpassAPI', { type: "FeatureCollection", features: [] });
};

document.querySelector<HTMLButtonElement>('#export-button').onclick = () => {
    const out = map.getSource("OverpassAPI").serialize();
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






// corner tooltip with coordinates
const infoDiv = document.getElementById('info');

function updateInfo(e: MapMouseEvent) {
    infoDiv.innerHTML = `${e.lngLat.wrap().lat.toFixed(8)},${e.lngLat.wrap().lng.toFixed(8)}`;
    if (distance) {
        infoDiv.innerHTML += `, distance: ${distance}m`;
    }
}
map.on('mousemove', updateInfo);
map.on('mouseup', updateInfo);

let distance = null;
let distanceMeasureGeojson = {
    'type': 'FeatureCollection',
    'features': []
};
map.on('load', () => {
    const linestring = {
        'type': 'Feature',
        'geometry': {
            'type': 'LineString',
            'coordinates': []
        }
    };

    map.addSource('distance-measure', {
        'type': 'geojson',
        'data': distanceMeasureGeojson
    });

    // Add styles to the map
    map.addLayer({
        id: 'measure-points',
        type: 'circle',
        source: 'distance-measure',
        paint: {
            'circle-radius': 5,
            'circle-color': '#000'
        },
        filter: ['in', '$type', 'Point']
    });
    map.addLayer({
        id: 'measure-lines',
        type: 'line',
        source: 'distance-measure',
        layout: {
            'line-cap': 'round',
            'line-join': 'round'
        },
        paint: {
            'line-color': '#000',
            'line-width': 2.5
        },
        filter: ['in', '$type', 'LineString']
    });


    map.on('click', (e) => {
        const on = distanceButton.dataset.on == 'true';
        if (!on) return;

        if (distanceMeasureGeojson.features.length > 1) distanceMeasureGeojson.features.pop();

        const point = {
            'type': 'Feature',
            'geometry': {
                'type': 'Point',
                'coordinates': [e.lngLat.lng, e.lngLat.lat]
            },
            'properties': {
                'id': String(new Date().getTime())
            }
        };

        distanceMeasureGeojson.features.push(point);

        if (distanceMeasureGeojson.features.length > 1) {
            linestring.geometry.coordinates = distanceMeasureGeojson.features.map(
                (point) => {
                    return point.geometry.coordinates;
                }
            );

            distanceMeasureGeojson.features.push(linestring);

            distance = (1000 * turf.length(linestring as Feature)).toLocaleString();
        }

        mapSetData('distance-measure', distanceMeasureGeojson as GeoJSON);
    });

});

const distanceButton = document.querySelector<HTMLButtonElement>('#distance-button');
function toggleDistance() {
    distanceButton.dataset.on = distanceButton.dataset.on == 'true' ? 'false' : 'true';

    const on = distanceButton.dataset.on == 'true';
    if (!on) {
        distance = null;
        mapSetData('distance-measure', { type: "FeatureCollection", features: [] });
        distanceMeasureGeojson = {
            'type': 'FeatureCollection',
            'features': []
        };
    }

    map.getCanvas().style.cursor = !on ? 'pointer' : 'crosshair';
};

distanceButton.onclick = toggleDistance;


// setup keybindings
const deltaDistance = 100;
function easing(t: number) {
    return t * (2 - t);
}
map.getCanvas().addEventListener(
    'keydown',
    (e) => {
        if (e.key === 'D') {
            toggleDistance();
        } else if (e.key === 'w') {
            map.panBy([0, -deltaDistance], {
                easing
            });
        } else if (e.key === 's') {
            map.panBy([0, deltaDistance], {
                easing
            });
        } else if (e.key === 'a') {
            map.panBy([-deltaDistance, 0], {
                easing
            });
        } else if (e.key === 'd') {
            map.panBy([deltaDistance, 0], {
                easing
            });
        }
    },
    true
);

import { GeoJSONSource, MapMouseEvent } from 'maplibre-gl';
import * as turf from '@turf/turf';
import { GeoJSON, Feature } from 'geojson';

import './style.css';
import 'maplibre-gl/dist/maplibre-gl.css';
import '@maplibre/maplibre-gl-geocoder/dist/maplibre-gl-geocoder.css';

import { map, mapBounds } from './map';
import { serializeGraph } from './graph/save';
import { settings } from './settings';



// resize
const resizer = document.querySelector("#resizer");
const left: HTMLDivElement = document.querySelector("#left");
const right: HTMLDivElement = document.querySelector("#right");

function resize(e: { x: number }) {
    left.style.width = `${e.x}px`;
    right.style.width = `calc(100vw - ${e.x}px)`;
    window.localStorage.setItem('codeWidth', e.x.toString());
}

const codeWidth = +window.localStorage.getItem('codeWidth') || (window.innerWidth * 0.3);
// so for some reason, when the codeWidth is too small, the map doesnt fill all of the available space
// resizing does force it to take all the space available, so we set it to +1, then immediately set it to the correct value
// this makes it work as expected tho it's *slightly* hacky and funky
resize({ x: codeWidth + 1 });
setTimeout(() => {
    resize({ x: codeWidth });
}, 100);

resizer.addEventListener("mousedown", () => {
    document.addEventListener("mousemove", resize, false);
    document.addEventListener("mouseup", () => {
        document.removeEventListener("mousemove", resize, false);
    }, false);
});




export let processedQueries = {};



let resultsDiv = document.querySelector("#results");

// loading modal doesnt use openModal cause we want this custom style which looks nicer imo
let loading = false;
let loadingModal: HTMLDivElement = document.querySelector("#loading-modal");

function mapSetSource(source: string, data: GeoJSON) {
    (map.getSource(source) as GeoJSONSource)
        .setData(data);
}

async function run() {
    if (loading) return;
    loading = true;
    loadingModal.style.display = 'flex';

    mapSetSource('OverpassAPI', { type: "FeatureCollection", features: [] });

    try {
        const r = await fetch('/search', {
            method: 'POST',
            body: JSON.stringify({
                bbox: mapBounds(),
                graph: serializeGraph(),
            }),
            headers: {
                'Content-Type': 'application/json'
            },
        });
        const res = await r.json();

        resultsDiv.innerHTML = '';
        if (r.status == 200) {
            const data = res.data;

            if (settings.hideEmptyNodes()) {
                data.features = data.features
                    .filter((f: Feature) => !(f.geometry.type == "Point" && Object.keys(f.properties).length == 0));
            }

            mapSetSource('OverpassAPI', data);

            if (res.geocode_areas.length > 0) {
                const areas = res.geocode_areas.map((a: any) => `${a.original} - <a href="//www.openstreetmap.org/${a.ty}/${a.id}" target="_blank" class="osm-link">${a.name}</a><br/>`).join('');
                resultsDiv.innerHTML = `<h2>Geocode areas found:</h2>${areas}`;
            }

            if (res.processed_queries) {
                processedQueries = res.processed_queries;
            }
        } else {
            if (res.format == 'xml') {
                const dom = new window.DOMParser().parseFromString(
                    res.message,
                    "text/xml"
                );
                alert(
                    Array.from(dom.body.querySelectorAll("p"))
                        .slice(1)
                        .map((p) => p.textContent)
                        .join("\n")
                );
            } else {
                alert(res.error);
            }

            // TODO do something to highlight node if node_id is set
        }
    } catch (e) {
        console.error(e);
        if (e instanceof TypeError) {
            alert(e.message);
        }
    }

    loading = false;
    loadingModal.style.display = 'none';
}

document.querySelector<HTMLButtonElement>('#run-button').onclick = run;
document.addEventListener('keydown', (event) => {
    if ((event.ctrlKey || event.metaKey) && event.key == "Enter") {
        event.preventDefault();
        run();
    }
});

document.querySelector<HTMLButtonElement>('#clear-button').onclick = () => {
    mapSetSource('OverpassAPI', { type: "FeatureCollection", features: [] });
};

document.querySelector<HTMLButtonElement>('#export-button').onclick = () => {
    const out = map.getSource("OverpassAPI").serialize();
    if (out.data.features.length == 0) {
        alert('No data to export!');
    } else {
        download('export.json', out);
    }
};
function download(filename: string, json: any) {
    var element = document.createElement('a');
    element.setAttribute('href', 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(json)));
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

        mapSetSource('distance-measure', distanceMeasureGeojson as GeoJSON);
    });

});

const distanceButton = document.querySelector<HTMLButtonElement>('#distance-button');
function toggleDistance() {
    distanceButton.dataset.on = distanceButton.dataset.on == 'true' ? 'false' : 'true';

    const on = distanceButton.dataset.on == 'true';
    if (!on) {
        distance = null;
        mapSetSource('distance-measure', { type: "FeatureCollection", features: [] });
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

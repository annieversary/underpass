import { MapLayerEventType, GeoJSONSource, AddLayerObject, Popup } from 'maplibre-gl';
import turfLength from '@turf/length';
import { GeoJSON, Feature } from 'geojson';

import './style.css';
import 'maplibre-gl/dist/maplibre-gl.css';
import '@maplibre/maplibre-gl-geocoder/dist/maplibre-gl-geocoder.css';

import { editor } from './codeEditor';
import { map, mapBounds } from './map';



// resize
const resizer = document.querySelector("#resizer");
const left: HTMLDivElement = document.querySelector("#left");
const right: HTMLDivElement = document.querySelector("#right");

function resize(e: { x: number }) {
    left.style.width = `${e.x}px`;
    right.style.width = `${window.innerWidth - e.x}px`;
    window.localStorage.setItem('codeWidth', e.x.toString());
}

const codeWidth = +window.localStorage.getItem('codeWidth') || (window.innerWidth * 0.3);
resize({ x: codeWidth });

resizer.addEventListener("mousedown", () => {
    document.addEventListener("mousemove", resize, false);
    document.addEventListener("mouseup", () => {
        document.removeEventListener("mousemove", resize, false);
    }, false);
});






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
                query: editor.state.doc.toString(),
                bbox: mapBounds(),
                road_angle: !document.querySelector<HTMLInputElement>('#road-angle-toggle').checked ? null : {
                    min: +document.querySelector<HTMLInputElement>('#road-angle-min').value,
                    max: +document.querySelector<HTMLInputElement>('#road-angle-max').value,
                },
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
        } else {
            if (res.format == 'xml') {
                const dom = new window.DOMParser().parseFromString(
                    res.error,
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
        }

        if ("query" in res) {
            let button = document.getElementById('view-query-button');
            if (!button) {
                button = document.createElement('button');
                button.innerHTML = 'View query';
                button.id = 'view-query-button';
                document.getElementById('header-buttons').appendChild(button);
            }
            button.onclick = () => openModal(`<pre>${res.query}</pre>`);
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
map.on('mousemove', (e) => {
    infoDiv.innerHTML = `${e.lngLat.wrap().lat.toFixed(8)},${e.lngLat.wrap().lng.toFixed(8)}`;
    if (distance) {
        infoDiv.innerHTML += `, Distance: ${distance}m`;
    }
});

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

            distance = (1000 * turfLength(linestring as Feature)).toLocaleString();
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
function easing(t) {
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



// throwaway modals
// content: string
function openModal(content: string) {
    const modal = document.createElement('div');
    modal.classList.add('modal');
    modal.innerHTML = `
    <div class="modal-content">
        <span class="close">&times;</span>
        <div class="modal-inner">
            ${content}
        </div>
    </div>`;
    document.body.appendChild(modal);
    modal.querySelector<HTMLSpanElement>('span.close').onclick = function() {
        modal.remove();
    };
    window.onclick = function(event: MouseEvent) {
        if (event.target == modal) {
            modal.remove();
        }
    };

    // TODO pressing esc should close the modal
}



// settings
// done by hand cause it's not a throwaway modal like with openModal
const settingsButton = document.getElementById('settings-button');
const settingsModal = document.getElementById('settings-modal');
settingsButton.onclick = function() {
    settingsModal.style.display = 'flex';

    settingsModal.querySelector<HTMLSpanElement>('span.close').onclick = function() {
        settingsModal.style.display = 'none';
    };
    window.onclick = function(event: MouseEvent) {
        if (event.target == settingsModal) {
            settingsModal.style.display = 'none';
        }
    };
};

const settings = {
    hideEmptyNodes: () => document.querySelector<HTMLInputElement>('#settings-hide-empty-nodes').checked,
};

// TODO we probably want a way to abstract this when we add more settings keys
const settingsHideEmptyNodes = document.querySelector<HTMLInputElement>('#settings-hide-empty-nodes');
settingsHideEmptyNodes.checked = window.localStorage.getItem('settings.hideEmptyNodes') === 'true';
settingsHideEmptyNodes.onchange = function() {
    window.localStorage.setItem('settings.hideEmptyNodes', settingsHideEmptyNodes.checked.toString());
};

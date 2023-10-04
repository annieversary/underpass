let [zoom, lat, lng] = JSON.parse(window.localStorage.getItem("viewport")) || [
    16, 51.945356463918586, -0.0175273074135589,
];

const map = new maplibregl.Map({
    container: 'map',
    style: 'https://demotiles.maplibre.org/style.json',
    style: {
        version: 8,
        sources: {
            'raster-tiles': {
                type: 'raster',
                tiles: [
                    'https://tile.openstreetmap.org/{z}/{x}/{y}.png'
                ],
                tileSize: 256,
                attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
            }
        },
        layers: [
            {
                id: 'simple-tiles',
                type: 'raster',
                source: 'raster-tiles',
                minzoom: 0,
                maxzoom: 22
            }
        ]
    },
    center: [lng, lat],
    zoom
});

var editor = CodeMirror(document.getElementById("code-container"), {
    styleActiveLine: true,
    lineNumbers: true,
    matchBrackets: true,
    autoCloseBrackets: true,
    autoCloseTags: true,
    mode: "htmlmixed",
});

const query = window.localStorage.getItem('query') || '[out:json][timeout:25];\n\n(way["highway"]({{bbox}}););\n\nout;>;out skel qt;';
editor.setValue(query);
editor.on('change', function () {
    window.localStorage.setItem('query', editor.getValue());
});

function mapBounds() {
    const b = map.getBounds();
    return {
        ne: [b._ne.lat, b._ne.lng],
        sw: [b._sw.lat, b._sw.lng],
    };
}



let resultsDiv = document.querySelector("#results");

let loading = false;
let loadingModal = document.querySelector("#loading-modal");
async function run() {
    if (loading) return;
    loading = true;
    loadingModal.style.display = 'flex';

    map.getSource("OverpassAPI")
        .setData({ type: "FeatureCollection", features: [] });

    try {
        const r = await fetch('/search', {
            method: 'POST',
            body: JSON.stringify({
                query: editor.getValue(),
                bbox: mapBounds(),
            }),
            headers: {
                'Content-Type': 'application/json'
            },
        });
        const res = await r.json();

        if (r.status == 200) {
            map.getSource("OverpassAPI").setData(res.data);

            const areas = res.geocode_areas.map(a => `<a href="//www.openstreetmap.org/${a.ty}/${a.id}" target="_blank" class="osm-link">${a.name}</a><br/>`);
            resultsDiv.innerHTML = `<h2>Geocode areas found:</h2>${areas}`;
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
    } catch (e) {
        console.log(e);
    }

    loading = false;
    loadingModal.style.display = 'none';
}

document.querySelector('#run-button').onclick = run;
document.addEventListener('keydown', (event) => {
    if((event.ctrlKey || event.metaKey) && event.key == "Enter") {
        run();
    }
});

document.querySelector('#clear-button').onclick = () => {
    map.getSource("OverpassAPI")
        .setData({ type: "FeatureCollection", features: [] });
};



map.on("style.load", () => {
    if (!map.getSource("OverpassAPI")) {
        map.addSource("OverpassAPI", {
            type: "geojson",
            data: { type: "FeatureCollection", features: [] },
        });

        let justOpenedPopup = false;

        const openPopup = (e) => {
            // debounce so we dont open two popups at once with the same click
            if (justOpenedPopup) return;
            justOpenedPopup = true;
            setTimeout(() => justOpenedPopup = false, 100);

            const f = e.features[0];
            const coordinates = f.geometry.coordinates.slice();

            // Ensure that if the map is zoomed out such that multiple
            // copies of the feature are visible, the popup appears
            // over the copy being pointed to.
            while (Math.abs(e.lngLat.lng - coordinates[0]) > 180) {
                coordinates[0] += e.lngLat.lng > coordinates[0] ? 360 : -360;
            }

            const props = Object.entries(f.properties)
                                .map(([k, v]) => `${k} = ${v}`)
                                .join('<br>');

            // this is not accurate but good enough
            const type = f.layer.type == 'fill' ? 'relation' : (f.layer.type == 'line' ? 'way' : 'node');

            const html = `<a href="//www.openstreetmap.org/${type}/${f.id}" target="_blank" class="osm-link">${f.id}</a><br/><br/>
            ${props}<br/><br/>
            <a href="https://google.co.uk/maps?q=${e.lngLat.lat},${e.lngLat.lng}" target="_blank" class="map-link"
                onclick="map.setFeatureState({source: 'OverpassAPI', id: ${f.id}}, {visited: true}); "
            >google maps</a><br/>
            <a href="javascript:navigator.clipboard.writeText('${e.lngLat.lat},${e.lngLat.lng}')" class="map-link">copy</a>
            `;

            new maplibregl.Popup()
                            .setLngLat(e.lngLat)
                            .setHTML(html)
                            .addTo(map)
                            .on('close', () => map.setFeatureState(
                                {source: 'OverpassAPI', id: f.id},
                                {selected: false}
                            ));

            // highlight the current thing
            map.setFeatureState(
                {source: 'OverpassAPI', id: f.id},
                {selected: true}
            );
        };

        function openContextMenu(e) {
            const f = e.features[0];
            map.setFeatureState(
                {source: 'OverpassAPI', id: f.id},
                {visited: !f.state.visited}
            );
        }

        const layers = [
            {
                id: "overpass-polygons",
                type: "fill",
                source: "OverpassAPI",
                filter: ["all", ["==", ["geometry-type"], "Polygon"]],
                paint: {
                    "fill-color": "rgba(255, 204, 0, .5)",
                },
            },
            {
                id: "overpass-polygons-stroke",
                type: "line",
                source: "OverpassAPI",
                filter: ["all", ["==", ["geometry-type"], "Polygon"]],
                paint: { "line-width": 2, "line-color": "rgba(0, 51, 255, 0.6)" },
            },
            {
                id: "overpass-lines",
                type: "line",
                source: "OverpassAPI",
                filter: ["all", ["==", ["geometry-type"], "LineString"]],
                paint: {
                    "line-width": 5,
                    "line-color": [
                        'case',
                        ['boolean', ['feature-state', 'selected'], false], "rgba(200, 51, 255, 0.6)",
                        ['boolean', ['feature-state', 'visited'], false], "rgba(0, 204, 200, 0.6)",
                        "rgba(0, 51, 255, 0.6)",
                    ]

                },
                layout: { "line-cap": "round" },
            },
            {
                id: "overpass-poi",
                type: "circle",
                source: "OverpassAPI",
                filter: ["all", ["==", ["geometry-type"], "Point"]],
                paint: {
                    "circle-stroke-width": 2,
                    "circle-stroke-color": "rgba(0, 51, 255, 0.6)",
                    "circle-color": [
                        'case',
                        ['boolean', ['feature-state', 'visited'], false], "rgba(0, 204, 200, 0.6)",
                        "rgba(250, 204, 0, 0.6)",
                    ]
                },
            },
        ];
        for (const layer of layers) {
            if (!map.getLayer(layer)) {
                map.addLayer(layer);

                map.on('click', layer.id, openPopup);
                map.on('mouseenter', layer.id, () => {map.getCanvas().style.cursor = 'pointer';});
                map.on('mouseleave', layer.id, () => {map.getCanvas().style.cursor = '';});

                map.on('contextmenu', layer.id, openContextMenu);
            }
        }
    }
});



// corner tooltip with coordinates
const infoDiv = document.getElementById('info');
map.on('mousemove', (e) => {
    infoDiv.innerHTML = `${e.lngLat.wrap().lat.toFixed(8)},${e.lngLat.wrap().lng.toFixed(8)}`;
    if (distance) {
        infoDiv.innerHTML += `, Distance: ${distance}m`;
    }
});

// store viewport position in localstorage
map.on("moveend", () => {
    window.localStorage.setItem("viewport", JSON.stringify([
        map.getZoom(0), map.getCenter().lat.toFixed(8), map.getCenter().lng.toFixed(8),
    ]));
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

            distance = (1000 * turf.length(linestring)).toLocaleString();
        }

        map.getSource('distance-measure').setData(distanceMeasureGeojson);
    });

});

const distanceButton = document.querySelector('#distance-button');
function toggleDistance() {
    distanceButton.dataset.on = distanceButton.dataset.on == 'true' ? 'false' : 'true';

    const on = distanceButton.dataset.on == 'true';
    if (!on) {
        distance = null;
        map.getSource("distance-measure")
            .setData({ type: "FeatureCollection", features: [] });
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
const deltaDegrees = 25;
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



// search
const geocoderApi = {
    forwardGeocode: async (config) => {
        const features = [];
        try {
            const request =
                `https://nominatim.openstreetmap.org/search?q=${
    config.query
}&format=geojson&polygon_geojson=1&addressdetails=1`;
            const response = await fetch(request);
            const geojson = await response.json();
            for (const feature of geojson.features) {
                const center = [
                    feature.bbox[0] +
            (feature.bbox[2] - feature.bbox[0]) / 2,
                    feature.bbox[1] +
            (feature.bbox[3] - feature.bbox[1]) / 2
                ];
                const point = {
                    type: 'Feature',
                    geometry: {
                        type: 'Point',
                        coordinates: center
                    },
                    place_name: feature.properties.display_name,
                    properties: feature.properties,
                    text: feature.properties.display_name,
                    place_type: ['place'],
                    center
                };
                features.push(point);
            }
        } catch (e) {
            console.error(`Failed to forwardGeocode with error: ${e}`);
        }

        return {
            features
        };
    }
};
map.addControl(
    new MaplibreGeocoder(geocoderApi, {
        maplibregl
    })
);

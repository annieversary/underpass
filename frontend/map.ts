import maplibregl, { MapLayerEventType, GeoJSONSource, AddLayerObject, Map, Popup, MapGeoJSONFeature, LngLat, MapMouseEvent } from 'maplibre-gl';
import MaplibreGeocoder from '@maplibre/maplibre-gl-geocoder';
import * as turf from '@turf/turf';
import { GeoJSON, Feature } from 'geojson';

let [zoom, lat, lng] = JSON.parse(window.localStorage.getItem("viewport")) || [
    16, 51.945356463918586, -0.0175273074135589,
];

const map = new Map({
    container: 'map',
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


/**
 * Sets the GeoJSON data for this source
 */
export function setMapData(source: string, data: GeoJSON): void {
    (map.getSource(source) as GeoJSONSource)
        .setData(data);
}

/**
 * @returns A plain (stringifiable) JS object representing the current state of the source.
 * Creating a source using the returned object as the `options` should result in a Source that is
 * equivalent to this one.
 */
export function getMapData(source: string): any {
    return map.getSource(source).serialize();
}

export type MapBounds = {
    ne: [number, number],
    sw: [number, number],
};

export function mapBounds(): MapBounds {
    const b = map.getBounds();
    return {
        ne: [b._ne.lat, b._ne.lng],
        sw: [b._sw.lat, b._sw.lng],
    };
}

// store viewport position in localstorage
map.on("moveend", () => {
    window.localStorage.setItem("viewport", JSON.stringify([
        map.getZoom(), map.getCenter().lat.toFixed(8), map.getCenter().lng.toFixed(8),
    ]));
});

map.on("style.load", () => {
    if (!map.getSource("OverpassAPI")) {
        map.addSource("OverpassAPI", {
            type: "geojson",
            data: { type: "FeatureCollection", features: [] },
        });

        let justOpenedPopup = false;

        const openPopup = (e: MapLayerEventType['click'] & Object) => {
            // debounce so we dont open two popups at once with the same click
            if (justOpenedPopup) return;
            justOpenedPopup = true;
            setTimeout(() => justOpenedPopup = false, 100);

            const f = e.features[0];
            const geometry = f.geometry;

            let point: LngLat;
            if (geometry.type == 'Point') {
                point = new LngLat(geometry.coordinates[0], geometry.coordinates[1]);
            } else if (geometry.type == 'LineString' || geometry.type == 'Polygon') {
                const nearest = turf.nearestPoint(
                    turf.point([e.lngLat.lng, e.lngLat.lat]),
                    turf.featureCollection(geometry.coordinates.map((p: any) => turf.point(p)))
                );
                point = new LngLat(nearest.geometry.coordinates[0], nearest.geometry.coordinates[1]);
            } else {
                point = e.lngLat;
            }

            const props = Object.entries(f.properties)
                .filter(([k, _]) => !k.startsWith("__"))
                .map(([k, v]) => `${k} = ${v}`)
                .join('<br>');

            const osm_id = f.properties.osm_id;
            const osm_type = f.properties.osm_type;

            const div = document.createElement('div');
            div.innerHTML = `<a href="//www.openstreetmap.org/${osm_type}/${osm_id}" target="_blank" class="osm-link">${osm_id}</a><br/><br/>
            ${props}<br/><br/>
            <a href="https://google.co.uk/maps?q=${point.lat},${point.lng}" target="_blank" class="map-link google-maps-link">google maps</a>
            <br/>
            <a href="javascript:navigator.clipboard.writeText('${point.lat},${point.lng}')" class="map-link">copy</a>`;

            div.querySelector<HTMLAnchorElement>('.google-maps-link').onclick = () => {
                markAsVisited(f, true);
            };

            new Popup()
                .setLngLat(point)
                .setDOMContent(div)
                .addTo(map)
                .on('close', () => f.id ? map.setFeatureState(
                    { source: 'OverpassAPI', id: f.id },
                    { selected: false }
                ) : null);

            // highlight the current thing
            map.setFeatureState(
                { source: 'OverpassAPI', id: f.id },
                { selected: true }
            );
        };

        function openContextMenu(e: MapLayerEventType['contextmenu'] & Object) {
            const f = e.features[0];

            if (f.properties.osm_type == 'node' && f.properties.__way_id) {
                const feats = map.querySourceFeatures('OverpassAPI', { filter: ['==', 'osm_id', f.properties.__way_id] });
                if (feats.length == 0) return;
                const feat = feats[0];
                const state = map.getFeatureState({
                    source: 'OverpassAPI',
                    id: feat.id,
                });
                markAsVisited(feat, !state.visited);
            } else {
                markAsVisited(f, !f.state.visited);
            }
        }

        let justMarkedAsVisited = false;

        /// Marks the feature as visited, and if it's a way, it marks all corresponding nodes too
        function markAsVisited(feature: MapGeoJSONFeature, visited: boolean) {
            // debounce so we dont open two popups at once with the same click
            if (justMarkedAsVisited) return;
            justMarkedAsVisited = true;
            setTimeout(() => justMarkedAsVisited = false, 100);

            if (feature.properties.osm_type == 'way' && feature.properties.__children_ids) {
                const ids = JSON.parse(feature.properties.__children_ids);
                for (const id of ids) {
                    map.setFeatureState(
                        { source: 'OverpassAPI', id: id },
                        { visited }
                    );
                }
            }

            map.setFeatureState(
                { source: 'OverpassAPI', id: feature.id },
                { visited }
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
            if (!map.getLayer(layer.id)) {
                map.addLayer(layer as AddLayerObject);

                map.on('click', layer.id, openPopup);
                map.on('mouseenter', layer.id, () => { map.getCanvas().style.cursor = 'pointer'; });
                map.on('mouseleave', layer.id, () => { map.getCanvas().style.cursor = ''; });

                map.on('contextmenu', layer.id, openContextMenu);
            }
        }
    }
});

// search
const geocoderApi = {
    forwardGeocode: async (config: any) => {
        const features = [];
        try {
            const request =
                `https://nominatim.openstreetmap.org/search?q=${config.query
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

        setMapData('distance-measure', distanceMeasureGeojson as GeoJSON);
    });

});

const distanceButton = document.querySelector<HTMLButtonElement>('#distance-button');
function toggleDistance() {
    distanceButton.dataset.on = distanceButton.dataset.on == 'true' ? 'false' : 'true';

    const on = distanceButton.dataset.on == 'true';
    if (!on) {
        distance = null;
        setMapData('distance-measure', { type: "FeatureCollection", features: [] });
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

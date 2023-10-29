import maplibregl, { MapLayerEventType, GeoJSONSource, AddLayerObject, Map, Popup } from 'maplibre-gl';
import MaplibreGeocoder from '@maplibre/maplibre-gl-geocoder';

let [zoom, lat, lng] = JSON.parse(window.localStorage.getItem("viewport")) || [
    16, 51.945356463918586, -0.0175273074135589,
];

export const map = new Map({
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

export function mapBounds() {
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

            new Popup()
                .setLngLat(e.lngLat)
                .setHTML(html)
                .addTo(map)
                .on('close', () => map.setFeatureState(
                    { source: 'OverpassAPI', id: f.id },
                    { selected: false }
                ));

            // highlight the current thing
            map.setFeatureState(
                { source: 'OverpassAPI', id: f.id },
                { selected: true }
            );
        };

        function openContextMenu(e: MapLayerEventType['contextmenu'] & Object) {
            const f = e.features[0];

            // TODO add [id, 'node|way|relation'] to a "visited" array
            // we dont have access to either of those things here sadly so idk if its doable

            map.setFeatureState(
                { source: 'OverpassAPI', id: f.id },
                { visited: !f.state.visited }
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

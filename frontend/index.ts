
import './style.css';
import 'maplibre-gl/dist/maplibre-gl.css';
import '@maplibre/maplibre-gl-geocoder/dist/maplibre-gl-geocoder.css';

import { mapBounds, setMapData, getMapData } from './map';
import { serializeGraph } from './graph/save';
import { settings } from './settings';
import './resizer';

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

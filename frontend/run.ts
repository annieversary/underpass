import { Feature } from 'geojson';

import { processedQueries } from './processed-queries';
import { SearchError, SearchSuccess, search } from './search';
import { setLoading, isLoading } from './loading';
import { mapBounds, setMapData } from './map';
import { serializeGraph } from './graph/save';
import { settings } from './settings';

let resultsDiv: HTMLDivElement = document.querySelector("#results") as HTMLDivElement;

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
            .filter((f: Feature) => !(f.geometry.type === "Point" && Object.keys(f.properties).length == 0));
    }

    setMapData('OverpassAPI', data);

    if (response.geocode_areas.length > 0) {
        const areas = response.geocode_areas.map((a: any) => `${a.original} - <a href="//www.openstreetmap.org/${a.ty}/${a.id}" target="_blank" class="osm-link">${a.name}</a><br/>`).join('');
        resultsDiv.innerHTML = `<h2>Geocode areas found:</h2>${areas}`;
    }

    if (response.processed_queries) {
        processedQueries.setAll(response.processed_queries);
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


document.querySelector<HTMLButtonElement>('#run-button')?.addEventListener('click', run);
document.addEventListener('keydown', (event) => {
    if ((event.ctrlKey || event.metaKey) && event.key == "Enter") {
        event.preventDefault();
        run();
    }
});

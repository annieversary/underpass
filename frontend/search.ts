import { MapBounds } from './map';
import { serializeGraph } from './graph/save';

export type SearchGraph = ReturnType<typeof serializeGraph>;
export type SearchResult = SearchSuccess | SearchError;

type SearchSuccess = {
    ok: 'true',
    data: any,
    processed_queries: {
        /// Node Id -> Processed query
        [nodeId: string]: string,
    },
    geocode_areas: any[]
};
type SearchError = {
    ok: 'false',
    /// Text representation of the error
    error: string,
    /// ID of the node that had an issue
    node_id: string | null,
    data: {
        format: "xml",
        message: string,
        query: string,
    } | {
        format: "text"
    }
};

export async function search(mapBounds: MapBounds, graph: SearchGraph): Promise<SearchResult> {
    const r = await fetch('/search', {
        method: 'POST',
        body: JSON.stringify({
            bbox: mapBounds,
            graph,
        }),
        headers: {
            'Content-Type': 'application/json'
        },
    });

    const data = await r.json();

    if (r.status === 200) {
        return {
            ok: 'true',
            ...data,
        };
    } else {
        return {
            ok: 'false',
            ...data,
        }
    }
}

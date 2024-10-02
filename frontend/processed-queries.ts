export type Queries = {
    [nodeId: string]: string,
}

class ProcessedQueries {
    private queries: Queries = {};

    public get(nodeId: string): string | null {
        return this.queries[nodeId] ?? null;
    }

    public setAll(queries: Queries) {
        this.queries = queries;
    }

    public delete(nodeId: string): void {
        delete this.queries[nodeId];
    }
}

export const processedQueries = new ProcessedQueries();

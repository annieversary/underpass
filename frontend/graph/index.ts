import { removeTab } from '../codeEditor/index';
import { openModal } from '../modal';
import { processedQueries } from '../processed-queries';

import { saveGraph, saveEvents } from './save';
import { Node, geojsonNodeList, queryNodeList, NodeList } from './nodes';
import { Control as ControlComponent } from './Control';
import { StyledNode } from './Node';
import { Socket } from './Socket';

import './style.css';

import { NodeEditor, GetSchemes, ClassicPreset, } from "rete";
import { createRoot } from "react-dom/client";
import { AreaPlugin, AreaExtensions } from "rete-area-plugin";
import { ReactPlugin, Presets, ReactArea2D } from "rete-react-plugin";
import { ConnectionPlugin, ClassicFlow, getSourceTarget } from "rete-connection-plugin"
import { ContextMenuExtra, ContextMenuPlugin } from "rete-context-menu-plugin";
import { ItemsCollection } from 'rete-context-menu-plugin/_types/types';

const container = document.querySelector<HTMLDivElement>('#graph-container');

export type Schemes = GetSchemes<
    Node,
    ClassicPreset.Connection<ClassicPreset.Node, ClassicPreset.Node>
>;

export const editor = new NodeEditor<Schemes>();

type AreaExtra = ReactArea2D<Schemes> | ContextMenuExtra;
export const area = new AreaPlugin<Schemes, AreaExtra>(container);
const render = new ReactPlugin<Schemes, AreaExtra>({ createRoot });
render.addPreset(Presets.classic.setup({
    customize: {
        control: () => ControlComponent,
        node: () => StyledNode,
        socket: () => Socket,
    }
}));
editor.use(area);
area.use(render);

const connection = new ConnectionPlugin<Schemes, AreaExtra>();
// connection.addPreset(ConnectionPresets.classic.setup())
connection.addPreset(() => new ClassicFlow({
    canMakeConnection(from, to) {
        const [source, target] = getSourceTarget(from, to) || [null, null];

        return source && target && (source as any).payload.name == (target as any).payload.name;
    },
    makeConnection(from, to, context) {
        const [source, target] = getSourceTarget(from, to) || [null, null];

        const { editor } = context;
        if (source && target) {
            editor.addConnection(
                new ClassicPreset.Connection(
                    editor.getNode(source.nodeId),
                    source.key,
                    editor.getNode(target.nodeId),
                    target.key
                ) as any
            );
            return true; // ensure that the connection has been successfully added
        }
    }
}));
area.use(connection);

export const nodeSelector = AreaExtensions.selectableNodes(area, AreaExtensions.selector(), {
    accumulating: {
        active: () => false,
    }
});

AreaExtensions.simpleNodesOrder(area);

// list addable nodes here
const contextMenu = new ContextMenuPlugin<Schemes>({
    items(context, _plugin): ItemsCollection {
        if (context === 'root') {
            const generators = (nodeList: NodeList) => nodeList.map(([label, factory], i) => {
                if (typeof factory != 'function') return;
                return {
                    label,
                    key: i.toString(),
                    async handler() {
                        const node = factory();
                        await editor.addNode(node)
                        area.translate(node.id, area.area.pointer)
                    }
                };
            });

            return {
                searchBar: true,
                list: [
                    {
                        label: 'GeoJson', key: '1', handler: () => null,
                        subitems: generators(geojsonNodeList)
                    },
                    {
                        label: 'Query', key: '2', handler: () => null,
                        subitems: generators(queryNodeList)
                    },
                    {
                        label: 'Tools', key: '3', handler: () => null,
                        subitems: [
                            {
                                label: 'Clear graph',
                                key: '1',
                                handler: async () => {
                                    const connections = editor.getConnections();
                                    for (const connection of connections) {
                                        await editor.removeConnection(connection.id);
                                    }
                                    const nodes = editor.getNodes();
                                    for (const n of nodes) {
                                        if (n.label == 'Map') continue;
                                        await editor.removeNode(n.id);
                                    }
                                }
                            }
                        ]
                    },
                ],
            };
        } else if ("label" in context) {
            if (context.label === 'Map') {
                return {
                    searchBar: false,
                    list: []
                };
            }

            const list = [
                {
                    label: 'Delete',
                    key: 'delete',
                    async handler() {
                        const nodeId = context.id
                        const connections = editor.getConnections().filter(c => {
                            return c.source === nodeId || c.target === nodeId
                        })

                        for (const connection of connections) {
                            await editor.removeConnection(connection.id)
                        }
                        await editor.removeNode(nodeId)
                    }
                },
            ];

            const nodeId = context.id;
            const query = processedQueries.get(nodeId);
            if (context.label == 'OQL Code' && query) {
                list.push({
                    label: 'View query',
                    key: 'view-query',
                    async handler() {

                        openModal(`<pre>${query}</pre>`);
                    },
                });
            }

            return {
                searchBar: false,
                list,
            }
        }

        return {
            searchBar: false,
            list: []
        }
    }
});

render.addPreset(Presets.contextMenu.setup({ delay: 0 }));
area.use(contextMenu);





export function zoomToNodes() {
    AreaExtensions.zoomAt(area, editor.getNodes());
}




// on node selected, select also that tab
area.addPipe(context => {
    // selecting an oql node also selects the corresponding code editor tab
    if (context.type === 'nodepicked') {
        const id = context.data.id;
        const tab = document.querySelector<HTMLDivElement>(`.tab[data-node-id="${id}"]`);
        if (tab) tab.onclick(new MouseEvent(''));
    }

    // delete tab and code editor when deleting oql nodes
    if (context.type == 'noderemoved') {
        if (context.data.label == 'OQL Code') {
            removeTab(context.data.id);
        }
    }

    if (saveEvents.includes(context.type)) {
        saveGraph();
    }

    return context
});

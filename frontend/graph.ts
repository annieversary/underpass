import { addTab, codeEditorMap, removeTab } from './codeEditor/index';
import { openModal } from './modal';
import { processedQueries } from './index';
import { nodeList, oqlNode, roadAngleFilter, map } from './graph-nodes';

import { Control as ControlComponent } from './Control';

import './graph.css';

import { NodeEditor, GetSchemes, ClassicPreset, getUID } from "rete";
import { createRoot } from "react-dom/client";
import { AreaPlugin, AreaExtensions } from "rete-area-plugin";
import { ReactPlugin, Presets, ReactArea2D } from "rete-react-plugin";
import { ConnectionPlugin, Presets as ConnectionPresets } from "rete-connection-plugin"
import { Control } from 'rete/_types/presets/classic';
import { ContextMenuPlugin } from "rete-context-menu-plugin";

const container = document.querySelector<HTMLDivElement>('#graph-container');

type Schemes = GetSchemes<
    ClassicPreset.Node,
    ClassicPreset.Connection<ClassicPreset.Node, ClassicPreset.Node>
>;

export const editor = new NodeEditor<Schemes>();

type AreaExtra = ReactArea2D<Schemes>;
export const area = new AreaPlugin<Schemes, AreaExtra>(container);
const render = new ReactPlugin<Schemes, AreaExtra>({ createRoot });
render.addPreset(Presets.classic.setup({
    customize: {
        control: (p) => ControlComponent
    }
}));
editor.use(area);
area.use(render);

const connection = new ConnectionPlugin<Schemes, AreaExtra>();
connection.addPreset(ConnectionPresets.classic.setup())
area.use(connection);

export const nodeSelector = AreaExtensions.selectableNodes(area, AreaExtensions.selector(), {
    accumulating: {
        active: () => false,
    }
});

AreaExtensions.simpleNodesOrder(area);

// list addable nodes here
const contextMenu = new ContextMenuPlugin<Schemes>({
    items(context, plugin) {
        if (context === 'root') {
            const nodeGenerators = nodeList.map(([label, factory], i) => {
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
                    ...nodeGenerators,
                    {
                        label: 'Tools', key: '1', handler: () => null,
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
            const query = processedQueries[nodeId];
            if (context.label == 'Overpass QL' && query) {
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

render.addPreset(Presets.contextMenu.setup());
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
        if (context.data.label == "Overpass QL") {
            removeTab(context.data.id);
        }
    }

    if (saveEvents.includes(context.type)) {
        saveGraph();
    }

    return context
});

export function serializeGraph() {
    const nodes: any[] = JSON.parse(JSON.stringify(editor.getNodes()));

    for (let i = 0; i < nodes.length; i++) {
        // set the position so we can use it when loading the graph from localStorage
        nodes[i].position = area.nodeViews.get(nodes[i].id).position;

        // add query as a control if this is an oql node
        if (nodes[i].label == "Overpass QL") {
            nodes[i].controls.query = {
                id: getUID(),
                value: codeEditorMap[nodes[i].id].state.doc.toString(),
            } as Control;
        }
    }

    return {
        nodes,
        connections: editor.getConnections(),
    };
}

/// Events after which to save the graph
const saveEvents = [
    'nodecreated',
    'noderemoved',
    'connectioncreated',
    'connectionremoved',
    'cleared',
    'nodetranslated',
    'noderesized',
    'nodepicked',
];

let loading = true;
export function saveGraph() {
    if (loading) return;
    const serialized = serializeGraph();
    window.localStorage.setItem('node-graph', JSON.stringify(serialized));
}

async function loadGraph() {
    let nodeGraph = window.localStorage.getItem('node-graph');
    // nodeGraph = null;
    if (nodeGraph == null) {
        await createDefaultGraph();
    } else {
        const data = JSON.parse(nodeGraph);

        const selectedNode = data.nodes.find(n => n.selected);
        const selectedIsOql = selectedNode?.label === 'Overpass QL';

        for (const { id, label, inputs, outputs, controls, position, selected } of data.nodes) {
            const node = new ClassicPreset.Node(label);
            node.id = id;
            Object.entries(inputs).forEach(([key, input]: [string, any]) => {
                const socket = new ClassicPreset.Socket(input.socket.name);
                const inp = new ClassicPreset.Input(socket, input.label);

                inp.id = input.id;

                node.addInput(key, input);
            });
            Object.entries(outputs).forEach(([key, output]: [string, any]) => {
                const socket = new ClassicPreset.Socket(output.socket.name);
                const out = new ClassicPreset.Output(socket, output.label);

                out.id = output.id;

                node.addOutput(key, out);
            });

            if (label == "Overpass QL") {
                const name = controls.name.value;
                const query = controls.query.value;
                const tab = addTab(node.id, name, selected || !selectedIsOql, () => {
                    nodeSelector.select(node.id, false);
                }, saveGraph, query);

                node.addControl("name", new ClassicPreset.InputControl("text", {
                    initial: name,
                    change(value) {
                        tab.innerHTML = `<p>${value}</p>`;
                        saveGraph();
                    }
                }));
            } else {
                Object.entries(controls).forEach(
                    ([key, control]) => {
                        if (!control) return;

                        const ctrl = new ClassicPreset.InputControl(control.type, {
                            initial: control.value,
                            readonly: control.readonly,
                            change() {
                                saveGraph();
                            }
                        });
                        node.addControl(key, ctrl);
                    }
                );
            }

            await editor.addNode(node);
            await area.translate(node.id, position);
            if (selected) {
                nodeSelector.select(node.id, false);
            }
        }

        for (const conn of data.connections) {
            await editor.addConnection(conn);
        }
    }

    setTimeout(zoomToNodes, 10);

    loading = false;
}
loadGraph();

async function createDefaultGraph() {
    const nodeA = oqlNode(true);
    const nodeB = roadAngleFilter();
    const nodeC = map();

    await editor.addNode(nodeA);
    await editor.addNode(nodeB);
    await editor.addNode(nodeC);

    await editor.addConnection(new ClassicPreset.Connection(nodeA, "out", nodeB, "in"));
    await editor.addConnection(new ClassicPreset.Connection(nodeB, "out", nodeC, "in"));

    await area.translate(nodeA.id, { x: 50, y: 100 });
    await area.translate(nodeB.id, { x: 280, y: 100 });
    await area.translate(nodeC.id, { x: 500, y: 100 });
}

import { addTab, codeEditorMap } from '../codeEditor/index';

import { editor, nodeSelector, area, zoomToNodes } from './index';
import { oqlNode, map } from './nodes';

import { ClassicPreset, getUID } from "rete";

type SerializedNode = {
    id: string;
    label: string;
    inputs: ClassicPreset.Node['inputs'];
    outputs: ClassicPreset.Node['outputs'];
    controls: {
        [key in string]?: {
            id: string;
            type: 'text' | 'number';
            value: string | number;
            readonly: boolean;
        }
    };
    position: {
        x: number;
        y: number;
    };
    selected: boolean;
};

export function serializeGraph(): {
    nodes: SerializedNode[],
    connections: ReturnType<typeof editor.getConnections>
} {
    const nodes: SerializedNode[] = JSON.parse(JSON.stringify(editor.getNodes()));

    for (let i = 0; i < nodes.length; i++) {
        // set the position so we can use it when loading the graph from localStorage
        nodes[i].position = area.nodeViews.get(nodes[i].id).position;

        // add query as a control if this is an oql node
        if (nodes[i].label == "Overpass QL") {
            nodes[i].controls.query = {
                id: getUID(),
                type: 'text',
                value: codeEditorMap[nodes[i].id].state.doc.toString(),
                readonly: true,
            };
        }
    }

    return {
        nodes,
        connections: editor.getConnections(),
    };
}

/// Events after which to save the graph
export const saveEvents = [
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
        const data: ReturnType<typeof serializeGraph> = JSON.parse(nodeGraph);

        const selectedNode = data.nodes.find((n: any) => n.selected);
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
                const name = controls.name.value as string;
                const query = controls.query.value as string;
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
                    ([key, control]: any) => {
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
    const codeNode = oqlNode(true);
    const mapNode = map();

    await editor.addNode(codeNode);
    await editor.addNode(mapNode);

    await editor.addConnection(new ClassicPreset.Connection(codeNode, "out", mapNode, "in"));

    await area.translate(codeNode.id, { x: 50, y: 100 });
    await area.translate(mapNode.id, { x: 500, y: 100 });
}

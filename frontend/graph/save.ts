import { addTab, codeEditorMap } from '../codeEditor/index';

import { editor, nodeSelector, area, zoomToNodes } from './index';
import { oqlNode, map, Control, ExtraProperties, Node, oqlCode } from './nodes';

import { ClassicPreset, getUID } from "rete";

const currentVersion = '2';

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
            properties: ExtraProperties<string | number>;
            label: string;
            tooltip: string;
        }
    };
    position: {
        x: number;
        y: number;
    };
    selected: boolean;
    type: Node["type"]
};

export function serializeGraph(): {
    nodes: SerializedNode[],
    connections: ReturnType<typeof editor.getConnections>
    version: string,
} {
    const nodes: SerializedNode[] = JSON.parse(JSON.stringify(editor.getNodes()));

    for (let i = 0; i < nodes.length; i++) {
        // set the position so we can use it when loading the graph from localStorage
        nodes[i].position = area.nodeViews.get(nodes[i].id).position;

        // add query as a control if this is an oql node
        if (nodes[i].label == "OQL Code") {
            nodes[i].controls.query = {
                id: getUID(),
                type: 'text',
                value: codeEditorMap[nodes[i].id].state.doc.toString(),
                readonly: true,
                properties: {},
                label: '',
                tooltip: '',
            };
        }
    }

    return {
        nodes,
        connections: editor.getConnections(),
        version: currentVersion,
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

    if (nodeGraph == null) {
        await createDefaultGraph();

        setTimeout(zoomToNodes, 10);
        loading = false;
        return;
    }

    const data: ReturnType<typeof serializeGraph> = JSON.parse(nodeGraph);

    if (data.version !== currentVersion) {
        await createDefaultGraph();

        setTimeout(zoomToNodes, 10);
        loading = false;
        return;
    }

    const selectedNode = data.nodes.find((n: any) => n.selected);
    const selectedIsOql = selectedNode?.label === 'OQL Code';

    for (const { id, label, inputs, outputs, controls, position, selected, type } of data.nodes) {
        const node = new ClassicPreset.Node(label) as Node;
        node.type = type;
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

        if (label == "OQL Code") {
            const name = controls.name.value as string;
            const query = controls.query.value as string;
            const tab = addTab(node.id, name, selected || !selectedIsOql, () => {
                nodeSelector.select(node.id, false);
            }, saveGraph, query);

            node.addControl("name", new Control("text", {
                initial: name,
                label: controls.name.label,
                tooltip: controls.name.tooltip,
                change(value) {
                    tab.innerHTML = `<p>${value}</p>`;
                }
            }));
        } else {
            Object.entries(controls).forEach(
                ([key, control]: any) => {
                    if (!control) return;

                    const ctrl = new Control(control.type, {
                        initial: control.value,
                        readonly: control.readonly,
                        properties: control.properties,
                        label: control.label,
                        tooltip: control.tooltip,
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

    setTimeout(zoomToNodes, 10);

    loading = false;
}
loadGraph();

async function createDefaultGraph() {
    const oql = oqlCode(true);
    const overpass = oqlNode();
    const mapNode = map();

    await editor.addNode(oql);
    await editor.addNode(overpass);
    await editor.addNode(mapNode);

    await editor.addConnection(new ClassicPreset.Connection(oql, "out", overpass, "query") as any);
    await editor.addConnection(new ClassicPreset.Connection(overpass, "out", mapNode, "in") as any);

    await area.translate(oql.id, { x: 50, y: 100 });
    await area.translate(overpass.id, { x: 300, y: 100 });
    await area.translate(mapNode.id, { x: 600, y: 100 });
}

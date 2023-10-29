import { addTab, codeEditorMap } from './codeEditor';

import './graph.css';

import { NodeEditor, GetSchemes, ClassicPreset, getUID } from "rete";
import { createRoot } from "react-dom/client";
import { AreaPlugin, AreaExtensions } from "rete-area-plugin";
import { ReactPlugin, Presets, ReactArea2D } from "rete-react-plugin";
import { ConnectionPlugin, Presets as ConnectionPresets } from "rete-connection-plugin"
import { Control } from 'rete/_types/presets/classic';

const container = document.querySelector<HTMLDivElement>('#graph-container');

type Schemes = GetSchemes<
    ClassicPreset.Node,
    ClassicPreset.Connection<ClassicPreset.Node, ClassicPreset.Node>
>;

export const editor = new NodeEditor<Schemes>();

type AreaExtra = ReactArea2D<Schemes>;
export const area = new AreaPlugin<Schemes, AreaExtra>(container);
const render = new ReactPlugin<Schemes, AreaExtra>({ createRoot });
render.addPreset(Presets.classic.setup());
editor.use(area);
area.use(render);

const connection = new ConnectionPlugin<Schemes, AreaExtra>();
connection.addPreset(ConnectionPresets.classic.setup())
area.use(connection);

const nodeSelector = AreaExtensions.selectableNodes(area, AreaExtensions.selector(), {
    accumulating: {
        active: () => false,
    }
});

AreaExtensions.simpleNodesOrder(area);



export function zoomToNodes() {
    AreaExtensions.zoomAt(area, editor.getNodes());
}



const socket = new ClassicPreset.Socket("socket");


// on node selected, select also that tab
area.addPipe(context => {
    if (context.type === 'nodepicked') {
        const id = context.data.id;
        const tab = document.querySelector<HTMLDivElement>(`.tab[data-node-id="${id}"]`);
        if (tab) tab.onclick(new MouseEvent(''));
    }

    if (saveEvents.includes(context.type)) {
        saveGraph();
    }

    // TODO serialize if it's not a mouse moving event

    return context
});

let codeBlockCount = 1;
async function oqlNode(selected: boolean): Promise<ClassicPreset.Node> {
    const nodeA = new ClassicPreset.Node("Oql");
    nodeA.addOutput("out", new ClassicPreset.Output(socket));
    await editor.addNode(nodeA);

    const name = `Code block ${codeBlockCount}`;
    const tab = addTab(nodeA.id, name, selected, () => {
        nodeSelector.select(nodeA.id, false);
    }, saveGraph);

    nodeA.addControl("name", new ClassicPreset.InputControl("text", {
        initial: name,
        change(value) {
            tab.innerHTML = `<p>${value}</p>`;
            saveGraph();
        }
    }));

    codeBlockCount++;

    return nodeA;
}

async function roadAngleFilter(): Promise<ClassicPreset.Node> {
    const nodeC = new ClassicPreset.Node("Road Angle Filter");
    nodeC.addInput("in", new ClassicPreset.Input(socket));
    nodeC.addOutput("out", new ClassicPreset.Output(socket));
    nodeC.addControl("min", new ClassicPreset.InputControl("number", {
        initial: 30.00,
        change() {
            saveGraph();
        }
    }));
    nodeC.addControl("max", new ClassicPreset.InputControl("number", {
        initial: 35.0,
        change() {
            saveGraph();
        }
    }));
    await editor.addNode(nodeC);
    return nodeC;
}

async function map(): Promise<ClassicPreset.Node> {
    const nodeB = new ClassicPreset.Node("Map");
    nodeB.addInput("in", new ClassicPreset.Input(socket));
    await editor.addNode(nodeB);
    return nodeB;
}


export function serializeGraph() {
    const nodes: any[] = editor.getNodes();

    for (let i = 0; i < nodes.length; i++) {
        // TODO add node position here
        nodes[i].position = area.nodeViews.get(nodes[i].id).position;

        // add query as a control if this is an oql node
        if (nodes[i].label == "Oql") {
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
function saveGraph() {
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
        const selectedIsOql = selectedNode?.label === 'Oql';

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

            if (label == "Oql") {
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

    loading = false;
}
loadGraph();

async function createDefaultGraph() {
    const nodeA = await oqlNode(true);
    const nodeOther = await oqlNode(false);

    const nodeC = await roadAngleFilter();

    const nodeB = await map();

    await editor.addConnection(new ClassicPreset.Connection(nodeA, "out", nodeC, "in"));
    await editor.addConnection(new ClassicPreset.Connection(nodeC, "out", nodeB, "in"));

    await area.translate(nodeOther.id, { x: 50, y: 300 });
    await area.translate(nodeA.id, { x: 50, y: 100 });
    await area.translate(nodeC.id, { x: 280, y: 100 });
    await area.translate(nodeB.id, { x: 500, y: 100 });
}

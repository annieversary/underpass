import { editor as codeEditor } from './codeEditor';

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

AreaExtensions.selectableNodes(area, AreaExtensions.selector(), {
    accumulating: AreaExtensions.accumulateOnCtrl()
});

AreaExtensions.simpleNodesOrder(area);




const socket = new ClassicPreset.Socket("socket");

async function setup() {
    const nodeA = new ClassicPreset.Node("Oql");
    nodeA.addOutput("out", new ClassicPreset.Output(socket));
    nodeA.addControl("name", new ClassicPreset.InputControl("text", {
        initial: "Code block 1"
    }));
    await editor.addNode(nodeA);

    const nodeOther = new ClassicPreset.Node("Oql");
    nodeOther.addOutput("out", new ClassicPreset.Output(socket));
    nodeOther.addControl("name", new ClassicPreset.InputControl("text", {
        initial: "Code block 2"
    }));
    await editor.addNode(nodeOther);

    const nodeC = new ClassicPreset.Node("Road Angle Filter");
    nodeC.addInput("in", new ClassicPreset.Input(socket));
    nodeC.addOutput("out", new ClassicPreset.Output(socket));
    nodeC.addControl("min", new ClassicPreset.InputControl("number", {
        initial: 30.00,
    }));
    nodeC.addControl("max", new ClassicPreset.InputControl("number", {
        initial: 35.0,
    }));
    await editor.addNode(nodeC);

    const nodeB = new ClassicPreset.Node("Map");
    nodeB.addInput("in", new ClassicPreset.Input(socket));
    await editor.addNode(nodeB);

    await editor.addConnection(new ClassicPreset.Connection(nodeA, "out", nodeC, "in"));
    await editor.addConnection(new ClassicPreset.Connection(nodeC, "out", nodeB, "in"));

    await area.translate(nodeOther.id, { x: 50, y: 200 });
    await area.translate(nodeA.id, { x: 50, y: 0 });
    await area.translate(nodeC.id, { x: 280, y: 0 });
    await area.translate(nodeB.id, { x: 500, y: 0 });
}
setup();

export function zoomToNodes() {
    AreaExtensions.zoomAt(area, editor.getNodes());
}

export function serializeGraph() {
    const nodes = editor.getNodes();

    for (let i = 0; i < nodes.length; i++) {
        if (nodes[i].label == "Oql") {
            nodes[i].controls.query = {
                id: getUID(),
                // TODO get the correct codeEditor code or whatever
                value: codeEditor.state.doc.toString(),
            } as Control;
        }
    }

    return {
        nodes,
        connections: editor.getConnections(),
    };
}

// TODO add an onchange thing that saves to localstorage
// TODO add something that reads from localstorage
// https://codesandbox.io/s/rete-js-v2-import-export-999y8z?file=/src/index.ts

// TODO add like buttons to insert nodes and stuff

// TODO make map node not deletable

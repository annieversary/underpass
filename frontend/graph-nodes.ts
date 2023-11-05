import { ClassicPreset, } from "rete";

import { addTab, } from './codeEditor/index';
import { nodeSelector, editor, saveGraph } from './graph';


export const nodeList = [
    ["Overpass QL", () => oqlNode(true)],
    ["Road Angle Filter", roadAngleFilter],
    ["Road Length Filter", roadLengthFilter],
];


const socket = new ClassicPreset.Socket("socket");

export function oqlNode(selected: boolean): ClassicPreset.Node {
    const nodeA = new ClassicPreset.Node("Overpass QL");
    nodeA.addOutput("out", new ClassicPreset.Output(socket));

    const codeBlockCount = editor.getNodes().filter(n => n.label == "Overpass QL").length + 1;

    const name = `OQL Block ${codeBlockCount}`;
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
    nodeA.addControl("timeout", new ClassicPreset.InputControl("number", {
        initial: 30,
        change() {
            saveGraph();
        }
    }));

    return nodeA;
}

export function roadAngleFilter(): ClassicPreset.Node {
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
    return nodeC;
}

export function roadLengthFilter(): ClassicPreset.Node {
    const nodeC = new ClassicPreset.Node("Road Length Filter");
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
    nodeC.addControl("tolerance", new ClassicPreset.InputControl("number", {
        initial: 10.0,
        change() {
            saveGraph();
        }
    }));
    return nodeC;
}

export function map(): ClassicPreset.Node {
    const nodeB = new ClassicPreset.Node("Map");
    nodeB.addInput("in", new ClassicPreset.Input(socket));
    return nodeB;
}

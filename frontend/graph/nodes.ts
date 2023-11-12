import { ClassicPreset, } from "rete";

import { addTab, } from '../codeEditor/index';
import { nodeSelector, editor, } from './index';
import { saveGraph } from './save';

/**
 * Input control options
 */
export type InputControlOptions<N> = {
    /** Whether the control is readonly. Default is `false` */
    readonly?: boolean,
    /** Initial value of the control */
    initial?: N,
    /** Callback function that is called when the control value changes */
    change?: (value: N) => void
    properties?: ExtraProperties<N>;

    label?: string;
    tooltip?: string;
}

export type ExtraProperties<N> = N extends number ? {
    min?: number;
    max?: number;
} : {};

/**
 * The input control class
 * @example new InputControl('text', { readonly: true, initial: 'hello' })
 */
export class Control<T extends 'text' | 'number', N = T extends 'text' ? string : number> extends ClassicPreset.InputControl<T, N> {
    properties: ExtraProperties<N>;
    label?: string;
    tooltip?: string;

    /**
     * @constructor
     * @param type Type of the control: `text` or `number`
     * @param options Control options
     */
    constructor(public type: T, public options?: InputControlOptions<N>) {
        let origChange = options?.change;

        options = options ?? {};
        options.change = (value: N) => {
            if (origChange) origChange(value);
            saveGraph();
        }

        super(type, options)
        this.properties = options.properties;
        this.label = options?.label;
        this.tooltip = options?.tooltip;
    }
}



export const nodeList: [key: string, factory: () => ClassicPreset.Node][] = [
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


    nodeA.addControl("name", new Control("text", {
        initial: name,
        label: 'name',
        tooltip: 'used to distinguish this Overpass QL block from others',
        change(value) {
            tab.innerHTML = `<p>${value}</p>`;
        }
    }));
    nodeA.addControl("timeout", new Control("number", {
        initial: 30,
        label: 'timeout',
        tooltip: 'timeout value to use for the Overpass API on this Overpass QL block',
        properties: {
            min: 0,
            max: 120,
        }
    }));

    return nodeA;
}

export function roadAngleFilter(): ClassicPreset.Node {
    const nodeC = new ClassicPreset.Node("Road Angle Filter");
    nodeC.addInput("in", new ClassicPreset.Input(socket));
    nodeC.addOutput("out", new ClassicPreset.Output(socket));
    nodeC.addControl("min", new Control("number", {
        initial: 30.00,
        label: 'min',
        properties: {
            min: -90.0,
            max: 90,
        }
    }));
    nodeC.addControl("max", new Control("number", {
        initial: 35.0,
        label: 'max',
        properties: {
            min: -90.0,
            max: 90,
        }
    }));
    return nodeC;
}

export function roadLengthFilter(): ClassicPreset.Node {
    const nodeC = new ClassicPreset.Node("Road Length Filter");
    nodeC.addInput("in", new ClassicPreset.Input(socket));
    nodeC.addOutput("out", new ClassicPreset.Output(socket));
    nodeC.addControl("min", new Control("number", {
        initial: 30.00,
        label: 'min',
        properties: {
            min: 0.0,
        }
    }));
    nodeC.addControl("max", new Control("number", {
        initial: 35.0,
        label: 'max',
        properties: {
            min: 0.0,
        }
    }));
    nodeC.addControl("tolerance", new Control("number", {
        initial: 10.0,
        properties: {
            min: 0.0,
        }
    }));
    return nodeC;
}

export function map(): ClassicPreset.Node {
    const nodeB = new ClassicPreset.Node("Map");
    nodeB.addInput("in", new ClassicPreset.Input(socket));
    return nodeB;
}

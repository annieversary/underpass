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


type ControlTypeMap = {
    text: string;
    number: number;
    checkbox: boolean;
}
export type ControlType = keyof ControlTypeMap;
export type ControlTypeValue<T extends ControlType> = ControlTypeMap[T];

/**
 * The input control class
 * @example new InputControl('text', { readonly: true, initial: 'hello' })
 */
export class Control<T extends ControlType, N = ControlTypeValue<T>> extends ClassicPreset.Control {
    properties: ExtraProperties<N>;
    label?: string;
    tooltip?: string;
    value?: N;
    readonly: boolean;

    constructor(public type: T, public options?: InputControlOptions<N>) {
        super()
        if (typeof options?.initial !== 'undefined') this.value = options.initial;
        this.readonly = options?.readonly;
        this.properties = options.properties;
        this.label = options?.label;
        this.tooltip = options?.tooltip;
    }

    setValue(value?: N) {
        this.value = value
        if (this.options?.change) this.options.change(value)
        saveGraph();
    }
}



// TODO make this an object, and make a separate thing that's the context menu
export const nodeList: [key: string, factory: () => Node][] = [
    ["Overpass", oqlNode],
    ["Union", union],
    ["Road Angle Filter", roadAngleFilter],
    ["Road Length Filter", roadLengthFilter],

    // TODO put this in separate category
    ["Filter", filter],
    ["OQL Code", () => oqlCode(true)],
];


const geojsonSocket = new ClassicPreset.Socket("geojson");
const querySocket = new ClassicPreset.Socket("query");

export type Node = ClassicPreset.Node & { type: "query" | "geojson" };

export function oqlNode(): Node {
    const node = new ClassicPreset.Node("Overpass") as Node;
    node.type = "geojson";

    node.addInput("query", new ClassicPreset.Input(querySocket));
    node.addOutput("out", new ClassicPreset.Output(geojsonSocket));

    node.addControl("timeout", new Control("number", {
        initial: 30,
        label: 'timeout',
        tooltip: 'timeout value to use for the Overpass API on this Overpass QL block',
        properties: {
            min: 0,
            max: 120,
        }
    }));

    return node;
}

export function roadAngleFilter(): Node {
    const node = new ClassicPreset.Node("Road Angle Filter") as Node;
    node.type = "geojson";
    node.addInput("in", new ClassicPreset.Input(geojsonSocket));
    node.addOutput("out", new ClassicPreset.Output(geojsonSocket));
    node.addControl("min", new Control("number", {
        initial: 30.00,
        label: 'min',
        properties: {
            min: -90.0,
            max: 90,
        }
    }));
    node.addControl("max", new Control("number", {
        initial: 35.0,
        label: 'max',
        properties: {
            min: -90.0,
            max: 90,
        }
    }));
    return node;
}

export function roadLengthFilter(): Node {
    const node = new ClassicPreset.Node("Road Length Filter") as Node;
    node.type = "geojson";
    node.addInput("in", new ClassicPreset.Input(geojsonSocket));
    node.addOutput("out", new ClassicPreset.Output(geojsonSocket));
    node.addControl("min", new Control("number", {
        initial: 30.00,
        label: 'min',
        properties: {
            min: 0.0,
        }
    }));
    node.addControl("max", new Control("number", {
        initial: 35.0,
        label: 'max',
        properties: {
            min: 0.0,
        }
    }));
    node.addControl("tolerance", new Control("number", {
        initial: 10.0,
        properties: {
            min: 0.0,
        }
    }));
    return node;
}

export function map(): Node {
    const node = new ClassicPreset.Node("Map") as Node;
    node.type = "geojson";
    node.addInput("in", new ClassicPreset.Input(geojsonSocket));
    return node;
}

export function union(): Node {
    const node = new ClassicPreset.Node("Union") as Node;
    node.type = "geojson";
    node.addInput("a", new ClassicPreset.Input(geojsonSocket));
    node.addInput("b", new ClassicPreset.Input(geojsonSocket));
    node.addOutput("out", new ClassicPreset.Output(geojsonSocket));
    return node;
}

export function oqlCode(selected: boolean): Node {
    const node = new ClassicPreset.Node('OQL Code') as Node;
    node.type = 'query';
    node.addOutput("out", new ClassicPreset.Output(querySocket));

    const codeBlockCount = editor.getNodes().filter(n => n.label == "Overpass QL").length + 1;

    const name = `OQL Block ${codeBlockCount}`;
    const tab = addTab(node.id, name, selected, () => {
        nodeSelector.select(node.id, false);
    }, saveGraph);

    node.addControl("name", new Control("text", {
        initial: name,
        label: 'name',
        tooltip: 'used to distinguish this Overpass QL block from others',
        change(value) {
            tab.innerHTML = `<p>${value}</p>`;
        }
    }));

    return node;
}

export function filter(): Node {
    const node = new ClassicPreset.Node('Osm Filter') as Node;
    node.type = 'query';
    node.addOutput("out", new ClassicPreset.Output(querySocket));

    node.addControl('nodes', new Control('checkbox', {
        initial: false,
        label: 'node',
    }));
    node.addControl('ways', new Control('checkbox', {
        initial: false,
        label: 'ways',
    }));
    node.addControl('relations', new Control('checkbox', {
        initial: false,
        label: 'relations',
    }));

    node.addControl('key', new Control('text', {
        initial: "highway",
        label: 'key',
    }));
    node.addControl('value', new Control('text', {
        initial: "primary",
        label: 'value',
    }));
    return node;
}

import { Schemes } from './index';
import { Node } from './nodes';

import * as React from 'react'
import { Presets, ReactArea2D } from "rete-react-plugin";
import { css } from "styled-components";

const nodeColor = (t: Node["type"]) => t == 'geojson' ? 'rgba(110,136,255,0.8)' : 'rgba(44, 191, 70, 0.8)';
const nodeBorderColor = (t: Node["type"]) => t == 'geojson' ? '#4e58bf' : '#3c8229';
const nodecolorselected = '#ffd92c'

const myStyles = (type: Node["type"]) => css<{ selected: boolean }>`
    background: ${nodeColor(type)};
    border: 2px solid ${nodeBorderColor(type)};
    &:hover {
        background: lighten(${nodeColor(type)},4%);
    }
    ${props => props.selected && css`
        background: ${nodecolorselected};
        border-color: #e3c000;
    `}
`;

export function StyledNode(props: {
    data: Schemes['Node'],
    emit: (props: ReactArea2D<Schemes>) => void;
}) {
    console.log(props.data.type);
    return <Presets.classic.Node styles={() => myStyles(props.data.type)} {...props} />;
}

import * as React from 'react'
import styled from 'styled-components'

import { Control as InputControl } from "./nodes";

const Label = styled.label`
    color: white;
`;
const Input = styled.input<{ styles?: (props: any) => any, style: any }>`
  width: 100%;
  border-radius: 30px;
  background-color: white;
  padding: 2px 6px;
  border: 1px solid #999;
  font-size: 110%;
  box-sizing: border-box;
  ${props => props.style}
  ${props => props.styles && props.styles(props)}
`;


export function Control<N extends 'text' | 'number'>(props: { data: InputControl<N>, styles?: () => any }) {
    const properties = props.data.options.properties;

    function isError(val: N extends 'text' ? string : number): boolean {
        if (properties) {
            if ("min" in properties && typeof val === 'number') {
                if (val < properties.min) return true;
            }
            if ("max" in properties && typeof val === 'number') {
                if (properties.max < val) return true;
            }
        }
        return false;
    }


    const [value, setValue] = React.useState(props.data.value)
    const ref = React.useRef(null)

    const [error, setError] = React.useState(isError(props.data.value));

    useNoDrag(ref)

    React.useEffect(() => {
        setValue(props.data.value)
    }, [props.data.value])


    return (
        <Label title={props.data.tooltip}>
            {props.data.label}
            <Input
                value={value}
                type={props.data.type}
                ref={ref}
                readOnly={props.data.readonly}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                    let val: typeof props.data['value'];
                    if (e.target.value !== "") {
                        val = (props.data.type === 'number'
                            ? +e.target.value
                            : e.target.value) as typeof props.data['value'];
                    }

                    setError(isError(val));

                    setValue(val)
                    props.data.setValue(val)
                }}
                style={error ? { background: "#f76464" } : {}}
                styles={props.styles}
                {...properties}
            />
        </Label>
    )
}

export function useNoDrag(ref: React.MutableRefObject<HTMLElement | null>, disabled?: boolean) {
    React.useEffect(() => {
        const handleClick = (e: PointerEvent) => {
            if (disabled) return

            const root = findReactRoot(e.target as HTMLElement)
            const target = React.version.startsWith('16') ? document : root

            if (target) {
                e.stopPropagation()
                target.dispatchEvent(copyEvent(e))
            }
        }
        const el = ref.current

        el?.addEventListener('pointerdown', handleClick)

        return () => {
            el?.removeEventListener('pointerdown', handleClick)
        }
    }, [ref, disabled])
}

function copyEvent<T extends Event & Record<string, any>>(e: T) {
    const newEvent = new (e.constructor as { new(type: string): T })(e.type)
    let current = newEvent

    while ((current = Object.getPrototypeOf(current))) {
        const keys = Object.getOwnPropertyNames(current)

        for (const k of keys) {
            const item = newEvent[k]

            if (typeof item === 'function') continue

            Object.defineProperty(newEvent, k, { value: e[k] })
        }
    }

    return newEvent
}

const rootPrefix = '__reactContainer$'

type Keys = `${typeof rootPrefix}${string}` | '_reactRootContainer';
type ReactNode = { [key in Keys]?: unknown } & HTMLElement

function findReactRoot(element: HTMLElement) {
    let current: ReactNode | null = element as ReactNode

    while (current) {
        if (current._reactRootContainer || Object.keys(current).some(key => key.startsWith(rootPrefix))) return current
        current = current.parentElement as ReactNode
    }
}

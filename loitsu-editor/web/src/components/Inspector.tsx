import { For, Match, Show, Switch, createEffect, createSignal } from 'solid-js';
import styles from './Inspector.module.css';
export default function Inspector() {
    const [entity, setEntity] = createSignal({} as object);
    // @ts-ignore
    window.select_entity = (entity: string) => {
        setEntity(JSON.parse(entity));
    };
    return (
        <div class={styles.inspector + " inspector"}>
            Inspector
            <Show when={entity()}>
                {/* @ts-ignore */ }
                <For each={entity().components}>
                {(item) => {
                    return <InspectorComponent component={item} />
                }}
                </For>
            </Show>
        </div>
    )
}

interface Component {
    name: string;
    id: string;
    properties: any;
}

function InspectorComponent(props: { component: Component }) {
    return (
    <div class={styles.inspector_component}>
        <span class={styles.inspector_component_name}>{props.component.name}</span>
        <For each={Object.keys(props.component.properties)}>
            {(key) => {
                return <InspectorInput label={key} value={props.component.properties[key]} />
            }}
        </For>
    </div>
    )
}

// @ts-ignore
function InspectorInput(props: { label: string, value: object }) {
    const [inputType, setInputType] = createSignal("String");
    createEffect(() => {
        setInputType(Object.keys(props.value)[0]);
    });
    return (
    <div class={styles.input}>
        <span>{props.label}</span>
        <Switch fallback={<span>Unknown</span>}>
            <Match when={inputType() == "String"}>
                {/* @ts-ignore */}
                <input value={props.value.String} />
            </Match>
        </Switch>
    </div>
    )
}

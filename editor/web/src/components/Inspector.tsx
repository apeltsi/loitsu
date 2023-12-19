import { For, Match, Show, Switch, createEffect, createSignal } from 'solid-js';
import styles from './Inspector.module.css';
import PanelTitle from './PanelTitle';
export default function Inspector() {
    const [entity, setEntity] = createSignal({} as object);
    // @ts-ignore
    window.select_entity = (entity: string) => {
        setEntity(JSON.parse(entity));
    };
    return (
        <div class={styles.inspector + " inspector"}>
            <PanelTitle title={"Inspector"}/>
            <Show when={entity()}>
                {/* @ts-ignore */ }
                <h3>{entity().name}</h3>
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
        <span class={styles.inspector_component_name}>{pretty_name(props.component.name)}</span>
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
        <span>{pretty_name(props.label)}</span>
        <Switch fallback={<span>Unknown</span>}>
            <Match when={inputType() == "String"}>
                {/* @ts-ignore */}
                <input value={props.value.String} />
            </Match>
        </Switch>
    </div>
    )
}

function pretty_name(name: string) {
    return name.replace(/_/g, " ").replace(/([A-Z])/g, ' $1').replace(/^./, (str) => str.toUpperCase());
}

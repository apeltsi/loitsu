import { For, Match, Show, Switch, createEffect, createSignal, onCleanup } from 'solid-js';
import styles from './Inspector.module.css';
import PanelTitle from './PanelTitle';
import { add_select_listener, remove_select_listener } from '..';

export default function Inspector() {
    const [entity, setEntity] = createSignal({} as object);
    const select_listener = (entity: any) => {
        setEntity(entity);
    };
    add_select_listener(select_listener);
    onCleanup(() => {
        remove_select_listener(select_listener);
    });
    return (
        <div class={styles.inspector + " inspector"}>
            <PanelTitle title={"Inspector"}/>
            <Show when={entity()}>
                { /* @ts-ignore */ }
                <h3>{entity().name}</h3>
                { /* @ts-ignore */ }
                <For each={entity().components}>
                {(item) => {
                    // @ts-ignore
                    return <InspectorComponent entity_id={entity().id} component={item} />
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

function InspectorComponent(props: { entity_id: string, component: Component }) {
    return (
    <div class={styles.inspector_component}>
        <div class={styles.inspector_component_names}> 
            <span class={styles.inspector_component_name}>{pretty_name(props.component.name)}</span>
            <span class={styles.inspector_component_true_name}>{props.component.name}</span>
        </div>
        <For each={Object.keys(props.component.properties)}>
            {(key) => {
                if (key.startsWith("_")) return;
                return <InspectorInput label={key} value={props.component.properties[key]} on_change={(value) => {
                    // @ts-ignore
                    window.set_component_property(props.entity_id, props.component.id, key, value);
                }} />
            }}
        </For>
    </div>
    )
}

// @ts-ignore
function InspectorInput(props: { label: string, value: object, on_change: (value: any) => void }) {
    const [inputType, setInputType] = createSignal("String");
    createEffect(() => {
        setInputType(Object.keys(props.value)[0]);
    });
    let ref: HTMLInputElement | undefined = undefined;
    return (
    <div class={styles.input}>
        <span>{pretty_name(props.label)}</span>
        <Switch fallback={<span>Unknown</span>}>
            <Match when={inputType() == "String"}>
                { /* @ts-ignore */}
                <input value={props.value.String} ref={ref} onChange={() => {props.on_change(ref.value)}}/>
            </Match>
        </Switch>
    </div>
    )
}

function pretty_name(name: string) {
    return name.replace(/_/g, " ").replace(/([A-Z])/g, ' $1').replace(/^./, (str) => str.toUpperCase());
}

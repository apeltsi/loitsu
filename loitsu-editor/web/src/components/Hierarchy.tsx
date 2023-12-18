import styles from './Hierarchy.module.css';
import { For, createSignal } from 'solid-js';

export default function Hierarchy() {
    let [hierarchy, set_hierarchy] = createSignal([]);
    // @ts-ignore
    window.set_hierarchy = (hierarchy) => {
        let hierarchy_tree = JSON.parse(hierarchy);
        set_hierarchy(hierarchy_tree);
        console.log(hierarchy_tree);
    };
    return (
        <div class={styles.hierarchy + " hierarchy"}>
            Hierarchy
            <div class={styles.hierarchyList}>
                <For each={hierarchy()}>
                {(item: any) => {
                    return (
                        <HierarchyItem item={item} />
                    )
                }}
                </For>
            </div>
        </div>
    )
}

function HierarchyItem(props: {item: any}) {
    // @ts-ignore
    return  <div class={styles.hierarchyItem} onClick={() => window.request_select_entity(props.item.id)}>
                {props.item.name}
            </div>
}

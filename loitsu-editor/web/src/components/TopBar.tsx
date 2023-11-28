import styles from './TopBar.module.css';
import { createSignal } from 'solid-js';
export default function TopBar() {
    const [sceneName, setSceneName] = createSignal('untitled');
    // @ts-ignore
    window.set_scene_name = (name) => setSceneName(name);
    return (
        <div class={styles.bar}>
            <span>loitsu editor</span>
            <span>{sceneName()}</span>
        </div>
    )
}

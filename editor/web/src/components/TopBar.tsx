import styles from './TopBar.module.css';
import { Show, createEffect, createSignal } from 'solid-js';
import spinner from "../assets/spinner.png";

export default function TopBar() {
    const [sceneName, setSceneName] = createSignal('untitled');
    const [tasks, setTasks] = createSignal<string[]>([]);
    const [tasksText, setTasksText] = createSignal('');
    // @ts-ignore
    window.set_scene_name = (name) => setSceneName(name);
    // @ts-ignore
    window.add_loading_task = (task: string) => {
        setTasks(tasks().concat(task));
    };
    // @ts-ignore
    window.remove_loading_task = (task: string) => {
        for (let i = 0; i < tasks().length; i++) {
            if (tasks()[i] === task) {
                let newTasks = tasks().slice();
                newTasks.splice(i, 1);
                setTasks(newTasks);
                return;
            }
        }
    };
    createEffect(() => {
        let text = '';
        if (tasks().length === 0) {
            setTasksText(text);
            return;
        }
        // if all of the tasks' names are the same, just show that with a number
        // if they're not, we'll just show the first one, plus the number of other tasks
        let firstTask = tasks()[0];
        let allSame = true;
        for (let i = 1; i < tasks().length; i++) {
            if (tasks()[i] !== firstTask) {
                allSame = false;
                break;
            }
        }
        if (allSame) {
            text = firstTask + ' (' + tasks().length + ')';
        } else {
            text = firstTask + ', and ' + (tasks().length - 1) + ' others';
        }
        setTasksText(text);
    })
    return (
        <div class={styles.bar}>
            <span>loitsu editor</span>
            <Show when={tasks().length > 0}>
                <div class={styles.tasks}>
                    <img src={spinner} class={styles.spinner} />
                    <span>{tasksText()}</span>
                </div>
            </Show>
            {/* @ts-ignore */}
            <button onClick={() => window.save_scene()}>Save</button>
            <span>{sceneName()}</span>
        </div>
    )
}

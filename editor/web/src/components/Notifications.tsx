import { For, createSignal, Show } from 'solid-js';
import styles from './Notifications.module.css';
export default function Notifications() {
    const [notifications, setNotifications] = createSignal<{type: number, title: string, text: string}[]>([]);
    // @ts-ignore
    window.add_notification = (type, title, text) => {
        setNotifications(notifications().concat({type, title, text}));
        setTimeout(() => {
            let newNotifications = notifications().slice();
            newNotifications.splice(0, 1);
            setNotifications(newNotifications);
        }, 10000);
    };
    return (<div class={styles.notificationContainer}>
        <For each={notifications()}>
            {(notification) => <Notification type={notification.type} title={notification.title} text={notification.text}/>}
        </For>
    </div>);
}

function Notification({ type, title, text }: { type: number, title: string, text: string }) {
    return (<div class={styles.notification + (type == 2 ? " " + styles.error : "")}>
        <Show when={title != ""}>
            <h3>{title}</h3>
        </Show>
        <span>{text}</span>
    </div>);
}

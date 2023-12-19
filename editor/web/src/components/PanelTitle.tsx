import styles from './PanelTitle.module.css';
import arrow from '../assets/arrow.png';

export default function PanelTitle(props: { title: string} ) {
    return <div class={styles.title}><img class={styles.arrow + " " + styles.rotated} draggable={false} src={arrow}/><h2>{props.title}</h2></div>;
}

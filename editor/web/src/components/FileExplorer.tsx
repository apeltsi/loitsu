import styles from './FileExplorer.module.css';
import PanelTitle from './PanelTitle';

export default function FileExplorer() {
    return (
        <div class={styles.explorer + " file-explorer"}>
            <PanelTitle title={"File Explorer"}/>
        </div>
    )
}

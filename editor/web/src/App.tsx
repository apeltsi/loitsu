import { createSignal } from 'solid-js';
import './App.css';
import FileExplorer from './components/FileExplorer';
import Hierarchy from './components/Hierarchy';
import Inspector from './components/Inspector';
import TopBar from './components/TopBar';
function App() {
    const [camera, setCamera] = createSignal("(0,0) x1");
    const [popOut, setPopOut] = createSignal(false);
    const [loading, setLoading] = createSignal(true);
    setTimeout(() => {
        setLoading(false);
    }, 1000);
    // @ts-ignore
    window.camera_moved = (x, y, zoom) => {
        x = Math.round(x * 100) / 100;
        y = Math.round(y * 100) / 100;
        zoom = Math.round(zoom * 100) / 100;
        setCamera(`(${x.toFixed(2)},${y.toFixed(2)}) x${zoom}`);
    };
    // pop out should activate on alt+enter
    
    add_key_listener((e: KeyboardEvent) => {
        if (e.altKey && e.key === "Enter") {
            setPopOut(!popOut());
        }
    });
    return (
    <>
        <div class={"splash" + (loading() ? "" : " done")}></div>
        <TopBar/>
        <div class={popOut() ? "" : "panel-shadow"}>
            <div class={"primary" + (popOut() ? " popout" : "")}>
                <Inspector/>
                <FileExplorer/>
                <Hierarchy/>
                <div class="overlays">
                    <span class="camera-state">{camera()}</span>
                </div>
            </div>
        </div>
    </>
    )
}
export let queued_events: any[] = [];
function add_key_listener(callback: (event: KeyboardEvent) => void) {
    window.addEventListener("keydown", (event) => {
        callback(event);
    });
    if (document.getElementsByTagName("canvas").length === 0) {
        queued_events.push(callback);
        return;
    }
    document.getElementsByTagName("canvas")[0].addEventListener("keydown", (event: KeyboardEvent) => {
        callback(event);
    });
}

export default App;

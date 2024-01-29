import { createSignal, onCleanup } from 'solid-js';
import './App.css';
import FileExplorer from './components/FileExplorer';
import Hierarchy from './components/Hierarchy';
import Inspector from './components/Inspector';
import TopBar from './components/TopBar';
import { Show } from 'solid-js/web';
import { add_select_listener, remove_select_listener } from '.';
import Notifications from './components/Notifications';

function App() {
    const [camera, setCamera] = createSignal("(0,0) x1");
    const [popOut, setPopOut] = createSignal(false);
    const [loading, setLoading] = createSignal(true);
    const [selectBounds, setSelectBounds] = createSignal(["0", "0", "0", "0"]);
    const [isMoving, setIsMoving] = createSignal<undefined | [number, number]>(undefined);
    let selected_entity = "";
    const [showSelection, setShowSelection] = createSignal(false);
    // @ts-ignore
    window.set_selected_bounds_pos = (x, y, width, height) => {
        setSelectBounds([(x - width / 2) * 100 + "vw", "calc(" + (y - height / 2) * 100 + "vh - 30px)", width * 100 + "vw", height * 100 + "vh"]);
    };

    const select_listener = (entity: any) => {
        if (entity.id !== selected_entity) {
            setShowSelection(false);
            selected_entity = entity.id;
            requestAnimationFrame(() => {
                setShowSelection(true);
            });
        }
    };
    add_select_listener(select_listener);
    const move_listener = (ev: MouseEvent) => {
        let moveVal = isMoving();
        if (moveVal) {
            const [startX, startY] = moveVal;
            const dx = ev.pageX / window.innerWidth - startX;
            const dy = ev.pageY / window.innerHeight - startY;
            // @ts-ignore
            window.move_selected(dx, -dy);
            setIsMoving([ev.pageX / window.innerWidth, ev.pageY / window.innerHeight]);
        }
    };
    const mouseup_listener = () => {
        setIsMoving(undefined);
    };
    window.addEventListener("mousemove", move_listener);
    window.addEventListener("mouseup", mouseup_listener);
    onCleanup(() => {
        remove_select_listener(select_listener);
        window.removeEventListener("mousemove", move_listener);
        window.removeEventListener("mouseup", mouseup_listener);
    });

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
                <Show when={showSelection()}>
                    <div draggable={false} class="select_bounds" style={{left: selectBounds()[0], top: selectBounds()[1], width: selectBounds()[2], height: selectBounds()[3]}}>
                        <div draggable={false} class="move_tool" onMouseDown={(ev: MouseEvent) => {
                            setIsMoving([ev.pageX / window.innerWidth, ev.pageY / window.innerHeight]);
                            ev.preventDefault();
                        }}/>
                    </div>
                </Show>
            </div>
            <Notifications/>
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

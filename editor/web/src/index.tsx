/* @refresh reload */
import { render } from 'solid-js/web';

import './index.css';
import App, { queued_events } from './App';
import init, {start_editor, override_asset_path, resize, request_select_entity, set_component_property} from '../public/wasm/loitsu-editor.js';

window.addEventListener("resize", () => resize());

function set_status(status: string) {
    console.log("Status reported: " + status);
}

async function run() {
	await init();
    if (window.location.search.includes("asset_path=")) {
        const asset_path = window.location.search.split("asset_path=")[1].split("&")[0];
        override_asset_path(asset_path);
    }
    console.log("Editor loaded.");
    start_editor();
    const interval = setInterval(() => {
        if (document.getElementsByTagName("canvas").length === 0) {
            return;
        }
        for (let i = 0; i < queued_events.length; i++) {
            document.getElementsByTagName("canvas")[0].addEventListener("keydown", (event: KeyboardEvent) => {
                queued_events[i](event);
            });
        }
        document.getElementsByTagName("canvas")[0].addEventListener("mousedown", (event: MouseEvent) => {
            if (event.button === 2) {
                document.getElementsByTagName("canvas")[0].classList.add("grabbing");
            }
        });
        document.getElementsByTagName("canvas")[0].addEventListener("mouseup", (event: MouseEvent) => {
            if (event.button === 2) {
                document.getElementsByTagName("canvas")[0].classList.remove("grabbing");
            }
        });
        clearInterval(interval);
    }, 50);
}
// @ts-ignore
if (window.set_status === undefined) {
    // @ts-ignore
    window.set_status = set_status;
    // @ts-ignore
    window.add_log = (prefix, color_hex, message) => {};
    // @ts-ignore
    window.add_warning = (message) => {};
    // @ts-ignore
    window.add_error = (message) => {};
    // @ts-ignore
    window.request_select_entity = request_select_entity;
    // @ts-ignore
    window.set_component_property = set_component_property;

    const root = document.getElementById('root');

    render(() => <App />, root!);
    run();
} else {
    window.location.reload();
}

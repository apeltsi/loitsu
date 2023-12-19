/* @refresh reload */
import { render } from 'solid-js/web';

import './index.css';
import App from './App';
import init, {start_editor, override_asset_path, resize, request_select_entity} from '../public/wasm/loitsu-editor.js';

window.addEventListener("resize", () => resize());

function set_status(status: string) {
    console.log(status);
}

async function run() {
	await init();
    if (window.location.search.includes("asset_path=")) {
        const asset_path = window.location.search.split("asset_path=")[1].split("&")[0];
        override_asset_path(asset_path);
    }
    console.log("Editor loaded.");
    start_editor();
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
    const root = document.getElementById('root');

    render(() => <App />, root!);
    run();
} else {
    window.location.reload();
}

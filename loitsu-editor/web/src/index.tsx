/* @refresh reload */
import { render } from 'solid-js/web';

import './index.css';
import App from './App';
import init, {resize, request_select_entity} from '../public/wasm/loitsu-editor.js';

window.addEventListener("resize", () => resize());

function set_status(status: string) {
    console.log(status);
}

async function run() {
	await init();
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

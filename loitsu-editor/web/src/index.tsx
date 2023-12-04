/* @refresh reload */
import { render } from 'solid-js/web';

import './index.css';
import App from './App';
import init, {resize} from '../public/wasm/loitsu-editor.js';

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
    run();
    const root = document.getElementById('root');

    render(() => <App />, root!);
} else {
    window.location.reload();
}

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
    run();
    const root = document.getElementById('root');

    render(() => <App />, root!);
} else {
    window.location.reload();
}

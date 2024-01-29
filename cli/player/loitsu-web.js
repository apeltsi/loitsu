import init, {resize} from "./{APP_NAME}.js";
const loadingText = document.getElementById("loadingText");
const log_container = document.getElementById("log_container");
export function set_status(status) {
    switch (status) {
        case 1:
            loadingText.innerHTML = "Starting renderer...";
            break;
        case 2:
            loadingText.innerHTML = "Finalizing...";
            break;
        case 3:
            loadingText.innerHTML = "Loading shards...";
            break;
        case 4:
            loadingText.innerHTML = "Enjoy!";
            document.getElementById("loading").classList.add("doneLoading");
            break;
    }
}

export function add_log(prefix, color_hex, message) {
    let log = document.createElement("p");
    log.innerHTML = `<span style="color: ${color_hex};">${prefix}</span> ${message}`;
    log_container.appendChild(log);
    setTimeout(() => {
        log_container.removeChild(log);
    }, 20000);
}

export function add_warning(message) {
    add_log("WARNING", "#FFA500", message);
}

export function add_error(message) {
    add_log("ERROR", "#FF0000", message);
}

window.addEventListener("resize", () => resize());

window.set_status = set_status;
window.add_log = add_log;
window.add_warning = add_warning;
window.add_error = add_error;

async function run() {
	await init();
}

run();

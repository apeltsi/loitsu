import init, {resize} from "./{APP_NAME}.js";
const loadingText = document.getElementById("loadingText");
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

window.addEventListener("resize", () => resize());

window.set_status = set_status;
async function run() {
	await init();
}
run();

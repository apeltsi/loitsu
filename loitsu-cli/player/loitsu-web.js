import init from "./{APP_NAME}.js";
const loadingText = document.getElementById("loadingText");
export function set_status(status) {
	if(status == 1) {
		loadingText.innerHTML = "Starting renderer...";
	}

	if(status == 2) {
		loadingText.innerHTML = "Finalizing...";
	}

	if(status == 3) {
		loadingText.innerHTML = "Enjoy!";
		document.getElementById("loading").classList.add("doneLoading");
	}
}

window.addEventListener('resize', function() {
});

window.set_status = set_status;
async function run() {
	await init();
}
run();

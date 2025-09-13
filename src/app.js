
var game;
var recorder;
var stream;
var link;

addEventListener("TrunkApplicationStarted", (event) => {
	beginWasmGame();
});

async function beginWasmGame() {
	let canvas = document.getElementById("flatland_canvas");
	link = document.getElementById("flatland_download");
	stream = canvas.captureStream(10);
	recorder = new MediaRecorder(stream, {
            mimeType: "video/webm; codecs=vp9"
        });
	recorder.start();

	recorder.ondataavailable = function(event) {
		let uri = URL.createObjectURL(event.data);
		link.download = "flatland.webm";
		link.href = uri;
	}

	let button = document.getElementById("flatland_restart");
	button.disabled = true;
	link.disabled = true;
	game = wasmBindings.WasmGame.new(0.25, 20, 5);
	game.render();
	// await new Promise(r => setTimeout(r, 1000));
	var WasmGameInterval = setInterval(function() {
		game.update_flatland();
		game.render();
		if (game.is_game_over()) {
			clearInterval(WasmGameInterval);
			button.disabled = false;
			link.disabled = false;
			recorder.stop();
		}
	}, 100);

}


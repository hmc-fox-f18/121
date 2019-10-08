//TODO: Adjust Constant Locations?

function sendPieceInfo() {
    let [x, y, rot] = pieces[playerNum];
    socket.send(`X: ${x} Y: ${y} Rotation: ${rot}`);
}

function initSocket() {
    socket = new WebSocket("ws://127.0.0.1:3012");

    socket.onopen = function(e) {
        socket.send("Started");
        socketOpen = true;
    };

    socket.onmessage = function(event) {
      alert(`[message] Data received from server: ${event.data}`);
    };

    socket.onclose = function(event) {
        socketOpen = false;
        if (event.wasClean) {
            alert(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
        } else {
            // e.g. server process killed or network down
            // event.code is usually 1006 in this case
            alert('[close] Connection died');
            }
        };

    socket.onerror = function(error) {
      alert(`[error] ${error.message}`);
    };
}

//TODO: Adjust Constant Locations?

function isMyPiece(piece) {
    return piece.player_id == player_id;
}

function getMyPiece() {
    // returns a piece that passes the isMyPiece check
    return gameState.piece_states.find(isMyPiece);
}

function sendPieceInfo(myUpdatedPiece) {
    socket.send(myUpdatedPiece);
}

function initSocket() {
    socket = new WebSocket("ws://127.0.0.1:3012");

    socket.onopen = function(e) {
        //socket.send("Started");
        console.log(e);
        socketOpen = true;
    };

    socket.onmessage = function(event) {
      console.log(event);
      alert(`[message] Data received from server: ${event.data}`);
      var game_state_raw = JSON.parse(event.data);
      gameState = Object.assign(new GameState, game_state_raw)
      console.log(gameState);
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

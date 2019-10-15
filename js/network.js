//TODO: Adjust Constant Locations?

function isMyPiece(piece) {
  return piece.player_id == player_id;
}

function getMyPiece() {
  return game_state.piece_states.find(isMyPiece);
}

function sendPieceInfo(myUpdatedPiece) {
    console.log(JSON.stringify(myUpdatedPiece));
    socket.send(JSON.stringify(myUpdatedPiece));
}

function initSocket() {
    socket = new WebSocket("ws://127.0.0.1:3012");

    socket.onopen = function(e) {
        socketOpen = true;
    };

    socket.onmessage = function(event) {
        let message = JSON.parse(event.data);
        if (message.type == 'init') {
            initializeFromServer(message)
        } else {

            var game_state_raw = JSON.parse(event.data);
            console.log(game_state_raw)
            gameState = Object.assign(new GameState, game_state_raw)
            //pieces = JSON.parse(event.data)["piece_states"];
        }
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

function initializeFromServer(message) {
    player_id = message.player_id;
    shape = message.piece_type;
    player_piece_state = new PieceState(shape, {x: 5, y: 5}, 0, player_id);
    game_state = new GameState([ player_piece_state ])
}

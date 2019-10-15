//TODO: Adjust Constant Locations?

function isMyPiece(piece) {
  console.log(piece)
  return piece.player_id == playerNum;
}

function getMyPiece() {
  return pieces.find(isMyPiece);
}

function sendPieceInfo(myUpdatedPiece) {
    socket.send(myUpdatedPiece);
}

// function sendPieceInfo() {
//     let [x, y, rot, shape_num] = getMyPiece().getNetworkInfo();
//     let message = {x: x, y: y, rotation: rot, shape: shape_num,
//          player_id: playerNum, type: "PieceState"};
//     socket.send(JSON.stringify(message));
// }

function initSocket() {
    socket = new WebSocket("ws://127.0.0.1:3012");

    socket.onopen = function(e) {
        socket.send("Started");
        socketOpen = true;
    };

    socket.onmessage = function(event) {
        let message = JSON.parse(event.data);
        if (message.type == 'init') {
            initializeFromServer(message)
        } else {
            console.log(event.data)
            var game_state_raw = JSON.parse(event.data);
            gameState = Object.assign(new GameState, game_state_raw)
            console.log(gameState)
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
    playerNum = message.player_id;
    shapeNum = message.piece_type;
    player_piece = shapes[shapeNum];
    player_piece.x = 5;
    player_piece.y = 5;
    pieces = [ player_piece ];
}

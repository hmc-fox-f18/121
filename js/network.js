//TODO: Adjust Constant Locations?

function isMyPiece(piece) {
  return piece.player_id == my_player_id;
}

function getMyPiece() {
  return game_state.piece_states.find(isMyPiece);
}


/*
@connectionCallback: function called game_state has been receive from server
*/
function initSocket(connectionCallback) {
    socket = new WebSocket("ws://127.0.0.1:3012");
    let made_callback = false;

    socket.onopen = function(e) {
        socketOpen = true;
    };

    socket.onmessage = function(event) {
        let message = JSON.parse(event.data);
        if (message.type == 'init') {
          my_player_id = message.player_id;
        } else {
          game_state = GameState.fromJson(event.data);

          if (!made_callback) {
            made_callback = true;
            connectionCallback();
          }
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

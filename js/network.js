/*jshint esversion: 6 */
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
    let hostname = location.hostname == "" ? "localhost" : location.hostname;

    let websocketAddress = `ws://${hostname}:3012`;
    console.log(`Connecting to WebSocket at: ${websocketAddress}`);

    socket = new WebSocket(websocketAddress);
    let made_callback = false;

    socket.onopen = function(e) {
        socketOpen = true;
    };

    socket.onmessage = function(event) {
        let message = JSON.parse(event.data);

        switch (message.type) {
          case 'init':
            my_player_id = message.player_id;
            break;

          case 'gameState':
            game_state = GameState.fromJson(event.data);

            if (!made_callback) {
              made_callback = true;
              connectionCallback();
            }
            break;

          case 'gameOver':
            gameOver = true;
            break;
          default:
            console.error(`Invalid message type ${message.type} received from server.`);
        }
    };

    socket.onclose = function(event) {
      socketOpen = false;
      // if (event.wasClean) {
      //     alert(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
      // } else {
      //   // e.g. server process killed or network down
      //   // event.code is usually 1006 in this case
      //   alert('[close] Connection died');
      // }
    };

    socket.onerror = function(error) {
      // alert(`[error] ${error.message}`);
    };
}

function sendInput(inputs) {
    let convertedArr = {};
    convertedArr.left = inputs.ArrowLeft || false;
    convertedArr.right = inputs.ArrowRight || false;
    convertedArr.counter_rot = inputs.ArrowUp || false;
    convertedArr.rot = inputs.z || false;
    convertedArr.hard_drop = false;
    convertedArr.fast_drop = false;
    convertedArr.player_id = my_player_id;
    let message = JSON.stringify(convertedArr);
    socket.send(message);
}

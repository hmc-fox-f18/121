let IBLOCK_HTML = `<table class='tetris-block i-block'>
   <tr>
     <td><div></td>
     <th><div></th>
      <th><div></th>
      <th><div></th>
   </tr>
</table>`;

let JBLOCK_HTML = `<table class='tetris-block j-block'>
   <tr>
     <th><div></th>
   </tr>
   <tr>
     <th><div></th>
     <td><div></td>
     <td><div></td>
   </tr>
</table>`;

let LBLOCK_HTML = `<table class='tetris-block l-block'>
   <tr>
     <th></th>
     <th></th>
     <th><div></th>
   </tr>
   <tr>
     <th><div></th>
     <td><div></td>
     <td><div></td>
   </tr>
</table>`;

let OBLOCK_HTML = `<table class='tetris-block o-block'>
   <tr>
     <th><div></th>
     <td><div></td>
   </tr>
   <tr>
     <th><div></th>
     <td><div></td>
   </tr>
</table>`;

let SBLOCK_HTML = `<table class='tetris-block s-block'>
   <tr>
     <th></th>
     <th><div></th>
     <td><div></td>
   </tr>
   <tr>
     <th><div></th>
     <td><div></td>
   </tr>
</table>`;

let TBLOCK_HTML = `<table class='tetris-block t-block'>
   <tr>
     <th></th>
     <th><div></th>
   </tr>
   <tr>
     <th><div></th>
     <td><div></td>
     <td><div></td>
   </tr>
</table>`;

let ZBLOCK_HTML = `<table class='tetris-block z-block'>
   <tr>
     <th><div></th>
     <th><div></th>
   </tr>
   <tr>
     <th></th>
     <td><div></td>
     <td><div></td>
   </tr>
</table>`;

function getPieceHTML(shape) {
  switch(shape) {
    case 0: return ZBLOCK_HTML;
    case 1: return SBLOCK_HTML;
    case 2: return JBLOCK_HTML;
    case 3: return LBLOCK_HTML;
    case 4: return TBLOCK_HTML;
    case 5: return IBLOCK_HTML;
    case 6: return OBLOCK_HTML;
  }
}

const MAX_ACTIVE_PIECES = 1;

// show the first three pieces
const PIECE_QUEUE_LENGTH = 3;

// returns -1 if player isn't in queue, else returns 0 or greater indicating
// position in queue
function getMyQueuePosition() {
  for (let i = 0; i < game_state.player_queue.length; i+=1) {
    if (game_state.player_queue[i] == my_player_id && i >= MAX_ACTIVE_PIECES) {
      return i - MAX_ACTIVE_PIECES;
    }
  }

  return -1;
}

// TODO: fill this in.
function getMyPieceShape() {
  return 2;
}

function drawMyPiece() {
  let queue_position = getMyQueuePosition();
  let my_piece_shape = getMyPieceShape();

  if (queue_position > PIECE_QUEUE_LENGTH) {
    $("#my-piece-shape").html(getPieceHTML(my_piece_shape))
    $("#my-piece-position").html(queue_position);
    $("#my-piece").removeClass("hidden");
  }
  else {
    $("#my-piece").addClass("hidden");
  }
}

function updateQueue() {


  console.log("update queue");
}

function initQueue() {

}

let IBLOCK_HTML = `<table class='tetris-block i-block'>
  <tr>
    <td><div></td>
    <td><div></td>
    <td><div></td>
    <td><div></td>
  </tr>
</table>`;

let JBLOCK_HTML = `<table class='tetris-block j-block'>
   <tr>
     <td><div></td>
   </tr>
   <tr>
     <td><div></td>
     <td><div></td>
     <td><div></td>
   </tr>
</table>`;

let LBLOCK_HTML = `<table class='tetris-block l-block'>
   <tr>
     <td></td>
     <td></td>
     <td><div></td>
   </tr>
   <tr>
     <td><div></td>
     <td><div></td>
     <td><div></td>
   </tr>
</table>`;

let OBLOCK_HTML = `<table class='tetris-block o-block'>
   <tr>
     <td><div></td>
     <td><div></td>
   </tr>
   <tr>
     <td><div></td>
     <td><div></td>
   </tr>
</table>`;

let SBLOCK_HTML = `<table class='tetris-block s-block'>
   <tr>
     <td></td>
     <td><div></td>
     <td><div></td>
   </tr>
   <tr>
     <td><div></td>
     <td><div></td>
   </tr>
</table>`;

let TBLOCK_HTML = `<table class='tetris-block t-block'>
   <tr>
     <td></td>
     <td><div></td>
   </tr>
   <tr>
     <td><div></td>
     <td><div></td>
     <td><div></td>
   </tr>
</table>`;

let ZBLOCK_HTML = `<table class='tetris-block z-block'>
   <tr>
     <td><div></td>
     <td><div></td>
   </tr>
   <tr>
     <td></td>
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

// how many pieces can be playing at one time
const MAX_ACTIVE_PIECES = 1;

// returns -1 if player isn't in queue, else returns 0 or greater indicating
// position in queue
function getMyQueuePosition() {
  for (let i = 0; i < game_state.player_queue.length; i+=1) {
    if (game_state.player_queue[i] == my_player_id) {
      return i;
    }
  }

  return -1;
}

// TODO: fill this in.
function getMyPieceShape() {
  for (let i = 0; i < game_state.player_queue.length; i+=1) {
    if (game_state.player_queue[i] == my_player_id) {
      return i;
    }
  }

  return -1;
}

function drawMyPiece() {
  let queue_position = getMyQueuePosition();

  console.log(queue_position);

  // if my piece is in the queue
  if (queue_position != -1) {
    $("#my-piece .shape").html(getPieceHTML(getMyPieceShape()));
    $("#my-piece .position").html("#" + queue_position);
  }
}

function drawBlockQueue() {
  let children = $("#block-queue").children();

  for (let i = 0; i < children.length; i++) {
    $(children[i]).find(".shape").html(getPieceHTML(game_state.piece_queue[i]));
  }
}

function updateQueue() {
  drawMyPiece();
  drawBlockQueue();

  console.log("update queue");
}

function initQueue() {

}

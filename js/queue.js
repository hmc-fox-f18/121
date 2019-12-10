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
      return game_state.piece_queue[i];
    }
  }

  return -1;
}

// draws the my-piece section which shows when my own
// piece will be put into the game
function drawMyPiece() {
  let display_position = getMyQueuePosition() + 1;

    // if my piece is in the queue
    switch (display_position) {
      case 0:
        $("#my-position div").html("You're playing!");
        break;
      case 1:
          $("#my-position div").html("<strong>You're next up!</strong>");
          break;
      default:
          let display_suffix = ((position) => {
            switch (display_position % 10) {
              case 1:
                return "st";
              case 2:
                return "nd";
              case 3:
                return "rd";
              default:
                return "th";
            }
          })(display_position);

          $("#my-position div").html(`You're ${display_position.toString() + display_suffix} up`);
          break;
    }
}

// draws the block queue which shows what the next three
// blocks will be
function drawBlockQueue() {
  let children = $("#block-queue").find(".shape");

  for (let i = 0; i < children.length; i++) {
    // get the ith child
    $(children).eq(i).html(getPieceHTML(game_state.piece_queue[i]));
  }
}

function updateQueue() {
  drawMyPiece();
  drawBlockQueue();
}

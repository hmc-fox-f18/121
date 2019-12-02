let READY_SIGNAL_DURATION = 500; // ms

// Actual Code
function clearBoard() {
    // white gridlines with black background
    ctx.fillStyle = bgColor;
    ctx.strokeStyle = "#FFFFFF";
    ctx.lineWidth   = 1;
    ctx.shadowBlur = 0;

    ctx.fillRect(0, 0, canvasWidth, canvasHeight);
    //Draw Gridlines
    ctx.stroke();
}

function drawShadowBlock(x, y) {
    ctx.fillStyle = "#444444FF";

    // Draw the blocks in the shape
    let posX = canvasWidth * x / BOARD_WIDTH;
    let posY = canvasHeight * y / BOARD_WIDTH;
    ctx.fillRect(posX, posY, blockWidth, blockHeight);
}

function drawBlock(x, y, color, glow) {
    ctx.fillStyle = color;

    // if piece is glowing, render with a shadow
    if (glow) {
        ctx.shadowColor = '#FFFF00ee';
        ctx.shadowBlur = 20;
    }
    else {
        ctx.shadowColor = '#00000000';
        ctx.shadowBlur = 0;
    }

    // Draw the blocks in the shape
    let posX = canvasWidth * x / BOARD_WIDTH;
    let posY = canvasHeight * y / BOARD_WIDTH;
    ctx.fillRect(posX, posY, blockWidth, blockHeight);

    // set the painting context again for no shadow
    ctx.shadowBlur = 0;
}

function drawPieces() {
    game_state.pieces.forEach((piece) => {
      let glow = piece.player_id == my_player_id;
      let min_y = 100;
      let cor_x = 100;
      // Draw the blocks in the shape
      piece.get_occupied_blocks((x, y) => {
          if (y < min_y) {
            min_y = y;
            cor_x = x;
          }
          drawBlock(x, y, piece.color, glow);
      });

      let nameX = canvasWidth * cor_x / BOARD_WIDTH;
      let nameY = canvasHeight * min_y / BOARD_WIDTH;

      ctx.fillStyle = "#FFFFFF";
      ctx.font = "bold 10pt Courier";
      ctx.fillText(piece.player_name, nameX, nameY)

  });
}

function drawFallenBlocks() {
  game_state.fallen_blocks.forEach((fallen_block) => {
    // TODO: this is a bit hacky
    let color = shapes[fallen_block.original_shape].color;

    // no glow on fallen blocks
    drawBlock(fallen_block.x, fallen_block.y, color, false);
  });
}

function drawMyPieceShadow() {
    let my_piece = getMyPiece();

    // sometimes we'll be in the queue and won't have a shadow to draw
    if (my_piece == undefined) return;

    // if for some reason my piece is currently colliding with something,
    // don't draw a shadow
    if (my_piece.collision()) return;

    // make a test piece and move it down until it hits something,
    // then back up by 1 block so we are no longer colliding
    let test_piece = my_piece.deepCopy();


    while (!test_piece.collision()) {
        test_piece.y += 1;
    }
    test_piece.y -= 1;


    // Draw the blocks in the shape
    test_piece.get_occupied_blocks((x, y) => {
        drawShadowBlock(x, y);
    });
}

var previously_playing = false;

function updateReadyMessage() {
    // if the player is not currently playing, they will not have a piece
    let currently_playing = (getMyPiece() != undefined);

    // if the player has just transitioned from not playing to playing,
    // give them a signal by making a shadow being the canvas flash green
    if (currently_playing && !previously_playing) {
        $("#board").addClass("emphasize");

        setTimeout(() => {
            // clearInterval(toggleInterval);
            $("#board").removeClass("emphasize");
        }, READY_SIGNAL_DURATION);
    }

    previously_playing = currently_playing;
}

function updateScore() {
  ctx.fillStyle = "#FFFFFF";
  ctx.font = "bold 20pt Courier";
  ctx.fillText(game_state.score.toString(), 10, 25)
}

function initGrid() {
    // Draw Vertical Gridlines
    for(let i = 0; i <= BOARD_WIDTH; ++i) {
        let posX = i * blockWidth;
        ctx.moveTo(posX, 0);
        ctx.lineTo(posX, canvasHeight);
    }
    // Draw Horizontal Gridlines
    for(let i = 0; i <= BOARD_WIDTH; ++i) {
        let posY = i * blockHeight;
        ctx.moveTo(0, posY);
        ctx.lineTo(canvasWidth, posY);
    }
}

function draw_frame() {
    clearBoard();
    drawMyPieceShadow(); // draw shadow before piece
    drawPieces();
    drawFallenBlocks();
    updateQueue();
    updateReadyMessage();
    updateScore();
}

//TODO: Adjust constant locations?

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
    ctx.fillStyle = "#999999FF";

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

      // Draw the blocks in the shape
      piece.get_occupied_blocks((x, y) => {
          drawBlock(x, y, piece.color, glow);
      });
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
    if (my_piece == undefined) {
        console.log("my piece is undefined");
        return;
    }

    // if for some reason my piece is currently colliding with something,
    // don't draw a shadow
    if (my_piece.collision()) {
        console.log("my piece collided with something");
        return;
    }

    console.log("ready to go.");


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
    drawPieces();
    drawMyPieceShadow();
    drawFallenBlocks();
    updateQueue();

    // Display Game Over if board has filled
    if (gameOver) {
      ctx.font = "30px Arial";
      ctx.fillStyle = "white";
      ctx.textAlign = "center";
      ctx.fillText("Game Over!", canvasWidth/2.0, canvasHeight/2.0);
    }
}

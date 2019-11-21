//TODO: Adjust constant locations?

// Actual Code
function clearBoard() {
    //
    //
    // console.log(`stroke style: ${ctx.strokeStyle}, ${ctx.lineWidth}`);

    // white gridlines with black background
    ctx.fillStyle = bgColor;
    ctx.strokeStyle = "#FFFFFF";
    ctx.lineWidth   = 1;
    ctx.shadowBlur = 0;

    ctx.fillRect(0, 0, canvasWidth, canvasHeight);
    //Draw Gridlines
    ctx.stroke();
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
  let posX = canvasWidth * x / boardWidth;
  let posY = canvasHeight * y / boardWidth;
  ctx.fillRect(posX, posY, blockWidth, blockHeight);

  // if (glow) {
  //   ctx.strokeStyle = "yellow";
  //   ctx.lineWidth   = 2;
  //   ctx.strokeRect(posX, posY, blockWidth, blockHeight);
  // }
}

function drawPieces() {
    game_state.piece_states.forEach((piece_state) => {

      let [x, y, color, width, rot_shape] = piece_state.getPiece().getRenderInfo();

      // Draw the blocks in the shape
      for (i in rot_shape) {
          if (rot_shape[i] == 1) {
              let x = (piece_state.pivot.x + i % width);
              let y = (piece_state.pivot.y + Math.floor(i / width));
              let glow = piece_state.player_id == my_player_id;

              drawBlock(x, y, color, glow);
          }
      }
    });
}

function drawFallenBlocks() {
  game_state.fallen_blocks.forEach((fallen_block) => {
    // TODO: this is a bit hacky
    let color = shapes[fallen_block.original_shape].color;

    // no glow on fallen blocks
    drawBlock(fallen_block.x, fallen_block.y, color, fallen_block.false);
  });
}

function initGrid() {
    // Draw Vertical Gridlines
    for(let i = 0; i <= boardWidth; ++i) {
        let posX = i * blockWidth;
        ctx.moveTo(posX, 0);
        ctx.lineTo(posX, canvasHeight);
    }
    // Draw Horizontal Gridlines
    for(let i = 0; i <= boardWidth; ++i) {
        let posY = i * blockHeight;
        ctx.moveTo(0, posY);
        ctx.lineTo(canvasWidth, posY);
    }
}

function draw_frame() {
    clearBoard();
    drawPieces();
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

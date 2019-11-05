//TODO: Adjust constant locations?

// Actual Code
function clearBoard() {
    // Clear Screen
    ctx.fillStyle = bgColor;
    ctx.fillRect(0, 0, canvasWidth, canvasHeight);
    //Draw Gridlines
    ctx.stroke();
}

function drawPieces() {
    game_state.piece_states.forEach((piece_state) => {
      let [x, y, color, width, rot_shape] = piece_state.getPiece().getRenderInfo();
      ctx.fillStyle = color;
      if (piece_state.player_id == my_player_id) {
        ctx.shadowColor = '#00ff00';
        ctx.shadowBlur = 40;
      }
      else {
        ctx.shadowColor = '#00000000';
        ctx.shadowBlur = 0;
      }
      // Draw the blocks in the shape
      for (i in rot_shape) {
          if (rot_shape[i] == 1) {
              let posX = canvasWidth * (piece_state.pivot.x + i % width) / boardWidth;
              let posY = canvasHeight * (piece_state.pivot.y + Math.floor(i / width) )
                              / boardWidth;
              ctx.fillRect(posX, posY, blockWidth, blockHeight);
          }
      }
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
}

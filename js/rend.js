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
    for (piece_state_num in gameState.piece_states) {
        let piece_state = gameState.piece_states[piece_state_num];
        let shape = shapes[piece_state.shape];
        let rot_shape = rotate_shape(piece_state.rotation, shape);
        let width = rot_shape.length == 9 ? 3 : 4;
        ctx.fillStyle = colors[piece_state.shape];
        // Draw the blocks in the shape
        for (i in rot_shape) {
            if (rot_shape[i] == 1) {
                let posX = canvasWidth * (piece_state.pivot.x + i % width) / boardWidth;
                let posY = canvasHeight * (piece_state.pivot.y + Math.floor(i / width) )
                                / boardWidth;
                if piece_state.player_id == playerNum {
                  ctx.shadowColor = '#00ff00';
                  ctx.shadowBlur = 40;
                }
                ctx.fillRect(posX, posY, blockWidth, blockHeight);
            }
        }
    }
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

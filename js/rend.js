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
    for (piece in pieces) {
        console.log(piece)
        let [x, y, color, width, rot_shape] = pieces[piece].getRenderInfo();
        ctx.fillStyle = color;
        // Draw the blocks in the shape
        for (i in rot_shape) {
            if (rot_shape[i] == 1) {
                let posX = canvasWidth * (x + i % width) / boardWidth;
                let posY = canvasHeight * (y + Math.floor(i / width) )
                                / boardWidth;
                if (piece.player_id == playerNum) {
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

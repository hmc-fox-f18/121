//TODO: Adjust constant locations?

var colors = ["#FF5B5B", "#3DE978", "#3D7AE9", "#FF894E", "#F27DFF",
                "#7DFFDC", "#FFDF92"]


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
        let [x, y, rot, shape, shape_num] = pieces[piece];
        let rot_shape = rotate_shape(rot, shape);
        let width = rot_shape.length == 9 ? 3 : 4;
        ctx.fillStyle = colors[shape_num];
        // Draw the blocks in the shape
        for (i in rot_shape) {
            if (rot_shape[i] == 1) {
                let posX = canvasWidth * (x + i % width) / boardWidth;
                let posY = canvasHeight * (y + Math.floor(i / width) )
                                / boardWidth;
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

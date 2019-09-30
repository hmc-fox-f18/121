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
        let [x, y, rot, shape] = pieces[piece];
        let rot_shape = rotate_shape(rot, shape);
        // Draw the blocks in the shape
        for (i in rot_shape) {
            if (rot_shape[i] == 1) {
                let posX = canvasWidth * (x + i % 3) / boardWidth;
                let posY = canvasHeight * (y + Math.floor(i / 3) )
                                / boardWidth;
                ctx.fillStyle = "red";
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

// Create new variables named in correct order, then return new array
function rotate_shape(rot, shape) {
    switch(rot) {
        case 1:
            return rotate_cw(shape);
            break;
        case 2:
            return rotate_180(shape);
            break;
        case 3:
            return rotate_ccw(shape);
            break;
        default:
            return shape;
            break;
    }
}

function rotate_ccw(shape) {
    let [b7, b4, b1, b8, b5, b2, b9, b6, b3] = shape;
    return [b1, b2, b3, b4, b5, b6, b7, b8, b9];
}

function rotate_cw(shape) {
    let [b3, b6, b9, b2, b5, b8, b1, b4, b7] = shape;
    return [b1, b2, b3, b4, b5, b6, b7, b8, b9];
}

function rotate_180(shape) {
    let [b9, b8, b7, b6, b5, b4, b3, b2, b1] = shape;
    return [b1, b2, b3, b4, b5, b6, b7, b8, b9];
}

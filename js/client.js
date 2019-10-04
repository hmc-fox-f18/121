//TODO: Consider using JS modules for imports

//TODO: Consider splitting game logic into separate code from base
// client?

// *******  Constants   ******
const bgColor= "black";
const strokeStyle = "white";

//The matrices representing the game pieces
const pieceZ = [ 1, 1, 0, 0, 1, 1, 0, 0, 0]; //0
const pieceS = [ 0, 1, 1, 1, 1, 0, 0, 0, 0]; //1
const pieceJ = [ 1, 0, 0, 1, 1, 1, 0, 0, 0]; //2
const pieceR = [ 0, 0, 1, 1, 1, 1, 0, 0, 0]; //3
const pieceT = [ 0, 1, 0, 1, 1, 1, 0, 0, 0]; //4
const pieceI = [ 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0]; //5
const pieceO = [ 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0]; //6
const shapes = [pieceZ, pieceS, pieceJ, pieceR, pieceT, pieceI, pieceO];

// ******   Variables   ********
//TODO: Consider removing global variables or moving relevant parts to
//other components
var boardWidth;
var boardHeight;

var canvasWidth;
var canvasHeight;

var blockHeight;
var blockWidth;

var board;
var ctx;
var pieces;

var playerNum;
var keystate = [];
var socket;
var socketOpen = false;

// Actual Code

/**
 *  Initializes the client state and all game logic variables
 */
function init() {
    //TODO: Receive board specifications from server
    boardWidth = 20;
    boardHeight = 20;
    board = Array(boardWidth).fill(Array(boardHeight));

    canvas = document.getElementById('board');
    ctx = canvas.getContext('2d');
    ctx.strokeStyle = strokeStyle;

    canvasWidth = canvas.width;
    canvasHeight = canvas.height;

    blockHeight = canvasHeight / boardHeight;
    blockWidth = canvasWidth / boardWidth;

    //Initialize network and rendering components
    //TODO: delegate to relevant components
    initSocket();
    initGrid();
    clearBoard();
    drawPieces();

    // Test code
    //TODO: Change piece code to use numeric representation
    shape_num = Math.floor(Math.random() * shapes.length);
    pieces = [ [5, 5, 0, shapes[shape_num], shape_num] ];
    playerNum = 0;

    window.addEventListener('keydown', (e) => {
        keystate[e.key] = true
        e.preventDefault();
    });

    window.requestAnimationFrame(handleFrame)
}

/**
 *  Delegates the logic for each frame and makes calls to the network
 *  and rendering logic to draw the frame and send data to the server
 */
function handleFrame() {
    updatePosition();
    if (socketOpen) {
        sendPieceInfo();
    }
    keystate = [];
    draw_frame();
    window.requestAnimationFrame(handleFrame)
}

/**
 *  Uses the input buffer for the current frame to adjust player
 *  position and rotation.
 */
function updatePosition() {
    //TODO: Consider abstracting indices to allow rebindable keys#FF5B5B#FF5B5B#FF5B5B#FF5B5B#3DE978#3DE978#3DE978
    // Move left
    if (keystate["ArrowLeft"]) {
        pieces[playerNum][0] -= 1;
        if (collision(pieces[playerNum])) {
            pieces[playerNum][0] += 1;
        }
    }
    // Move right
    if (keystate["ArrowRight"]) {
        pieces[playerNum][0] += 1;
        if (collision(pieces[playerNum])) {
            pieces[playerNum][0] -= 1;
        }
    }
    // Rotate clockwise
    if (keystate["ArrowUp"]) {
        wallkick(pieces[playerNum], true);
    }
    // Rotate counter-clockwise
    if (keystate["z"]) {
        wallkick(pieces[playerNum], false);
    }
}

/**
 *
 * Takes in a piece and whether the rotation is clockwise as an input,
 * and tries to perform a series of wallkicks to determine what the
 * resulting position from this rotation should be. If no wall kicks are
 * valid positions, the function will return the original position and
 * rotation.
 *
 */
function wallkick(piece, clockwise) {
    let rot = piece[2];
    if (clockwise) {
        piece[2] = (piece[2] + 1) % 4;
    }
    else {
        piece[2] = piece[2] - 1;
        piece[2] = piece[2] == -1 ? 3 : piece[2];
    }

    // No Change, Test 1
    if (!collision(piece)) {
        return piece;
    }
    // Test 2
    let x = piece[0];
    let y = piece[1];
    let x_test = (piece[2] == 2 || piece[2] == 3) ? 1 : -1;
    x_test = clockwise ? x_test : -1 * x_test;
    let y_test = 0;
    piece[0] = x + x_test;
    piece[1] = y + y_test;
    if (!collision(piece)) {
        return piece;
    }
    // Test 3
    y_test = (piece[2] == 1 || piece[2] == 3) ? 1 : -1;
    piece[1] = y + y_test;
    if (!collision(piece)) {
        return piece;
    }
    // Test 4
    piece[0] = x;
    piece[1] = y + 2 * y_test;
    if (!collision(piece)) {
        return piece;
    }
    // Test 5
    piece[0] = x + x_test;
    if (!collision(piece)) {
        return piece;
    }
    // All failed, return to original
    piece[0] = x;
    piece[1] = y;
    piece[2] = rot;
    return piece;
    //TODO: Implement I block wall kicks
}

/**
 *
 * Takes in a piece, and checks for any collisions with other game
 * elements which would invalidate the pieces position.
 *
 * Returns true if there is a collision.
 */
function collision(piece) {
    let rot_shape = rotate_shape(piece[2], piece[3]);
    let x = piece[0];
    let y = piece[1];
    //Determine whether the block is in a 3x3 or 4x4 bounding box
    let bound_width = rot_shape.length == 9 ? 3 : 4;
    for (block in rot_shape) {
        if (rot_shape[block] == 1) {
            if (x + (block % bound_width) < 0) {
                return true;
            }
            else if (x + (block % bound_width) >= boardWidth) {
                return true;
            }
        }
    }
    //TODO: Check for collision with other pieces and floor
    return false;
}

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

//TODO: Better rotation method
/**
 *
 * Takes in a piece array and returns a new one rotated
 * counter-clockwise
 *
 */
function rotate_ccw(shape) {
    if (shape.length == 9) {
        let [b7, b4, b1, b8, b5, b2, b9, b6, b3] = shape;
        return [b1, b2, b3, b4, b5, b6, b7, b8, b9];
    }
    else {
        let [b4, b8, b12, b16, b3, b7, b11, b15, b2, b6, b10, b14, b1, b5, b9, b13] = shape;
        return [b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16];
    }
}

/**
 *
 * Takes in a piece array and returns a new one rotated
 * clockwise
 *
 */
function rotate_cw(shape) {
    if (shape.length == 9) {
        let [b3, b6, b9, b2, b5, b8, b1, b4, b7] = shape;
        return [b1, b2, b3, b4, b5, b6, b7, b8, b9];
    }
    else {
        let [b13, b9, b5, b1, b14, b10, b6, b2, b15, b11, b7, b3, b16, b12, b8, b4] = shape;
        return [b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16];
    }
}

/**
 *
 * Takes in a piece array and returns a new one rotated
 * 180 degrees
 *
 */
function rotate_180(shape) {
    if (shape.length == 9) {
        let [b9, b8, b7, b6, b5, b4, b3, b2, b1] = shape;
        return [b1, b2, b3, b4, b5, b6, b7, b8, b9];
    }
    else {
        let [b16, b15, b14, b13, b12, b11, b10, b9, b8, b7, b6, b5, b4, b3, b2, b1] = shape;
        return [b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16];
    }
}

//TODO: Consider using JS modules for imports

//TODO: Consider splitting game logic into separate code from base
// client?

// *******  Constants   ******
const bgColor= "black";
const strokeStyle = "white";

//The matrices representing the game pieces
const pieceZ = [ 1, 1, 0, 0, 1, 1, 0, 0, 0]; //0
const pieceS = [ 0, 1, 1, 1, 1, 0, 0, 0, 0]; //1
const pieceL = [ 1, 1, 0, 0, 1, 0, 0, 1, 0]; //2
const pieceR = [ 0, 1, 1, 0, 1, 0, 0, 1, 0]; //3
const pieceT = [ 0, 1, 0, 1, 1, 1, 0, 0, 0]; //4
const pieceI = [ 1, 1, 0, 0, 1, 1, 0, 0, 0]; //5
const pieceO = [ 1, 1, 0, 0, 1, 1, 0, 0, 0]; //6

// ******   Variables   ********
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
    pieces = [ [5, 5, 0, pieceL]];
    playerNum = 0;

    window.addEventListener('keydown', (e) => {
        keystate[e.key] = true
        e.preventDefault();
    });

    window.requestAnimationFrame(handle_frame)
}

/**
 *  Delegates the logic for each frame and makes calls to the network
 *  and rendering logic to draw the frame and send data to the server
 */
function handle_frame() {
    updatePosition();
    if (socketOpen) {
        sendPieceInfo();
    }
    keystate = [];
    draw_frame();
    window.requestAnimationFrame(handle_frame)
}

/**
 *  Uses the input buffer for the current frame to adjust player
 *  position and rotation.
 */
function updatePosition() {
    if (keystate["ArrowLeft"]) {
        pieces[playerNum][0] -= 1;
    }
    if (keystate["ArrowRight"]) {
        pieces[playerNum][0] += 1;
    }
    if (keystate["ArrowUp"]) {
        pieces[playerNum][2] = (pieces[playerNum][2] + 1) % 4;
    }
    if (keystate["z"]) {
        pieces[playerNum][2] = (pieces[playerNum][2] - 1);
        if (pieces[playerNum][2] == -1) {
            pieces[playerNum][2] = 3;
        }
    }
}

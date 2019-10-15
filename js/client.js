//TODO: Consider using JS modules for imports

//TODO: Consider splitting game logic into separate code from base
// client?

// *******  Constants   ******
const bgColor= "black";
const strokeStyle = "white";

//The matrices representing the game pieces
const pieceZ = new Piece([ 1, 1, 0, 0, 1, 1, 0, 0, 0], 0,  "#FF5B5B", 0, 0, 0, 3, -1); //0
const pieceS = new Piece([ 0, 1, 1, 1, 1, 0, 0, 0, 0], 1, "#3DE978", 0, 0, 0, 3, -1); //1
const pieceJ = new Piece([ 1, 0, 0, 1, 1, 1, 0, 0, 0], 2, "#3D7AE9", 0, 0, 0, 3, -1); //2
const pieceR = new Piece([ 0, 0, 1, 1, 1, 1, 0, 0, 0], 3, "#FF894E", 0, 0, 0, 3, -1); //3
const pieceT = new Piece([ 0, 1, 0, 1, 1, 1, 0, 0, 0], 4, "#F27DFF", 0, 0, 0, 3, -1); //4
const pieceI = new Piece([ 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0], 5, "#7DFFDC", 0, 0, 0, 4, -1); //5
const pieceO = new Piece([ 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0], 6, "#FFDF92", 0, 0, 0, 4, -1); //6
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
var pieceNum;
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
    if (playerNum != undefined) {
        updatePosition();
        keystate = [];
        draw_frame();
    }
    window.requestAnimationFrame(handleFrame)
}

/**
 *  Uses the input buffer for the current frame to adjust player
 *  position and rotation.
 */
function updatePosition() {
    // Use JSON to deep copy our current piece
    // we want to do this so we do not end up updating the client without the server's permission
    var myUpdatedPiece = JSON.parse(JSON.stringify(getMyPiece()));

    //TODO: Consider abstracting indices to allow rebindable keys#FF5B5B#FF5B5B#FF5B5B#FF5B5B#3DE978#3DE978#3DE978
    // Move left
    if (keystate["ArrowLeft"]) {
        myUpdatedPiece.x -= 1;
        if (myUpdatedPiece.collision()) {
            myUpdatedPiece.x += 1;
        }
    }
    // Move right
    if (keystate["ArrowRight"]) {
        myUpdatedPiece.x += 1;
        if (myUpdatedPiece.collision()) {
            myUpdatedPiece.x -= 1;
        }
    }
    // Rotate clockwise
    if (keystate["ArrowUp"]) {
        myUpdatedPiece.wallkick(true);
    }
    // Rotate counter-clockwise
    if (keystate["z"]) {
        myUpdatedPiece.wallkick(false);
    }

    // send piece info on every position update
    if (socketOpen) {
        sendPieceInfo(myUpdatedPiece);
    }
}

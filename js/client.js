//TODO: Consider using JS modules for imports

//TODO: Consider splitting game logic into separate code from base
// client?

// *******  Constants   ******
const bgColor= "black";
const strokeStyle = "white";

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

var player_id;
var pieceNum;
var keystate = [];
var socket;
var socketOpen = false;

var game_state = new GameState([]);

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

    // Test code
    //TODO: Change piece code to use numeric representation
    shape_num = Math.floor(Math.random() * shapes.length);
    gameState = new GameState([ new PieceState(shape_num, { x: 5, y: 5}, 1, 1) ]);
    player_id = 1;

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
    if (player_id != undefined) {
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
    var myPiece = getMyPiece();
    // if we do not have a piece yet, do not take input
    if (!myPiece) {
      return
    }
    // Use Prototype to deep copy our current piece
    // we want to do this so we do not end up updating the client without the server's permission
    var myUpdatedPiece = Object.assign( Object.create( Object.getPrototypeOf(myPiece)), myPiece);

    //TODO: Consider abstracting indices to allow rebindable keys#FF5B5B#FF5B5B#FF5B5B#FF5B5B#3DE978#3DE978#3DE978
    // Move left
    if (keystate["ArrowLeft"]) {
        myUpdatedPiece.pivot.x -= 1;
        if (myUpdatedPiece.collision()) {
            myUpdatedPiece.pivot.x += 1;
        }
    }
    // Move right
    if (keystate["ArrowRight"]) {
        myUpdatedPiece.pivot.x += 1;
        if (myUpdatedPiece.getPiece().collision()) {
            myUpdatedPiece.pivot.x -= 1;
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

    // send piece info on every position update
    if (socketOpen) {
        sendPieceInfo(myUpdatedPiece);
    }
}

// Constants

const bgColor= "black";
const strokeStyle = "white";

const pieceZ = [ 1, 1, 0, 0, 1, 1, 0, 0, 0];

//Variables
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
function clearBoard() {
    // Clear Screen
    ctx.fillStyle = bgColor;
    ctx.fillRect(0, 0, canvasWidth, canvasHeight);
    //Draw Gridlines
    ctx.stroke();
}

function drawPieces() {
    for (piece in pieces) {
        let [x, y, shape] = pieces[piece];
        // Draw the blocks in the shape
        for (i in shape) {
            if (shape[i] == 1) {
                let posX = canvasWidth * (x + i % 3) / boardWidth;
                let posY = canvasHeight * (y + Math.floor(i / 3) )
                                / boardWidth;
                ctx.fillStyle = "red";
                ctx.fillRect(posX, posY, blockWidth, blockHeight);
            }
        }
    }
}

function init() {

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

    pieces = [ [5, 5, pieceZ, 0, "Z"], [] ];

    initSocket();
    initGrid();
    clearBoard();
    drawPieces();

    playerNum = 0;

    window.addEventListener('keydown', (e) => {
        keystate[e.key] = true
        e.preventDefault();
    });

    window.requestAnimationFrame(drawFrame)
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

function drawFrame() {
    updatePosition();
    if (socketOpen) {
        sendPieceInfo();
    }
    keystate = [];
    clearBoard();
    drawPieces();
    window.requestAnimationFrame(drawFrame)
}

function updatePosition() {
    if (keystate["ArrowLeft"]) {
        pieces[playerNum][0] -= 1;
    }
    if (keystate["ArrowRight"]) {
        pieces[playerNum][0] += 1;
    }
    if (keystate["ArrowUp"]) {
        pieces[playerNum][2] = rotateCW(pieces[playerNum][2]);
        pieces[playerNum][3] = (pieces[playerNum][3] + 1) % 4;
    }
    if (keystate["z"]) {
        pieces[playerNum][2] = rotateCCW(pieces[playerNum][2]);
        pieces[playerNum][3] = (pieces[playerNum][3] - 1) % 4;
        if (pieces[playerNum][3] == -1) {
            pieces[playerNum][3] = 3;
        }
    }
}

// Create new variables named in correct order, then return new array
function rotateCW(shape) {
    let [b7, b4, b1, b8, b5, b2, b9, b6, b3] = shape;
    return [b1, b2, b3, b4, b5, b6, b7, b8, b9];
}

function rotateCCW(shape) {
    let [b3, b6, b9, b2, b5, b8, b1, b4, b7] = shape;
    return [b1, b2, b3, b4, b5, b6, b7, b8, b9];
}

// Experimental networking stuffs

function sendPieceInfo() {
    let [x, y, , rot, piece] = pieces[playerNum];
    socket.send(`X: ${x} Y: ${y} Rotation: ${rot} Piece: ${piece}`);
}

function initSocket() {
    socket = new WebSocket("ws://127.0.0.1:3012");

    socket.onopen = function(e) {
        socket.send("Started");
        socketOpen = true;
    };

    socket.onmessage = function(event) {
      alert(`[message] Data received from server: ${event.data}`);
    };

    socket.onclose = function(event) {
        socketOpen = false;
        if (event.wasClean) {
            alert(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
        } else {
            // e.g. server process killed or network down
            // event.code is usually 1006 in this case
            alert('[close] Connection died');
            }
        };

    socket.onerror = function(error) {
      alert(`[error] ${error.message}`);
    };
}

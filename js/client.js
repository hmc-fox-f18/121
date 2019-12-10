/*jshint esversion: 6 */
//TODO: Consider using JS modules for imports

//TODO: Consider splitting game logic into separate code from base
// client?

// *******  Constants   ******
const bgColor= "black";
const strokeStyle = "white";

// ******   Variables   ********
//TODO: Consider removing global variables or moving relevant parts to
//other components
let BOARD_WIDTH = 20;
let BOARD_HEIGHT = 20;

// let GAME_START_DELAY = 1000;

var name = 'Guest';

var canvasWidth;
var canvasHeight;

var blockHeight;
var blockWidth;

var board;
var ctx;

var my_player_id;

var socket;
var socketOpen = false;

var gameOver = false;

var game_state = new GameState([], [], [], [], 0);
// Actual Code

$(document).ready(() => {
    $("#splash .splash-blocks").append(`<div>${IBLOCK_HTML}</div>`);
    $("#splash .splash-blocks").append(`<div>${JBLOCK_HTML}</div>`);
    $("#splash .splash-blocks").append(`<div>${LBLOCK_HTML}</div>`);
    $("#splash .splash-blocks").append(`<div>${OBLOCK_HTML}</div>`);
    $("#splash .splash-blocks").append(`<div>${SBLOCK_HTML}</div>`);
    $("#splash .splash-blocks").append(`<div>${TBLOCK_HTML}</div>`);
    $("#splash .splash-blocks").append(`<div>${ZBLOCK_HTML}</div>`);
});

/**
 *  Initializes the client state and all game logic variables
 */
function play() {
    $("#splash").fadeOut(200);
    name = $("#name-textbox").val() || 'Guest';

    init();
}

function init() {
    //TODO: Receive board specifications from server

    board = Array(BOARD_WIDTH).fill(Array(BOARD_HEIGHT));

    canvas = document.getElementById('board');
    ctx = canvas.getContext('2d');
    ctx.strokeStyle = strokeStyle;

    canvasWidth = canvas.width;
    canvasHeight = canvas.height;

    blockHeight = canvasHeight / BOARD_HEIGHT;
    blockWidth = canvasWidth / BOARD_WIDTH;

    initKeypressHandler();

    setTimeout(() => {
        //Initialize network and rendering components
        //TODO: delegate to relevant components
        initSocket(() => {
            initGrid();
            clearBoard();
            window.requestAnimationFrame(handleFrame);
        });
    });
}

/**
 *  Delegates the logic for each frame and makes calls to the network
 *  and rendering logic to draw the frame and send data to the server
 */
function handleFrame() {
    updatePosition();
    draw_frame();

    // only paint another frame if the game isn't over
    if (!gameOver) {
        window.requestAnimationFrame(handleFrame);
    } else {
        $('#restart-modal').show();
    }
}

/**
 *  Uses the input buffer for the current frame to adjust player
 *  position and rotation.
 */
function updatePosition() {
    var myPiece = getMyPiece();
    // if we do not have a piece yet, do not take input
    if (!myPiece) {
      return;
    }

    // send update piece position to the server
    // getKeypresses gets whichever pieces have been pressed since the last
    // call of this function
    if (socketOpen) {
        sendInput(getKeypresses());
    }
}

function restart_game() {
    location.reload();
}

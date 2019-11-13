extern crate ws;
extern crate rand;
extern crate slab;

use std::collections::HashMap;
mod piece_state;
mod input;
mod tetris;
mod tests;

use crate::piece_state::{PieceState, Pivot, BlockState};
use crate::input::{KeyState};
use crate::tetris::{update_state, fallen_blocks_collision, read_block, get_shape};

use std::time::{SystemTime, UNIX_EPOCH};

use rand::thread_rng;
use rand::prelude::SliceRandom;
use std::sync::{Arc, Mutex};
use std::{time, thread};
use std::collections::VecDeque;
use std::cmp::min;

use ws::{CloseCode, Handler, Handshake, Message, Result,
     Sender, WebSocket, util::Token, util::Timeout};

use serde_json::json;

const FRAME_MILLIS : u64 = (1000.0 / 60.0) as u64;
const FRAME_TIME : time::Duration = time::Duration::from_millis(FRAME_MILLIS);

const TIMEOUT_MILLIS : u64 = 10000;

const NUM_BAGS : usize = 3;
const BAG_SIZE : usize = 14;
const MAX_NUM_ACTIVE : usize = 2;
// how long it takes between when pieces move down 1 square
const SHIFT_PERIOD_MILLIS : u128 = 1000;

const PIECE_START_X : i8 = 5;
const PIECE_START_Y : i8 = 5;


/**
 *
 * The representation of an individual client
 *
 * TODO: Implement saving data frames for rollback?
 *
 * TODO: Split client into separate module for code clarity?
 */
struct Client<'a> {
    out: Sender,
    player_queue: &'a Mutex<VecDeque<PieceState>>,
    inactive_queue: &'a Mutex<VecDeque<PieceState>>,
    block_queue: &'a Mutex<[ [u8 ; 14] ; NUM_BAGS ]>,
    block_index: &'a Mutex<usize>,
    fallen_blocks: &'a Mutex<HashMap<Pivot, u8>>,
    timeout: Option<Timeout>,
}

impl Handler for Client<'_> {
    /**
     *
     * Function called when a connection is opened with a client
     *
     * Clients are added to the shared players Slab, and the initial
     * state is messaged back to the client.
     *
     * TODO: Consider breaking new vs. returning client to different
     * helper methods
     *
     */
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("Request: {}", shake.request);
        let player_id : usize = self.out.token().into();
        let response;

        // Player doesn't exist, add to players list
        // TODO: Genericize initial piece state
        let mut block_queue = self.block_queue.lock().unwrap();
        let mut block_index = self.block_index.lock().unwrap();

        let piece_type: u8 = next_piece(&mut block_queue, &mut block_index);

        let new_piece_state = PieceState {
            shape: piece_type,
            pivot: Pivot {
                x: PIECE_START_X,
                y: PIECE_START_Y,
            },
            rotation: 0,
            player_id: player_id
        };

        // Insert player into back of inactive queue
        let mut inactive_queue = self.inactive_queue.lock().unwrap();
        inactive_queue.push_back(new_piece_state);
        drop(inactive_queue);

        response = json!({
            "player_id": player_id,
            "piece_type": piece_type,
            "type": "init"
        });

        // setup ping every second
        self.out.timeout(TIMEOUT_MILLIS, self.out.token()).unwrap();

        self.out.send(response.to_string())
    }

    //TODO: Deal with different messages if applicable
    fn on_message(&mut self, msg: Message) -> Result<()> {

        match self.out.timeout(TIMEOUT_MILLIS, self.out.token()) {
            Ok(_) => {},
            Err(e) => println!("Error registering new timeout: {}", e)
        };

        // Parse the msg as text
        if let Ok(text) = msg.into_text() {
            // Try to parse the message as a piece state
            match serde_json::from_str::<KeyState>(&text) {
                Ok(mut player_input) => {
                    let mut players_queue = self.player_queue.lock().unwrap();
                    let fallen_blocks = self.fallen_blocks.lock().unwrap();

                    // Don't trust input, ensure labelled properly
                    let player_id : usize = self.out.token().into();
                    player_input.player_id = player_id;
                    // Update state for player
                    update_state(&mut players_queue, &player_input, &fallen_blocks);
                    return Ok(());
                }
                Err(e) => {
                    // Piece state is not valid
                    println!("Could not parse status: {}\n", e);
                    return Ok(());
                },
            }
        }
        // default to blank result if message is not parseable
        return Ok(());
    }

    /**
     *
     * Method invoked when a client ceases to be connected
     * to the server.
     *
     * Sets a timeout to remove a client
     *
     * TODO: Add more complex behavior for a more seamless tetris game
     *
     */
    fn on_close(&mut self, code: CloseCode, _reason: &str) {
        // Print reason for connection loss
        let player_id : usize = self.out.token().into();
        match code {
            CloseCode::Normal => println!("Client {} is done with the connection.", player_id),
            CloseCode::Away => println!("Client {} is leaving the site.", player_id),
            _ => println!("Client {} encountered an error: {:?}", player_id, code),
        }

        let mut players = self.player_queue.lock().unwrap();
        let mut inactive_players = self.inactive_queue.lock().unwrap();
        remove_player(player_id, &mut *players, &mut *inactive_players);
    }

    /**
     *
     *  Method invoked when a client times out.
     *
     *  Logs the disconnection, then proceeds to remove the player
     *  from the game state.
     *
     */
    fn on_timeout(&mut self, _event: Token) -> Result<()> {
        // close the connection, send Error close code because we shouldn't
        // hit a timeout unless the server dies
        // this will trigger on_close which will remove the player
        match self.out.ping(vec![]) {
            Ok(()) => self.out.timeout(TIMEOUT_MILLIS, self.out.token()).unwrap(),
            _ => self.out.close(CloseCode::Error).unwrap(),
        }
        // Note: timeouts will actually occur if the client refreshes
        // the page
        Ok(())
    }

    /**
     *
     *  Code called when a new timeout event is created.
     *
     *  Should be usable to cancel previous timeouts as data is
     *  received from the client
     *
     *  //TODO: Make this actually work properly
     *
     */
    fn on_new_timeout(&mut self, _event: Token, timeout: Timeout) -> Result<()> {
        // take() transfers ownership of the underlying data stored in self.timeout
        if let Some(t) = self.timeout.take() {
            // if cancel is successful, set we don't have a timeout until
            // on_new_timeout is called
            // if cancel fails, the old timeout is still active
            match self.out.cancel(t) {
                Ok(_) => self.timeout = None,
                Err(_) => {},
            };
        }

        self.timeout = Some(timeout);
        return Ok(());
    }
}

/**
 *
 *  Function which removes a given player from the player slab.
 *  This removes the player from the entire game, not just the
 *  board.
 *
 */
fn remove_player(player_id: usize,
                 players: &mut VecDeque<PieceState>,
                 inactive_players: &mut VecDeque<PieceState>) {

    let mut active_remove_index = None;
    let mut inactive_remove_index = None;

    // find where in queue the player lives
    for (index, active_player) in players.iter_mut().enumerate() {
        if active_player.player_id == player_id {
            active_remove_index = Some(index);
        }
    }
    for (index, inactive_player) in inactive_players.iter_mut().enumerate() {
        if inactive_player.player_id == player_id {
            inactive_remove_index = Some(index);
        }
    }

    // remove the player from queue
    match active_remove_index {
        Some(index) => { players.remove(index); },
        None => { },
    };
    match inactive_remove_index {
        Some(index) => { inactive_players.remove(index); },
        None => { },
    };
}

/**
 *
 *  Removes a player from the active queue and puts their piece in the inactive queue.
 *
 */
fn move_to_inactive(player_id : usize, players: &mut VecDeque<PieceState>, inactive_players: &mut VecDeque<PieceState>) {
    let mut index = 0;
    loop {
        // Let crash if not in queue for now.
        let player = players.get(index).unwrap();
        if player.player_id == player_id {
            // Remove player then push them to the back of the queue
            let removed_piece = players.remove(index).unwrap();
            inactive_players.push_back(removed_piece);
            break;
        }
        index += 1;
    };
}

/**
 *
 *  Generates the next piece to be output
 *
 *  TODO: Implement Tetris bag generation for better distribution
 *
 */
pub fn next_piece(block_queue: &mut [[u8 ; 14] ; NUM_BAGS ],
                  stored_index: &mut usize) -> u8 {

    let mut rng = thread_rng();
    let index = *stored_index;
    let next_piece = block_queue[index / 14][index % 14];

    // if we just used all of a bag, shuffle it so its good
    // for next time
    if index % 14 == 13 {
        block_queue[index / 14].shuffle(&mut rng);
    }
    *stored_index = (index + 1) % (14 * NUM_BAGS);
    return next_piece;
}

/*
Gets the next fourteen pieces that will be put into play and returns them as a Vec.
*/
pub fn peek_next_pieces(block_queue: &[ [u8 ; 14] ; NUM_BAGS ],
                   block_index: usize) -> Vec<u8> {

    let mut next_bag = Vec::new();

    for global_index in block_index..(block_index + BAG_SIZE) {
        let bag = (global_index / BAG_SIZE) % NUM_BAGS;
        let index = global_index % BAG_SIZE;

        next_bag.push(block_queue[bag][index]);
    }

    return next_bag;
}


fn millis_since_epoch() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    return since_the_epoch.as_millis();
}

fn add_fallen_blocks(piece : &PieceState, fallen_blocks : &mut HashMap<Pivot, u8>) {
    let this_shape = get_shape(piece.shape);
    let width = if this_shape.len() == 9 {3} else {4};
    let this_origin = piece.pivot;

    // iterate through all of the blocks that make up the
    // current piece and add them to fallen_blocks.
    for y in 0..width {
        for x in 0..width {
            let abs_x = x + this_origin.x;
            let abs_y = y + this_origin.y;

            if read_block(this_shape, x, y, piece.rotation) {
                let pivot = Pivot {
                    x: abs_x,
                    y: abs_y,
                };

                fallen_blocks.insert(pivot, piece.shape);
            }
        }
    }
}

fn shift_pieces(players : &mut VecDeque<PieceState>, inactive_players : &mut VecDeque<PieceState>, fallen_blocks : &mut HashMap<Pivot, u8>) {
    let mut player_ids_to_remove : Vec<usize> = vec![];

    for mut player in players.iter_mut() {
        // make a copy which we shift down and check for collision
        let mut player_copy = player.clone();
        player_copy.pivot.y += 1;

        // If piece is off of the screen, remove it from play
        // We do this later, not in the iterator, since removing
        // elements while iterating is not safe.

        if fallen_blocks_collision(&player_copy, fallen_blocks) {
            add_fallen_blocks(player, fallen_blocks);

            // let t = json!({"fallen_blocks": fallen_blocks});
            // println!("{}", t);

            player_ids_to_remove.push(player.player_id);
        } else {
            player.pivot.y += 1;
        }
    }

    // actually remove players from the board
    for player_id in player_ids_to_remove {
        move_to_inactive(player_id, players, inactive_players);
    }
}

// activates exactly one piece !
fn activate_piece(players : &mut VecDeque<PieceState>,
                  inactive_players : &mut VecDeque<PieceState>,
                  block_queue : &mut [ [u8 ; 14] ; NUM_BAGS ],
                  block_index : &mut usize) {

    // if we have more pieces in play and there are inactive pieces in the queue
    if MAX_NUM_ACTIVE - players.len() > 0 && inactive_players.len() > 0 {
        let mut player = inactive_players.pop_front().unwrap();

        // get the new piece type
        let piece_type: u8 = next_piece(block_queue,
                                        block_index);

        // update the player's piece type
        player.shape = piece_type;

        // update the player's position so it starts in the middle of the screen
        player.pivot.x = PIECE_START_X;
        player.pivot.y = PIECE_START_Y;

        players.push_front(player);
    }
}

/**
 *
 *  Runs the actual game logic at regular intervals, then sends out a
 *  state update to all the clients.
 *
 */
fn game_frame<'a>(broadcaster: Sender,
                  thread_block_queue: Arc<Mutex<[ [u8 ; 14] ; NUM_BAGS ]>>,
                  thread_block_index: Arc<Mutex<usize>>,
                  thread_player_queue: Arc<Mutex<VecDeque<PieceState>>>,
                  thread_inactive_queue: Arc<Mutex<VecDeque<PieceState>>>,
                  thread_fallen_blocks : Arc<Mutex<HashMap<Pivot, u8>>>) {

    // the time when we last shifted the pieces down
    let mut last_shift_time : u128 = 0;

    loop {
        let mut player_queue = thread_player_queue.lock().unwrap();
        let mut inactive_player_queue = thread_inactive_queue.lock().unwrap();
        let mut fallen_blocks = thread_fallen_blocks.lock().unwrap();
        let mut block_queue = thread_block_queue.lock().unwrap();
        let mut block_index = thread_block_index.lock().unwrap();

        // drop the pieces 1 square if they need to be dropped
        let current_time = millis_since_epoch();
        if current_time - last_shift_time > SHIFT_PERIOD_MILLIS {

            // check to make sure shift works
            shift_pieces(&mut player_queue, &mut inactive_player_queue, &mut fallen_blocks);

            // actives a single piece
            activate_piece(&mut player_queue, &mut inactive_player_queue, &mut block_queue, &mut block_index);
            last_shift_time = current_time;
        }

        let fallen_blocks_list : Vec<BlockState> = fallen_blocks.iter().map(|(pivot, shape)| {
            return BlockState {
                position: pivot.clone(),
                original_shape: *shape,
            };
        }).collect();

        // Get the active players from the front of the deque
        let states : Vec<&PieceState> = player_queue.iter().collect();

        // get the player ids of the players who are in the queue
        let inactive_player_ids : Vec<usize> =
            inactive_player_queue.iter().map(|player| player.player_id).collect();

        // get the next 14 pieces that will be deployed
        let next_pieces = peek_next_pieces(&block_queue, *block_index);


        let response = json!({
            "piece_states": states,
            "type": "gameState",
            "fallen_blocks": fallen_blocks_list,
            "player_queue": inactive_player_ids,
            "piece_queue": next_pieces,
        });

        // Unlock players so main thread can take in player updates
        drop(player_queue);
        drop(inactive_player_queue);
        drop(fallen_blocks);
        drop(block_queue);
        drop(block_index);

        // Send game state update to all connected clients
        match broadcaster.send(response.to_string()) {
            Ok(v) => v,
            Err(e) => println!("Unable to broadcast info: {}", e)
        };

        // Wait until next frame
        thread::sleep(FRAME_TIME);
    }
}


/**
 *
 *  The code which initializes the server.
 *
 *  After this block is executed, the main thread will take care
 *  of the incoming client updates, while the _game_thread will run
 *  the server logic and send out game state updates
 *
 *
 */
fn main() {
    let block_queue = Arc::new(Mutex::new([[0, 1, 2, 3, 4, 5, 6, 0, 1, 2, 3, 4, 5, 6] ; NUM_BAGS]));
    let block_index = Arc::new(Mutex::new(0));
    let player_queue = Arc::new(Mutex::new(VecDeque::new()));
    let inactive_queue = Arc::new(Mutex::new(VecDeque::new()));
    let fallen_blocks = Arc::new(Mutex::new(HashMap::new()));

    let thread_block_queue = block_queue.clone();
    let thread_block_index = block_index.clone();
    let thread_player_queue = player_queue.clone();
    let thread_inactive_queue = inactive_queue.clone();
    let thread_fallen_blocks = fallen_blocks.clone();

    // Code that initializes client structs
    let server_gen  = |out : Sender| {
        Client {
            out: out,
            timeout: None,
            player_queue: &player_queue,
            inactive_queue: &inactive_queue,
            block_queue: &block_queue,
            block_index: &block_index,
            fallen_blocks: &fallen_blocks,
        }
    };

    // Same functionality as listen command, but actually compiles?
    let socket = WebSocket::new(server_gen).unwrap();
    let socket = match socket.bind("127.0.0.1:3012") {
        Ok(v) => v,
        Err(_e) => {
            panic!("Socket in Use, Please Close Other Server")
        },
    };

    // Clone broadcaster to send data to clients on other thread
    let broadcaster = socket.broadcaster().clone();
    let _game_thread = thread::spawn(move || {
        game_frame(broadcaster,
                   thread_block_queue,
                   thread_block_index,
                   thread_player_queue,
                   thread_inactive_queue,
                   thread_fallen_blocks);
    });
    // Run the server on this thread
    socket.run().unwrap();
}

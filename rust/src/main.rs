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
const SHIFT_PERIOD_MILLIS : u128 = 500;

const PIECE_START_X : i8 = 5;
const PIECE_START_Y : i8 = 5;


type BlockQueueType = [[u8 ; BAG_SIZE] ; NUM_BAGS ];

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
    active_players: &'a Mutex<HashMap<usize, PieceState>>,
    inactive_players: &'a Mutex<VecDeque<PieceState>>,
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

        // a "null" struct, this will be set properly when the piece becomes active
        let new_piece_state = PieceState {
            shape: 0,
            pivot: Pivot {
                x: 0,
                y: 0,
            },
            rotation: 0,
            player_id: player_id
        };

        // Insert player into back of inactive queue
        let mut inactive_players = self.inactive_players.lock().unwrap();
        inactive_players.push_back(new_piece_state);
        drop(inactive_players);

        response = json!({
            "player_id": player_id,
            "type": "init",
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
                    let mut players_queue = self.active_players.lock().unwrap();
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

        let mut players = self.active_players.lock().unwrap();
        let mut inactive_players = self.inactive_players.lock().unwrap();
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
                 active_players: &mut HashMap<usize, PieceState>,
                 inactive_players: &mut VecDeque<PieceState>) {

    // remove player from active_players
    active_players.remove(&player_id).unwrap();

    // remove player from inactive_players
    let mut inactive_remove_index = None;
    for (index, inactive_player) in inactive_players.iter_mut().enumerate() {
        if inactive_player.player_id == player_id {
            inactive_remove_index = Some(index);
        }
    }
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
fn move_to_inactive(player_id : usize,
                    active_players: &mut HashMap<usize, PieceState>,
                    inactive_players: &mut VecDeque<PieceState>) {

    let player = active_players.remove(&player_id).unwrap();
    inactive_players.push_back(player);
}

/**
 *
 *  Generates the next piece to be output
 *
 *  TODO: Implement Tetris bag generation for better distribution
 *
 */
pub fn next_piece(block_queue: &mut BlockQueueType,
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
Don't actually change the piece queue at all, hence the word "peek."
*/
pub fn peek_next_pieces(block_queue: &BlockQueueType,
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

fn shift_pieces(active_players : &mut HashMap<usize, PieceState>,
                inactive_players : &mut VecDeque<PieceState>,
                fallen_blocks : &mut HashMap<Pivot, u8>) {

    let mut player_ids_to_remove : Vec<usize> = vec![];

    for (player_id, mut player) in active_players.iter_mut() {
        // make a copy which we shift down and check for collision
        let mut player_copy = player.clone();
        player_copy.pivot.y += 1;

        // If piece is off of the screen, remove it from play
        // We do this later, not in the iterator, since removing
        // elements while iterating is not safe.

        if fallen_blocks_collision(&player_copy, fallen_blocks) {
            add_fallen_blocks(player, fallen_blocks);
            player_ids_to_remove.push(player.player_id);
        } else {
            player.pivot.y += 1;
        }
    }

    // actually remove players from the board
    for player_id in player_ids_to_remove {
        move_to_inactive(player_id, active_players, inactive_players);
    }
}

// activates exactly one piece !
fn activate_piece(active_players : &mut HashMap<usize, PieceState>,
                  inactive_players : &mut VecDeque<PieceState>,
                  block_queue : &mut [ [u8 ; 14] ; NUM_BAGS ],
                  block_index : &mut usize) {

    // if we have more pieces in play and there are inactive pieces in the queue
    if MAX_NUM_ACTIVE - active_players.len() > 0 && inactive_players.len() > 0 {
        let mut player = inactive_players.pop_front().unwrap();

        // get the new piece type
        let piece_type: u8 = next_piece(block_queue,
                                        block_index);

        // update the player's piece type
        player.shape = piece_type;

        // update the player's position so it starts in the middle of the screen
        player.pivot.x = PIECE_START_X;
        player.pivot.y = PIECE_START_Y;

        // make sure that we didn't insert a duplicate into the set
        match active_players.insert(player.player_id, player) {
            Some(_) => { panic!("Already a player with id {} in active_players set.", player.player_id); },
            None => {},
        }
    }
}

/**
 *
 *  Runs the actual game logic at regular intervals, then sends out a
 *  state update to all the clients.
 *
 */
fn game_frame<'a>(broadcaster: Sender,
                  thread_active_players: Arc<Mutex<HashMap<usize, PieceState>>>,
                  thread_inactive_players: Arc<Mutex<VecDeque<PieceState>>>,
                  thread_fallen_blocks : Arc<Mutex<HashMap<Pivot, u8>>>) {

    // the time when we last shifted the pieces down
    let mut last_shift_time : u128 = 0;

    //
    let mut block_queue = [[0, 1, 2, 3, 4, 5, 6, 0, 1, 2, 3, 4, 5, 6] ; NUM_BAGS];
    let mut block_index = 0;

    loop {
        let mut active_players = thread_active_players.lock().unwrap();
        let mut inactive_players = thread_inactive_players.lock().unwrap();
        let mut fallen_blocks = thread_fallen_blocks.lock().unwrap();


        // drop the pieces 1 square if they need to be dropped
        let current_time = millis_since_epoch();
        if current_time - last_shift_time > SHIFT_PERIOD_MILLIS {

            // check to make sure shift works
            shift_pieces(&mut active_players, &mut inactive_players, &mut fallen_blocks);

            // actives a single piece
            activate_piece(&mut active_players, &mut inactive_players, &mut block_queue, &mut block_index);
            last_shift_time = current_time;
        }

        let fallen_blocks_list : Vec<BlockState> = fallen_blocks.iter().map(|(pivot, shape)| {
            return BlockState {
                position: pivot.clone(),
                original_shape: *shape,
            };
        }).collect();

        // Get the active players from the front of the deque
        let states : Vec<&PieceState> = active_players.iter().map(|(player_id, player)| player).collect();

        // get the player ids of the players who are in the queue
        let inactive_player_ids : Vec<usize> =
            inactive_players.iter().map(|player| player.player_id).collect();

        // get the next 14 pieces that will be deployed
        let next_pieces = peek_next_pieces(&block_queue, block_index);


        let response = json!({
            "piece_states": states,
            "type": "gameState",
            "fallen_blocks": fallen_blocks_list,
            "player_queue": inactive_player_ids,
            "piece_queue": next_pieces,
        });

        // Unlock players so main thread can take in player updates
        drop(active_players);
        drop(inactive_players);
        drop(fallen_blocks);

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
    let active_players = Arc::new(Mutex::new(HashMap::new()));
    let inactive_players = Arc::new(Mutex::new(VecDeque::new()));
    let fallen_blocks = Arc::new(Mutex::new(HashMap::new()));

    let thread_active_players = active_players.clone();
    let thread_inactive_players = inactive_players.clone();
    let thread_fallen_blocks = fallen_blocks.clone();

    // Code that initializes client structs
    let server_gen  = |out : Sender| {
        Client {
            out: out,
            timeout: None,
            active_players: &active_players,
            inactive_players: &inactive_players,
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
                   thread_active_players,
                   thread_inactive_players,
                   thread_fallen_blocks);
    });
    // Run the server on this thread
    socket.run().unwrap();
}

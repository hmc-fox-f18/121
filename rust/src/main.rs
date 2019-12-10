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
use crate::tetris::{update_state, fallen_blocks_collision, player_collision, clear_lines, read_block, get_shape};

use std::time::{SystemTime, UNIX_EPOCH};

use rand::thread_rng;
use rand::prelude::SliceRandom;
use std::sync::{Arc, Mutex};
use std::{time, thread};
use std::collections::VecDeque;

use ws::{CloseCode, Handler, Handshake, Message, Result,
     Sender, WebSocket, util::Token, util::Timeout, OpCode, Frame, Error};

use serde_json::json;

const FRAME_MILLIS : u64 = (1000.0 / 60.0) as u64;
const FRAME_TIME : time::Duration = time::Duration::from_millis(FRAME_MILLIS);

const NUM_BAGS : usize = 3;
const BAG_SIZE : usize = 14;
const MAX_NUM_ACTIVE : usize = 2;

/// CONSTANTS RELATED TO THE TIMING OF PIECE SHIFTING ///

// how long it takes between when pieces move down 1 square
const MAX_SHIFT_PERIOD : f32 = 400.0;

// how long it takes between when pieces move down 1 square
const MIN_SHIFT_PERIOD : f32 = 100.0;

// the score at which block drop at MIN_SHIFT_PERIOD and blocks can't drop any faster
const SPEED_CAPPED_SCORE : f32 = 10000.0; // 100 lines cleared


// how long piece may move when touching bottom of the board before it freezes
const BOTTOM_TOUCH_MS : u128 = 500;

const FAST_DROP_SHIFT_MS : u128 = 25;


const PIECE_START_X_LEFT : i8 = 5;
const PIECE_START_Y_LEFT : i8 = 0;
const PIECE_START_X_RIGHT : i8 = 12;
const PIECE_START_Y_RIGHT : i8 = 0;

/// CONSTANTS RELATED TO DETECTING NETWORK TIMEOUTS

const DISCONNECT_MILLIS : u64 = 3000; // 3 seconds
const PING_MILLIS : u64 = 1000; // 1 second
const PING: Token = Token(1);
const DISCONNECT: Token = Token(2);


type BlockQueueType = [[u8 ; BAG_SIZE] ; NUM_BAGS ];
type ActivePlayersType = HashMap<usize, PieceState>;
type InactivePlayersType = VecDeque<PieceState>;
type FallenBlocksType = HashMap<Pivot, u8>;

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
    active_players: &'a Mutex<ActivePlayersType>,
    inactive_players: &'a Mutex<InactivePlayersType>,
    fallen_blocks: &'a Mutex<FallenBlocksType>,
    timeout: Option<Timeout>,
    shutdown: bool,
}

// For accessing the default handler implementation
struct DefaultHandler;

impl Handler for DefaultHandler {}

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
            player_id: player_id,
            player_name: ['g', 'u', 'e', 's', 't', ' ', ' ', ' '],
            next_shift_time: None,
            fast_drop: false,
        };

        // Insert player into back of inactive queue
        let mut inactive_players = self.inactive_players.lock().unwrap();
        inactive_players.push_back(new_piece_state);
        drop(inactive_players);

        response = json!({
            "player_id": player_id,
            "type": "init",
        });

        // start pinging the client to detect if disconnected
        self.out.timeout(PING_MILLIS, PING).unwrap();

        self.out.send(response.to_string())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        if self.shutdown { return Ok(()); } // if connection is shutdown, do nothing

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
        if self.shutdown { return; } // if connection is shutdown, do nothing

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

    fn on_error(&mut self, err: Error) {
        if self.shutdown { return; }// if connection is shutdown, do nothing

        println!("The server encountered an error: {:?}", err);
    }

    /**
     *
     *  Method invoked when a client times out.
     *
     *  Logs the disconnection, then proceeds to remove the player
     *  from the game state.
     *
     */
    fn on_timeout(&mut self, event: Token) -> Result<()> {
        if self.shutdown { return Ok(()); } // if connection is shutdown, do nothing

        // if the event is PING, send a ping message and setup the next timeout
        match event {
            PING => {
                match self.out.ping(vec![]) {
                    Err(_) => { panic!("Unable to send ping."); },
                    _ => { },
                };
                self.out.timeout(PING_MILLIS, PING).unwrap();

                // if there is currently no disconnect timeout, start one
                if self.timeout.is_none() {
                    self.out.timeout(DISCONNECT_MILLIS, DISCONNECT).unwrap();
                }
            },
            DISCONNECT => {
                /*
                This code is run if the client becomes unresponsive and won't respond to a close
                message.

                We call remove_player now so that the other clients don't have to worry about
                this inactive player.

                We set self.shutdown == true so that all future data on this connection is ignored.
                */

                self.shutdown = true;

                match self.out.close(CloseCode::Away) {
                    Err(_) => println!("Unable to send close message to unresponsive client."),
                    _ => { },
                };

                let player_id : usize = self.out.token().into();
                let mut players = self.active_players.lock().unwrap();
                let mut inactive_players = self.inactive_players.lock().unwrap();
                remove_player(player_id, &mut *players, &mut *inactive_players);
            },
            Token(_) => panic!("Unexpected timoeout token."),
        };

        // Note: timeouts will actually occur if the client refreshes
        // the page
        Ok(())
    }

    fn on_new_timeout(&mut self, event: Token, timeout: Timeout) -> Result<()> {
        if self.shutdown { return Ok(()); } // if connection is shutdown, do nothing

        if event == DISCONNECT {
            // if there was no timeout registered, register one
            if self.timeout.is_none() {
                self.timeout = Some(timeout);
            }
            // if there was already a timeout registered and we just registered a duplicate
            else {
                match self.out.cancel(timeout) {
                    Err(_) => println!("Unable to cancel redundant timeout."),
                    _ => { },
                }
            }
        }

        return Ok(());
    }

    fn on_frame(&mut self, frame: Frame) -> Result<Option<Frame>> {
        if self.shutdown { return Ok(None); } // if connection is shutdown, do nothing

        if frame.opcode() == OpCode::Pong {
            // if there is a timeout, cancel it
            if let Some(t) = self.timeout.take() {
                match self.out.cancel(t) {
                    Err(_) => { panic!("Unable to cancel timeout."); },
                    _ => { },
                };
                self.timeout = None;
            }
        }

        // Run default frame validation
        DefaultHandler.on_frame(frame)
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
                 active_players: &mut ActivePlayersType,
                 inactive_players: &mut InactivePlayersType) {

    // remove player from active_players
    match active_players.remove(&player_id) {
        None => println!("{} wasn't in active_players", player_id),
        Some(_) => {},
    };

    // remove player from inactive_players
    let mut inactive_remove_index = None;
    for (index, inactive_player) in inactive_players.iter_mut().enumerate() {
        if inactive_player.player_id == player_id {
            inactive_remove_index = Some(index);
        }
    }
    match inactive_remove_index {
        // use .unwrap() because we are certain that a piece with inactive_remove_index
        // is in inactive_players
        Some(index) => { inactive_players.remove(index).unwrap(); },
        None => { println!("{} wasn't in inactive_players", player_id); },
    };
}

/**
 *
 *  Removes a player from the active queue and puts their piece in the inactive queue.
 *
 */
fn move_to_inactive(player_id : usize,
                    active_players: &mut ActivePlayersType,
                    inactive_players: &mut InactivePlayersType) {

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
    let next_piece = block_queue[index / BAG_SIZE][index % BAG_SIZE];

    // if we just used all of a bag, shuffle it so its good
    // for next time
    if index % BAG_SIZE == BAG_SIZE-1 {
        block_queue[index / BAG_SIZE].shuffle(&mut rng);
    }
    *stored_index = (index + 1) % (BAG_SIZE * NUM_BAGS);
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

fn add_fallen_blocks(piece : &PieceState, fallen_blocks : &mut FallenBlocksType) {
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

// move piece down by 1 square
// returns true if the player is no longer active
fn drop_piece(player_id : usize, fallen_blocks : &mut FallenBlocksType, active_players : &mut ActivePlayersType, shift_period : &f32) -> bool {
    // make a copy which we shift down and check for collision
    let mut player_copy = active_players.get(&player_id).unwrap().clone();

    player_copy.pivot.y += 1;

    // If piece has fallen off of the screen, remove it from play
    if fallen_blocks_collision(&player_copy, fallen_blocks) {
        let player : &mut PieceState = active_players.get_mut(&player_id).unwrap();
        add_fallen_blocks(player, fallen_blocks);
        (*player).next_shift_time = None;
        return true;
    }

    // if there is another piece blocking the way, don't shift down yet
    // and stop fast drop
    if player_collision(&player_copy, active_players) {
        active_players.get_mut(&player_id).unwrap().fast_drop = false;
        return false;
    }

    // if we've reache this point, the piece has not about to fall off of the screen
    // and there is no other player in the way
    let player : &mut PieceState = active_players.get_mut(&player_id).unwrap();
    (*player).pivot.y += 1; // move the piece down by 1

    // if we are doing fast drop, there is no extra time added when we're about to
    // hit the bottom
    if (*player).fast_drop {
        (*player).next_shift_time = Some(millis_since_epoch() + FAST_DROP_SHIFT_MS);
        return false;
    }

    // if piece is about to freeze, setup next_shift_time so that we can
    // allow the player longer to move around when their piece is almost about to collide
    player_copy.pivot.y += 1;
    if fallen_blocks_collision(&player_copy, fallen_blocks) {
        (*player).next_shift_time = Some(millis_since_epoch() + BOTTOM_TOUCH_MS);
        println!("Player is about to collide with ground!");
    }
    // if the player is not about to be off the screen, just do regular dropping
    else {
        (*player).next_shift_time = Some(millis_since_epoch() + *shift_period as u128);
    }

    return false;
}

fn shift_pieces(active_players : &mut ActivePlayersType,
                inactive_players : &mut InactivePlayersType,
                fallen_blocks : &mut FallenBlocksType,
                block_queue : &mut BlockQueueType,
                block_index : &mut usize,
                last_spawn_time : &mut u128,
                score : &u32) {

    // calculate shift period from score
    let shift_period = get_shift_period(score);

    let current_time = millis_since_epoch();

    // convert to i128 before subtracting so that negative result doesn't cause panic
    let spawn_ready = (current_time as i128 - *last_spawn_time as i128) as f32 > shift_period;

    let mut player_ids_to_drop : Vec<usize> = vec![];

    for player in active_players.values() {
        match player.next_shift_time {
            Some(next_shift_time) => {
                if current_time > next_shift_time {
                    player_ids_to_drop.push(player.player_id);
                }
            },
            None => panic!("There should be a next_shift_time field."),
        };
    }

    let mut player_ids_to_remove : Vec<usize> = vec![];

    // actually remove players from the board
    for player_id in player_ids_to_drop {
        if drop_piece(player_id, fallen_blocks, active_players, &shift_period) {
            player_ids_to_remove.push(player_id);
        }
    }

    // actually remove players from the board
    for player_id in player_ids_to_remove {
        move_to_inactive(player_id, active_players, inactive_players);
    }

    if spawn_ready {
        // actives a single piece
        activate_piece(active_players, inactive_players, block_queue, block_index, &shift_period);

        *last_spawn_time = current_time;
    }
}

// activates exactly one piece !
fn activate_piece(active_players : &mut ActivePlayersType,
                  inactive_players : &mut InactivePlayersType,
                  block_queue : &mut BlockQueueType,
                  block_index : &mut usize,
                  shift_period : & f32) {

    // if we have more pieces in play and there are inactive pieces in the queue
    if MAX_NUM_ACTIVE - active_players.len() > 0 && inactive_players.len() > 0 {
        let mut player = inactive_players.pop_front().unwrap();

        // get the new piece type
        let piece_type: u8 = next_piece(block_queue,
                                        block_index);

        player.rotation = 0; // reset the rotation
        player.shape = piece_type; // update the player's piece type

        // Alternate between 2 start positions
        if *block_index % 2 == 0 {
            player.pivot.x = PIECE_START_X_LEFT;
            player.pivot.y = PIECE_START_Y_LEFT;
        } else {
            player.pivot.x = PIECE_START_X_RIGHT;
            player.pivot.y = PIECE_START_Y_RIGHT;
        }

        // we lose a bit of precision on shift_period
        player.next_shift_time = Some(millis_since_epoch() + (*shift_period as u128));

        // piece are NOT fast dropping by default
        player.fast_drop = false;

        // make sure that we didn't insert a duplicate into the set
        match active_players.insert(player.player_id, player) {
            Some(_) => { panic!("Already a player with id {} in active_players set.", player.player_id); },
            None => {},
        }
    }
}

fn get_shift_period(score : &u32) -> f32 {
    // fraction from 0 to 1 indicates where we are between 0 and 100 lines cleared
    let mut frac = *score as f32 / SPEED_CAPPED_SCORE;
    if frac > 1.0 {
        frac = 1.0;
    }

    return MAX_SHIFT_PERIOD - ((MAX_SHIFT_PERIOD - MIN_SHIFT_PERIOD) * frac);
}

/**
 *
 *  Runs the actual game logic at regular intervals, then sends out a
 *  state update to all the clients.
 *
 */
fn game_frame<'a>(broadcaster: Sender,
                  thread_active_players: Arc<Mutex<ActivePlayersType>>,
                  thread_inactive_players: Arc<Mutex<InactivePlayersType>>,
                  thread_fallen_blocks : Arc<Mutex<FallenBlocksType>>,
                  thread_score : Arc<Mutex<u32>>) {

    // the time when we last shifted the pieces down
    let mut last_spawn_time : u128 = 0;

    let mut block_queue = [[0, 1, 2, 3, 4, 5, 6, 0, 1, 2, 3, 4, 5, 6] ; NUM_BAGS];
    let mut block_index = 0;

    loop {
        let mut active_players = thread_active_players.lock().unwrap();
        let mut inactive_players = thread_inactive_players.lock().unwrap();
        let mut fallen_blocks = thread_fallen_blocks.lock().unwrap();
        let mut score = thread_score.lock().unwrap();

        // check to make sure shift works
        shift_pieces(&mut active_players,
                     &mut inactive_players,
                     &mut fallen_blocks,
                     &mut block_queue,
                     &mut block_index,
                     &mut last_spawn_time,
                     & score);

        // Clear all completed fallen lines
        clear_lines(&mut fallen_blocks, &mut score);

        // Test for game-over criteria
        // If either starting point is blocked, end the game
        let start_left_pivot = &Pivot {
            x: PIECE_START_X_LEFT,
            y: PIECE_START_Y_LEFT,
        };
        let start_right_pivot = &Pivot {
            x: PIECE_START_X_RIGHT,
            y: PIECE_START_Y_RIGHT,
        };

        let mut is_collision = false;
        let starting_tiles = [start_left_pivot, start_right_pivot];
        let iter = starting_tiles.iter();
        for starting_tile in iter {
            let right = &Pivot {
                x: starting_tile.x + 1,
                y: starting_tile.y,
            };
            let left = &Pivot {
                x: starting_tile.x - 1,
                y: starting_tile.y,
            };
            let above = &Pivot {
                x: starting_tile.x,
                y: starting_tile.y - 1,
            };
            let below = &Pivot {
                x: starting_tile.x,
                y: starting_tile.y + 1,
            };

            let to_check = [starting_tile, right, left, above, below];
            for c in to_check.iter() {
                if fallen_blocks.contains_key(c) {
                    is_collision = true;
                }
            }
        }

        if is_collision {
            // Trigger Game Over
            let response = json!({
                "type": "gameOver"
            });
            // Send game state update to all connected clients
            match broadcaster.send(response.to_string()) {
                Ok(v) => v,
                Err(e) => println!("Unable to broadcast info: {}", e)
            };

            // Reset all data structure for next game
            active_players.clear();
            inactive_players.clear();
            fallen_blocks.clear();
            *score = 0;
        }

        let fallen_blocks_list : Vec<BlockState> = fallen_blocks.iter().map(|(pivot, shape)| {
            return BlockState {
                position: pivot.clone(),
                original_shape: *shape,
            };
        }).collect();

        // Get the active players from the front of the deque
        let states : Vec<&PieceState> = active_players.iter().map(|(_, player)| player).collect();

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
            "score": score.clone(),
        });

        // Unlock players so main thread can take in player updates
        drop(active_players);
        drop(inactive_players);
        drop(fallen_blocks);
        drop(score);

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
    let score = Arc::new(Mutex::new(0));

    let thread_active_players = active_players.clone();
    let thread_inactive_players = inactive_players.clone();
    let thread_fallen_blocks = fallen_blocks.clone();
    let thread_score = score.clone();

    // Code that initializes client structs
    let server_gen  = |out : Sender| {
        Client {
            out: out,
            timeout: None,
            active_players: &active_players,
            inactive_players: &inactive_players,
            fallen_blocks: &fallen_blocks,
            shutdown: false,
        }
    };

    // Same functionality as listen command, but actually compiles?
    let socket = WebSocket::new(server_gen).unwrap();
    let socket = match socket.bind("0.0.0.0:3012") {
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
                   thread_fallen_blocks,
                   thread_score);
    });
    // Run the server on this thread
    socket.run().unwrap();
}

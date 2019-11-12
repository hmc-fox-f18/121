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

const TIMEOUT_MILLIS : u64 = 3000;

const NUM_BAGS : usize = 3;
const NUM_ACTIVE : usize = 10;
// how long it takes between when pieces move down 1 square
const SHIFT_PERIOD_MILLIS : u128 = 1000;

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
    block_queue: &'a Mutex<[ [u8 ; 14] ; NUM_BAGS ]>,
    block_index: &'a Mutex<usize>,
    timeout: Option<Timeout>
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
        let mut player_queue = self.player_queue.lock().unwrap();
        let response;

        println!("Players: {:?}", player_queue);
        // Player doesn't exist, add to players list
        // TODO: Genericize initial piece state
        let piece_type: u8 = next_piece(self.block_queue,
                                            self.block_index);
        let new_piece_state = PieceState{
            shape: piece_type,
            pivot: Pivot{
                x: 5,
                y: 5
            },
            rotation: 0,
            player_id: player_id
        };

        // Insert player into back of queue
        player_queue.push_back(new_piece_state);

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
                    // Don't trust input, ensure labelled properly
                    let player_id : usize = self.out.token().into();
                    player_input.player_id = player_id;
                    // Update state for player
                    update_state(&mut players_queue, &player_input);
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
        remove_player(player_id, &mut *players);
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
        println!("Timed Out Player");
        match self.out.ping(vec![]) {
            Ok(_response) => {
                self.out.timeout(TIMEOUT_MILLIS, self.out.token()).unwrap();
                println!("Player ponged with {:?}", _response);
            }
            _err => {
                let player_id : usize = self.out.token().into();
                let mut players = self.player_queue.lock().unwrap();
                remove_player(player_id, &mut *players);
                self.out.close(CloseCode::Error).unwrap()
            }
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
                 players: &mut VecDeque<PieceState>) {
    let mut index = 0;
    loop {
        // Let crash if not in queue for now.
        let player = players.get(index).unwrap();
        if player.player_id == player_id {
            // Remove player
            players.remove(index).unwrap();
            break;
        }
        index += 1;
    };
    println!("remove_player called on {}", player_id);
}

/**
 *
 *  Removes a player from the board and puts their piece in the queue.
 *
 */
fn remove_from_play(player_id : usize, players: &mut VecDeque<PieceState>) {
    // this is temporary, change it
    // remove_player(player_id, players);
    let mut index = 0;
    loop {
        // Let crash if not in queue for now.
        println!("Trying to remove player {} from Players: \n {:?}", player_id, players);
        let player = players.get(index).unwrap();
        if player.player_id == player_id {
            // Remove player then push them to the back of the queue
            let removed_piece = players.remove(index).unwrap();
            players.push_back(removed_piece);
            break;
        }
        index += 1;
    };
    println!("remove_from_play called on {}", player_id);
}

/**
 *
 *  Generates the next piece to be output
 *
 *  TODO: Implement Tetris bag generation for better distribution
 *
 */
pub fn next_piece(thread_block_queue: &Mutex<[ [u8 ; 14] ; NUM_BAGS ]>,
                thread_block_index: &Mutex<usize>) -> u8 {
    let mut rng = thread_rng();
    let mut stored_index = thread_block_index.lock().unwrap();
    let index = *stored_index;
    let mut block_queue = thread_block_queue.lock().unwrap();
    let next_piece = block_queue[index / 14][index % 14];
    if index % 14 == 13 {
        block_queue[index / 14].shuffle(&mut rng);
    }
    *stored_index = (index + 1) % (14 * NUM_BAGS);
    return next_piece;
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

fn shift_pieces(players : &mut VecDeque<PieceState>, fallen_blocks : &mut HashMap<Pivot, u8>) {

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
        remove_from_play(player_id, players);
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
                thread_player_queue: Arc<Mutex<VecDeque<PieceState>>>) {

    // the time when we last shifted the pieces down
    let mut last_shift_time : u128 = 0;

    // stores PieceStates for all of the pieces that have
    // fallen to the bottom of the screen
    let mut fallen_blocks = HashMap::new();

    loop {
        let mut player_queue = thread_player_queue.lock().unwrap();


        // drop the pieces 1 square if they need to be dropped
        let current_time = millis_since_epoch();
        if current_time - last_shift_time > SHIFT_PERIOD_MILLIS {
            // check to make sure shift works
            shift_pieces(&mut player_queue, &mut fallen_blocks);
            last_shift_time = current_time;
        }

        let fallen_blocks_list : Vec<BlockState> = fallen_blocks.iter().map(|(pivot, shape)| {
            return BlockState {
                position: pivot.clone(),
                original_shape: *shape,
            };
        }).collect();

        // // for debugging
        // print!("blocks: ");
        // for block in fallen_blocks_list.iter() {
        //     print!("({}, {}), ", block.position.x, block.position.y);
        // }
        // print!("\n");

        let num_active = min(NUM_ACTIVE, player_queue.len());
        // Get the active players from the front of the deque
        let states : Vec<&PieceState> = player_queue.iter()
                                                    .take(num_active)
                                                    .collect();
        let response = json!({
            "piece_states": states,
            "type": "gameState",
            "fallen_blocks": fallen_blocks_list,
        });


        // Unlock players so main thread can take in player updates
        drop(player_queue);
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

    let thread_block_queue = block_queue.clone();
    let thread_block_index = block_index.clone();
    let thread_player_queue = player_queue.clone();
    // Code that initializes client structs
    let server_gen  = |out : Sender| {
        Client {
            out: out,
            timeout: None,
            player_queue: &player_queue,
            block_queue: &block_queue,
            block_index: &block_index
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
        game_frame(broadcaster, thread_block_queue,
                    thread_block_index, thread_player_queue);
    });
    // Run the server on this thread
    socket.run().unwrap();
}

extern crate ws;
extern crate rand;
extern crate slab;

mod piece_state;

use crate::piece_state::PieceState;
use rand::Rng;

use ws::{CloseCode, Handler, Handshake, Message, Result,
     Sender, WebSocket};

use slab::Slab;

use serde_json::json;

struct Server<'a> {
    out: Sender,
    player_key: usize,
    players: &'a mut Slab<PieceState>
}

impl Handler for Server<'_> {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("Request: {}", shake.request);
        let address = shake.remote_addr();
        match address {
            Ok(v) => println!("User Connected with Address: {:?}", v),
            Err(e) => println!("Unable to get user address: {:?}", e),
        }
        let player_id : usize = self.out.token().into();
        println!("This is {:?}", player_id);

        //TODO: Consider if different responses are needed for
        //Reconnecting players
        let response;
        if self.players.contains(player_id) {
            let new_piece_state = self.players.get(player_id).unwrap();
            let piece_type = new_piece_state.shape;
            response = json!({
                "player_num": player_id,
                "piece_type": piece_type,
                "type": "Initialize"
            });
        }
        else {
            // Player doesn't exist, add too players list
            let piece_type: u8 = next_piece();
            let new_piece_state = PieceState{
                shape: piece_type,
                x: 5,
                y: 5,
                rotation: 0,
                player_id: player_id
            };
            println!("{:?}", new_piece_state);
            self.player_key = self.players.insert(new_piece_state);
            response = json!({
                "player_num": player_id,
                "piece_type": piece_type,
                "type": "Initialize"
            });
        }
        println!("{:?}", self.players);
        self.out.send(response.to_string()).unwrap();
        return Ok(());
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        // parse the msg as text
        if let Ok(text) = msg.into_text() {
            // try to parse the message as json,
            // if valid json, echo the json to everyone connected
            // else send blank response OK(()) (send nothing)
            match serde_json::from_str::<PieceState>(&text) {
                Ok(new_piece_state) => {
                    let state = self.players.get_mut(self.player_key).unwrap();
                    *state = new_piece_state;
                    let mut states : Vec<&PieceState> = self.players
                                        .iter()
                                        .map(|(key, val)| val)
                                        .collect();
                    let response = json!({
                        "player_states": states,
                        "type": "gameState"
                    });
                    //println!("{}", text);
                    return self.out.send(response.to_string());
                }
                Err(_e) => {
                    //println!("Could not parse status: {}\n", e);
                    return Ok(());
                },
            }
        }
        // default to blank result
        return Ok(());
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => println!("The client is leaving the site."),
            _ => println!("The client encountered an error: {}", reason),
        }
    }
}

fn next_piece() -> u8 {
    let mut rng = rand::thread_rng();
    return rng.gen_range(0, 7);
}

fn main() {
    let players : &mut Slab<PieceState> = &mut Slab::new();
    let server_gen  = |out : Sender| {
        Server {
            out: out,
            player_key: 0,
            players: players
        }
    };
    // Same functionality as listen command, but actually compiles?
    let socket = WebSocket::new(server_gen).unwrap();
    let socket = match socket.bind("127.0.0.1:3012") {
        Ok(v) => v,
        Err(e) => panic!("Socket in Use, Please Close Other Server"),
    };
    socket.run().unwrap();
}


extern crate ws;
extern crate rand;

mod piece_state;

use crate::piece_state::PieceState;
use rand::Rng;

use std::cell::Cell;
use std::rc::Rc;

use ws::{listen, CloseCode, Handler, Handshake, Message, Result, Sender};

use serde_json::json;

struct Server {
    out: Sender,
    count: Rc<Cell<u32>>,
    players: Vec<PieceState>
}

impl Handler for Server {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("Request: {}", shake.request);
        let address = shake.remote_addr();
        match address {
            Ok(v) => println!("User Connected with Address: {:?}", v),
            Err(e) => println!("Unable to get user address: {:?}", e),
        }
        let piece_type: u8 = next_piece();
        let player_id = self.count.get();
        let response = json!({
            "player_num": player_id,
            "piece_type": piece_type,
            "type": "Initialize"
        });
        println!("Piece Type Given: {:?}", piece_type);
        println!("This is {:?}", self.out.token());
        let new_piece_state = PieceState{
                shape: piece_type,
                x: 5,
                y: 5,
                rotation: 0,
                player_id: player_id
        };
        println!("{:?}", self.players);
        self.players.push(new_piece_state);
        println!("{:?}", self.players);
        self.out.send(response.to_string());
        return Ok(self.count.set(self.count.get() + 1));
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        // parse the msg as text
        if let Ok(text) = msg.into_text() {
            // try to parse the message as json,
            // if valid json, echo the json to everyone connected
            // else send blank response OK(()) (send nothing)
            match serde_json::from_str::<PieceState>(&text) {
                Ok(new_piece_state) => {
                    let player_id = new_piece_state.player_id;
                    for state in self.players.iter_mut() {
                        if state.player_id == player_id {
                            *state = new_piece_state;
                            break;
                        }
                    }
                    //println!("Received status:\n{:?}\n", new_piece_state);
                    //println!("Stored {} Players as: {:?}",
                    //    self.players.len(), self.players);
                    return self.out.send(text);
                }
                Err(e) => {
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
    let count = Rc::new(Cell::new(0));
    listen("127.0.0.1:3012", |out : Sender| Server {
        out: out,
        count: count.clone(),
        players: Vec::new()
    })
    .unwrap();
}

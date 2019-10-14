use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PieceState {
    pub shape: u8,
    pub x: u8,
    pub y:u8,
    pub rotation: u8,
    pub player_id: usize
}

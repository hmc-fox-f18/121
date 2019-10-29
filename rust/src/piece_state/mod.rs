use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Pivot {
    pub x: u8,
    pub y:u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct PieceState {
    pub shape: u8,
    pub pivot: Pivot,
    pub rotation: u8,
    pub player_id: usize
}

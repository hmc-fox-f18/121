use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pivot {
    pub x: i8,
    pub y: i8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct PieceState {
    pub shape: u8,
    pub pivot: Pivot,
    pub rotation: u8,
    pub player_id: usize,
    pub player_name: [char; 8]
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct BlockState {
    pub original_shape: u8, // this is so we can use existing methods to determine color
    pub position: Pivot,
}

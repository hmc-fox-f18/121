use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct KeyState {
    pub left: bool,
    pub right: bool,
    pub rot: bool,
    pub counter_rot: bool,
    pub hard_drop: bool,
    pub fast_drop: bool,
    pub player_id: usize,
    pub player_name: String
}

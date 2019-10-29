extern crate slab;

use crate::piece_state::{PieceState};
use crate::input::{KeyState};

use std::sync::{Mutex};
use slab::Slab;
// TODO: Cleaner representation of pieces for calculations
// consider classes
const piece_z : [bool ; 9] = [ true, true, false, false, true, true, false, false, false]; //0
const piece_s : [bool ; 9] = [ false, true, true, true, true, false, false, false, false]; //1
const piece_j : [bool ; 9] = [ true, false, false, true, true, true, false, false, false]; //2
const piece_r : [bool ; 9] = [ false, false, true, true, true, true, false, false, false]; //3
const piece_t : [bool ; 9] = [ false, true, false, true, true, true, false, false, false]; //4
const piece_i : [bool ; 16] = [ false, false, false, false, true, true, true, true, false, false, false, false, false, false, false, false]; //5
const piece_o : [bool ; 16] = [ false, false, false, false, false, true, true, false, false, true, true, false, false, false, false, false]; //6

const rot_limit : u8 = 4;

pub struct Game<'a> {
    players: &'a Mutex<Slab<PieceState>>
}

/**
 *
 *
 * TODO: Consider making methods into a trait to more easily support
 * different games or gamestyles on the same codebase
 */
impl Game<'_> {
}

pub fn update_state(players : &mut Slab<PieceState>, player_input : &KeyState) {
    let mut curr_player = players.get_mut(player_input.player_id)
                                 .unwrap();
    let new_state = apply_input(player_input, curr_player);
    if new_state != *curr_player {
        let states : Vec<&PieceState> = players
                            .iter()
                            .map(|(_key, val)| val)
                            .collect();
        if !collision(&new_state, states) {
            *curr_player = new_state;
        }
    }
}


fn apply_input(player_input : &KeyState,
            player_piece : &PieceState) -> PieceState {
    let new_state = (*player_piece).clone();
    // Move left
    if player_input.left {
        new_state.pivot.x -= 1;
    }
    // Move right
    if player_input.right {
        new_state.pivot.x += 1;
    }
    // Rotate clockwise
    if player_input.rot {
        new_state.rotation = (new_state.rotation + 1) % rot_limit;
    }
    // Rotate counter-clockwise
    if player_input.counter_rot {
        new_state.rotation -= 1;
        // Wrap around
        if new_state.rotation < 0 {
            new_state.rotation = rot_limit - 1;
        }
    }
    return new_state;
}
/**
     *
     *  Determines whether there exists a block in the corresponding
     *  rotated piece at the inputed (x, y) coordinate relative to the
     *  top-left corner of the piece's bounding box.
     *
     *  This is done in place by taking advantage of how we can
     *  represent both the original and rotated matrices as arrays, and
     *  then represent the array of the rotated piece in terms of the
     *  original.
     *
     */
    fn read_block(piece : &[bool], x : u8, y : u8, rot : u8) -> bool {
    let length : u8 = piece.len().try_into().unwrap();
    // Matrix width, 3 if there are 9 elements, 4 if there are 16
    let width = if length == 9 {3} else {4};
    // Return false if requested element is outside matrix
    if x >= width || y >= width || x < 0 || y < 0 {
        return false;
    }
    match rot {
        // Unrotated
        0 => {
            let index = y * width + x;
            return piece[index as usize];
        }
        // Rotated clockwise, goes down with x, left with y
        1 => {
            let index = x * width + (width - (y + 1) % width);
            return piece[index as usize];
        }
        // 180 degrees, like unrotated but backwards
        2 => {
            let index = length - (y * width + x) - 1;
            return piece[index as usize];
        }
        // Rotated counter-clockwise, goes up with x, right with y
        3 => {
            let index = (width - (x + 1) % width) * width + y;
            return piece[index as usize];
        }
        // Invalid Rotation
        _ => false,
    }
}

fn get_shape(shape_num : u8) -> &'static [bool] {
    match shape_num {
        0 => &piece_z,
        1 => &piece_s,
        2 => &piece_j,
        3 => &piece_r,
        5 => &piece_t,
        6 => &piece_i,
        7 => &piece_o,
        _ => panic!("Unknown Piece Number")
    }
}

fn collision(piece : &PieceState, players : Vec<&PieceState>) -> bool {
    for other_piece in &players {
        if piece.player_id != other_piece.player_id {
            let this_origin = piece.pivot;
            let other_origin = other_piece.pivot;

            let x_offset = other_origin.x - this_origin.x;
            let y_offset = other_origin.y - this_origin.y;
            // TODO: add wallkicks, wall collision
            let this_shape = get_shape(piece.shape);
            let other_shape = get_shape(other_piece.shape);
            let width = if this_shape.len() == 9 {3} else {4};
            for x in 0..width {
                for y in 0..width {
                    if read_block(this_shape, x, y, piece.rotation) &&
                            read_block(other_shape, x + x_offset,
                            y + y_offset, other_piece.rotation) {
                        return true;
                    }
                }
            }
        }
    }
    return false;
}

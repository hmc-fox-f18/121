extern crate slab;

use crate::piece_state::{PieceState};
use crate::input::{KeyState};

use slab::Slab;
use std::convert::TryInto;
// TODO: Cleaner representation of pieces for calculations
// consider classes
const PIECE_Z : [bool ; 9] = [ true, true, false, false, true, true, false, false, false]; //0
const PIECE_S : [bool ; 9] = [ false, true, true, true, true, false, false, false, false]; //1
const PIECE_J : [bool ; 9] = [ true, false, false, true, true, true, false, false, false]; //2
const PIECE_R : [bool ; 9] = [ false, false, true, true, true, true, false, false, false]; //3
const PIECE_T : [bool ; 9] = [ false, true, false, true, true, true, false, false, false]; //4
const PIECE_I : [bool ; 16] = [ false, false, false, false, true, true, true, true, false, false, false, false, false, false, false, false]; //5
const PIECE_O : [bool ; 16] = [ false, false, false, false, false, true, true, false, false, true, true, false, false, false, false, false]; //6

const ROT_LIMIT : u8 = 4;
const BOARD_WIDTH : i8 = 20;

pub fn update_state(players : &mut Slab<PieceState>, player_input : &KeyState) {
    let new_state = apply_input(player_input, players);
    if !collision(&new_state, players) {
        let player_piece = players.get_mut(player_input.player_id)
                                 .unwrap();
        *player_piece = new_state;
    }
}


fn apply_input(player_input : &KeyState,
            players : &mut Slab<PieceState>) -> PieceState {
    let player_piece = players.get_mut(player_input.player_id)
                                 .unwrap();
    let mut new_state = (*player_piece).clone();
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
        new_state.rotation = (new_state.rotation + 1) % ROT_LIMIT;
    }
    // Rotate counter-clockwise
    if player_input.counter_rot {
        // Wrap around
        if new_state.rotation == 0 {
            new_state.rotation = ROT_LIMIT - 1;
        }
        else {
            new_state.rotation -= 1;
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
**/
fn read_block(piece : &[bool], x : i8, y : i8, rot : u8) -> bool {
    let length : i8 = piece.len().try_into().unwrap();
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
            let index = x * width + (width - (y + 1) );
            return piece[index as usize];
        }
        // 180 degrees, like unrotated but backwards
        2 => {
            let index = length - (y * width + x) - 1;
            return piece[index as usize];
        }
        // Rotated counter-clockwise, goes up with x, right with y
        3 => {
            let index = (width - (x + 1) ) * width + y;
            return piece[index as usize];
        }
        // Invalid Rotation
        _ => false,
    }
}

fn get_shape(shape_num : u8) -> &'static [bool] {
    match shape_num {
        0 => &PIECE_Z,
        1 => &PIECE_S,
        2 => &PIECE_J,
        3 => &PIECE_R,
        4 => &PIECE_T,
        5 => &PIECE_I,
        6 => &PIECE_O,
        _unknown => panic!("Unknown Piece Number: {}", _unknown)
    }
}

fn collision(piece : &PieceState, players_slab : &mut Slab<PieceState>) -> bool {
    // Check if in bounds
    let this_shape = get_shape(piece.shape);
    let width = if this_shape.len() == 9 {3} else {4};
    let this_origin = piece.pivot;
    for x in 0..width {
        for y in 0..width {
            let abs_x = x + this_origin.x;
            let abs_y = y + this_origin.y;
            if read_block(this_shape, x, y, piece.rotation) &&
                    (abs_x > BOARD_WIDTH || abs_x < 0 || abs_y < 0) {
                return true;
            }
        }
    }

    // Check if collides with other players
    let players : Vec<&mut PieceState> = players_slab
                            .iter_mut()
                            .map(|(_key, val)| val)
                            .collect();
    for other_piece in &players {
        if piece.player_id != other_piece.player_id {
            let other_origin = other_piece.pivot;
            let other_shape = get_shape(other_piece.shape);
            let x_offset = this_origin.x - other_origin.x;
            let y_offset = this_origin.y - other_origin.y;
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
    // TODO: add wallkicks
    return false;
}

extern crate slab;

use std::convert::TryInto;

use crate::piece_state::{PieceState, Pivot};
use crate::input::{KeyState};
use crate::{ActivePlayersType, FallenBlocksType};

// TODO: Cleaner representation of pieces for calculations
// consider classes
const PIECE_Z : [bool ; 9] = [ true, true, false, false, true, true, false, false, false]; //0
const PIECE_S : [bool ; 9] = [ false, true, true, true, true, false, false, false, false]; //1
const PIECE_J : [bool ; 9] = [ true, false, false, true, true, true, false, false, false]; //2
const PIECE_L : [bool ; 9] = [ false, false, true, true, true, true, false, false, false]; //3
const PIECE_T : [bool ; 9] = [ false, true, false, true, true, true, false, false, false]; //4
const PIECE_I : [bool ; 16] = [ false, false, false, false, true, true, true, true, false, false, false, false, false, false, false, false]; //5
const PIECE_O : [bool ; 16] = [ false, false, false, false, false, true, true, false, false, true, true, false, false, false, false, false]; //6

const X_KICKS : [ [i8 ; 8] ; 4] = [
                                    [-1, 1, 1, 1, -1, 1, -1, -1],
                                    [-1, 1, 1, 1, -1, 1, -1, -1],
                                    [ 0, 0, 0, 0, 0, 0, 0, 0],
                                    [-1, 1, 1, 1, -1, 1, -1, -1]
                                ];

const Y_KICKS : [ [i8 ; 8] ; 4] = [
                                    [ 0, 0, 0, 0, 0, 0, 0, 0],
                                    [ 1, 1, -1, -1, 1, 1, -1, -1],
                                    [-2, -2, 2, 2, -2, -2, 2, 2],
                                    [-2, -2, 2, 2, -2, -2, 2, 2]
                                ];

const I_BLOCK_X_KICKS : [ [i8 ; 8] ; 4] = [
                                    [-2, -1, 2, -1, 1, 2, -2, 1],
                                    [1, 2, -1, 2, -2, -1, 1, -2],
                                    [-2, -1, 2, -1, 1, 2, -2, 1],
                                    [1, 2, -1, 2, -2, -1, 1, -2]
                                ];

const I_BLOCK_Y_KICKS : [ [i8 ; 8] ; 4] = [
                                    [ 0, 0, 0, 0, 0, 0, 0, 0],
                                    [ 0, 0, 0, 0, 0, 0, 0, 0],
                                    [-1, 2, 1, 2, -2, 1, -1, -2],
                                    [2, -1, -2, -1, 1, -2, 2, 1]
                                ];


const ROT_LIMIT : u8 = 4;
pub const BOARD_WIDTH : i8 = 20;

pub fn update_state(active_players : &mut ActivePlayersType,
                    player_input : &KeyState,
                    fallen_blocks : &FallenBlocksType) {

    let player_id = player_input.player_id;
    let active_player_ids : Vec<usize> = active_players.keys().map(|key| *key).collect();

    // only apply the update if the player specified in player_id is active
    if !active_player_ids.contains(&player_id) {
        return
    }

    let new_state = apply_input(player_input, active_players, fallen_blocks);
    if !collision(&new_state, active_players, fallen_blocks) {
        // call unwrap() to make sure that there was already a key present and
        // that we are just updating an existing entry
        active_players.insert(player_id, new_state).unwrap();
    }
}

fn apply_input(player_input : &KeyState,
               active_players : &mut ActivePlayersType,
               fallen_blocks : &FallenBlocksType) -> PieceState {


    // make a copy of the current player state and work with this
    let mut new_state = active_players.get(&player_input.player_id).unwrap().clone();

    // Move left
    if player_input.left {
        new_state.pivot.x -= 1;
    }
    // Move right
    if player_input.right {
        new_state.pivot.x += 1;
    }
    // Rotate clockwise
    let mut rotated = false;
    let mut clockwise = false;
    if player_input.rot {
        clockwise = true;
        rotated = !rotated;
        new_state.rotation = (new_state.rotation + 1) % ROT_LIMIT;
    }
    // Rotate counter-clockwise
    if player_input.counter_rot {
        rotated = !rotated;
        new_state.rotation = (ROT_LIMIT + new_state.rotation - 1)
                                % ROT_LIMIT;
    }
    // Only do wallkick calculations when there is a net rotation
    if rotated {
        return wallkick(&mut new_state, clockwise, active_players, fallen_blocks);
    }
    else {
        return new_state;
    }
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
pub fn read_block(piece : &[bool], x : i8, y : i8, rot : u8) -> bool {
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
            let index = (width - (x + 1) ) * width + y;
            return piece[index as usize];
        }
        // 180 degrees, like unrotated but backwards
        2 => {
            let index = length - (y * width + x) - 1;
            return piece[index as usize];
        }
        // Rotated counter-clockwise, goes up with x, right with y
        3 => {
            let index = x * width + (width - (y + 1) );
            return piece[index as usize];
        }
        // Invalid Rotation
        _ => false,
    }
}

pub fn get_shape(shape_num : u8) -> &'static [bool] {
    match shape_num {
        0 => &PIECE_Z,
        1 => &PIECE_S,
        2 => &PIECE_J,
        3 => &PIECE_L,
        4 => &PIECE_T,
        5 => &PIECE_I,
        6 => &PIECE_O,
        _unknown => panic!("Unknown Piece Number: {}", _unknown)
    }
}

pub enum CollisionType {
    Ceiling,
    Wall,
    Floor,
    None,
}

pub fn screen_collision(piece : &PieceState) -> CollisionType {
    let this_shape = get_shape(piece.shape);
    let width = if this_shape.len() == 9 {3} else {4};
    let this_origin = piece.pivot;

    for y in 0..width {
        for x in 0..width {
            let abs_x = x + this_origin.x;
            let abs_y = y + this_origin.y;

            if read_block(this_shape, x, y, piece.rotation) {
                if abs_x >= BOARD_WIDTH || abs_x < 0 { return CollisionType::Wall };
                if abs_y < 0 { return CollisionType::Ceiling; }
                if abs_y >= BOARD_WIDTH { return CollisionType::Floor; }
            }
        }
    }

    return CollisionType::None;
}

pub fn fallen_blocks_collision(piece : &PieceState, fallen_blocks : &FallenBlocksType) -> bool {
    // Check if we collide with the bottom of the screen
    let bottom_screen_collision = match screen_collision(piece) {
        CollisionType::Floor => true,
        _ => false,
    };
    if bottom_screen_collision { return true; }

    // check if we collide with any of the bottom blocks

    let this_shape = get_shape(piece.shape);
    let width = if this_shape.len() == 9 {3} else {4};
    let this_origin = piece.pivot;

    for y in 0..width {
        for x in 0..width {
            let abs_x = x + this_origin.x;
            let abs_y = y + this_origin.y;

            if read_block(this_shape, x, y, piece.rotation) {
                // if the position of one of the blocks that makes up piece overlaps
                // with the location of a block in fallen_blocks, we have a collision

                if fallen_blocks.contains_key(&Pivot{x: abs_x, y: abs_y}) {
                    return true;
                }
            }
        }
    }

    return false;
}

// Clears any lines necessary, modifying fallen_blocks as appropriate
pub fn clear_lines(fallen_blocks : &mut FallenBlocksType, score : &mut u32) {
    let mut offset = 0;
    let mut lines_cleared = 0;
    for row in (0..BOARD_WIDTH).rev() {
        let mut is_full = true;
        for col in 0..BOARD_WIDTH {
            let pivot = &Pivot {
                x: col,
                y: row,
            };
            if !fallen_blocks.contains_key(pivot) {
                is_full = false;
                break;
            }
        }

        // Need to clear the row if it is full
        if is_full {
            offset += 1;
            for col in 0..BOARD_WIDTH {
                let pivot = &Pivot {
                    x: col,
                    y: row,
                };
                fallen_blocks.remove(pivot);
            }
            lines_cleared += 1;
        } else if offset != 0 {
            // If did not clear, add to new_fallen_blocks
            for col in 0..BOARD_WIDTH {
                let pivot = &Pivot {
                    x: col,
                    y: row,
                };
                let fallen_pivot = Pivot {
                    x: col,
                    y: row + offset,
                };

                match fallen_blocks.get(pivot) {
                    Some(shape) => {
                        fallen_blocks.insert(fallen_pivot, *shape)
                    },
                    None => None
                };
                fallen_blocks.remove(pivot);
            }
        }
    }
    // Update the score, tetrises are scored diffrently
    if lines_cleared == 4 {
        *score += 800;
    } else {
        *score += 100 * lines_cleared;
    }
}

fn collision(piece : &PieceState,
             active_players: &mut ActivePlayersType,
             fallen_blocks : &FallenBlocksType) -> bool {

    // if we hit a wall, return true
    let wall_collision = match screen_collision(piece) {
        CollisionType::Wall => true,
        _ => false,
    };
    if wall_collision { return true; }

    // if we hit a fallen block, return true
    if fallen_blocks_collision(piece, fallen_blocks) {
        return true;
    }

    let this_shape = get_shape(piece.shape);
    let width = if this_shape.len() == 9 {3} else {4};
    let this_origin = piece.pivot;

    // Check if collides with other players
    for (other_piece_id, other_piece) in active_players {
        if piece.player_id != *other_piece_id {
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

fn wallkick(mut new_state : &mut PieceState,
            clockwise : bool,
            active_players : &mut ActivePlayersType,
            fallen_blocks : &FallenBlocksType) -> PieceState {
    // No Change, Test 1
    if !collision(&mut new_state, active_players, fallen_blocks) {
        return *new_state;
    }
    let prev_rotation = if clockwise {
            (ROT_LIMIT + new_state.rotation - 1) % ROT_LIMIT
        }
        else {
            (new_state.rotation + 1) % ROT_LIMIT
        };

    for i in 0..4 {
        let index : usize = (2 * prev_rotation +
                                (if clockwise {0} else {1})) as usize;
        let x_test;
        let y_test;
        if new_state.shape < 5 {
            x_test = X_KICKS[i][index];
            y_test = Y_KICKS[i][index];
        }
        else {
            x_test = I_BLOCK_X_KICKS[i][index];
            y_test = I_BLOCK_Y_KICKS[i][index];
        }
        new_state.pivot.x += x_test;
        new_state.pivot.y += y_test;
        if !collision(&mut new_state, active_players, fallen_blocks) {
            return *new_state;
        }
        new_state.pivot.x -= x_test;
        new_state.pivot.y -= y_test;
    }
    // TODO: Prevent from re-checking collision afterwards
    return *new_state;
}

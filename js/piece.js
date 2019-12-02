/* jshint esversion: 6 */

/*
Piece objects should not be used directly in the game because they have no
associated player_id. Rather, they should be used to create PlayerPiece
objects.
*/
class Piece {
    constructor(shape, shape_num, color, x, y, rot, boundWidth) {
        this.shape = shape;
        this.shape_num = shape_num;
        this.color = color;
        this.x = x;
        this.y = y;
        this.rot = rot;
        this.boundWidth = boundWidth;
    }
}


const pieceZ = new Piece([ 1, 1, 0, 0, 1, 1, 0, 0, 0], 0,  "#FF5B5B", 0, 0, 0, 3); //0
const pieceS = new Piece([ 0, 1, 1, 1, 1, 0, 0, 0, 0], 1, "#3DE978", 0, 0, 0, 3); //1
const pieceJ = new Piece([ 1, 0, 0, 1, 1, 1, 0, 0, 0], 2, "#3D7AE9", 0, 0, 0, 3); //2
const pieceL = new Piece([ 0, 0, 1, 1, 1, 1, 0, 0, 0], 3, "#FF894E", 0, 0, 0, 3); //3
const pieceT = new Piece([ 0, 1, 0, 1, 1, 1, 0, 0, 0], 4, "#F27DFF", 0, 0, 0, 3); //4
const pieceI = new Piece([ 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0], 5, "#7DFFDC", 0, 0, 0, 4); //5
const pieceO = new Piece([ 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0], 6, "#FFDF92", 0, 0, 0, 4); //6
const shapes = [pieceZ, pieceS, pieceJ, pieceL, pieceT, pieceI, pieceO];

/*
PlayerPiece objects are typically built using the fromNetworkInfo static method.
This class is similar to Piece class, but has the added player_id field and
supports collision detection because these objects are intended for use in the
game.
*/
class PlayerPiece extends Piece {
    constructor(shape, shape_num, color, x, y, rot, boundWidth, player_id, player_name) {
        // call base constructor to set fields
        super(shape, shape_num, color, x, y, rot, boundWidth);
        // player_id field isn't in Piece class
        this.player_id = player_id;
        this.player_name = player_name;
    }

    deepCopy() {
        return new PlayerPiece(this.shape,
                               this.shape_num,
                               this.color,
                               this.x, this.y, this.rot,
                               this.boundWidth,
                               this.player_id,
                               this.player_name);
    }

    // Call this to construct a player_piece from info sent from
    // the server.
    static fromNetworkInfo(shape_num, x, y, rot, player_id, player_name) {
        // get the Piece template piece from the list
        var piece_template = shapes[shape_num];

        // create a new PlayerPiece using this Piece template
        return new PlayerPiece(
                piece_template.shape,
                piece_template.shape_num,
                piece_template.color,
                x, y, rot,
                piece_template.boundWidth,
                player_id,
                player_name.join(""));
    }

    getRenderInfo() {
        return [this.x, this.y, this.color, this.boundWidth,
                    this.rotate_shape()];
    }

    getNetworkInfo() {
        return [this.x, this.y, this.rot, this.shape_num];
    }

    /**
     *
     * Takes in a piece, and checks for any collisions with other game
     * elements which would invalidate the pieces position.
     *
     * Returns true if there is a collision.
     */
     collision() {
         return this.wall_collision() || this.player_collision() || this.fallen_block_collision();
     }

    // calls callback(x, y) for each occupied block
    get_occupied_blocks(callback) {
        // get the rotated version of the matrix
        let rot_shape = this.rotate_shape();
        let x = this.x;
        let y = this.y;

        //Determine whether the block is in a 3x3 or 4x4 bounding box
        let bound_width = this.boundWidth;
        for (let block in rot_shape) {
            if (rot_shape[block] == 1) {
                let abs_x = x + (block % bound_width);
                let abs_y = y + Math.floor(block / bound_width);

                callback(abs_x, abs_y);
            }
        }
    }

    wall_collision() {
        var off_screen = false;

        this.get_occupied_blocks((x, y) => {
            if (x < 0 || x >= BOARD_WIDTH || y >= BOARD_HEIGHT) {
                off_screen = true;
            }
        });

        return off_screen;
    }

    player_collision() {
        // first, get a list of all the squares that are occupied by pieces
        // that aren't our piece
        var is_overlap = false;

        game_state.pieces.forEach((piece) => {
            // if the piece isn't my piece, add all the blocks it occupies
            // to the set
            if (isMyPiece(piece)) return;

            piece.get_occupied_blocks((other_x, other_y) => {
                this.get_occupied_blocks((my_x, my_y) => {
                    if (other_x == my_x && other_y == my_y) {
                        is_overlap = true;
                    }
                });
            });
        });

        return is_overlap;
    }

    fallen_block_collision() {
        var is_overlap = false;

        this.get_occupied_blocks((my_x, my_y) => {
            game_state.fallen_blocks.forEach((fallen_block) => {
                if (fallen_block.x == my_x &&
                    fallen_block.y == my_y) {
                    is_overlap = true;
                }
            });
        });

        return is_overlap;
    }

    //TODO: Better rotation method
    /**
     * Returns the piece array rotated 90 degrees counter-clockwise
     */
    rotate_ccw() {
        if (this.boundWidth == 3) {
            let [b7, b4, b1, b8, b5, b2, b9, b6, b3] = this.shape;
            return [b1, b2, b3, b4, b5, b6, b7, b8, b9];
        }
        else {
            let [b13, b9, b5, b1, b14, b10, b6, b2, b15, b11, b7, b3, b16, b12, b8, b4] = this.shape;
            return [b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16];
        }
    }

    /**
     * Returns the piece array rotated 90 degrees clockwise
     */
    rotate_cw() {
        if (this.boundWidth == 3) {
            let [b3, b6, b9, b2, b5, b8, b1, b4, b7] = this.shape;
            return [b1, b2, b3, b4, b5, b6, b7, b8, b9];
        }
        else {
            let [b4, b8, b12, b16, b3, b7, b11, b15, b2, b6, b10, b14, b1, b5, b9, b13] = this.shape;
            return [b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16];
        }
    }

    /**
     * Returns the piece array rotated 180 degrees
     */
    rotate_180() {
        if (this.boundWidth == 3) {
            let [b9, b8, b7, b6, b5, b4, b3, b2, b1] = this.shape;
            return [b1, b2, b3, b4, b5, b6, b7, b8, b9];
        }
        else {
            let [b16, b15, b14, b13, b12, b11, b10, b9, b8, b7, b6, b5, b4, b3, b2, b1] = this.shape;
            return [b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16];
        }
    }

/**
 *
 *  Returns a shape array representing the piece's current rotation
 *
 */
    rotate_shape() {
        switch(this.rot) {
            case 1:
                return this.rotate_cw();
            case 2:
                return this.rotate_180();
            case 3:
                return this.rotate_ccw();
            default:
                return this.shape;
        }
    }
}

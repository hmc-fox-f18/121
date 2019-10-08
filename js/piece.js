/* jshint esversion: 6 */
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
        let rot_shape = this.rotate_shape();
        let x = this.x;
        let y = this.y;
        //Determine whether the block is in a 3x3 or 4x4 bounding box
        let bound_width = this.boundWidth;
        for (let block in rot_shape) {
            if (rot_shape[block] == 1) {
                if (x + (block % bound_width) < 0) {
                    return true;
                }
                //TODO: make boardWidth not global
                else if (x + (block % bound_width) >= boardWidth) {
                    return true;
                }
            }
        }
        //TODO: Check for collision with other pieces and floor
        return false;
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
            let [b4, b8, b12, b16, b3, b7, b11, b15, b2, b6, b10, b14, b1, b5, b9, b13] = this.shape;
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
            let [b13, b9, b5, b1, b14, b10, b6, b2, b15, b11, b7, b3, b16, b12, b8, b4] = this.shape;
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

    /**
     *
     * Takes whether the rotation is clockwise as an input, and tries to
     * perform a series of wallkicks to determine what the resulting position
     * from this rotation should be. If no wall kicks are valid
     * positions, the piece will return the original position and
     * rotation.
     *
     * Returns true if the piece can be rotated, and false otherwise
     */
    wallkick(clockwise) {
        let rot = this.rot;
        if (clockwise) {
            this.rot = (this.rot + 1) % 4;
        }
        else {
            this.rot = this.rot - 1;
            this.rot = this.rot == -1 ? 3 : this.rot;
        }

        // No Change, Test 1
        if (!this.collision()) {
            return true;
        }

        let x = this.x;
        let y = this.y;
        if (this.wkTest2(x, y, clockwise, rot)) {
            return true;
        }
        else if (this.wkTest3(x, y)) {
            return true;
        }
        else if (this.wkTest4(x, y, clockwise, rot)) {
            return true;
        }
        else if (this.wkTest5(x, y, clockwise, rot)) {
            return true;
        }

        // All failed, return to original
        this.x = x;
        this.y = y;
        this.rot = rot;
        return false;
    }

    //TODO: Consider having I block test override super class tests
    wkTest2(x, y, clockwise, rot) {
        if (this.boundWidth == 3) {
            let x_test = (this.rot == 2 || this.rot == 3) ? 1 : -1;
            x_test = clockwise ? x_test : -1 * x_test;
            let y_test = 0;
            this.x = x + x_test;
            this.y = y + y_test;
            if (!this.collision()) {
                return true;
            }
        }
        else { //I block rotation
            let x_test = (this.rot % 2) == 1 ? -1 : 1;
            x_test = (this.rot == 1 || this.rot == 3) ||
                    (rot == 1 || rot == 3) ? 2 * x_test : x_test;
            let y_test = 0;
            this.x = x + x_test;
            this.y = y + y_test;
            if (!this.collision()) {
                return true;
            }
        }
    }

    wkTest3(x, y, rot) {
        if (this.boundWidth == 3) {
            let y_test = (this.rot == 1 || this.rot == 3) ? 1 : -1;
            this.y = y + y_test;
            if (!this.collision()) {
                return true;
            }
        }
        else { //I block wallkick
            let x_test = (this.rot % 2) == 1 ? 1 : -1;
            x_test = (this.rot == 1 || this.rot == 3) ||
                    (rot == 1 || rot == 3) ? x_test : 2 * x_test;
            this.x = x + x_test;
            this.y = y;
            if (!this.collision()) {
                return true;
            }
        }
    }

    wkTest4(x, y, clockwise, rot) {
        if (this.boundWidth == 3) {
            let y_test = (this.rot == 1 || this.rot == 3) ? 1 : -1;
            this.x = x;
            this.y = y + 2 * y_test;
            if (!this.collision()) {
                return true;
            }
        }
        else {
            let x_test = (this.rot % 2) == 1 ? -1 : 1;
            x_test = (this.rot == 1 || this.rot == 3) ||
                    (rot == 1 || rot == 3) ? 2 * x_test : x_test;

            let y_test = (this.rot == 1) || (rot == 3) ? 1 : -1;
            y_test = (clockwise && (this.rot == 1 || this.rot == 3) ) ||
                (!clockwise && (this.rot == 0 || this.rot == 2) ) ?
                y_test : 2 * y_test;

            this.x = x + x_test;
            this.y = y + y_test;
            if (!this.collision()) {
                return true;
            }
        }
    }

    wkTest5(x, y, clockwise, rot) {
        if (this.boundWidth == 3) {
            this.x = x + x_test;
            if (!this.collision()) {
                return;
            }
        }
        else {
            let x_test = (this.rot == 1) || (rot == 3) ? -1 : 1;
            x_test = (clockwise && (this.rot == 1 || this.rot == 3) ) ||
                (!clockwise && (this.rot == 0 || this.rot == 2) ) ?
                x_test : 2 * x_test;

            let y_test = (this.rot % 2) == 1 ? 1 : -1;
            y_test = (this.rot == 1 || this.rot == 3) ||
                    (rot == 1 || rot == 3) ? 2 * y_test : y_test;

            this.x = x + x_test;
            this.y = y + y_test;
            if (!this.collision()) {
                return true;
            }
        }
    }
}

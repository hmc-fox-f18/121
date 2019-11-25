/*jshint esversion: 6 */

class BlockState {
  constructor(x, y, original_shape) {
    this.original_shape = original_shape;
    this.x = x;
    this.y = y;
  }
}

// TODO: implement this within our code
class GameState {
  constructor(pieces, piece_queue, player_queue, fallen_blocks, score) {
    this.pieces = pieces;
    this.piece_queue = piece_queue;
    this.player_queue = player_queue;
    this.fallen_blocks = fallen_blocks;
    this.score = score;
  }

  static fromJson(json) {
    const server_state = JSON.parse(json);

    let pieces = server_state.piece_states.map((x) => {
      return PlayerPiece.fromNetworkInfo(
        x.shape,
        x.pivot.x,
        x.pivot.y,
        x.rotation,
        x.player_id);
    });

    let fallen_blocks = server_state.fallen_blocks.map((fallen_block) => {
      return new BlockState(
        fallen_block.position.x,
        fallen_block.position.y,
        fallen_block.original_shape);
    });

    let piece_queue = [0, 4, 5, 6, 1];
    if (server_state.hasOwnProperty('piece_queue')) {
      piece_queue = [...server_state.piece_queue]; // clone an array ES6-style
    }

    let player_queue = [0, 1, 2, 3];
    if (server_state.hasOwnProperty('player_queue')) {
      player_queue = [...server_state.player_queue];
    }

    let score = 0;
    if (server_state.hasOwnProperty('score')) {
      score = server_state.score; // clone an array ES6-style
    }

    return new GameState(pieces, piece_queue, player_queue, fallen_blocks, score);
  }
}

class PieceState {
  constructor(shape, pivot, rotation, player_id) {
    this.shape = shape;
    this.pivot = pivot;
    this.rotation = rotation;
    this.player_id = player_id;
  }
}


class GameState {
  constructor(piece_states) {
    this.piece_states = piece_states;
  }
}

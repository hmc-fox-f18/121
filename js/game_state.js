//The matrices representing the game pieces
const pieceZ = new Piece([ 1, 1, 0, 0, 1, 1, 0, 0, 0], 0,  "#FF5B5B", 0, 0, 0, 3); //0
const pieceS = new Piece([ 0, 1, 1, 1, 1, 0, 0, 0, 0], 1, "#3DE978", 0, 0, 0, 3); //1
const pieceJ = new Piece([ 1, 0, 0, 1, 1, 1, 0, 0, 0], 2, "#3D7AE9", 0, 0, 0, 3); //2
const pieceR = new Piece([ 0, 0, 1, 1, 1, 1, 0, 0, 0], 3, "#FF894E", 0, 0, 0, 3); //3
const pieceT = new Piece([ 0, 1, 0, 1, 1, 1, 0, 0, 0], 4, "#F27DFF", 0, 0, 0, 3); //4
const pieceI = new Piece([ 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0], 5, "#7DFFDC", 0, 0, 0, 4); //5
const pieceO = new Piece([ 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0], 6, "#FFDF92", 0, 0, 0, 4); //6
const shapes = [pieceZ, pieceS, pieceJ, pieceR, pieceT, pieceI, pieceO];

class PieceState {
  constructor(shape, pivot, rotation, player_id) {
    this.shape = shape;
    this.pivot = pivot;
    this.rotation = rotation;
    this.player_id = player_id;
  }

  getPiece() {
    var piece_obj = shapes[this.shape];
    piece_obj.x = this.pivot.x;
    piece_obj.y = this.pivot.y;
    piece_obj.rot = this.rotation;
    return piece_obj;
  }

  collision() {
    return this.getPiece().collision();
  }

  // TODO: this doesn't actually change this.rotation, which doesn't make
  // much sense to me.
  wallkick(clockwise) {
    return this.getPiece().wallkick(clockwise)
  }
}

// TODO: implement this within our code
class GameState {
  constructor(piece_states) {
    this.piece_states = piece_states;
  }

  static fromJson(json) {
    const json_obj = JSON.parse(json);

    let piece_states = json_obj['piece_states'].map((x) => {
      return new PieceState(
        x['shape'],
        x['pivot'],
        x['rotation'],
        x['player_id'])
    });

    return new GameState(piece_states);
  }
}

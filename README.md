# Multiplayer Tetris

Play Tetris -- cooperatively with your friends! Play instantly on the web without any plug-ins.

## Game Controls

 - `LeftArrow`: move left
 - `RightArrow`: move right
 - `UpArrow`: rotate clockwise
 - `z`: rotate counter-clockwise
 - `Space`: hard drop

## Development

Since it's a trademark infringement to distribute this, you have to run it yourself! Do that easily, by following the guide below.

### Requirements
 - cargo 1.39.0-nightly
 - python3

### Startup

First, clone this repo.

 1. Run `python3 -m http.server 8080` from the project root directory to start the web server
 2. run `cargo run` from the `[root dir]/rust` to start the game server

Navigate to `localhost:8080` to play!

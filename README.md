# Multiplayer Tetris

Tetris -- but with your friends! Play instantly on the web, now in our custom-featured-classic-groovy *collaborative mode*.

## How to Play

`LeftArrow`: move left
`RightArrow`: move right
`UpArrow`: rotate clockwise
`z`: rotate counter-clockwise
`Space`: hard drop

## Development

Since it's a trademark infringement to distribute this, you gotta run it yourself! Do that easily, by following the guide below.

### Requirements
Requirements:
 - cargo 1.39.0-nightly
 - python3

### Startup

 1. Run `python3 -m http.server 8080` from the project root directory to start the web server
 2. run `cargo run` from the `[root dir]/rust` to start the game server

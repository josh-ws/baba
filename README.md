# baba

`baba` is a small game, written in Rust, that mimics the mechanics of the popular puzzle game [Baba Is You](https://en.wikipedia.org/wiki/Baba_Is_You) by Arvi Teikari.

All assets and code are original. Nothing from the original game is used and all art is redrawn (poorly) by me.

## Building

`cargo run` from the root. No build is provided yet and assets are relative to the root directory.

## Controls
- WASD or arrow keys for movement
- Z: Undo
- Enter: Enter selected level
- Backspace: Return to the previous map/level
- F5: Reload the level pack from disk
- Escape: Exit the game

## Features 

**Mechanical**:
- A live grid that responds to rule changes.
- Simple sentences for properties and transformations. (Note: `NOT` and conditions unsupported so far.)
- Tiling sprites that update on grid changes (for liquids, paths, etc.)
- Both maps and levels, with correct nesting.
- Hot reload for level packs.
- Full undo stack (press Z to undo last movement).


**Text**:
- `IS` transformations e.g. `BABA IS ROCK`
- `IS` properties e.g. `BABA IS YOU`
- `HAS` e.g. `ROCK HAS KEY`
- `STOP`
- `PUSH`
- `WIN`
- `SELECT` (for the map cursor)
- `SINK`
- `DEFEAT`
- `MOVE`/`AUTO`
- `HOT`/`MELT`
- `AND` for nouns e.g. `BABA AND KEY IS YOU`
- `AND` for properties e.g. `BABA IS YOU AND WIN`
- ...and many nouns.

## License

MIT

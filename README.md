# Game-Saver

![Alt Text](https://github.com/Nukesor/images/blob/master/game_saver.png)

Game-Saver is a small utility which aims to provide multi-slot saves and auto-saving for games that don't provide it out of the box.

The goal is to enable users to easily save and restore multiple versions of a savegame folder.
This tool is designed to extend a game's save logic and not to replace it.

### Features:

- Create new saves on-demand.
- Create autosaves when files in the watched folder change.
- Restore saves
- Support for multiple games.
- Rename and delete saves


### How to use

- Use `CTRL+[h|l|j|k]` or `CTRL+[left|right|up|down]` to navigate the windows.
- `a` to create a new save for the currently selected game.
- `r` to rename a selected savefile.
- `d` to delete a selected savefile.
- `ENTER` to restore a selected savefile.

### Installation guide

**This tool is build for Unix Systems**

1. Install the Rust toolchain, `stable` is ensured to work.
2. Clone and install the game:
    ```sh
    git clone git@github.com:Nukesor/game-saver.git
    cd game-saver
    cargo install --locked --path .
    ```
3. Either copy the binary in `~/.cargo/bin/` somewhere your executable live, or add it to your `$PATH`.


### TODOS:

- Show timestamp floating on the right of the savegame lists
- Add how-to section, which explains how to use it.

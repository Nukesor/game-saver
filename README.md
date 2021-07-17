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


### TODOS:

- Refactor everything once the MVP is finished
    * Think about generics for lists
- Show timestamp floating on the right of the savegame lists
- Add option to define timeout between autosaves. E.g. only save on changes every 10 mins.
- Add how-to section, which explains how to use it.

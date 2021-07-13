# Game-Saver

Game-Saver is a small utility which aims to provide multi-slot saves and auto-saving for games that don't provide it out of the box.

The goal is to enable users to easily save and restore multiple versions of a savegame folder.
This tool is designed to extend a game's save logic and not to replace it.

### Features:

- Create new saves on-demand.
- Create autosaves when files in the watched folder change.
- Restore saves
- Support for multiple games.


### TODOS:

- Build prompts for overwriting and/or renaming files.
- Refactor everything once the MVP is finished
    * Reorganize and restructure event handling
    * Think about generics for lists
    * Think about generics/parameterization for draw functions.
- Write comments in `example_game_saver.toml` and deploy it as default if no config exists yet.
- Function to delete saves (`d` for delete)
- Show timestamp floating on the right of the savegame lists
- Add option to define timeout between autosaves. E.g. only save on changes every 10 mins.
- Add how-to section, which explains how to use it.

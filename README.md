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

- Refactor everything once the MVP is finished
    * Think about generics for lists
- Show timestamp floating on the right of the savegame lists
- Add option to define timeout between autosaves. E.g. only save on changes every 10 mins.
- Add how-to section, which explains how to use it.

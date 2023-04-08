# Bmark
Have you ever opened a terminal, just to spend the next few seconds trying to navigate to the right project folder? No more! Introducing `bmark`, the local bookmark manager. Search through your local bookmarks with a dmenu-like fuzzy finder, and open a terminal right in that directory!

# Features
- Quickly add and manage local bookmarks from the terminal
- Open a terminal emulator directly in a bookmarked location, picked with a dmenu-like fuzzy-finder.
- Navigate to bookmarked locations with shell aliases (see [aliases](#aliases)).

# Commands
Get a quick overview by running with no arguments:
```bash
bmark
```
*Output:*
```
usage: bmark <command>

Commands:
   add [<name>]       add a bookmark to the current working directory
   edit               edit bookmarks in a text editor
   list               list all stored bookmarks
   open               open a new terminal in a bookmarked location
   rm <name>          remove a bookmark with a given name
   config <command>   commands for managing bmark configuration
   update             update shell aliases file
```

### add
Add a bookmark to the current working directory. By default this bookmark will be named the same as to the current directory (ex: "foo/bar" -> "bar"), but you can specify a different name by providing it.

### edit
Edit the `bookmarks.toml` file directly in your editor. The editor is determined by the `editor_cmd` [option](#configuration)  (default is 'nvim').

### list
List the current bookmarks in the terminal.

### open
Launch dmenu-like program, search through bookmarks and open a terminal in the selected location. The terminal and dmenu-like program is determined by the user [configuration](#configuration) (default is 'kitty' and 'dmenu').

### rm
Remove a bookmark by its name.

### update
Update the aliases file (see [alases](#Aliases)). 

# Aliases
`bmark` automatically created a file called `aliases.sh` in the data directory. This file defines shell aliases, that navigate to all your bookmarks.

If you you have a bookmark called "myMark" you can navigate it like this:
```bash
-myMark
```
All the bookmark aliases are prefixed with '_' by default. This can be changed with the `alias_prefix` [option](#configuration). 

To enable this in your shell, you need to source the alias file from your shell configuration file (ex: `.bashrc` for `.zshrc`).

# Configuration
`bmark` configuration is done using the configuration file. This file is located in the configuration directory (ex: `~/.config/bmark/config.toml`). Edit the configuration file easily with this command:
```bash
bmark config edit
```
To show the current config, run this command:
```bash
bmark config show
```

### List of Configuration Values

| Option       | Default Value                   |
| ------------ | ------------------------------- |
| data_dir     | "~/.config/bmark/config.toml"   |
| dmenu_cmd    | "dmenu"                         |
| editor_cmd   | "nvim"                          |
| terminal_cmd | "kitty --detach"                |
| alias_prefix | "_"                             |
| display_sep  | " : "                           |

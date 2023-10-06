# Bmark
Have you ever opened a terminal, just to spend the next few seconds navigating to the right folder? No more! Introducing `bmark`, the local bookmark manager. Search through your local bookmarks with a dmenu-like fuzzy finder, and open a terminal right in that directory!

# Features
- Quickly add and manage local bookmarks from the terminal
- Open a terminal emulator directly in a bookmarked location, picked with a dmenu-like fuzzy-finder.
- Navigate to bookmarked locations with shell aliases (see [aliases](#aliases)).

# Dependencies
This tool has been tested on fedora linux and nixos, and should work on all linux distros. MacOs and Windows are currently not supported, but may be implemented in the future.

For this tool to work out of the box, you will need the following installed on your system:
- [rofi](https://github.com/davatorium/rofi)
- [kitty](https://sw.kovidgoyal.net/kitty/)
- [neovim](https://neovim.io/)
- posix shell like [bash](https://www.gnu.org/software/bash/) or [zsh](https://zsh.sourceforge.io/)

Even though these programs are the default, **they are not strictly required** as they can be swapped out for similar ones using the [configuration file](#configuration).

# Quick Start
From zero to `bmark`.

Install cargo and the default programs.

#### Fedora
```bash
sudo dnf install cargo rofi kitty neovim
```
Now install `bmark`.
```bash
cargo install bmark
```
If you have not already, add the `~/.cargo/bin` folder to your `$PATH`. This can be done by adding this line to your `~/.bashrc` or `~/.zshrc`:
```bash
PATH=$HOME/.cargo/bin:$PATH
```
You should now be able to run `bmark` in your terminal!

To enable bmark's navigation aliases, you also need to source the aliases.sh file. The command for doing this can be automatically added to your shell config like this:

```bash
# For bash:
bmark config source-cmd >> ~/.bashrc && bash

# For zsh: 
bmark config source-cmd >> ~/.zshrc && zsh
```
All done!

#### Nixos
Derivation can be found [here](https://github.com/BalderHolst/nix-hyprland-config/blob/main/pkgs/bmark.nix).

After installation, run the following command:

```
bmark config source-cmd
```

This will output a line of bash/zsh, that should be put in your `.zshrc` or `.bashrc`. This is required to use the bookmark aliases.

# Commands
Get a quick overview by running with no arguments:

```bash
bmark
```

*Output:*
```
Optional arguments:
  -h, --help     print help message
  -v, --verbose  be verbose

Subcommands:
  add     add a bookmark to the current working directory
  edit    edit bookmarks in a text editor
  list    list all stored bookmarks
  open    open a new terminal in a bookmarked location
  rm      remove a bookmark with a given name
  config  commands for managing bmark configuration
  update  update shell aliases file
```

### add
Add a bookmark to the current working directory. By default this bookmark will be named the same as to the current directory (ex: "foo/bar" -> "bar"), but you can specify a different name by providing it.

### edit
Edit the `bookmarks.toml` file directly in your editor. The editor is determined by the `editor_cmd` [option](#configuration)  (default is 'nvim').

### list
List the current bookmarks in the terminal.

### open
Launch dmenu-like program, search through bookmarks and open a terminal in the selected location. The terminal and dmenu-like program is determined by the user [configuration](#configuration) (default is 'kitty' and 'rofi').

### rm
Remove a bookmark by its name.

### Config
Commands for managing configuration.

See [configuration](#Configuration).

### update
Update the aliases file (see [aliases](#Aliases)). This is done automatically whenever you alter your bookmarks file through. This command is only usefull if you manually open and the `bookmarks.toml` file without using `bmark edit`. The command for doing this can be generated like this:
```bash
bmark config source-cmd
```

# Aliases
`bmark` automatically creates a file called `aliases.sh` in the data directory. This file defines shell aliases, that navigate to all your bookmarks.

If you you have a bookmark called "myMark" you can navigate to it like this:
```bash
_myMark
```
All the bookmark aliases are prefixed with '_' by default. This can be changed with the `alias_prefix` [option](#configuration). 

To enable this in your shell, you need to source the alias file from your shell configuration file (ex: `.bashrc` for `.zshrc`).

# Configuration
`bmark` configuration is done using the configuration file. This file is located in the configuration directory (ex: `~/.config/bmark/config.toml`). To create a config file with the default values run the following command:

```bash
bmark config create
```

Edit the configuration file easily with this command:

```bash
bmark config edit
```

To show the current config, run this command:

```bash
bmark config show
```

### List of Configuration Values

| Option                        | Default Value                   |
| ----------------------------- | ------------------------------- |
| [data_dir](#data_dir)         | "~/.local/share/bmark"   |
| [dmenu_cmd](#dmenu_cmd)       | "rofi -matching fuzzy -dmenu"                   |
| [editor_cmd](#editor_cmd)     | "nvim"                          |
| [terminal_cmd](#terminal_cmd) | "kitty --detach"                |
| [alias_prefix](#alias_prefix) | "_"                             |
| [show_paths](#show_paths)   | false
| [display_sep](#display_sep)   | " : "                           |

## Description of Values

### data_dir
The directory where the `bookmarks.toml` and `aliases.sh` files are stored.

### dmenu_cmd
The dmenu-like command is used for fuzzy-finding through bookmarks. This program should (like [dmenu](https://tools.suckless.org/dmenu/)) take input from a pipe, and output the selected line to stdout. To check if a program is suitable for this you can run the following:
```bash
seq 20 | <dmenu_cmd>
```
This should give you a menu with the numbers from 1 to 20, when you pick one, it should be output in the terminal.

If your command works, you can make bmark use that by adding a like to the '~/.config/bmark/config.toml' file:
```toml
dmenu_cmd = "<dmenu_cmd>"
```
The default command is `rofi -dmenu`.

#### Using Actual Dmenu
If you want to use the actual dmenu instead of rofi, simply add this to your [config](#configuration) .
```toml
dmenu_cmd = "dmenu"
```
(be sure to have dmenu installed)

### editor_cmd
The editor command is run whenever you ask bmark to edit a file (ex: `bmark edit`). This command should be able to be used like this:
```bash
<editor_cmd> <file>
```
For this a terminal based editor like [neovim](https://neovim.io/) is recommended.

### terminal_cmd
The terminal command is the command used for spawning terminal emulators at the desired locations. This command should be able to spawn a terminal in the root directory like this:
```bash
<terminal_cmd> /
```
If you are not using the `kitty` terminal, you should probably change this.

### alias_prefix
The prefix in front of bookmark names for generated aliases.

### show_paths
Whether or not, to show paths in the dmenu-like fuzzy finder. If false, the `display_sep` has no effect.

### display_sep
The characters separating the bookmark names from their paths when listing or searching through your bookmarks.

use std::fmt;
use serde::Deserialize;
use std::collections::HashMap;
use std::{env, fs};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::process::{exit, Command};

static BOOKMARKS_FILE: &str = "bookmarks.toml";
static ALIAS_FILE: &str = "aliases.sh";
static BOOKMARKS_SEP: &str = " = ";

struct Bookmarks {
    file: PathBuf,
}

impl Bookmarks {
    fn from(path: PathBuf) -> Bookmarks {
        Bookmarks { 
            file: path 
        }
    }
    fn from_config(config: &Config) -> Bookmarks {
        Bookmarks { 
            file: config.get_bookmarks_file() 
        }
    }
    fn get_raw(&self) -> String {
        let mut contents = String::new();
        match File::open(&self.file) {
            Ok(mut file) => {
                if let Err(_) = file.read_to_string(&mut contents) {
                        eprintln!("ERROR: opened, but could not read from bookmarks file: `{}`", self.file.display());
                        exit(1);
                    }
                },
            Err(_) => {
                eprintln!("ERROR: could not open bookmarks file: `{}`", self.file.display());
                exit(1);
            }
        }
        contents
    }
    fn get_map(&self) -> HashMap<String, String> {
        match toml::from_str(&self.get_raw()) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("{e}");
                    eprintln!("ERROR: Could not parse bookmarks file: `{}`", self.file.display());
                    exit(1);
            },
        }
    }
    fn readable(&self) -> String {
        let mut res = String::new();
        let config = Config::get_user_config();
        let map = self.get_map();
        let mut max_len: usize = 0;
        for l in map.keys() {
            if l.len() > max_len {
                max_len = l.len();
            }
        }
        for (k, v) in map {
            let mut padding = "".to_string();
            for _ in 0..(max_len - k.len()) { padding.push(' ') }
            res += format!("{}{}{}{}\n", k, padding, config.display_sep, v).as_str();
        }
        res
    }
}

impl fmt::Display for Bookmarks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.readable())?;
        Ok(())
    }
}


// User defined config. These options get merged with the defaults for Config.
#[derive(Deserialize)]
struct UserConfig {
    data_dir: Option<String>,
    editor_cmd: Option<String>,
    dmenu_cmd: Option<String>,
    terminal_cmd: Option<String>,
    alias_prefix: Option<String>,
    display_sep: Option<String>,
}

impl UserConfig {
    fn empty() -> UserConfig {
        UserConfig {  
            data_dir: None,
            editor_cmd: None,
            dmenu_cmd: None,
            terminal_cmd: None,
            alias_prefix: None,
            display_sep: None,
        }
    }
}

// The configuration used by the progmem.
struct Config {
    data_dir: String,
    editor_cmd: String,
    dmenu_cmd: String,
    terminal_cmd: String,
    alias_prefix: String,
    display_sep: String
}

impl Config {
    fn default() -> Config {
        Config { 
            data_dir: "/home/balder/.local/share/bmark".to_string(),
            editor_cmd: "nvim".to_string(),
            dmenu_cmd: "dmenu".to_string(),
            terminal_cmd: "kitty --detach".to_string(),
            alias_prefix: "_".to_string(),
            display_sep: " : ".to_string(),
        }
    }
    fn user_config_file() -> String {
        "/home/balder/.config/bmark/config.toml".to_string()
    }
    fn get_user_config() -> Config {
        let config_file = Config::user_config_file();
        match File::open(&config_file) {
            Ok(mut file) => {
                let mut lines = String::new();
                if let Err(_) = file.read_to_string(&mut lines){
                    eprintln!("ERROR: Can not read lines from file: `{}`", config_file);
                    exit(1);
                }
                let uc: UserConfig = match toml::from_str(lines.as_str()) {
                    Ok(val) => val,
                    Err(e) => {
                        eprintln!("{e}");
                        eprintln!("WARNING: could not parse `config.toml` file. Using default settings.");
                        UserConfig::empty()
                    }
                };
                let mut c = Config::default();
                if let Some(data_dir) = uc.data_dir { c.data_dir = data_dir }
                if let Some(editor_cmd) = uc.editor_cmd { c.editor_cmd = editor_cmd }
                if let Some(dmenu_cmd) = uc.dmenu_cmd { c.dmenu_cmd = dmenu_cmd }
                if let Some(terminal_cmd) = uc.terminal_cmd { c.terminal_cmd = terminal_cmd }
                if let Some(alias_prefix) = uc.alias_prefix { c.alias_prefix = alias_prefix }
                if let Some(display_sep) = uc.display_sep { c.display_sep = display_sep }
                c
            },
            Err(_) => {
                Config::default()
            },
        }
    }
    fn get_bookmarks_file(&self) -> PathBuf {
        let mut bookmarks_file = PathBuf::from(&self.data_dir);
        bookmarks_file.push(BOOKMARKS_FILE);
        bookmarks_file
    }
    fn get_alias_file(&self) -> PathBuf {
        let mut bookmarks_file = PathBuf::from(&self.data_dir);
        bookmarks_file.push(ALIAS_FILE);
        bookmarks_file
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n{}\n{}\n{}", 
               self.data_dir, 
               self.dmenu_cmd,
               self.editor_cmd,
               self.terminal_cmd,
               )
    }
}

fn bmark_config_usage () {
    eprintln!("Usage: bmark config <subcommand>\n");
    eprintln!("Commands:");
    eprintln!("    show         Show the current configuration");
    eprintln!("    edit         Edit the configuration file");
}

fn bmark_config(subcommand: String) {
    let config = Config::get_user_config();
    match subcommand.as_str() {
        "show" => { // TODO: show options
            println!("{}", config);
        },
        "create" => {
            assert!(false, "Not Implemented") // TODO
        }
        "edit" => {
            let editor_cmd = config.editor_cmd + " " + Config::user_config_file().as_str();
            Command::new("sh")
                .arg("-c")
                .arg(editor_cmd)
                .status()
                .expect("ERROR: Failed to execute editor command.");
        }
        _ => {
            eprintln!("ERROR: not subcommand called `{}`\n", subcommand);
            bmark_config_usage();
        }
    }
}

fn bmark_add(name: Option<String>) {
    let bookmarks_file = Config::get_user_config().get_bookmarks_file();

    let data_dir = bookmarks_file.parent()
        .expect("Found no parrent to bookmarks file.");

    if !data_dir.exists() {
        match fs::create_dir_all(data_dir) {
            Ok(_) => {},
            Err(e) => {
                eprint!("ERROR: Could not create data directory `{}`", e);
                exit(1);
            }
        };
    }


    match OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&bookmarks_file)
    {
        Ok(mut file) => {
            let cwd = env::current_dir().unwrap();
            if let Err(_) = match name {
                Some(n) => writeln!(file, "{}{}\"{}\"", n, BOOKMARKS_SEP, cwd.display()), 
                None => {
                    let stem = cwd.file_stem().unwrap();
                    writeln!(file, "{}{}\"{}\"", stem.to_str().unwrap(), BOOKMARKS_SEP, cwd.display())
                },
            } {
                eprintln!("ERROR: Could not write to file: {}", bookmarks_file.display());
                exit(1);
            }
        }
        Err(_) => {
            eprintln!("ERROR: Could not open bookmarks file: `{}`", bookmarks_file.display());
            exit(1);
        }
    }
    bmark_update();
}

fn bmark_edit() {
    let config = Config::get_user_config();
    let path = config.get_bookmarks_file();
    let editor_cmd = config.editor_cmd + " " + path.to_str().unwrap();
    Command::new("sh")
        .arg("-c")
        .arg(editor_cmd)
        .status()
        .expect("ERROR: Failed to execute editor command.");
    bmark_update();
}

fn bmark_list() {
    let config = Config::get_user_config();
    let bookmarks = Bookmarks::from_config(&config);
    print!("{}", bookmarks);
}

fn bmark_open(){
    let config = Config::get_user_config();
    let bookmarks = Bookmarks::from_config(&config);
    let cmd = "echo '".to_owned() + bookmarks.readable().as_str()+ "'" + " | " + config.dmenu_cmd.as_str();
    let mut path = match Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
    {
        Ok(output) => {
            let choice = String::from_utf8(output.stdout).unwrap();
            if choice == "" {
                eprintln!("No bookmark chosen.");
                exit(1);
            }
            let mut split = choice.split(&config.display_sep);
            split.next();
            match split.next() { // TODO: fix with .remainder()
                Some(p) => p.to_owned(),
                None => {
                    eprintln!("ERROR: Could not parse line bookmark: `{}`", choice);
                    exit(1);
                }
            }
        },
        Err(_) => {
            eprintln!("ERROR: Error running dmenu command: `{}`", cmd);
            exit(1);
        }
    };
    path.pop(); // Remove newline
    let cmd = config.terminal_cmd + " \"" + path.as_str() + "\"";
    println!("{cmd}");

    if let Err(_) = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .status()
    {
        eprintln!("ERROR: Could not open terminal with this command: `{}`", cmd);
        exit(1);
    }
}

fn bmark_rm(bmark: String){

    let config = Config::get_user_config();
    let bookmarks = Bookmarks::from_config(&config);
    let mut bookmarks_str = String::new();
    let mut removed = false;

    for (k, v) in bookmarks.get_map() {
        if k == bmark { 
            removed = true;
            continue;
        }
        bookmarks_str += format!("{}{}{}\n", k, BOOKMARKS_SEP, v).as_str();
    }
    if !removed {
        eprintln!("ERROR: could not find bookmark `{}`.", bmark);
        exit(1);
    }
    let bytes = bookmarks_str.as_bytes();
    match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(config.get_bookmarks_file())
    {
        Ok(mut file) => {
            match file.write_all(bytes) {
                Ok(_) => { },
                Err(_) => {
                    eprintln!("ERROR: Could not open bookmarks file");
                    exit(1);
                }
            }
        },
        Err(_) => {
            eprintln!("ERROR: Could not open bookmarks file");
            exit(1);
        }
    }
}

fn bmark_update(){
    let config = Config::get_user_config();
    let bookmarks = Bookmarks::from(config.get_bookmarks_file());
    let mut aliases = String::new();
    for (name, path) in bookmarks.get_map() {
        aliases += format!("alias {}{}=\"{}\"\n", config.alias_prefix, name, path).as_str();
    }
    let bytes = aliases.as_bytes();
    match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open( Config::get_user_config().get_alias_file())
    {
        Ok(mut file) => {
            match file.write_all(bytes) {
                Ok(_) => { },
                Err(e) => {
                    eprintln!("{e}");
                    eprintln!("ERROR: Could not write to aliases file");
                    exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("{e}");
            eprintln!("ERROR: Could not open aliases file");
            exit(1);
        }
    }
}

fn usage() {
    println!("usage: bmark <command>\n"                                             );
    println!("Commands:"                                                            );
    println!("   add [<name>]       add a bookmark to the current working directory");
    println!("   edit               edit bookmarks in a text editor"                );
    println!("   list               list all stored bookmarks"                      );
    println!("   open               open a new terminal in a bookmarked location"   );
    println!("   rm <name>          remove a bookmark with a given name"            );
    println!("   config <command>   commands for managing bmark configuration"      );
    println!("   update             update shell aliases file"                      );
}

fn main() {

    let mut args: Vec<String> = Vec::new();

    for argument in env::args() {
        args.push(argument);
    }

    if args.len() <= 1 {
        usage();
        exit(1);
    }

    let cmd = args[1].as_str();

    match cmd {
        "add" => {
            if args.len() == 3 {
                bmark_add(Some(args[2].clone()))
            }
            else if args.len() == 2 {
                bmark_add(None)
            }
            else {
                eprintln!("ERROR: Add commands takes zero or one argument.\n");
                usage();
                exit(1);
            }
        },
        "edit" => bmark_edit(),
        "list" => bmark_list(),
        "open" => bmark_open(),
        "rm" => {
            if args.len() < 3 {
                eprintln!("ERROR: Please provide a bookmark to remove.\n");
                usage();
                exit(1);
            }
            bmark_rm(args[2].clone())
        },
        "update" => bmark_update(),
        "config" => {
            if args.len() < 3 {
                eprintln!("ERROR: Please provide a subcommand.\n");
                bmark_config_usage();
                exit(1);
            }
            bmark_config(args[2].clone());
        },
        _ => {
            eprintln!("ERROR: command `{}` not known.\n", cmd);
            usage();
        }
    }
}

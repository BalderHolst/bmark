use std::fmt;
use serde::Deserialize;
use std::collections::HashMap;
use std::{env, fs};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::process::{exit, Command};

static BOOKMARKS_FILE: &str = "bookmarks.txt";
static ALIAS_FILE: &str = "aliases.sh";

fn get_bookmark_map(config: &Config) -> HashMap<String, String> {
    let mut hmap: HashMap<String, String> = HashMap::new();

    for line in get_bookmarks(config).split("\n") {
        let mut parts = line.split(" - ");
        let name = parts.next().unwrap();
        let path = match parts.next() {
            Some(p) => p,
            None => {
                if line != "" { eprintln!("WARNING: Could not parse bookmark: `{}`. Skipping.", line) };
                continue;
            }
        };
        hmap.insert(name.to_owned(), path.to_owned());
    }
    hmap
}

fn get_bookmarks(config: &Config) -> String {
    let bookmarks_file = config.get_bookmarks_file();
    let mut contents = String::new();
    match File::open(&bookmarks_file) {
        Ok(mut file) => {
            match file.read_to_string(&mut contents) {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("ERROR: opened, but could not read from bookmarks file: `{}`", bookmarks_file.display());
                    exit(1);
                }
            }
        }
        Err(_) => {
            eprintln!("ERROR: could not open bookmarks file: `{}`", bookmarks_file.display());
            exit(1);
        }
    }
    contents
}

fn usage() {
    println!("usage: bmark <command>\n"                                          );
    println!("Commands:"                                                         );
    println!("   add [<name>]    add a bookmark to the current working directory");
    println!("   edit            edit bookmarks in a text editor"                );
    println!("   list            list all stored bookmarks"                      );
    println!("   open            open a new terminal in a bookmarked location"   );
    println!("   rm <name>       remove a bookmark with a given name"            );
    println!("   update          update shell aliases file"                      );
}

// User defined config. These options get merged with the defaults for Config.
#[derive(Deserialize)]
struct UserConfig {
    data_dir: Option<String>,
    editor_cmd: Option<String>,
    dmenu_cmd: Option<String>,
    terminal_cmd: Option<String>,
    alias_prefix: Option<String>,
}

impl UserConfig {
    fn empty() -> UserConfig {
        UserConfig {  
            data_dir: None,
            editor_cmd: None,
            dmenu_cmd: None,
            terminal_cmd: None,
            alias_prefix: None,
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
}

impl Config {
    fn default() -> Config {
        Config { 
            data_dir: "/home/balder/.local/share/bmark".to_string(),
            editor_cmd: "nvim".to_string(),
            dmenu_cmd: "dmenu".to_string(),
            terminal_cmd: "kitty".to_string(),
            alias_prefix: "_".to_string(),
        }
    }
    fn get_user_config() -> Config {
        let config_path = PathBuf::from("/home/balder/.config/bmark/config.toml");
        match File::open(&config_path) {
            Ok(mut file) => {
                let mut lines = String::new();
                if let Err(_) = file.read_to_string(&mut lines){
                    eprintln!("ERROR: Can not read lines from file: `{}`", config_path.display());
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

fn bmark_config() {
    let config = Config::get_user_config();

    println!("{}", config);
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
                Some(n) => writeln!(file, "{} - \"{}\"", n, cwd.display()), 
                None => {
                    let stem = cwd.file_stem().unwrap();
                    writeln!(file, "{} - \"{}\"", stem.to_str().unwrap(), cwd.display())
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
    print!("{}", get_bookmarks(&config));
}

fn bmark_open(){
    let config = Config::get_user_config();
    let cmd = "echo '".to_owned() + get_bookmarks(&config).as_str()+ "'" + " | " + config.dmenu_cmd.as_str();
    let path = match Command::new("sh")
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
            let mut split = choice.split(" - ");
            split.next();
            match split.next() {
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

    let cmd = config.terminal_cmd + " " + path.as_str();

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

    let mut bookmarks_str = String::new();
    let mut removed = false;

    for (k, v) in get_bookmark_map(&config) {
        if k == bmark { 
            removed = true;
            continue;
        }
        bookmarks_str += format!("{} - {}\n", k, v).as_str();
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
    let bookmarks = get_bookmarks(&config);
    let mut aliases = String::new();
    for line in bookmarks.split("\n") {
        let mut parts = line.split(" - ");
        let name = parts.next().unwrap();
        let path = match parts.next() {
            Some(p) => p,
            None => {
                if line != "" { eprintln!("WARNING: Could not parse bookmark: `{}`. Skipping.", line) };
                continue;
            }
        };
        aliases += format!("alias _{}={}\n", name, path).as_str();
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
                Err(_) => {
                    eprintln!("ERROR: Could not open aliases file");
                    exit(1);
                }
            }
        },
        Err(_) => {
            eprintln!("ERROR: Could not open aliases file");
            exit(1);
        }
    }
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
        "config" => bmark_config(),
        _ => {
            eprintln!("ERROR: command `{}` not known.\n", cmd);
            usage();
        }
    }
}

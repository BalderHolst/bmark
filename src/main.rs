mod cli;

use gumdrop::Options;
use fuzzy_finder::{item::Item, FuzzyFinder};
use std::{fmt, io};
use std::collections::{HashMap, BTreeMap};
use std::{env, fs};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::process::{exit, Command};
use directories::ProjectDirs;

static BOOKMARKS_FILE: &str = "bookmarks.toml";
static ALIAS_FILE: &str = "aliases.sh";

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
    fn get_map(&self) -> BTreeMap<String, String> {
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
            res += format!("{}{}{}{}\n", k, padding, config.get_display_sep(), v).as_str();
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


struct Config {
    map: HashMap<String, toml::Value>,
}

impl Config {
    fn get_data_dir(&self) -> PathBuf {
        match &self.map.get("data_dir") {
            Some(toml::Value::String(str)) => PathBuf::from(str.to_string()),
            _ => {
                match ProjectDirs::from("com", "bmark",  "bmark") {
                    Some(proj_dirs) => PathBuf::from(proj_dirs.data_dir()),
                    None => {
                        eprintln!("ERROR: could not determine data directory");
                        exit(1);
                    }
                }
            },
        }
    }
    fn get_editor_cmd(&self) -> String {
        match &self.map.get("editor_cmd") {
            Some(toml::Value::String(str)) => str.to_string(),
            _ => "nvim".to_string(),
        }
    }
    fn get_dmenu_cmd(&self) -> String {
        match &self.map.get("dmenu_cmd") {
            Some(toml::Value::String(str)) => str.to_string(),
            _ => "rofi -dmenu".to_string(),
        }
    }
    fn get_terminal_cmd(&self) -> String {
        match &self.map.get("terminal_cmd") {
            Some(toml::Value::String(str)) => str.to_string(),
            _ => "kitty --detach -d".to_string(),
        }
    }
    fn get_alias_prefix(&self) -> String {
        match &self.map.get("alias_prefix") {
            Some(toml::Value::String(str)) => str.to_string(),
            _ => "_".to_string(),
        }
    }
    fn get_display_sep(&self) -> String {
        match &self.map.get("get_display_sep") {
            Some(toml::Value::String(str)) => str.to_string(),
            _ => " : ".to_string(),
        }
    }

    fn user_config_file() -> PathBuf {
        match ProjectDirs::from("com", "bmark",  "bmark") {
            Some(proj_dirs) => PathBuf::from(proj_dirs.config_dir()).join("config.toml"),
            None => {
                eprintln!("ERROR: could not determine config directory");
                exit(1);
            }
        }
    }
    fn default() -> Config {
        Config { map: HashMap::new() }
    } fn get_user_config() -> Config {
        let config_file = Config::user_config_file();
        let mut m: HashMap<String, toml::Value> = Default::default();
        match File::open(&config_file) {
            Ok(mut file) => {
                let mut lines = String::new();
                if let Err(_) = file.read_to_string(&mut lines){
                    eprintln!("ERROR: Can not read lines from file: `{}`", config_file.display());
                    exit(1);
                }
                let user_config: HashMap<String, toml::Value> = toml::from_str(lines.as_str()).unwrap();
                for (k, v) in user_config.iter() {
                    m.insert(k.to_owned(), v.to_owned());
                }
                Config { map: m }
            },
            Err(_) => {
                Config { map: m }
            },
        }
    }
    fn get_bookmarks_file(&self) -> PathBuf {
        let mut bookmarks_file = PathBuf::from(&self.get_data_dir());
        bookmarks_file.push(BOOKMARKS_FILE);
        bookmarks_file
    }
    fn get_alias_file(&self) -> PathBuf {
        let mut bookmarks_file = PathBuf::from(&self.get_data_dir());
        bookmarks_file.push(ALIAS_FILE);
        bookmarks_file
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "data_dir = \"{}\"
dmenu_cmd = \"{}\"
editor_cmd = \"{}\"
terminal_cmd = \"{}\"
alias_prefix = \"{}\"
display_sep = \"{}\"", 
               self.get_data_dir().display(), 
               self.get_dmenu_cmd(),
               self.get_editor_cmd(),
               self.get_terminal_cmd(),
               self.get_alias_prefix(),
               self.get_display_sep(),
               )
    }
}

fn bmark_config_usage () {
    eprintln!("Usage: bmark config <subcommand>\n");
    eprintln!("Commands:");
    eprintln!("    show         Show the current configuration");
    eprintln!("    edit         Edit the configuration file");
}

// Add: source_cmd subcommand to output the command to source the alias file
fn bmark_config(subcommand: cli::ConfigCommand) -> BmarkResult {
    let config = Config::get_user_config();
    match subcommand {
        cli::ConfigCommand::Show(_) => {
            let config_file = Config::user_config_file();
            if !config_file.exists() {
                println!("No config file found at `{}`. Create one by running `bmark config create`", config_file.display());
            }
        },
        cli::ConfigCommand::Create(_) => {
            let config_file = Config::user_config_file();
            let config = Config::default();
            if config_file.exists() {
                eprintln!("ERROR: Cannot create default config file, a config file already exists at `{}`.", 
                          config_file.display());
                exit(1);
            }
            fs::create_dir_all(config_file.parent().expect("No parrent of config file."))
                .expect("Could not create config directory.");
            match OpenOptions::new()
                .write(true)
                .create(true)
                .open(&config_file)
            {
                Ok(mut file) => {
                    let buf = format!("data_dir = \"{}\"\ndmenu_cmd = \"{}\"\neditor_cmd = \"{}\"\nterminal_cmd = \"{}\"\nalias_prefix = \"{}\"\ndisplay_sep = \"{}\"", 
                        config.get_data_dir().display(), 
                        config.get_dmenu_cmd(),
                        config.get_editor_cmd(),
                        config.get_terminal_cmd(),
                        config.get_alias_prefix(),
                        config.get_display_sep(),
                    );

                    if let Err(e) = file.write_all(buf.as_bytes()) {
                        eprintln!("ERROR: Could not write to config file : {e}");
                    }
                }
                Err(_) => {
                    eprintln!("ERROR: Could not open config file: `{}`", config_file.display());
                    exit(1);
                }
            }
        }
        cli::ConfigCommand::Edit(_) => {
            let path = Config::user_config_file();

            if !path.exists() {
                fs::create_dir_all(path.parent().unwrap()).unwrap();
            }
            let editor_cmd = config.get_editor_cmd() + " " + path.to_str().unwrap();
            Command::new("sh")
                .arg("-c")
                .arg(editor_cmd)
                .status()
                .expect("ERROR: Failed to execute editor command.");
        }
        cli::ConfigCommand::SourceCmd(_) => {
            println!("source \"{}/{}\"", 
                     Config::get_user_config().get_data_dir().display(), 
                     ALIAS_FILE,
                     );
            exit(0);
        }
    }

    Ok(())
}

// Make sure that there are no duplicates
fn bmark_add(name: Option<String>) -> BmarkResult {
    let bookmarks = Bookmarks::from_config(&Config::get_user_config());
    let bookmarks_file = bookmarks.file.clone();

    let data_dir = bookmarks_file.parent()
        .expect("There should always be a parrent in the path.");

    if !data_dir.exists() {
        if let Err(e) = fs::create_dir_all(data_dir) {
            return Err(BmarkError::IoExplained(format!("ERROR: Could not create data directory: `{}`", e)));
        };
    }

    let cwd = env::current_dir().unwrap();
    let mut bmark_name = match name {
        Some(n) => n,
        None => cwd.file_stem().unwrap().to_str().unwrap().to_string(),
    };

    if bookmarks.get_map().contains_key(&bmark_name) {
        return Err(BmarkError::IoExplained(format!("A bookmark with the name '{bmark_name}' already exists.")));
    }

    if bmark_name.rfind(' ') != None {
        eprintln!("WARNING: Bookmarks with spaces cannot be accesed through aliases. Added it anyway.");
        bmark_name = "\"".to_string() + bmark_name.as_str() + "\"";
    }
    
    match OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&bookmarks_file)
    {
        Ok(mut file) => {
            let cwd = env::current_dir().unwrap();
            if let Err(_) = writeln!(file, "{} = \"{}\"", bmark_name, cwd.display()) {
                eprintln!("ERROR: Could not write to file: {}", bookmarks_file.display());
                exit(1);
            }
        }
        Err(_) => {
            eprintln!("ERROR: Could not open bookmarks file: `{}`", bookmarks_file.display());
            exit(1);
        }
    }
    bmark_update()
}

fn bmark_edit() -> BmarkResult {
    let config = Config::get_user_config();
    let path = config.get_bookmarks_file();
    let editor_cmd = config.get_editor_cmd() + " " + path.to_str().unwrap();
    if let Err(e) = Command::new("sh")
        .arg("-c")
        .arg(editor_cmd)
        .status()
    {
        return Err(BmarkError::IoExplained(format!("ERROR: Failed to execute editor command:\n{e}")));
    }
    bmark_update()
}

fn bmark_list() -> BmarkResult {
    let config = Config::get_user_config();
    let bookmarks = Bookmarks::from_config(&config);
    print!("{}", bookmarks);
    Ok(())
}

fn bmark_open() -> BmarkResult {
    let config = Config::get_user_config();
    let bookmarks = Bookmarks::from_config(&config);
    let mut items: Vec<Item<String>> = Vec::new();
    for (k, v) in bookmarks.get_map().iter() {
        items.push(Item::new(k.to_owned(), v.to_owned()));
    }
    let path = match FuzzyFinder::find(items, 8) {
        Ok(s) => {
            match s {
                Some(s) => s,
                None => exit(0), // If nothing was selected
            }
        },
        Err(_) => {
            return Err(BmarkError::IoExplained("No selected bookmark.".to_string()))
        }
    };
    println!("");

    Command::new("zsh")
            .current_dir(path)
            .status()
            .expect("bash command failed to start");
    
    Ok(())
}

// TODO: Check that dmenu-like program is executable
// TODO: Support fzf
fn bmark_dmenu(){
    let config = Config::get_user_config();
    let bookmarks = Bookmarks::from_config(&config);
    let cmd = "echo '".to_owned() + bookmarks.readable().as_str()+ "'" + " | " + config.get_dmenu_cmd().as_str();
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
            let sep = config.get_display_sep();
            let mut split = choice.split(&sep);
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
            eprintln!("ERROR: Error running dmenu-command: `{}`", cmd);
            exit(1);
        }
    };
    path.pop(); // Remove newline
    let cmd = config.get_terminal_cmd() + " \"" + path.as_str() + "\"";

    if let Err(_) = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .status()
    {
        eprintln!("ERROR: Could not open terminal with this command: `{}`", cmd);
        exit(1);
    }
}

fn bmark_rm(bmark: String) -> BmarkResult {

    let config = Config::get_user_config();
    let bookmarks = Bookmarks::from_config(&config);
    let mut bookmarks_str = String::new();
    let mut removed = false;

    for (mut k, v) in bookmarks.get_map() {
        if k == bmark { 
            removed = true;
            continue;
        }
        if k.rfind(' ') != None { k = "\"".to_string() + k.as_str() + "\"" } 
        bookmarks_str += format!("{} = \"{}\"\n", k, v).as_str();
    }
    if !removed {
        return Err(BmarkError::IoExplained(format!("ERROR: could not find bookmark `{}`.", bmark)))
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
                Ok(_) => {},
                Err(_) => {
                    return Err(BmarkError::IoExplained("ERROR: Could not open bookmarks file".to_string()))
                }
            }
        },
        Err(_) => {
            return Err(BmarkError::IoExplained("ERROR: Could not open bookmarks file".to_string()))
        }
    }
    Ok(())
}

fn bmark_update() -> BmarkResult {
    let config = Config::get_user_config();
    let bookmarks = Bookmarks::from(config.get_bookmarks_file());
    let mut aliases = String::new();
    for (name, path) in bookmarks.get_map() {
        if name.rfind(' ') != None { continue; } // Skip bookmark names with spaces
        aliases += format!("alias {}{}='cd \"{}\"'\n", config.get_alias_prefix(), name, path).as_str();
    }
    let bytes = aliases.as_bytes();
    match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open( config.get_alias_file())
    {
        Ok(mut file) => {
            match file.write_all(bytes) {
                Ok(_) => {},
                Err(_) => {
                    return Err(BmarkError::IoExplained(format!("ERROR: Could not write to aliases file")))
                }
            }
        },
        Err(_) => {
            return Err(BmarkError::IoExplained(format!("ERROR: Could not open aliases file")))
        }
    }
    Ok(())
}

type BmarkResult = Result<(), BmarkError>;

enum BmarkError {

    /// For errors regargin cli commands and options parsing
    Cli(String),

    /// IO errors that have a custom error message
    IoExplained(String),
}

fn main() {

    let opts = cli::Opts::parse_args_default_or_exit();

    let cmd = if let Some(c) = opts.command { c }
    else {
        eprintln!("{}\n\nSubcommands:\n{}", opts.self_usage(), opts.self_command_list().unwrap());
        exit(1)
    };

    let res = match cmd {
        cli::Command::Add(add_opts) => bmark_add(add_opts.name),
        cli::Command::Edit(_) => bmark_edit(),
        cli::Command::List(_) => bmark_list(),
        cli::Command::Open(_) => bmark_open(),
        cli::Command::Rm(rm_opts) => {
            bmark_rm(rm_opts.name)
        },
        cli::Command::Update(_) => bmark_update(), cli::Command::Config(config_opts) => {
            if let Some(cmd) = config_opts.command {
                bmark_config(cmd)
            }
            else {
                let msg = format!("Please supply a subcommand for `bmark config`.\n\nSubcommands:\n{}", config_opts.self_command_list().unwrap());
                Err(BmarkError::Cli(msg))
            }
        },
    };

    if let Err(e) = res {
        match e {
            BmarkError::Cli(msg) => eprintln!("{msg}"),
            BmarkError::IoExplained(msg) => eprintln!("{msg}"),
        }
        exit(1);
    }
}

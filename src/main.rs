mod cli;

use directories::ProjectDirs;
use gumdrop::Options;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{exit, Command};
use std::{env, fs};

static BOOKMARKS_FILE: &str = "bookmarks.toml";
static ALIAS_FILE: &str = "aliases.sh";

struct Bookmarks {
    file: PathBuf,
}

impl Bookmarks {
    fn from(path: PathBuf) -> Bookmarks {
        Bookmarks { file: path }
    }
    fn from_config(config: &Config) -> Bookmarks {
        Bookmarks {
            file: config.get_bookmarks_file(),
        }
    }
    fn get_raw(&self) -> String {
        let mut contents = String::new();
        match File::open(&self.file) {
            Ok(mut file) => {
                if let Err(_) = file.read_to_string(&mut contents) {
                    eprintln!(
                        "ERROR: opened, but could not read from bookmarks file: `{}`",
                        self.file.display()
                    );
                    exit(1);
                }
            }
            Err(_) => {
                eprintln!(
                    "ERROR: could not open bookmarks file: `{}`",
                    self.file.display()
                );
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
                eprintln!(
                    "ERROR: Could not parse bookmarks file: `{}`",
                    self.file.display()
                );
                exit(1);
            }
        }
    }
    fn readable(&self, config: &Config) -> String {
        let mut res = String::new();
        let map = self.get_map();
        let mut max_len: usize = 0;
        for l in map.keys() {
            if l.len() > max_len {
                max_len = l.len();
            }
        }
        for (k, v) in map {
            let mut padding = "".to_string();
            for _ in 0..(max_len - k.len()) {
                padding.push(' ')
            }
            res += format!("{}{}{}{}\n", k, padding, config.display_sep, v).as_str();
        }
        res
    }
}

impl fmt::Display for Bookmarks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (k, v) in self.get_map() {
            write!(f, "{} - {}", k, v)?;
        }
        Ok(())
    }
}

struct Config {
    dmenu_cmd: String,
    editor_cmd: String,
    display_sep: String,
    show_paths: bool,
    terminal_cmd: String,
    alias_prefix: String,
    data_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::new(),
            dmenu_cmd: "rofi -dmenu".to_string(),
            editor_cmd: "nvim".to_string(),
            terminal_cmd: "kitty --detach".to_string(),
            alias_prefix: "_".to_string(),
            display_sep: ":".to_string(),
            show_paths: false,
        }
    }
}

impl Config {
    fn new(_opts: &cli::Opts) -> Result<Self, String> {
        let mut dmenu_cmd: Option<String> = None;
        let mut editor_cmd: Option<String> = None;
        let mut display_sep: Option<String> = None;
        let mut terminal_cmd: Option<String> = None;
        let mut alias_prefix: Option<String> = None;
        let mut data_dir: Option<PathBuf> = None;
        let mut show_paths: Option<bool> = None;

        // Default data_dir
        data_dir = match ProjectDirs::from("com", "bmark", "bmark") {
            Some(proj_dirs) => Some(PathBuf::from(proj_dirs.data_dir())),
            None => data_dir,
        };

        // Read config form toml file
        match Self::get_user_config() {
            Ok(toml_config) => {
                data_dir = match toml_config.get("data_dir") {
                    Some(toml::Value::String(p)) => Some(PathBuf::from(p)),
                    _ => data_dir,
                };
                Self::try_get_string_option(&toml_config, &mut dmenu_cmd, "dmenu_cmd");
                Self::try_get_string_option(&toml_config, &mut editor_cmd, "editor_cmd");
                Self::try_get_string_option(&toml_config, &mut display_sep, "display_sep");
                Self::try_get_string_option(&toml_config, &mut terminal_cmd, "terminal_cmd");
                Self::try_get_string_option(&toml_config, &mut alias_prefix, "alias_prefix");
                Self::try_get_bool_option(&toml_config, &mut show_paths, "show_paths");
            }
            Err(e) => eprintln!("{e}"),
        };

        let mut config = Self::default();

        if let Some(o) = dmenu_cmd {
            config.dmenu_cmd = o;
        }
        if let Some(o) = editor_cmd {
            config.editor_cmd = o;
        }
        if let Some(o) = data_dir {
            config.data_dir = PathBuf::from(o);
        }
        if let Some(o) = display_sep {
            config.display_sep = o;
        }
        if let Some(o) = terminal_cmd {
            config.terminal_cmd = o;
        }
        if let Some(o) = alias_prefix {
            config.alias_prefix = o;
        }
        if let Some(o) = show_paths {
            config.show_paths = o;
        }

        if !config.data_dir.is_dir() {
            return Err("ERROR: Could not deternine data directory.".to_string());
        }

        Ok(config)
    }

    fn try_get_string_option(
        config: &HashMap<String, toml::Value>,
        field: &mut Option<String>,
        option: &str,
    ) {
        if let Some(toml::Value::String(s)) = config.get(option) {
            *field = Some(s.clone());
        }
    }

    fn try_get_bool_option(
        config: &HashMap<String, toml::Value>,
        field: &mut Option<bool>,
        option: &str,
    ) {
        if let Some(toml::Value::Boolean(b)) = config.get(option) {
            *field = Some(b.clone());
        }
    }

    fn user_config_file() -> Result<PathBuf, String> {
        match ProjectDirs::from("com", "bmark", "bmark") {
            Some(proj_dirs) => Ok(PathBuf::from(proj_dirs.config_dir()).join("config.toml")),
            None => return Err(format!("ERROR: could not determine config directory")),
        }
    }

    fn get_user_config() -> Result<HashMap<String, toml::Value>, String> {
        let config_file = Config::user_config_file()?;
        let mut m: HashMap<String, toml::Value> = Default::default();
        match File::open(&config_file) {
            Ok(mut file) => {
                let mut lines = String::new();
                if let Err(_) = file.read_to_string(&mut lines) {
                    return Err(format!(
                        "ERROR: Can not read config file: `{}`",
                        config_file.display()
                    ));
                }
                let user_config: HashMap<String, toml::Value> =
                    toml::from_str(lines.as_str()).unwrap();
                for (k, v) in user_config.iter() {
                    m.insert(k.to_owned(), v.to_owned());
                }
                Ok(m)
            }
            Err(_) => Ok(m),
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
        write!(
            f,
            "data_dir = \"{}\"
dmenu_cmd = \"{}\"
editor_cmd = \"{}\"
terminal_cmd = \"{}\"
alias_prefix = \"{}\"
display_sep = \"{}\",
show_paths = \"{}\"",
            self.data_dir.display(),
            self.dmenu_cmd,
            self.editor_cmd,
            self.terminal_cmd,
            self.alias_prefix,
            self.display_sep,
            self.show_paths,
        )
    }
}

// Add: source_cmd subcommand to output the command to source the alias file
fn bmark_config(config: &Config, subcommand: cli::ConfigCommand) -> BmarkResult {
    match subcommand {
        cli::ConfigCommand::Show(_) => println!("{config}"),
        cli::ConfigCommand::Create(_) => {
            let config_file = Config::user_config_file()?;
            if config_file.exists() {
                eprintln!("ERROR: Cannot create default config file, a config file already exists at `{}`.", 
                          config_file.display());
                exit(1);
            }
            let config = Config::default();
            fs::create_dir_all(config_file.parent().expect("No parrent of config file."))
                .expect("Could not create config directory.");
            match OpenOptions::new()
                .write(true)
                .create(true)
                .open(&config_file)
            {
                Ok(mut file) => {
                    let buf = format!("data_dir = \"{}\"\ndmenu_cmd = \"{}\"\neditor_cmd = \"{}\"\nterminal_cmd = \"{}\"\nalias_prefix = \"{}\"\ndisplay_sep = \"{}\"", 
                        config.data_dir.display(),
                        config.dmenu_cmd,
                        config.editor_cmd,
                        config.terminal_cmd,
                        config.alias_prefix,
                        config.display_sep,
                    );

                    if let Err(e) = file.write_all(buf.as_bytes()) {
                        eprintln!("ERROR: Could not write to config file : {e}");
                    }
                }
                Err(_) => {
                    eprintln!(
                        "ERROR: Could not open config file: `{}`",
                        config_file.display()
                    );
                    exit(1);
                }
            }
        }
        cli::ConfigCommand::Edit(_) => {
            let path = Config::user_config_file()?;

            if !path.exists() {
                fs::create_dir_all(path.parent().unwrap()).unwrap();
            }
            let editor_cmd = config.editor_cmd.clone() + " " + path.to_str().unwrap();
            Command::new("sh")
                .arg("-c")
                .arg(editor_cmd)
                .status()
                .expect("ERROR: Failed to execute editor command.");
        }
        cli::ConfigCommand::SourceCmd(_) => {
            println!("source \"{}/{}\"", config.data_dir.display(), ALIAS_FILE,);
            exit(0);
        }
    }

    Ok(())
}

// Make sure that there are no duplicates
fn bmark_add(config: &Config, name: Option<String>) -> BmarkResult {
    let bookmarks = Bookmarks::from_config(config);
    let bookmarks_file = bookmarks.file.clone();

    let data_dir = bookmarks_file
        .parent()
        .expect("There should always be a parrent in the path.");

    if !data_dir.exists() {
        if let Err(e) = fs::create_dir_all(data_dir) {
            return Err(format!("ERROR: Could not create data directory: `{}`", e));
        };
    }

    let cwd = env::current_dir().unwrap();
    let mut bmark_name = match name {
        Some(n) => n,
        None => cwd.file_stem().unwrap().to_str().unwrap().to_string(),
    };

    if bookmarks.get_map().contains_key(&bmark_name) {
        return Err(format!(
            "A bookmark with the name '{bmark_name}' already exists."
        ));
    }

    if bmark_name.rfind(' ') != None {
        eprintln!(
            "WARNING: Bookmarks with spaces cannot be accesed through aliases. Added it anyway."
        );
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
                eprintln!(
                    "ERROR: Could not write to file: {}",
                    bookmarks_file.display()
                );
                exit(1);
            }
        }
        Err(_) => {
            eprintln!(
                "ERROR: Could not open bookmarks file: `{}`",
                bookmarks_file.display()
            );
            exit(1);
        }
    }
    bmark_update(config)
}

fn bmark_edit(config: &Config) -> BmarkResult {
    let path = config.get_bookmarks_file();
    let editor_cmd = config.editor_cmd.clone() + " " + path.to_str().unwrap();
    if let Err(e) = Command::new("sh").arg("-c").arg(editor_cmd).status() {
        return Err(format!("ERROR: Failed to execute editor command:\n{e}"));
    }
    bmark_update(config)
}

fn bmark_list(config: &Config) -> BmarkResult {
    let bookmarks = Bookmarks::from_config(&config);
    print!("{}", bookmarks);
    Ok(())
}

// TODO: Check that dmenu-like program is executable
// TODO: Support fzf
fn bmark_open(config: &Config) -> BmarkResult {
    let bookmarks = Bookmarks::from_config(&config);
    let cmd = "echo '".to_owned()
        + bookmarks.readable(&config).as_str()
        + "'"
        + " | "
        + config.dmenu_cmd.as_str();
    let mut path = match Command::new("sh").arg("-c").arg(&cmd).output() {
        Ok(output) => {
            let choice = String::from_utf8(output.stdout).unwrap();
            if choice == "" {
                eprintln!("No bookmark chosen.");
                exit(1);
            }
            let sep = config.display_sep.clone();
            let mut split = choice.split(&sep);
            split.next();
            match split.next() {
                // TODO: fix with .remainder()
                Some(p) => p.to_owned(),
                None => {
                    eprintln!("ERROR: Could not parse line bookmark: `{}`", choice);
                    exit(1);
                }
            }
        }
        Err(_) => {
            eprintln!("ERROR: Error running dmenu-command: `{}`", cmd);
            exit(1);
        }
    };
    path.pop(); // Remove newline
    let cmd = config.terminal_cmd.clone() + " \"" + path.as_str() + "\"";

    if let Err(_) = Command::new("sh").arg("-c").arg(&cmd).status() {
        eprintln!(
            "ERROR: Could not open terminal with this command: `{}`",
            cmd
        );
        exit(1);
    }

    Ok(())
}

fn bmark_rm(config: &Config, bmark: String) -> BmarkResult {
    let bookmarks = Bookmarks::from_config(&config);
    let mut bookmarks_str = String::new();
    let mut removed = false;

    for (mut k, v) in bookmarks.get_map() {
        if k == bmark {
            removed = true;
            continue;
        }
        if k.rfind(' ') != None {
            k = "\"".to_string() + k.as_str() + "\""
        }
        bookmarks_str += format!("{} = \"{}\"\n", k, v).as_str();
    }
    if !removed {
        return Err(format!("ERROR: could not find bookmark `{}`.", bmark));
    }
    let bytes = bookmarks_str.as_bytes();
    match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(config.get_bookmarks_file())
    {
        Ok(mut file) => match file.write_all(bytes) {
            Ok(_) => {}
            Err(_) => return Err("ERROR: Could not open bookmarks file".to_string()),
        },
        Err(_) => return Err("ERROR: Could not open bookmarks file".to_string()),
    }
    Ok(())
}

fn bmark_update(config: &Config) -> BmarkResult {
    let bookmarks = Bookmarks::from(config.get_bookmarks_file());
    let mut aliases = String::new();
    for (name, path) in bookmarks.get_map() {
        if name.rfind(' ') != None {
            continue;
        } // Skip bookmark names with spaces
        aliases += format!("alias {}{}='cd \"{}\"'\n", config.alias_prefix, name, path).as_str();
    }
    let bytes = aliases.as_bytes();
    match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(config.get_alias_file())
    {
        Ok(mut file) => match file.write_all(bytes) {
            Ok(_) => {}
            Err(_) => return Err(format!("ERROR: Could not write to aliases file")),
        },
        Err(_) => return Err(format!("ERROR: Could not open aliases file")),
    }
    Ok(())
}

type BmarkResult = Result<(), String>;

fn main() {
    let opts = cli::Opts::parse_args_default_or_exit();

    let config = match Config::new(&opts) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            exit(1)
        }
    };

    let cmd = if let Some(c) = opts.command {
        c
    } else {
        eprintln!(
            "{}\n\nSubcommands:\n{}",
            opts.self_usage(),
            opts.self_command_list().unwrap()
        );
        exit(1)
    };

    let res = match cmd {
        cli::Command::Add(add_opts) => bmark_add(&config, add_opts.name),
        cli::Command::Edit(_) => bmark_edit(&config),
        cli::Command::List(_) => bmark_list(&config),
        cli::Command::Open(_) => bmark_open(&config),
        cli::Command::Rm(rm_opts) => bmark_rm(&config, rm_opts.name),
        cli::Command::Update(_) => bmark_update(&config),
        cli::Command::Config(config_opts) => {
            if let Some(cmd) = config_opts.command {
                bmark_config(&config, cmd)
            } else {
                let msg = format!(
                    "Please supply a subcommand for `bmark config`.\n\nSubcommands:\n{}",
                    config_opts.self_command_list().unwrap()
                );
                Err(msg)
            }
        }
    };

    if let Err(e) = res {
        eprintln!("{e}");
        exit(1);
    }
}

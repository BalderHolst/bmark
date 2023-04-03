use std::collections::HashMap;
use std::{env, fs};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::process::{exit, Command};

fn get_dmenu_cmd() -> String {
    "rofi -dmenu".to_owned()
}

fn get_editor_cmd() -> String {
    "nvim".to_owned()
}

fn get_open_term_cmd() -> String {
    "kitty --detach".to_owned()
}

fn get_bookmarks_path() -> PathBuf { 
    PathBuf::from("/home/balder/.local/share/bmark/bookmarks.txt")
}

fn get_aliases_path() -> PathBuf { 
    PathBuf::from("/home/balder/.local/share/bmark/aliases.sh")
}

fn get_bookmark_map() -> HashMap<String, String> {
    let mut hmap: HashMap<String, String> = HashMap::new();

    for line in get_bookmarks().split("\n") {
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

fn get_bookmarks() -> String {
    let bookmarks_file = get_bookmarks_path();
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

fn bmark_add(name: Option<String>) {
    let bookmarks_file = get_bookmarks_path();

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
    let path = get_bookmarks_path();
    let editor_cmd = get_editor_cmd() + " " + path.to_str().unwrap();
    Command::new("sh")
        .arg("-c")
        .arg(editor_cmd)
        .status()
        .expect("ERROR: Failed to execute editor command.");
    bmark_update();
}

fn bmark_list() {
    print!("{}", get_bookmarks());
}

fn bmark_open(){
    let cmd = "echo '".to_owned() + get_bookmarks().as_str()+ "'" + " | " + get_dmenu_cmd().as_str();
    let path = match Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
    {
        Ok(output) => {
            let choice = String::from_utf8(output.stdout).unwrap();
            let mut split = choice.split(" - ");
            split.next();
            match split.next() {
                Some(p) => p.to_owned(),
                None => {
                    eprintln!("ERROR: Could not parse line bookmark: `{}`", "test");
                    exit(1);
                }
            }
        },
        Err(_) => {
            eprintln!("ERROR: Error running dmenu command: `{}`", cmd);
            exit(1);
        }
    };

    let cmd = get_open_term_cmd() + " " + path.as_str();

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

    let mut bookmarks_str = String::new();
    let mut removed = false;

    for (k, v) in get_bookmark_map() {
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
        .open(get_bookmarks_path())
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
    let bookmarks = get_bookmarks();
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
        .open(get_aliases_path())
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
        "add"    => {
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
        "edit"   => bmark_edit(),
        "list"   => bmark_list(),
        "open"   => bmark_open(),
        "rm"     => {
            if args.len() < 3 {
                eprintln!("ERROR: Please provide a bookmark to remove.\n");
                usage();
                exit(1);
            }
            bmark_rm(args[2].clone())
        },
        "update" => bmark_update(),
        _ => {
            eprintln!("ERROR: command `{}` not known.\n", cmd);
            usage();
        }
    }
}

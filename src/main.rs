use std::env;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::process::exit;

fn get_bookmarks_path() -> PathBuf { 
    PathBuf::from("/home/balder/.local/share/bmark/bookmarks.txt")
}
fn get_aliases_path() -> PathBuf { 
    PathBuf::from("/home/balder/.local/share/bmark/aliases.sh")
}

fn usage() {
    println!("usage: bmark <command>\n"                                          );
    println!("Commands:"                                                         );
    println!("   add [<name>]    add a bookmark to the current working directory");
    println!("   list            list all stored bookmarks"                      );
    println!("   edit            edit bookmarks in a text editor"                );
    println!("   open            open a new terminal in a bookmarked location"   );
    println!("   rm <name>       remove a bookmark with a given name"            );
    println!("   update          update shell aliases file"                      );
}

fn bmark_list() {
    let bookmarks_file = get_bookmarks_path();
    let mut contents = String::new();

    match File::open(&bookmarks_file) {
        Ok(mut file) => {
            match file.read_to_string(&mut contents) {
                Ok(_) => {},
                Err(_) => {
                    println!("ERROR: opened, but could not read from bookmarks file: `{}`", bookmarks_file.display());
                    exit(1);
                }
            }
        }
        Err(_) => {
            println!("ERROR: could not open bookmarks file: `{}`", bookmarks_file.display());
            exit(1);
        }
    }
    println!("{}", contents);
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
        "list" => bmark_list(),
        _ => {
            println!("ERROR: command `{}` not known.\n", cmd);
            usage();
        }
    }
}

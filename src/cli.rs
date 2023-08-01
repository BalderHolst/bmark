use gumdrop::Options;

#[derive(Debug, Options)]
pub(crate) struct Opts {
    #[options(short = "h", help = "print help message")]
    pub(crate) help: bool,

    #[options(short = "v", help = "be verbose")]
    pub(crate) verbose: bool,

    #[options(command)]
    pub(crate) command: Option<Command>,
}

#[derive(Debug, Options)]
pub(crate) enum Command {
    #[options(help = "add a bookmark to the current working directory")]
    Add(AddOpts),
    #[options(help = "edit bookmarks in a text editor")]
    Edit(EditOpts),
    #[options(help = "list all stored bookmarks")]
    List(ListOpts),
    #[options(help = "open a new terminal in a bookmarked location")]
    Open(OpenOpts),
    #[options(help = "remove a bookmark with a given name")]
    Rm(RmOpts),
    #[options(help = "commands for managing bmark configuration")]
    Config(ConfigOpts),
    #[options(help = "update shell aliases file")]
    Update(UpdateOpts),
}

#[derive(Debug, Options)]
pub(crate) struct AddOpts {
    #[options(short = "h", help = "print help message")]
    pub(crate) help: bool,

    pub(crate) name: Option<String>,
}

#[derive(Debug, Options)]
pub struct EditOpts {
    #[options(short = "h", help = "print help message")]
    pub(crate) help: bool,
}

#[derive(Debug, Options)]
pub struct ListOpts {
    #[options(short = "h", help = "print help message")]
    pub(crate) help: bool,
}

#[derive(Debug, Options)]
pub struct OpenOpts {
    #[options(short = "h", help = "print help message")]
    pub(crate) help: bool,

    #[options(short = "P", help = "Show corrsponding bookmark paths")]
    pub(crate) show_paths: bool,

    #[options(short = "D", help = "Dmenu-like command to be used for fuzzyfinding")]
    pub(crate) cmd: Option<String>,

    #[options(short = "T", help = "Terminal command")]
    pub(crate) terminal: Option<String>,
}

#[derive(Debug, Options)]
pub struct RmOpts {
    #[options(short = "h", help = "print help message")]
    pub(crate) help: bool,

    #[options(short = "n", help = "Name of a bookmark")]
    pub(crate) name: String,
}

#[derive(Debug, Options)]
pub struct ConfigOpts {
    #[options(short = "h", help = "print help message")]
    pub(crate) help: bool,
    
    #[options(command)]
    pub(crate) command: Option<ConfigCommand>,
}

#[derive(Debug, Options)]
pub struct UpdateOpts {
    #[options(short = "h", help = "print help message")]
    pub(crate) help: bool,
}

#[derive(Debug, Options)]
pub enum ConfigCommand {
    #[options(help = "Show the current configuration")]
    Show(ConfigShow),
    #[options(help = "Create a configuration file with default configuration")]
    Create(ConfigCreate),
    #[options(help = "Edit the configuration file")]
    Edit(ConfigEdit),
    #[options(help = "Print the command used to source the bookmark aliases file")]
    SourceCmd(ConfigSourceCmd),
}

#[derive(Debug, Options)]
pub struct ConfigShow {
    #[options(short = "h", help = "print help message")]
    pub(crate) help: bool,
}

#[derive(Debug, Options)]
pub struct ConfigCreate {
    #[options(short = "h", help = "print help message")]
    pub(crate) help: bool,
}

#[derive(Debug, Options)]
pub struct ConfigEdit {
    #[options(short = "h", help = "print help message")]
    pub(crate) help: bool,

    #[options(short = "E", help = "Command to launch a text editor")]
    editor: Option<String>,
}

#[derive(Debug, Options)]
pub struct ConfigSourceCmd {
    #[options(short = "h", help = "print help message")]
    pub(crate) help: bool,
}

use gumdrop::Options;

#[derive(Debug, Options)]
pub(crate) struct Opts {
    #[options(help = "print help message")]
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
    pub(crate) name: Option<String>,
}

#[derive(Debug, Options)]
pub struct EditOpts {}

#[derive(Debug, Options)]
pub struct ListOpts {}

#[derive(Debug, Options)]
pub struct OpenOpts {}

#[derive(Debug, Options)]
pub struct RmOpts {
    pub(crate) name: String,
}

#[derive(Debug, Options)]
pub struct ConfigOpts {
    #[options(command)]
    pub(crate) command: Option<ConfigCommand>,
}

#[derive(Debug, Options)]
pub struct UpdateOpts {}

#[derive(Debug, Options)]
pub enum ConfigCommand {
    Show(ConfigShow),
    Create(ConfigCreate),
    Edit(ConfigEdit),
    SourceCmd(ConfigSourceCmd),
}

#[derive(Debug, Options)]
pub struct ConfigShow {}

#[derive(Debug, Options)]
pub struct ConfigCreate {}

#[derive(Debug, Options)]
pub struct ConfigEdit {}

#[derive(Debug, Options)]
pub struct ConfigSourceCmd {}

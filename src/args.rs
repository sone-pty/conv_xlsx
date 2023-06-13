use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Name of the xlsx file
    pub name: String,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(name = "build", visible_alias = "b")]
    Build,

    #[command(name = "clean", visible_alias = "c")]
    Clean,
}
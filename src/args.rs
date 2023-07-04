use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Name of the xlsx file
    #[arg(default_value_t = String::from(""), short, long)]
    pub name: String,
    /// need to update svn or not
    #[arg(default_value_t = false, short, long)]
    pub update_svn: bool,
    /// need to pull file or not
    #[arg(default_value_t = false, short, long)]
    pub pull_file: bool,
    /// path of the config dir
    #[arg(default_value_t = String::from("D:/Config-beta/"), long)]
    pub src_table_dir: String,
    /// path of the output script dir
    #[arg(default_value_t = String::from("ExportScripts/"), long)]
    pub output_script_dir: String,
    /// path of the output enum dir
    #[arg(default_value_t = String::from("ConfigExportEnum/"), long)]
    pub output_enum_dir: String,
    /// path of the config ref mapping dir
    #[arg(default_value_t = String::from("ConfigRefNameMapping/"), long)]
    pub ref_mapping_dir: String,
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
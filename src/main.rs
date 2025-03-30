use anyhow::Context;
use clap::{Args, Parser, Subcommand, ValueEnum};
use legit::Repository;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "legit")]
#[command(about = "A git implementation in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Initialize a new git repository
    Init {
        #[arg()]
        /// The path to the repository
        path: Option<OsString>,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Command::Init { path } => {
            println!("Initializing repository...");
            let path = path.map_or(std::env::current_dir().unwrap(), PathBuf::from);
            let repo = Repository::create(&path);
            if let Err(e) = repo {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }
}

use clap::Parser;
use legit::Repository;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "legit")]
#[command(about = "A git implementation in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// The path to the repository
    path: Option<OsString>,
}

#[derive(Parser, Debug)]
enum Command {
    /// Initialize a new git repository
    Init {
        /// The path to the repository
        path: Option<OsString>,
    },

    /// Display information about the repository
    Config,
}

fn main() {
    let args = Cli::parse();

    let base_path = args
        .path
        .clone()
        .map_or(std::env::current_dir().unwrap(), PathBuf::from);

    match args.command {
        Command::Init { path } => {
            println!("Initializing repository...");
            let path = path.map_or(base_path.clone(), PathBuf::from);
            let repo = Repository::new(&path);
            match repo {
                Ok(repo) => {
                    if let Err(e) = repo.create() {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                    println!("Initialized empty git repository in {}", path.display());
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::Config => {
            let repo = Repository::new(&base_path);
            match repo {
                Ok(repo) => {
                    println!("{:#?}", repo.settings);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

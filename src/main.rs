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
    }
}

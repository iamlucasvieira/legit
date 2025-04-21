use clap::Parser;
use legit::objects::{read_object, write_object, Object, ObjectHash, ObjectType};
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

    /// Provide contents of a repository object
    CatFile {
        /// The type of the object (e.g., commit, tree, blob)
        #[arg(value_enum)]
        object_type: ObjectType,

        /// The hash of the object
        hash: String,
    },

    /// Hash a file
    HashFile {
        /// The type of the object (e.g., commit, tree, blob)
        #[arg(value_enum)]
        object_type: ObjectType,

        /// The path to the file
        path: OsString,

        /// If true, the object will be stored in the repository
        #[arg(long)]
        store: bool,
    },
}

fn main() {
    let args = Cli::parse();

    let base_path = match args.path {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir().unwrap_or_else(|_| {
            eprintln!("Failed to get current directory");
            std::process::exit(1);
        }),
    };

    match args.command {
        Command::Init { path } => {
            println!("Initializing repository...");
            let path = path.map_or(base_path.clone(), PathBuf::from);
            let repo = Repository::new(&path);
            match repo {
                Ok(_) => {
                    println!("Initialized empty git repository in {}", path.display());
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::Config => {
            let repo = Repository::find(&base_path);
            match repo {
                Ok(repo) => {
                    println!("{:#?}", repo.settings());
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::CatFile { hash, .. } => {
            let repo = Repository::find(&base_path);
            let hash = ObjectHash::from_hex(hash.as_str()).unwrap_or_else(|_| {
                eprintln!("Invalid hash format");
                std::process::exit(1);
            });
            match repo {
                Ok(repo) => {
                    let object = read_object(&repo, &hash);
                    match object {
                        Ok(obj) => {
                            println!("{:#?}", obj);
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::HashFile {
            object_type,
            path,
            store,
        } => {
            let path = PathBuf::from(path);
            let data = std::fs::read(&path).unwrap_or_else(|e| {
                eprintln!("Failed to read file {}: {}", path.display(), e);
                std::process::exit(1);
            });
            let object = Object::new(object_type, data).unwrap_or_else(|e| {
                eprintln!("Failed to create object: {}", e);
                std::process::exit(1);
            });
            if store {
                let repo = Repository::find(&base_path).unwrap_or_else(|e| {
                    eprintln!("{}", e);
                    std::process::exit(1);
                });
                write_object(&object, &repo).unwrap_or_else(|e| {
                    eprintln!("Failed to write object: {}", e);
                    std::process::exit(1);
                });
                println!("Stored object with hash: {}", object.hash);
            } else {
                println!("Hash of file {}: {}", path.display(), object.hash);
            }
        }
    }
}

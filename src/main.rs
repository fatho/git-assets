use std::env;
use std::io::{self};
use std::path::{PathBuf};

use structopt::StructOpt;

mod hash;
mod store;
use store::{Store, StoreFileRef};

#[derive(StructOpt)]
#[structopt(about = "binary asset handling for git")]
struct GitAssets {
    #[structopt(long, short, parse(from_os_str))]
    store: Option<PathBuf>,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    Clean,
    Smudge,
}

fn default_store() -> io::Result<PathBuf> {
    Ok(env::current_dir()?.join(".git").join("x-assets"))
}

fn main() {
    let opts = GitAssets::from_args();
    let store_path = opts.store.unwrap_or_else(|| default_store().unwrap());

    let result = match opts.command {
        Command::Clean => clean(store_path),
        Command::Smudge => smudge(store_path),
    };

    match result {
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1)
        }
        Ok(()) => {}
    }
}

/// Store a file from the working directory in the store
fn clean(store_path: PathBuf) -> io::Result<()> {
    let store = Store::open_or_create(store_path)?;

    // Copy stdin (where git provides the file contents) to a temporary file,
    // which also computes the hash while writing.
    let mut staging_file = store.new_staging_file()?;
    io::copy(&mut io::stdin().lock(), &mut staging_file)?;
    // If writing was successful, we make the file permanent.
    let store_ref = store.make_permanent(staging_file)?;

    // Print reference to stdout so that we can fetch the contents back during smudge
    println!("{}", store_ref.to_string());

    Ok(())
}

/// Read a file from the store and put it in the working directory.
fn smudge(store_path: PathBuf) -> io::Result<()> {
    // Parse the reference to the actual file
    let store_ref = StoreFileRef::parse_from_stream(&mut io::stdin().lock())?;
    // And dereference it using the given store
    let store = Store::open_or_create(store_path)?;
    let mut file = store.open_ref(&store_ref)?;
    io::copy(&mut file, &mut io::stdout().lock())?;

    Ok(())
}

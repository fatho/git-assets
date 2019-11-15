use std::env;
use std::io::{self};
use std::path::PathBuf;

use structopt::StructOpt;

use git_assets;
use git_assets::store;

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
    /// Store the contents received on stdin in the store, and print a reference to the file on stdout.
    ///
    /// To be used as a git clean filter.
    Store,
    /// Read a reference to the file contents from stdin, and write the contents to stdout.
    ///
    /// To be used as a git smudge filter.
    Retrieve,
    /// Validate the store contents, i.e. that all data files are consistent (their name matches the hash),
    /// and that there are no unexpected files that don't belong there.
    Validate,
}

fn default_store() -> io::Result<PathBuf> {
    Ok(env::current_dir()?.join(".git").join("x-assets"))
}

fn main() {
    let opts = GitAssets::from_args();

    match run(opts) {
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1)
        }
        Ok(()) => {}
    }
}

fn run(opts: GitAssets) -> io::Result<()> {
    let store_path = opts.store.unwrap_or_else(|| default_store().unwrap());

    match opts.command {
        Command::Store => store(store_path),
        Command::Retrieve => retrieve(store_path),
        Command::Validate => validate(store_path),
    }
}

/// Store a file from the working directory in the store
fn store(store_path: PathBuf) -> io::Result<()> {
    let store = store::Store::open_or_create(store_path)?;

    // Copy stdin (where git provides the file contents) to a temporary file,
    // which also computes the hash while writing.
    let mut staging_file = store.new_staging_file()?;
    io::copy(&mut io::stdin().lock(), &mut staging_file)?;
    // If writing was successful, we make the file permanent.
    let store_ref = store.make_permanent(staging_file)?;

    // Print reference to stdout so that we can fetch the contents back during retrieve
    println!("{}", store_ref.to_string());

    Ok(())
}

/// Read a file from the store and put it in the working directory.
fn retrieve(store_path: PathBuf) -> io::Result<()> {
    // Parse the reference to the actual file
    let store_ref = store::StoreFileRef::parse_from_stream(&mut io::stdin().lock())?;
    // And dereference it using the given store
    let store = store::Store::open_or_create(store_path)?;
    let mut file = store.open_ref(&store_ref)?;
    io::copy(&mut file, &mut io::stdout().lock())?;

    Ok(())
}

/// Check whether the store contents are consistent.
fn validate(store_path: PathBuf) -> io::Result<()> {
    // And dereference it using the given store
    let store = store::Store::open_or_create(store_path)?;
    let report = store.validate()?;

    if report.is_valid() {
        Ok(())
    } else {
        for hash_mismatch in &report.hash_mismatches {
            println!(
                "hash-mismatch: {}: {} != {}",
                hash_mismatch.file_name.display(),
                hash_mismatch.expected_hash,
                hash_mismatch.actual_hash
            );
        }

        for unexpected_file in &report.unexpected_files {
            println!("unexpected: {}", unexpected_file.display());
        }

        Err(io::ErrorKind::InvalidData.into())
    }
}

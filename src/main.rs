use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;

#[macro_use]
mod macros;

mod catalog_mirrors;
mod catalog_releases;
mod mirrors;
mod releases;
mod str_error;
mod wares;

#[derive(Parser)]
#[command(name = "catalog-manifest")]
#[command(bin_name = "catalog-manifest")]
#[command(about = "Walks a warpforge catalog and joins information.")]
struct Cli {
    /// The directory to walk. This is expected to be a warpforge catalog directory.
    #[arg(short, long, value_name = "DIRECTORY")]
    catalog_path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print a JSON object of references and ware IDs
    Releases,
    /// Print a unified mirrors JSON object
    Mirrors,
    ///Prints a list of ware IDs and fully qualified mirror locations.
    Wares,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    match args.command {
        Commands::Releases {} => cmd_releases(args)?,
        Commands::Mirrors {} => cmd_mirrors(args)?,
        Commands::Wares {} => cmd_wares(args)?,
    }
    Ok(())
}

fn cmd_releases(args: Cli) -> Result<(), Box<dyn Error>> {
    let dir = PathBuf::from(args.catalog_path);
    let result = releases::collect(&dir)?;
    let output = serde_json::to_string_pretty(&result)?;
    println!("{output}");
    Ok(())
}

fn cmd_mirrors(args: Cli) -> Result<(), Box<dyn Error>> {
    let start_dir = PathBuf::from(args.catalog_path);
    let result = mirrors::collect(&start_dir)?;
    let output = serde_json::to_string_pretty(&result)?;
    println!("{output}");
    Ok(())
}

fn cmd_wares(args: Cli) -> Result<(), Box<dyn Error>> {
    let start_dir = PathBuf::from(args.catalog_path);
    let result = wares::resolve_all(&start_dir)?;
    let output = serde_json::to_string_pretty(&result)?;
    println!("{output}");
    Ok(())
}

pub mod modules;

use modules::declaration_trait::{Declaration, reconcile};
use modules::declarations::packages::PackagesDeclaration;

use clap::{Parser, Subcommand};
use modules::config::Config;
use std::fs;

#[derive(Parser)]
#[command(name = "declarative-alpine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Apply {
        #[arg(short, long, default_value = "config.toml")]
        config: String,
        #[arg(long)]
        dry_run: bool,
    },
    Diff {
        #[arg(short, long, default_value = "config.toml")]
        config: String,
    },
    // Add init, validate, etc.
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config_path = match &cli.command {
        Commands::Apply { config, .. } => config,
        Commands::Diff { config } => config,
    };
    let config_str = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_str)?;

    match cli.command {
        Commands::Apply {
            config: _path,
            dry_run,
        } => {
            reconcile(&PackagesDeclaration, &config.packages, dry_run)?;
        }

        Commands::Diff { config: _path } => {
            // Similar, but only print diffs, no apply
            // Similar: Get current, compute diff, print for each
            let packages_current = PackagesDeclaration.get_current()?;
            let packages_diff =
                PackagesDeclaration.compute_diff(&packages_current, &config.packages);
            println!("Packages Diff: {:?}", packages_diff);
        }
    }

    Ok(())
}

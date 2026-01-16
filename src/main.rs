pub mod modules;

use modules::reconcilers;

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

    match cli.command {
        Commands::Apply {
            config: path,
            dry_run,
        } => {
            let config_str = fs::read_to_string(path)?;
            let config: Config = toml::from_str(&config_str)?;

            reconcilers::packages::reconcile_packages(&config.packages, dry_run)?;
            // reconcilers::users::reconcile_users(&config.users, dry_run)?;
            // reconcilers::services::reconcile_services(&config.services, dry_run)?;
            // Call others...

            if !dry_run {
                // Optional: Git commit if repo set up
                // Command::new("git").args(["add", "."]).output()?;
                // Command::new("git")
                //     .args(["commit", "-m", "Applied declarative config"])
                //     .output()?;
            }
        }
        Commands::Diff { config: path } => {
            // Similar, but only print diffs, no apply
        }
    }

    Ok(())
}

use clap::{Parser, Subcommand};
use std::error::Error;

/// LoopHouse CLI: auto-generate ZABAL hackathon submissions from git history
#[derive(Parser)]
#[command(name = "zabal")]
#[command(about = "Generate ZABAL hackathon submission documents from your Git commit history", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a submission document from commits in the last N days
    Submit {
        /// Number of days to scan for commits
        #[arg(short = 'd', long, default_value = "7")]
        days: u64,
        /// Output format: markdown or json
        #[arg(short = 'f', long, default_value = "markdown")]
        format: String,
    },
    /// Preview what will be submitted without writing to disk
    Preview {
        /// Number of days to scan for commits
        #[arg(short = 'd', long, default_value = "7")]
        days: u64,
    },
    /// Initialize a project tracker with a handle
    Init {
        /// Your ZABAL handle (e.g., @username)
        #[arg(short = 'u', long)]
        username: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Submit { days, format } => {
            // TODO: implement in src/parser.rs and src/generator.rs
            println!("(not implemented yet) Submitting last {} days in {} format", days, format);
        }
        Commands::Preview { days } => {
            // TODO: implement in src/parser.rs and src/ui.rs
            println!("(not implemented yet) Previewing last {} days", days);
        }
        Commands::Init { username } => {
            // TODO: implement in src/ui.rs or separate tracker
            println!("(not implemented yet) Initializing project tracker for {}", username);
        }
    }

    Ok(())
}

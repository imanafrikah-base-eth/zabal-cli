use clap::{Parser, Subcommand};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use zabal_cli::generator::{self, Format};
use zabal_cli::parser as commits;
use zabal_cli::ui;

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
        /// Write to this file instead of the default name
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
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
    let root = std::env::current_dir()?;

    match cli.command {
        Commands::Submit {
            days,
            format,
            output,
        } => {
            let format: Format = format.parse()?;
            let tracker = ui::load_tracker(&root);
            let handle = tracker.as_ref().map(|t| t.username.as_str());
            let project = tracker
                .as_ref()
                .map(|t| t.project.clone())
                .unwrap_or_else(|| ui::project_name(&root));

            let scanned = commits::scan_commits(&root, days)?;
            let document =
                generator::generate_submission(&scanned, format, handle, Some(project.as_str()), days)?;

            let path = output
                .unwrap_or_else(|| root.join(format!("zabal-submission.{}", format.extension())));
            fs::write(&path, document)?;

            let totals = generator::totals(&scanned);
            println!(
                "Wrote {} commits ({} files, +{} / -{}) to {}",
                totals.commits,
                totals.files_changed,
                totals.insertions,
                totals.deletions,
                path.display()
            );

            if scanned.is_empty() {
                println!(
                    "Heads up: no commits landed in the last {} days, so the document is empty.",
                    days
                );
            }
        }

        Commands::Preview { days } => {
            let tracker = ui::load_tracker(&root);
            let handle = tracker.as_ref().map(|t| t.username.as_str());
            let scanned = commits::scan_commits(&root, days)?;
            print!("{}", ui::render_preview(&scanned, days, handle));
        }

        Commands::Init { username } => {
            let (tracker, path) = ui::init_tracker(&root, &username)?;
            println!(
                "Tracking {} as {} in {}",
                tracker.project,
                tracker.username,
                path.display()
            );
            println!("Next: run `zabal preview` to see what your recent commits look like.");
        }
    }

    Ok(())
}

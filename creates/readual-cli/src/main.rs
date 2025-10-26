use clap::{Parser, Subcommand};
use readual_command_info::{InfoCommand, execute_info_command};
use readual_command_run::{RunCommand, execute_run_command};

/// Readual - repository assistant utility
#[derive(Parser)]
#[command(name = "readual")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Repository assistant utility")]
#[command(long_about = "Readual - a utility that runs commands for repository management, described in README.md file.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Executes commands and scripts described in README.md file
    Run(RunCommand),
    /// Shows repository information
    Info(InfoCommand),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Info(args) => {
            if let Err(e) = execute_info_command(args) {
                eprintln!("❌ Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Run(args) => {
            if let Err(e) = execute_run_command(args) {
                eprintln!("❌ Error executing run command: {}", e);
                std::process::exit(1);
            }
        }
    }
}
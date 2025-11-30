use clap::{Parser, Subcommand};
use readual_command_info::{InfoCommand, execute_info_command};
use readual_command_run::{RunCommand, execute_run_command};
use readual_output::{set_verbosity, OutputVerbosity};

/// Readual - repository assistant utility
#[derive(Parser)]
#[command(name = "readual")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Repository assistant utility")]
#[command(long_about = "Readual - a utility that runs commands for repository management, described in README.md file.")]
struct Cli {
    /// Silent mode - блокирует весь вывод
    #[arg(short = 's', long = "silent", conflicts_with_all = ["verbose", "debug"])]
    silent: bool,
    
    /// Verbose mode - показывает информационные сообщения (по умолчанию)
    #[arg(short = 'v', long = "verbose", conflicts_with_all = ["silent", "debug"])]
    verbose: bool,
    
    /// Debug mode - показывает весь вывод, включая отладочные сообщения
    #[arg(short = 'd', long = "debug", conflicts_with_all = ["silent", "verbose"])]
    debug: bool,
    
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

    // Устанавливаем уровень детализации вывода (глобально для всего приложения)
    let verbosity = if cli.silent {
        OutputVerbosity::Silent
    } else if cli.debug {
        OutputVerbosity::Debug
    } else if cli.verbose {
        OutputVerbosity::Info
    } else {
        OutputVerbosity::Info // По умолчанию Info
    };
    
    set_verbosity(verbosity);

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
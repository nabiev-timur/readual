use clap::{Parser, Subcommand};
use readual_command_info::{InfoCommand, execute_info_command};
use readual_command_run::{RunCommand, execute_run_command};

/// Readual - утилита для работы с репозиторием
#[derive(Parser)]
#[command(name = "readual")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Утилита для работы с репозиторием")]
#[command(long_about = "Readual - утилита, которая запускает команды для работы с репозиторием, описанные в README.md файле.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Выполняет команды и скрипты, описанные в README.md файле
    Run(RunCommand),
    /// Выводит информацию о репозитории
    Info(InfoCommand),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Info(args) => {
            if let Err(e) = execute_info_command(args) {
                eprintln!("❌ Ошибка: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Run(args) => {
            if let Err(e) = execute_run_command(args) {
                eprintln!("❌ Ошибка выполнения команды run: {}", e);
                std::process::exit(1);
            }
        }
    }
}
use clap::Parser;
use readual_md::extract_commands;
use std::fs;

/// Executes commands and scripts described in README.md file
#[derive(Parser, Debug)]
pub struct RunCommand {
    /// Path to command in format "Heading::Subheading::Command"
    #[arg(short, long)]
    pub path: Option<String>,
}

/// Gets commands from README.md
pub fn get_commands_from_readme() -> Result<Vec<String>, String> {
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Error getting current directory: {}", e))?;
    let readme_path = current_dir.join("README.md");
    
    if !readme_path.exists() {
        return Err("README.md not found in current directory".to_string());
    }
    
    let content = fs::read_to_string(&readme_path)
        .map_err(|e| format!("Error reading README.md: {}", e))?;
    
    Ok(extract_commands(&content))
}

/// Prints command to execute
pub fn print_command(command: &str) {
    println!("{}", command);
}

/// Executes run command
pub fn execute_run_command(args: &RunCommand) -> Result<(), String> {
    // Get commands from README.md
    let commands = get_commands_from_readme()?;
    
    // Print all available commands
    for cmd in &commands {
        print_command(cmd);
    }
    
    Ok(())
}

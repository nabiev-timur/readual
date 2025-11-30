use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[macro_use]
extern crate readual_output;

pub mod index;

use index::PathIndex;
use readual_md::parse_file_to_tree;

/// Валидирует путь к README файлу
fn validate_readme_path(path: &str) -> Result<PathBuf, String> {
    let path_buf = PathBuf::from(path);
    
    // Проверяем, что путь существует
    if !path_buf.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    
    // Проверяем, что это файл, а не директория
    if !path_buf.is_file() {
        return Err(format!("Path is not a file: {}", path));
    }
    
    Ok(path_buf)
}

/// Executes commands and scripts described in README.md file
#[derive(Parser, Debug)]
pub struct RunCommand {
    /// Path to command in format "Heading::Subheading::Command"
    pub path: String,
    /// Path to README.md file
    #[arg(short = 'r', long = "readme", default_value = "./README.md", value_parser = validate_readme_path)]
    pub file: PathBuf,
}

/// Executes run command
pub fn execute_run_command(args: &RunCommand) -> Result<(), String> {
    // Парсим файл в дерево
    info!("Parsing file: {}", args.file.display());
    let parsed = parse_file_to_tree(&args.file)
        .map_err(|e| format!("Error parsing file: {:?}", e))?;
    
    debug!("Parsed document:");
    debug!("  Document size: {} bytes", parsed.document.buf.len());
    debug!("  Tree root id: {}", parsed.tree.id);
    debug!("  Tree root children: {}", parsed.tree.children.len());
    
    // Строим индекс путей
    info!("Building path index...");
    let index = PathIndex::from_tree(&parsed.tree, &parsed.document);
    
    // Выводим debug информацию об индексе
    index.debug_print();
    
    // Ищем узлы по переданному пути
    info!("Searching for path: \"{}\"", args.path);
    let node_ids = index.find_nodes(&args.path);
    
    if node_ids.is_empty() {
        return Err(format!("No nodes found for path: \"{}\"", args.path));
    }
    
    info!("Found {} node(s) to execute", node_ids.len());
    
    // Выполняем команды для найденных узлов
    for node_id in &node_ids {
        // Находим узел в дереве
        let node = match parsed.tree.find_node_by_id(*node_id) {
            Some(n) => n,
            None => {
                warning!("Node with ID {} not found in tree, skipping", node_id);
                continue;
            }
        };
        
        // Проверяем, есть ли код для выполнения
        let code_span = match &node.code_span {
            Some(span) => span,
            None => {
                debug!("  Node ID {} has no code to execute, skipping", node_id);
                continue;
            }
        };
        
        // Извлекаем код из документа
        let code = parsed.document.slice(code_span).trim().to_string();
        
        if code.is_empty() {
            debug!("  Node ID {} has empty code, skipping", node_id);
            continue;
        }
        
        info!("Executing command for node ID {}: {}", node_id, code);
        
        // Выполняем команду через bash -c с перенаправлением потоков
        let mut child = Command::new("bash")
            .arg("-c")
            .arg(&code)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to execute command: {}", e))?;
        
        // Ждем завершения процесса
        let status = child.wait()
            .map_err(|e| format!("Failed to wait for command: {}", e))?;
        
        // Проверяем статус выполнения
        if !status.success() {
            return Err(format!(
                "Command failed with exit code {}: {}",
                status.code().unwrap_or(-1),
                code
            ));
        }
        
        success!("Command executed successfully");
    }
    
    Ok(())
}

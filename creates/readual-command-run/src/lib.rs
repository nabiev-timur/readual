use clap::Parser;
use std::process::Command;

/// Выполняет команды и скрипты, описанные в README.md файле
#[derive(Parser, Debug)]
pub struct RunCommand {
    /// Показать подробную информацию о выполнении команды
    #[arg(short, long)]
    pub verbose: bool,
    /// Не выполнять команды, только показать, что будет выполнено
    #[arg(short, long)]
    pub dry: bool,
    /// Путь к команде в формате "Заголовок::Подзаголовок::Команда"
    #[arg(short, long)]
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub path: String,
    pub command: String,
    pub line_number: usize,
}

#[derive(Debug)]
pub struct DocumentCommands {
    pub commands: Vec<CommandInfo>,
}

/// Парсит README.md и извлекает команды из блоков кода
pub fn parse_commands_from_readme() -> Result<DocumentCommands, String> {
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Ошибка получения текущей директории: {}", e))?;
    let readme_path = current_dir.join("README.md");
    
    if !readme_path.exists() {
        return Err("README.md не найден в текущей директории".to_string());
    }
    
    let content = std::fs::read_to_string(&readme_path)
        .map_err(|e| format!("Ошибка чтения README.md: {}", e))?;
    
    let mut commands = DocumentCommands { commands: Vec::new() };
    let lines: Vec<&str> = content.lines().collect();
    let mut current_path = Vec::new();
    
    for (line_number, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Обрабатываем заголовки для построения пути
        if trimmed.starts_with('#') {
            let level = trimmed.chars()
                .take_while(|&c| c == '#')
                .count();
            
            let text = trimmed[level..].trim().to_string();
            if !text.is_empty() {
                // Обновляем путь в зависимости от уровня заголовка
                if level <= current_path.len() {
                    current_path.truncate(level - 1);
                }
                current_path.push(text);
            }
        }
        
        // Ищем блоки кода bash
        if trimmed.starts_with("```bash") {
            // Начало bash блока - пропускаем
            continue;
        }
        
        if trimmed == "```" {
            // Конец блока кода - пропускаем
            continue;
        }
        
        // Проверяем, находимся ли мы внутри bash блока
        let mut in_bash_block = false;
        for i in (0..line_number).rev() {
            let prev_line = lines[i].trim();
            if prev_line == "```" {
                break;
            } else if prev_line == "```bash" {
                in_bash_block = true;
                break;
            }
        }
        
        // Если это команда внутри bash блока
        if in_bash_block && !trimmed.starts_with('#') && !trimmed.starts_with("```") && !trimmed.is_empty() {
            // Проверяем, что это похоже на команду (начинается с буквы или содержит специальные символы)
            if trimmed.chars().next().map_or(false, |c| c.is_alphabetic()) || 
               trimmed.contains("cargo") || 
               trimmed.contains("npm") || 
               trimmed.contains("git") ||
               trimmed.contains("readual") {
                let path_str = current_path.join("::");
                commands.commands.push(CommandInfo {
                    path: path_str,
                    command: trimmed.to_string(),
                    line_number: line_number + 1,
                });
            }
        }
    }
    
    Ok(commands)
}

/// Находит команду по пути
pub fn find_command_by_path<'a>(commands: &'a DocumentCommands, path: &str) -> Option<&'a CommandInfo> {
    commands.commands.iter().find(|cmd| cmd.path == path)
}

/// Выполняет команду
pub fn execute_command(command: &str, dry_run: bool) -> Result<(), String> {
    if dry_run {
        println!("🧪 DRY RUN: {}", command);
        return Ok(());
    }
    
    println!("⚡ Выполнение: {}", command);
    
    // Разбиваем команду на части
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Пустая команда".to_string());
    }
    
    let mut cmd = Command::new(parts[0]);
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }
    
    let output = cmd.output()
        .map_err(|e| format!("Ошибка выполнения команды: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Команда завершилась с ошибкой: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        println!("{}", stdout);
    }
    
    Ok(())
}

/// Выполняет команду run
pub fn execute_run_command(args: &RunCommand) -> Result<(), String> {
    println!("🚀 Выполнение команды run");
    
    if args.verbose {
        println!("📋 Параметры:");
        println!("  🔍 Подробный режим: {}", args.verbose);
        println!("  🧪 Dry run: {}", args.dry);
        if let Some(path) = &args.path {
            println!("  🎯 Путь: {}", path);
        }
    }
    
    // Парсим команды из README.md
    let commands = parse_commands_from_readme()?;
    
    if args.verbose {
        println!("📄 Найдено команд: {}", commands.commands.len());
        for cmd in &commands.commands {
            println!("  📍 {} -> {}", cmd.path, cmd.command);
        }
    }
    
    if let Some(path) = &args.path {
        // Выполняем конкретную команду по пути
        match find_command_by_path(&commands, path) {
            Some(command_info) => {
                println!("🎯 Найдена команда: {}", command_info.command);
                execute_command(&command_info.command, args.dry)?;
            }
            None => {
                println!("❌ Команда не найдена по пути: {}", path);
                println!("💡 Доступные пути:");
                for cmd in &commands.commands {
                    println!("  - {}", cmd.path);
                }
                return Err(format!("Команда не найдена: {}", path));
            }
        }
    } else {
        // Показываем все доступные команды
        println!("📋 Доступные команды:");
        for cmd in &commands.commands {
            println!("  📍 {} -> {}", cmd.path, cmd.command);
        }
        println!("💡 Используйте --path для выполнения конкретной команды");
    }
    
    Ok(())
}

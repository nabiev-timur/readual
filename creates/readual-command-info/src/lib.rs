use clap::Parser;
use readual_md::{DocumentHierarchy, parse_markdown_file};

/// Выводит информацию о репозитории
#[derive(Parser, Debug)]
pub struct InfoCommand {
    /// Показать подробную информацию о репозитории
    #[arg(short, long)]
    pub verbose: bool,
    /// Стиль вывода иерархии
    #[arg(short, long, default_value = "tree")]
    pub format: String,
}

/// Отображает иерархию документа в виде дерева
pub fn print_tree(hierarchy: &DocumentHierarchy) {
    println!("📄 Структура документа:");
    println!();

    for heading in &hierarchy.headings {
        let indent = "  ".repeat((heading.level - 1) as usize);
        let marker = match heading.level {
            1 => "📌",
            2 => "📋",
            3 => "📝",
            4 => "🔸",
            5 => "🔹",
            6 => "🔺",
            _ => "•",
        };
        
        println!("{}{} {} {}", 
            indent, 
            marker, 
            "#".repeat(heading.level as usize), 
            heading.text
        );
    }
}

/// Отображает иерархию без символов
pub fn print_clean(hierarchy: &DocumentHierarchy) {
    println!("Структура документа:");
    println!();

    for heading in &hierarchy.headings {
        let indent = "  ".repeat((heading.level - 1) as usize);
        println!("{}{}", indent, heading.text);
    }
}

/// Отображает иерархию с разделением по главным заголовкам
pub fn print_sections(hierarchy: &DocumentHierarchy) {
    println!("Структура документа:");
    println!();

    let mut current_section = String::new();
    
    for heading in &hierarchy.headings {
        if heading.level == 1 {
            if !current_section.is_empty() {
                println!();
            }
            println!("{}", "=".repeat(50));
            println!("{}", heading.text);
            println!("{}", "=".repeat(50));
            current_section = heading.text.clone();
        } else {
            let indent = "  ".repeat((heading.level - 2) as usize);
            println!("{}{}", indent, heading.text);
        }
    }
}

/// Отображает иерархию в виде списка
pub fn print_list(hierarchy: &DocumentHierarchy) {
    println!("Структура документа:");
    println!();

    for (i, heading) in hierarchy.headings.iter().enumerate() {
        let indent = "  ".repeat((heading.level - 1) as usize);
        println!("{}{}. {}", indent, i + 1, heading.text);
    }
}

/// Выводит информацию о репозитории
pub fn execute_info_command(args: &InfoCommand) -> Result<(), String> {
    if args.verbose {
        println!("🔍 Анализ документа с подробной информацией...");
    }
    
    println!("🔍 Поиск README.md в текущей директории...");
    
    // Ищем README.md в текущей директории
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Ошибка получения текущей директории: {}", e))?;
    let readme_path = current_dir.join("README.md");
    
    if !readme_path.exists() {
        println!("❌ README.md не найден в текущей директории");
        println!("💡 Убедитесь, что файл README.md существует в текущей папке");
        return Err("README.md not found".to_string());
    }
    
    println!("✅ Найден README.md");
    println!();
    
    // Парсим файл используя readual-md
    let hierarchy = parse_markdown_file(&readme_path)?;
    
    if hierarchy.headings.is_empty() {
        println!("⚠️  В файле не найдено заголовков");
        return Ok(());
    }
    
    // Отображаем иерархию в выбранном формате
    match args.format.as_str() {
        "clean" => print_clean(&hierarchy),
        "sections" => print_sections(&hierarchy),
        "list" => print_list(&hierarchy),
        "tree" | _ => print_tree(&hierarchy),
    }
    Ok(())
}

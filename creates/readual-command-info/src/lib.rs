use clap::Parser;
use std::path::PathBuf;
use readual_md::{Title, Document, read_document, parse_titles, HeadingTree, NodeTitle, parse_file_to_tree};

#[macro_use]
extern crate readual_output;

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

/// Shows repository information
#[derive(Parser, Debug)]
pub struct InfoCommand {
    /// Path to README.md file
    #[arg(short = 'r', long = "readme", default_value = "./README.md", value_parser = validate_readme_path)]
    pub file: PathBuf,
    /// Hierarchy output style
    #[arg(short = 'f', long = "format", default_value = "list")]
    pub format: String,
}

/// Форматирует NodeTitle в строку
fn format_node_title(title: &NodeTitle, doc: &Document) -> String {
    match title {
        NodeTitle::None => "<empty>".to_string(),
        NodeTitle::Span(span) => doc.slice(span).trim().to_string(),
        NodeTitle::Text(text) => text.clone(),
    }
}

/// Рекурсивная функция для отрисовки дерева с ANSI символами
fn print_tree_node(node: &HeadingTree, doc: &Document, prefix: &str, is_last: bool) {
    // Определяем символы для текущего узла
    let connector = if is_last { "└── " } else { "├── " };
    let continuation = if is_last { "    " } else { "│   " };
    
    // Форматируем заголовок узла
    let title_text = format_node_title(&node.title, doc);
    
    // Выводим узел
    output!("{}{}{}", prefix, connector, title_text);
    
    // Debug вывод для узла
    debug!("  Node[{}]:", node.id);
    debug!("    title: {:?} -> \"{}\"", node.title, title_text);
    debug!("    level: {}", node.level);
    debug!("    parent: {:?}", node.parent);
    debug!("    aliases: {:?}", node.aliases);
    debug!("    dependencies: {:?}", node.dependencies);
    if let Some(code_span) = &node.code_span {
        let code_text = doc.slice(&code_span);
        debug!("    code_span: {:?} -> \"{}\"", code_span, code_text.trim());
    }
    debug!("    children: {}", node.children.len());
    
    // Рекурсивно выводим детей
    let children_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == children_count - 1;
        let new_prefix = format!("{}{}", prefix, continuation);
        print_tree_node(child, doc, &new_prefix, is_last_child);
    }
}

/// Displays hierarchy as a tree
fn print_tree(tree: &HeadingTree, doc: &Document) {
    info!("Document Structure (Tree):");
    println!();
    
    // Debug вывод корневого узла
    debug!("Root node:");
    debug!("  id: {}", tree.id);
    debug!("  level: {}", tree.level);
    debug!("  children: {}", tree.children.len());
    
    // Выводим дерево, начиная с детей корня (сам корень не выводим)
    let children_count = tree.children.len();
    for (i, child) in tree.children.iter().enumerate() {
        let is_last = i == children_count - 1;
        print_tree_node(child, doc, "", is_last);
    }
}

/// Displays hierarchy as a list
fn print_list(titles: &Vec<Title>, document: &Document) {
    // Debug вывод всего списка заголовков
    debug!("Parsed {} titles:", titles.len());
    for (i, title) in titles.iter().enumerate() {
        let header_text = document.slice(&title.header).trim();
        
        debug!("  Title[{}]:", i);
        debug!("    level: {}", title.level);
        debug!("    header: {:?} -> \"{}\"", title.header, header_text);
        debug!("    body: {:?}", title.body);
        debug!("    directives ({}):", title.directives.len());
        
        // Выводим каждую директиву на отдельной строке
        for (j, dir) in title.directives.iter().enumerate() {
            let span_text = document.slice(&dir.span);
            let payload_text = document.slice(&dir.payload_span);
            debug!("      Directive[{}]:", j);
            debug!("        kind: {:?}", dir.kind);
            debug!("        span: {:?} -> \"{}\"", dir.span, span_text.trim());
            debug!("        payload_span: {:?} -> \"{}\"", dir.payload_span, payload_text.trim());
        }
    }
    
    info!("Document Titles:");
    println!();

    for (i, title) in titles.iter().enumerate() {
        let indent = "  ".repeat((title.level - 1) as usize);
        let header_text = document.slice(&title.header).trim();
        output!("{}.{} [{}] {}", i + 1, indent, title.level, header_text);
    }
}

/// Shows repository information
pub fn execute_info_command(args: &InfoCommand) -> Result<(), String> {
    // Выбираем формат вывода
    match args.format.as_str() {
        "tree" => {
            // Парсим файл в дерево (вся логика парсинга в readual-md)
            let parsed = parse_file_to_tree(&args.file)
                .map_err(|e| format!("Error parsing file: {:?}", e))?;
            
            // Выводим дерево
            print_tree(&parsed.tree, &parsed.document);
        },
        "list" => {
            // Парсим файл в список заголовков
            let document = read_document(&args.file)
                .map_err(|e| format!("Error reading document: {:?}", e))?;
            
            let titles = parse_titles(&document)
                .map_err(|e| format!("Error parsing titles: {:?}", e))?;
            
            // Выводим список
            print_list(&titles, &document);
        },
        _ => {
            return Err(format!("Unknown format: {}. Use 'tree' or 'list'", args.format));
        }
    }

    Ok(())
}

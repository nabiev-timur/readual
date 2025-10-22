use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Heading {
    pub level: u32,
    pub text: String,
    pub line_number: usize,
}

#[derive(Debug)]
pub struct DocumentHierarchy {
    pub headings: Vec<Heading>,
}

impl DocumentHierarchy {
    pub fn new() -> Self {
        Self {
            headings: Vec::new(),
        }
    }

    pub fn add_heading(&mut self, level: u32, text: String, line_number: usize) {
        self.headings.push(Heading {
            level,
            text,
            line_number,
        });
    }
}

/// Парсит Markdown файл по указанному пути и возвращает иерархию заголовков
/// 
/// # Аргументы
/// 
/// * `file_path` - путь к Markdown файлу
/// 
/// # Возвращает
/// 
/// * `Result<DocumentHierarchy, String>` - структура с заголовками или ошибка
/// 
/// # Примеры
/// 
/// ```rust
/// use readual_md::parse_markdown_file;
/// 
/// let hierarchy = parse_markdown_file("README.md")?;
/// for heading in &hierarchy.headings {
///     println!("{} {}", "#".repeat(heading.level as usize), heading.text);
/// }
/// ```
pub fn parse_markdown_file<P: AsRef<Path>>(file_path: P) -> Result<DocumentHierarchy, String> {
    let path = file_path.as_ref();
    
    // Проверяем, что файл существует
    if !path.exists() {
        return Err(format!("Файл не найден: {}", path.display()));
    }
    
    // Проверяем, что это файл
    if !path.is_file() {
        return Err(format!("Путь не является файлом: {}", path.display()));
    }
    
    // Читаем содержимое файла
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Ошибка чтения файла {}: {}", path.display(), e))?;
    
    // Парсим содержимое
    Ok(parse_markdown_content(&content))
}

/// Парсит содержимое Markdown и возвращает иерархию заголовков
/// 
/// # Аргументы
/// 
/// * `content` - содержимое Markdown файла
/// 
/// # Возвращает
/// 
/// * `DocumentHierarchy` - структура с заголовками
pub fn parse_markdown_content(content: &str) -> DocumentHierarchy {
    let mut hierarchy = DocumentHierarchy::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_number, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Проверяем, является ли строка заголовком
        if trimmed.starts_with('#') {
            let level = trimmed.chars()
                .take_while(|&c| c == '#')
                .count() as u32;
            
            // Ограничиваем уровень заголовка (Markdown поддерживает до 6 уровней)
            if level <= 6 {
                let text = trimmed[level as usize..].trim().to_string();
                if !text.is_empty() {
                    hierarchy.add_heading(level, text, line_number + 1);
                }
            }
        }
    }

    hierarchy
}

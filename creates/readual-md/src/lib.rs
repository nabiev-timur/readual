pub mod document;
pub mod directive;
pub mod titles;
pub mod structure;

// Реэкспорт основных типов
pub use document::{Document, ReadError, Span};
pub use document::{from_string, read_document};
pub use document::{LinesIterator, SpanLinesIterator, StrLinesExt, StrLinesIterator};
pub use directive::{Directive, DirectiveType};
pub use titles::{ParseError, Title};
pub use titles::parse_titles;
pub use structure::{HeadingTree, NodeTitle, from_titles};

/// Результат парсинга файла в дерево заголовков
pub struct ParsedDocument {
    pub document: Document,
    pub tree: HeadingTree,
}

/// Парсит файл и возвращает документ и дерево заголовков
/// 
/// # Параметры
/// - `file_path` - путь к файлу для парсинга
/// 
/// # Возвращает
/// - `ParsedDocument` с документом и деревом заголовков
/// 
/// # Ошибки
/// - `ReadError` - ошибка чтения файла
/// - `ParseError` - ошибка парсинга заголовков или построения дерева
pub fn parse_file_to_tree(file_path: &std::path::Path) -> Result<ParsedDocument, ParseError> {
    // Читаем документ
    let document = read_document(file_path)
        .map_err(|e| ParseError::DocumentInvariant(format!("Error reading document: {:?}", e)))?;
    
    // Парсим заголовки
    let titles = parse_titles(&document)?;
    
    // Строим дерево (передаем путь к файлу)
    let tree = from_titles(&titles, &document, file_path)?;
    
    Ok(ParsedDocument { document, tree })
}
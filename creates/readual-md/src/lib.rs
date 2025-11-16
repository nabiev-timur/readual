pub mod document;
pub mod titles;

// Реэкспорт основных типов
pub use document::{Document, ReadError, Span};
pub use document::{from_string, read_document};
pub use titles::{Directive, ParseError, Title};
pub use titles::parse_titles;

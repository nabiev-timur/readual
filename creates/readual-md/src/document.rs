use std::io;
use std::ops::Range;
use std::path::Path;

pub type Span = Range<usize>;

/// Документ с быстрым доступом к строкам и позициям
#[derive(Debug, Clone)]
pub struct Document {
    pub buf: String,
    pub line_starts: Vec<usize>,
}

/// Ошибки чтения документа
#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Utf8(std::string::FromUtf8Error),
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> Self {
        ReadError::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for ReadError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ReadError::Utf8(err)
    }
}

/// Чтение документа из файла
pub fn read_document(path: &Path) -> Result<Document, ReadError> {
    let bytes = std::fs::read(path)?;
    let (buf, _bom_offset) = skip_bom(&bytes)?;
    let mut line_starts = vec![0];
    
    // Обработка разных типов переводов строк: \n, \r\n, \r
    let mut i = 0;
    while i < buf.len() {
        match buf.as_bytes().get(i) {
            Some(b'\r') => {
                // Проверяем, не \r\n ли это
                if i + 1 < buf.len() && buf.as_bytes()[i + 1] == b'\n' {
                    i += 2; // Пропускаем \r\n
                    line_starts.push(i);
                } else {
                    i += 1; // Просто \r
                    line_starts.push(i);
                }
            }
            Some(b'\n') => {
                i += 1;
                line_starts.push(i);
            }
            _ => {
                i += 1;
            }
        }
    }
    
    // Всегда добавляем конец файла
    if line_starts.last() != Some(&buf.len()) {
        line_starts.push(buf.len());
    }
    
    Ok(Document { buf, line_starts })
}

/// Создание документа из строки
pub fn from_string(s: String) -> Document {
    let mut line_starts = vec![0];
    
    let mut i = 0;
    while i < s.len() {
        match s.as_bytes().get(i) {
            Some(b'\r') => {
                if i + 1 < s.len() && s.as_bytes()[i + 1] == b'\n' {
                    i += 2;
                    line_starts.push(i);
                } else {
                    i += 1;
                    line_starts.push(i);
                }
            }
            Some(b'\n') => {
                i += 1;
                line_starts.push(i);
            }
            _ => {
                i += 1;
            }
        }
    }
    
    if line_starts.last() != Some(&s.len()) {
        line_starts.push(s.len());
    }
    
    Document {
        buf: s,
        line_starts,
    }
}

/// Пропуск BOM (Byte Order Mark) в начале файла
fn skip_bom(bytes: &[u8]) -> Result<(String, usize), ReadError> {
    const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
    
    let (buf, bom_offset) = if bytes.starts_with(UTF8_BOM) {
        let without_bom = &bytes[UTF8_BOM.len()..];
        (String::from_utf8(without_bom.to_vec())?, UTF8_BOM.len())
    } else {
        (String::from_utf8(bytes.to_vec())?, 0)
    };
    
    Ok((buf, bom_offset))
}

impl Document {
    /// Получить номер строки для байтовой позиции
    pub fn line_of(&self, byte_pos: usize) -> usize {
        if byte_pos >= self.buf.len() {
            return self.line_starts.len().saturating_sub(1);
        }
        
        // Бинарный поиск в line_starts
        match self.line_starts.binary_search(&byte_pos) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        }
    }
    
    /// Получить Span для строки по индексу
    pub fn line_span(&self, line_idx: usize) -> Span {
        let start = self.line_starts.get(line_idx).copied().unwrap_or(self.buf.len());
        let end = self
            .line_starts
            .get(line_idx + 1)
            .copied()
            .unwrap_or(self.buf.len());
        start..end
    }
    
    /// Получить подстроку по Span
    pub fn slice(&self, span: &Span) -> &str {
        if span.start >= self.buf.len() {
            return "";
        }
        let end = span.end.min(self.buf.len());
        &self.buf[span.start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_file() {
        let doc = from_string(String::new());
        assert_eq!(doc.buf.len(), 0);
        assert_eq!(doc.line_starts, vec![0, 0]);
        assert_eq!(doc.line_span(0), 0..0);
    }

    #[test]
    fn test_unix_newlines() {
        let doc = from_string("line1\nline2\nline3".to_string());
        assert_eq!(doc.line_starts, vec![0, 6, 12, 18]);
        assert_eq!(doc.line_of(0), 0);
        assert_eq!(doc.line_of(5), 0);
        assert_eq!(doc.line_of(6), 1);
        assert_eq!(doc.slice(&doc.line_span(0)), "line1\n");
        assert_eq!(doc.slice(&doc.line_span(1)), "line2\n");
    }

    #[test]
    fn test_windows_newlines() {
        let doc = from_string("line1\r\nline2\r\nline3".to_string());
        assert_eq!(doc.line_starts, vec![0, 7, 14, 20]);
        assert_eq!(doc.slice(&doc.line_span(0)), "line1\r\n");
    }

    #[test]
    fn test_mac_newlines() {
        let doc = from_string("line1\rline2\rline3".to_string());
        assert_eq!(doc.line_starts, vec![0, 6, 12, 18]);
    }

    #[test]
    fn test_mixed_newlines() {
        let doc = from_string("a\nb\r\nc\rd".to_string());
        assert_eq!(doc.line_starts.len(), 5);
    }

    #[test]
    fn test_no_trailing_newline() {
        let doc = from_string("line1\nline2".to_string());
        assert_eq!(doc.line_starts, vec![0, 6, 11]);
        assert_eq!(doc.slice(&doc.line_span(1)), "line2");
    }

    #[test]
    fn test_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"content");
        let (buf, offset) = skip_bom(&bytes).unwrap();
        assert_eq!(offset, 3);
        assert_eq!(buf, "content");
    }

    #[test]
    fn test_read_document() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "line1\nline2\n").unwrap();
        let path = file.path();
        let doc = read_document(path).unwrap();
        assert_eq!(doc.buf, "line1\nline2\n");
        assert!(doc.line_starts.len() >= 3);
    }

    #[test]
    fn test_line_of_boundaries() {
        let doc = from_string("a\nb".to_string());
        assert_eq!(doc.line_of(0), 0);
        assert_eq!(doc.line_of(1), 0);
        assert_eq!(doc.line_of(2), 1);
        assert_eq!(doc.line_of(100), 2); // За пределами
    }

    #[test]
    fn test_slice_boundaries() {
        let doc = from_string("hello".to_string());
        assert_eq!(doc.slice(&(0..5)), "hello");
        assert_eq!(doc.slice(&(0..100)), "hello");
        assert_eq!(doc.slice(&(100..200)), "");
    }
}


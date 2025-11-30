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

		if span.end < span.start {
			return "";
		}
		
        let end = span.end.min(self.buf.len());
        &self.buf[span.start..end]
    }

    /// Итератор по всем строкам документа, возвращает Span для каждой строки
    pub fn lines(&self) -> LinesIterator<'_> {
        LinesIterator {
            doc: self,
            current_line: 0,
        }
    }

    /// Итератор по строкам внутри указанного Span, возвращает Span для каждой строки
    pub fn lines_in_span(&self, span: &Span) -> SpanLinesIterator<'_> {
        let start_line = self.line_of(span.start);
        let end_line = self.line_of(span.end.min(self.buf.len()));
        SpanLinesIterator {
            doc: self,
            span: span.clone(),
            current_line: start_line,
            end_line: end_line + 1, // +1 чтобы включить последнюю строку
        }
    }

	/// Функция для проверки начала строки
	pub fn starts_with(&self, span: &Span, prefix: &str) -> bool {
		self.slice(span).starts_with(prefix)
	}

	/// Функция для проверки вхождения подстроки в span
	pub fn contains(&self, span: &Span, substring: &str) -> bool {
		self.slice(span).contains(substring)
	}

	/// Функция для получения позиции подстроки в документе
	/// Возвращает позицию относительно документа, а не span
	pub fn pos_of_str(&self, span: &Span, str: &str) -> usize {
		let text = self.slice(span);
		if let Some(pos) = text.find(str) {
			span.start + pos
		} else {
			span.end // Если не найдено, возвращаем конец span
		}
	}

	/// Обрезать пробелы в начале span
	pub fn trim_start(&self, span: &Span) -> Span {
		let text = self.slice(span);
		let trimmed = text.trim_start();
		let trim_len = text.len() - trimmed.len();
		(span.start + trim_len)..span.end
	}

	/// Обрезать пробелы в конце span
	pub fn trim_end(&self, span: &Span) -> Span {
		let text = self.slice(span);
		let trimmed = text.trim_end();
		let trim_len = text.len() - trimmed.len();
		span.start..(span.end - trim_len)
	}

	/// Обрезать пробелы в начале и конце span
	pub fn trim(&self, span: &Span) -> Span {
		let trimmed_start = self.trim_start(span);
		self.trim_end(&trimmed_start)
	}

	/// Подсчитать количество вхождений символа в span
	pub fn count_chars(&self, span: &Span, ch: &str) -> u8 {
		let text = self.slice(span);
		text.matches(ch).count() as u8
	}

	/// Функция для захвата групп регулярного выражения в span
	/// Возвращает Vec<Option<Span>>, где:
	/// - [0] - полное совпадение (всегда Some, если есть совпадение)
	/// - [1..] - захваченные группы (Some если группа захвачена, None если нет)
	/// 
	/// ВАЖНО: позиции в Span относительны к документу, а не к строке
	pub fn captures(&self, span: &Span, regex: &str) -> Vec<Option<Span>> {
		use regex::Regex;
		
		let re = match Regex::new(regex) {
			Ok(re) => re,
			Err(_) => return vec![], // Возвращаем пустой вектор при ошибке регулярки
		};
		
		let text = self.slice(span);
		let captures = match re.captures(text) {
			Some(caps) => caps,
			None => return vec![], // Нет совпадений
		};
		
		// Преобразуем все группы в Span с учётом смещения span.start
		captures.iter()
			.map(|opt_match| {
				opt_match.map(|m| {
					let match_span = m.range();
					// Добавляем смещение span.start, чтобы позиции были относительно документа
					(span.start + match_span.start)..(span.start + match_span.end)
				})
			})
			.collect()
	}
}

/// Итератор по строкам документа, возвращает Span для каждой строки
pub struct LinesIterator<'a> {
    doc: &'a Document,
    current_line: usize,
}

impl<'a> Iterator for LinesIterator<'a> {
    type Item = Span;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_line >= self.doc.line_starts.len().saturating_sub(1) {
            return None;
        }

        let span = self.doc.line_span(self.current_line);
        self.current_line += 1;
        Some(span)
    }
}

/// Итератор по строкам внутри Span, возвращает Span для каждой строки
/// Использует line_starts напрямую для определения диапазона строк
pub struct SpanLinesIterator<'a> {
    doc: &'a Document,
    span: Span,
    current_line: usize,
    end_line: usize,
}

impl<'a> Iterator for SpanLinesIterator<'a> {
    type Item = Span;

    fn next(&mut self) -> Option<Self::Item> {
        // Используем line_starts напрямую для определения границ
        if self.current_line >= self.end_line {
            return None;
        }

        // Получаем границы текущей строки из line_starts
        let line_start = *self.doc.line_starts.get(self.current_line)?;
        let line_end = self.doc.line_starts
            .get(self.current_line + 1)
            .copied()
            .unwrap_or(self.doc.buf.len());
        
        // Обрезаем строку по границам span
        let intersection_start = line_start.max(self.span.start);
        let intersection_end = line_end.min(self.span.end);
        
        // Пропускаем строки, которые не пересекаются с span
        if intersection_start >= intersection_end {
            self.current_line += 1;
            return self.next();
        }

        self.current_line += 1;
        Some(intersection_start..intersection_end)
    }
}

/// Extension trait для &str, предоставляющий итератор по строкам
pub trait StrLinesExt {
    /// Итератор по строкам, возвращает &str для каждой строки
    fn lines_iter(&self) -> StrLinesIterator<'_>;
}

impl StrLinesExt for str {
    fn lines_iter(&self) -> StrLinesIterator<'_> {
        StrLinesIterator {
            s: self,
            pos: 0,
        }
    }
}

/// Итератор по строкам для &str, возвращает &str для каждой строки
pub struct StrLinesIterator<'a> {
    s: &'a str,
    pos: usize,
}

impl<'a> Iterator for StrLinesIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.s.len() {
            return None;
        }

        let start = self.pos;
        let bytes = self.s.as_bytes();
        
        // Ищем конец строки
        while self.pos < self.s.len() {
            match bytes.get(self.pos) {
                Some(b'\r') => {
                    // Проверяем, не \r\n ли это
                    if self.pos + 1 < self.s.len() && bytes[self.pos + 1] == b'\n' {
                        let line = &self.s[start..self.pos];
                        self.pos += 2; // Пропускаем \r\n
                        return Some(line);
                    } else {
                        let line = &self.s[start..self.pos];
                        self.pos += 1; // Пропускаем \r
                        return Some(line);
                    }
                }
                Some(b'\n') => {
                    let line = &self.s[start..self.pos];
                    self.pos += 1; // Пропускаем \n
                    return Some(line);
                }
                _ => {
                    self.pos += 1;
                }
            }
        }

        // Последняя строка без завершающего переноса
        if start < self.s.len() {
            let line = &self.s[start..];
            self.pos = self.s.len();
            Some(line)
        } else {
            None
        }
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

    #[test]
    fn test_document_lines_iterator() {
        let doc = from_string("line1\nline2\nline3".to_string());
        let lines: Vec<Span> = doc.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(doc.slice(&lines[0]), "line1\n");
        assert_eq!(doc.slice(&lines[1]), "line2\n");
        assert_eq!(doc.slice(&lines[2]), "line3");
    }

    #[test]
    fn test_document_lines_iterator_empty() {
        let doc = from_string(String::new());
        let lines: Vec<Span> = doc.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(doc.slice(&lines[0]), "");
    }

    #[test]
    fn test_document_lines_iterator_no_trailing_newline() {
        let doc = from_string("line1\nline2".to_string());
        let lines: Vec<Span> = doc.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(doc.slice(&lines[0]), "line1\n");
        assert_eq!(doc.slice(&lines[1]), "line2");
    }

    #[test]
    fn test_span_lines_iterator() {
        let doc = from_string("line1\nline2\nline3\nline4".to_string());
        let span = 6..18; // От начала line2 до конца line3
        let lines: Vec<Span> = doc.lines_in_span(&span).collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(doc.slice(&lines[0]), "line2\n");
        assert_eq!(doc.slice(&lines[1]), "line3");
    }

    #[test]
    fn test_span_lines_iterator_partial() {
        let doc = from_string("line1\nline2\nline3".to_string());
        let span = 7..13; // Часть line2 и часть line3
        let lines: Vec<Span> = doc.lines_in_span(&span).collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(doc.slice(&lines[0]), "ine2");
        assert_eq!(doc.slice(&lines[1]), "line");
    }

    #[test]
    fn test_span_lines_iterator_empty() {
        let doc = from_string("line1\nline2".to_string());
        let span = 0..0;
        let lines: Vec<Span> = doc.lines_in_span(&span).collect();
        assert_eq!(lines.len(), 0);
    }

    #[test]
    fn test_str_lines_iterator() {
        let s = "line1\nline2\nline3";
        let lines: Vec<&str> = s.lines_iter().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    #[test]
    fn test_str_lines_iterator_windows() {
        let s = "line1\r\nline2\r\nline3";
        let lines: Vec<&str> = s.lines_iter().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    #[test]
    fn test_str_lines_iterator_mac() {
        let s = "line1\rline2\rline3";
        let lines: Vec<&str> = s.lines_iter().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    #[test]
    fn test_str_lines_iterator_mixed() {
        let s = "a\nb\r\nc\rd";
        let lines: Vec<&str> = s.lines_iter().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "a");
        assert_eq!(lines[1], "b");
        assert_eq!(lines[2], "c");
        assert_eq!(lines[3], "d");
    }

    #[test]
    fn test_str_lines_iterator_empty() {
        let s = "";
        let lines: Vec<&str> = s.lines_iter().collect();
        assert_eq!(lines.len(), 0);
    }

    #[test]
    fn test_str_lines_iterator_no_trailing_newline() {
        let s = "line1\nline2";
        let lines: Vec<&str> = s.lines_iter().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
    }

    #[test]
    fn test_str_lines_iterator_single_line() {
        let s = "single line";
        let lines: Vec<&str> = s.lines_iter().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "single line");
    }
}


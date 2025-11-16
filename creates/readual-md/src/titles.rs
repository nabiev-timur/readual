use crate::document::{Document, Span};

/// Директива в документе
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive {
    pub span: Span,
    pub generated: Option<String>,
}

/// Заголовок с его секцией и директивами
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title {
    pub span: Option<Span>,
    pub directives: Vec<Directive>,
}

/// Ошибки парсинга
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    DocumentInvariant(String),
}

/// Парсинг заголовков и директив из документа
pub fn parse_titles(doc: &Document) -> Result<Vec<Title>, ParseError> {
    let mut titles = Vec::new();
    let mut header_positions = Vec::new();
    
    // Шаг 1: Найти все заголовки и их уровни
    let bytes = doc.buf.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        // Пропускаем пробелы в начале строки
        let line_start = i;
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        
        // Проверяем, начинается ли строка с #
        let mut level = 0;
        let mut j = i;
        while j < bytes.len() && bytes[j] == b'#' && level < 6 {
            level += 1;
            j += 1;
        }
        
        if level > 0 && level <= 6 {
            // Проверяем, что после # идёт пробел
            if j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                header_positions.push((line_start, level));
            }
        }
        
        // Переходим к следующей строке
        while i < bytes.len() && bytes[i] != b'\n' && bytes[i] != b'\r' {
            i += 1;
        }
		
        if i < bytes.len() {
            if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                i += 2;
            } else {
                i += 1;
            }
        }
    }
    
    if header_positions.is_empty() {
        return Ok(vec![]);
    }
    
    // Шаг 2: Определить секции для каждого заголовка
    for (idx, &(header_start, header_level)) in header_positions.iter().enumerate() {
        // Находим конец секции: следующий заголовок уровня <= текущего или конец файла
        let section_end = if idx + 1 < header_positions.len() {
            let mut next_idx = idx + 1;
            while next_idx < header_positions.len() {
                let (_, next_level) = header_positions[next_idx];
                if next_level <= header_level {
                    break;
                }
                next_idx += 1;
            }
            if next_idx < header_positions.len() {
                header_positions[next_idx].0
            } else {
                doc.buf.len()
            }
        } else {
            doc.buf.len()
        };
        
        let span = header_start..section_end;
        
        // Шаг 3: Собрать все директивы для этой секции
        let mut directives = Vec::new();
        
        // 3.1: Поиск сокращённых алиасов в строке заголовка
        let header_line = doc.slice(&doc.line_span(doc.line_of(header_start)));
        find_short_aliases(header_line, header_start, &mut directives);
        
        // 3.2: Поиск явных директив <!-- rdl:... --> в теле секции
        find_explicit_directives(&doc.buf, span.start, span.end, &mut directives);
        
        // 3.3: Поиск блоков кода ```
        find_code_blocks(&doc.buf, span.start, span.end, &mut directives);
        
        // 3.4: Сортировка директив по позиции
        directives.sort_by_key(|d| d.span.start);
        
        titles.push(Title { span, directives });
    }
    
    Ok(titles)
}

/// Поиск сокращённых алиасов [a, b|c] в строке заголовка
fn find_short_aliases(header_line: &str, line_start: usize, directives: &mut Vec<Directive>) {
    let mut chars = header_line.char_indices().peekable();
    
    while let Some((i, ch)) = chars.next() {
        if ch == '[' {
            let bracket_start_byte = i;
            
            // Ищем закрывающую скобку
            let mut bracket_end_byte = None;
            while let Some((j, ch)) = chars.next() {
                if ch == ']' {
                    bracket_end_byte = Some(j + ch.len_utf8());
                    break;
                }
                if ch == '\n' || ch == '\r' {
                    // Незакрытая скобка - игнорируем
                    break;
                }
            }
            
            if let Some(end_byte) = bracket_end_byte {
                // Извлекаем содержимое между скобками
                let content = &header_line[bracket_start_byte + 1..end_byte - 1];
                
                // Разбиваем по запятым, вертикальным чертам и пробелам
                let tokens: Vec<&str> = content
                    .split(|c: char| c == ',' || c == '|' || c.is_whitespace())
                    .filter(|s| !s.is_empty())
                    .collect();
                
                // Создаём директиву для каждого токена
                for token in tokens {
                    let token_trimmed = token.trim();
                    if !token_trimmed.is_empty() {
                        // Находим позицию этого токена в исходной строке (в байтах)
                        if let Some(rel_pos) = content.find(token_trimmed) {
                            let token_start_byte = line_start + bracket_start_byte + 1 + rel_pos;
                            let token_end_byte = token_start_byte + token_trimmed.len();
                            let span = token_start_byte..token_end_byte;
                            
                            directives.push(Directive {
                                span,
                                generated: Some(format!("<!-- rdl:alias={} -->", token_trimmed)),
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Поиск явных директив <!-- rdl:... -->
fn find_explicit_directives(
    buf: &str,
    start: usize,
    end: usize,
    directives: &mut Vec<Directive>,
) {
    // Безопасно получаем подстроку, проверяя границы символов
    let search_text = if end <= buf.len() && buf.is_char_boundary(start) && buf.is_char_boundary(end) {
        &buf[start..end]
    } else {
        return;
    };
    
    // Используем простой поиск подстроки
    let mut search_from = 0;
    while let Some(comment_start_rel) = search_text[search_from..].find("<!--") {
        let comment_start_abs = start + search_from + comment_start_rel;
        let after_comment = search_from + comment_start_rel + 4;
        
        // Проверяем, что после <!-- есть rdl:
        if let Some(rdl_pos) = search_text[after_comment..].find("rdl:") {
            // Ищем закрывающий -->
            if let Some(close_pos) = search_text[after_comment + rdl_pos..].find("-->") {
                let comment_end_abs = start + after_comment + rdl_pos + close_pos + 3;
                let span = comment_start_abs..comment_end_abs;
                
                directives.push(Directive {
                    span,
                    generated: None,
                });
                
                search_from = after_comment + rdl_pos + close_pos + 3;
            } else {
                break;
            }
        } else {
            search_from = after_comment;
        }
    }
}

/// Поиск блоков кода ```
fn find_code_blocks(
    buf: &str,
    start: usize,
    end: usize,
    directives: &mut Vec<Directive>,
) {
    // Безопасно получаем подстроку, проверяя границы символов
    let search_text = if end <= buf.len() && buf.is_char_boundary(start) && buf.is_char_boundary(end) {
        &buf[start..end]
    } else {
        return;
    };
    
    let mut search_from = 0;
    while let Some(open_pos) = search_text[search_from..].find("```") {
        let block_start_abs = start + search_from + open_pos;
        let after_open = search_from + open_pos + 3;
        
        // Ищем закрывающий ```
        if let Some(close_pos) = search_text[after_open..].find("```") {
            let block_end_abs = start + after_open + close_pos + 3;
            let span = block_start_abs..block_end_abs;
            
            directives.push(Directive {
                span,
                generated: Some("<!-- rdl:code -->".to_string()),
            });
            
            search_from = after_open + close_pos + 3;
        } else {
            // Незакрытый блок - игнорируем
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::from_string;

    #[test]
    fn test_single_title_no_body() {
        let doc = from_string("# Title".to_string());
        let titles = parse_titles(&doc).unwrap();
        assert_eq!(titles.len(), 1);
        assert_eq!(titles[0].directives.len(), 0);
    }

    #[test]
    fn test_multiple_levels() {
        let doc = from_string("# H1\ncontent\n## H2\nmore\n### H3\ntext\n## H2-2\nend".to_string());
        let titles = parse_titles(&doc).unwrap();
        assert_eq!(titles.len(), 3);
        // H1 должен включать всё до H2-2
        assert!(titles[0].span.end > titles[1].span.start);
    }

    #[test]
    fn test_short_aliases() {
        let doc = from_string("## [build] Title".to_string());
        let titles = parse_titles(&doc).unwrap();
        assert_eq!(titles.len(), 1);
        assert_eq!(titles[0].directives.len(), 1);
        assert_eq!(
            titles[0].directives[0].generated,
            Some("<!-- rdl:alias=build -->".to_string())
        );
    }

    #[test]
    fn test_aliases_with_separators() {
        let doc = from_string("## [a, b|c] Title".to_string());
        let titles = parse_titles(&doc).unwrap();
        assert_eq!(titles.len(), 1);
        // Должно быть 3 алиаса: a, b, c
        let alias_dirs: Vec<_> = titles[0]
            .directives
            .iter()
            .filter(|d| d.generated.is_some())
            .collect();
        assert!(alias_dirs.len() >= 3);
    }

    #[test]
    fn test_explicit_directive() {
        let doc = from_string("## Title\n<!-- rdl:note=keep -->\ntext".to_string());
        let titles = parse_titles(&doc).unwrap();
        assert_eq!(titles.len(), 1);
        let explicit: Vec<_> = titles[0]
            .directives
            .iter()
            .filter(|d| d.generated.is_none())
            .collect();
        assert_eq!(explicit.len(), 1);
    }

    #[test]
    fn test_code_block() {
        let doc = from_string("## Title\n```sh\necho hi\n```\ntext".to_string());
        let titles = parse_titles(&doc).unwrap();
        assert_eq!(titles.len(), 1);
        let code_dirs: Vec<_> = titles[0]
            .directives
            .iter()
            .filter(|d| {
                d.generated
                    .as_ref()
                    .map(|s| s == "<!-- rdl:code -->")
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(code_dirs.len(), 1);
    }

    #[test]
    fn test_unclosed_code_block() {
        let doc = from_string("## Title\n```sh\necho hi\ntext".to_string());
        let titles = parse_titles(&doc).unwrap();
        assert_eq!(titles.len(), 1);
        // Незакрытый блок должен быть проигнорирован
        let code_dirs: Vec<_> = titles[0]
            .directives
            .iter()
            .filter(|d| {
                d.generated
                    .as_ref()
                    .map(|s| s == "<!-- rdl:code -->")
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(code_dirs.len(), 0);
    }

    #[test]
    fn test_mixed_directives() {
        let doc = from_string(
            "## [build] Title\n<!-- rdl:note=keep -->\n```sh\necho\n```".to_string(),
        );
        let titles = parse_titles(&doc).unwrap();
        assert_eq!(titles.len(), 1);
        // Должны быть: alias, explicit, code
        assert!(titles[0].directives.len() >= 3);
        // Проверяем сортировку
        for i in 1..titles[0].directives.len() {
            assert!(
                titles[0].directives[i - 1].span.start
                    <= titles[0].directives[i].span.start
            );
        }
    }

    #[test]
    fn test_example_from_spec() {
        let input = "## [build] Build-linux-x64\n\nSome text\n\n<!-- rdl:note=keep -->\n\n```sh\necho hi\n```\n\n### [ci|test] Sub\n";
        let doc = from_string(input.to_string());
        let titles = parse_titles(&doc).unwrap();
        
        assert_eq!(titles.len(), 2);
        
        // Первый заголовок должен содержать alias=build, rdl:note=keep, rdl:code
        let first = &titles[0];
        assert!(first.directives.iter().any(|d| d
            .generated
            .as_ref()
            .map(|s| s.contains("alias=build"))
            .unwrap_or(false)));
        assert!(first.directives.iter().any(|d| d.generated.is_none()));
        assert!(first.directives.iter().any(|d| d
            .generated
            .as_ref()
            .map(|s| s == "<!-- rdl:code -->")
            .unwrap_or(false)));
        
        // Второй заголовок должен содержать alias=ci и alias=test
        let second = &titles[1];
        let aliases: Vec<_> = second
            .directives
            .iter()
            .filter_map(|d| d.generated.as_ref())
            .filter(|s| s.contains("alias="))
            .collect();
        assert!(aliases.len() >= 2);
    }

    #[test]
    fn test_multiple_brackets_in_header() {
        let doc = from_string("## [a] [b] Title".to_string());
        let titles = parse_titles(&doc).unwrap();
        assert_eq!(titles.len(), 1);
        // Оба алиаса должны быть найдены
        let alias_count = titles[0]
            .directives
            .iter()
            .filter(|d| d.generated.is_some())
            .count();
        assert!(alias_count >= 2);
    }
}


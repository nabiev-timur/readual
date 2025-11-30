use crate::document::{Document, Span};
use crate::directive::{Directive, DirectiveType};

/// Заголовок с его секцией и директивами
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title {
	pub header: Span,
	pub level: u8,
    pub body: Span,
    pub directives: Vec<Directive>,
}

/// Ошибки парсинга
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    DocumentInvariant(String),
}

/// Парсинг заголовков и директив из документа
pub fn parse_titles(doc: &Document) -> Result<Vec<Title>, ParseError> {
    let mut titles = Vec::<Title>::new();
	let mut code_block_start = Option::<usize>::None; // Позиция начала ``` для кода

	for line in doc.lines() {
		// При нахождении нового заголовка, если есть предыдущий, завершаем его body
		if doc.starts_with(&doc.trim_start(&line), "#") {
			// Заканчиваем body предыдущего заголовка на начало текущей строки
			if let Some(prev_title) = titles.last_mut() {
				prev_title.body = prev_title.header.start..line.start;
			}
			
			let level = doc.count_chars(&line, "#");
			
			if code_block_start.is_some() {
				return Err(ParseError::DocumentInvariant("Invalid block".to_string())); // TODO add Span line
			}

			// Регулярка для парсинга заголовка: захватывает опциональный [alias] и обязательный текст заголовка
			// Группа 1 (опциональная): [alias] со скобками
			// Группа 2 (обязательная): текст заголовка
			let result = doc.captures(&line, r"^\s*#+\s+(\[[^\]]+\])?\s*(.+)\s*$");
			
			// Проверяем, что регулярка сработала и есть обязательная группа 2 (текст заголовка)
			let title_span = match result.get(2) {
				Some(Some(span)) => span.clone(),
				_ => return Err(ParseError::DocumentInvariant("Title header is empty or invalid format".to_string())),
			};

			// Обрабатываем опциональную группу 1 (alias из заголовка)
			let mut directives = Vec::new();
			if let Some(Some(alias_span)) = result.get(1) {
				// alias_span содержит [alias] со скобками
				// payload_span - только содержимое без скобок
				let payload_start = alias_span.start + 1; // Пропускаем [
				let payload_end = alias_span.end - 1; // Пропускаем ]
				let payload_span = payload_start..payload_end;
				
				directives.push(Directive::new(
					DirectiveType::Alias,
					alias_span.clone(), // span включает скобки
					payload_span,       // payload_span только содержимое
				));
			}

			let header = title_span;
			let body = line.end..line.end;
			titles.push(Title { header, level, body, directives });
			continue;
		}

		if titles.is_empty() {
			continue;
		}

		// проверяем многострочные кодовые блоки ```
		if doc.contains(&line, "```") {
			if code_block_start.is_none() {
				// Начало блока кода: запоминаем позицию начала ```
				let code_start_pos = doc.pos_of_str(&line, "```");
				code_block_start = Some(code_start_pos); // Сохраняем начало ```
			} else {
				// Конец блока кода
				let span_start = code_block_start.unwrap(); // Начало первого ```
				let closing_code_pos = doc.pos_of_str(&line, "```");
				let span_end = closing_code_pos + 3; // Конец закрывающих ```
				let span = span_start..span_end;
				
				// payload_span - только код, без ``` и языка
				// Находим конец строки с первым ``` (после языка)
				let first_line_idx = doc.line_of(span_start);
				let first_line_span = doc.line_span(first_line_idx);
				let payload_start = first_line_span.end; // Начало payload (после ``` и языка, на следующей строке)
				let payload_end = closing_code_pos; // Конец payload (до закрывающих ```)
				let payload_span = payload_start..payload_end;
				
				if let Some(last_title) = titles.last_mut() {
					last_title.directives.push(Directive::new(
						DirectiveType::Code,
						span,
						payload_span,
					));
				}

				code_block_start = None;
			}
		}
		
		// проверяем однострочные кодовые блоки ``
		if doc.contains(&line, "`") && !doc.contains(&line, "```") {
			// Ищем пару `` на одной строке
			let line_text = doc.slice(&line);
			if let Some(first_pos) = line_text.find('`') {
				let first_backtick = line.start + first_pos;
				// Ищем вторую обратную кавычку после первой
				let after_first_text = &line_text[first_pos + 1..];
				if let Some(second_pos) = after_first_text.find('`') {
					let second_backtick = first_backtick + 1 + second_pos;
					
					// Нашли однострочный блок кода
					let span_start = first_backtick;
					let span_end = second_backtick + 1; // Включаем вторую `
					let span = span_start..span_end;
					
					// payload_span - только код, без обратных кавычек
					let payload_start = first_backtick + 1; // После первой `
					let payload_end = second_backtick; // До второй `
					let payload_span = payload_start..payload_end;
					
					if let Some(last_title) = titles.last_mut() {
						last_title.directives.push(Directive::new(
							DirectiveType::Code,
							span,
							payload_span,
						));
					}
				}
			}
		}

		// Парсим комментарии <!-- rdl:alias=... --> и <!-- rdl:deps=... -->
		if doc.contains(&line, "<!-- rdl:") {
			if doc.contains(&line, "-->") {
				// Однострочный комментарий
				let span_start = doc.pos_of_str(&line, "<!-- rdl:");
				let span_end = doc.pos_of_str(&line, "-->") + "-->".len();
				let span = span_start..span_end;
				
				// Определяем тип директивы и payload
				let comment_text = doc.slice(&span);
				if let Some(alias_pos) = comment_text.find("alias=") {
					// Директива alias
					let payload_start = span_start + alias_pos + "alias=".len();
					let payload_end = span_end - "-->".len();
					let payload_span = payload_start..payload_end;
					
					if let Some(last_title) = titles.last_mut() {
						last_title.directives.push(Directive::new(
							DirectiveType::Alias,
							span,
							payload_span,
						));
					}
				} else if let Some(deps_pos) = comment_text.find("deps=") {
					// Директива deps
					let payload_start = span_start + deps_pos + "deps=".len();
					let payload_end = span_end - "-->".len();
					let payload_span = payload_start..payload_end;
					
					if let Some(last_title) = titles.last_mut() {
						last_title.directives.push(Directive::new(
							DirectiveType::Dependencies,
							span,
							payload_span,
						));
					}
				}
			}
			// Многострочные комментарии пока не обрабатываем
		}
	}
	
	if let Some(prev_title) = titles.last_mut() {
		// Заканчиваем body предыдущего заголовка на начало текущей строки
		prev_title.body = prev_title.header.start..*doc.line_starts.last().unwrap();
	}
	
    Ok(titles)
}

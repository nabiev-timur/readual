use crate::document::{Document, Span};
use crate::titles::{Title, ParseError};
use crate::directive::{DirectiveType};

#[derive(Debug, Clone)]
pub enum NodeTitle {
    None,
    Span(Span),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct HeadingTree {
	pub id: usize,
	pub title: NodeTitle,
	pub level: u8,
	pub aliases: Vec<String>,
	pub code_span: Option<Span>,
	pub dependencies: Vec<usize>,
	pub children: Vec<HeadingTree>,
	pub parent: Option<usize>,
}

impl HeadingTree {
	pub fn new(id: usize, title: NodeTitle, level: u8, parent: Option<usize>) -> Self {
		Self { id, title, level, aliases: Vec::new(), code_span: None, dependencies: Vec::new(), children: Vec::new(), parent }
	}

	pub fn add_child(&mut self, child: HeadingTree) -> &mut Self {
		self.children.push(child);
		self
	}

	pub fn add_dependency(&mut self, dependency: usize) -> &mut Self {
		self.dependencies.push(dependency);
		self
	}

	pub fn add_alias(&mut self, alias: String) -> &mut Self {
		self.aliases.push(alias);
		self
	}

	pub fn add_code_span(&mut self, code_span: Span) -> &mut Self {
		self.code_span = Some(code_span);
		self
	}

	/// Найти узел по ID (рекурсивный поиск)
	pub fn find_node_by_id(&self, target_id: usize) -> Option<&HeadingTree> {
		if self.id == target_id {
			return Some(self);
		}
		
		for child in &self.children {
			if let Some(found) = child.find_node_by_id(target_id) {
				return Some(found);
			}
		}
		
		None
	}
}

/// Вспомогательная функция для получения мутабельной ссылки на узел по пути
/// Путь - это вектор индексов в children, начиная от root
fn get_node_mut<'a>(node: &'a mut HeadingTree, path: &[usize]) -> Option<&'a mut HeadingTree> {
	if path.is_empty() {
		return Some(node);
	}
	
	let idx = path[0];
	if idx >= node.children.len() {
		return None;
	}
	
	get_node_mut(&mut node.children[idx], &path[1..])
}

/// Получить уровень последнего узла по пути
fn get_node_level(node: &HeadingTree, path: &[usize]) -> Option<u8> {
	if path.is_empty() {
		return Some(node.level);
	}
	
	let idx = path[0];
	if idx >= node.children.len() {
		return None;
	}
	
	get_node_level(&node.children[idx], &path[1..])
}

pub fn from_titles(titles: &Vec<Title>, doc: &Document, file_path: &std::path::Path) -> Result<HeadingTree, ParseError> {
	// Получаем имя родительской папки файла для корневого узла
	let doc_folder = file_path
		.parent()
		.and_then(|p| p.file_name())
		.and_then(|n| n.to_str())
		.unwrap_or("root")
		.to_string();
	let mut root = HeadingTree::new(0, NodeTitle::Text(doc_folder), 0, None);
	
	// Стек для хранения пути к последнему узлу (вектор индексов в children)
	let mut path_stack: Vec<usize> = Vec::new();
	
	// Счетчик для уникальных id (начинаем с 1, так как 0 - это root)
	let mut next_id = 1;
	
	for title in titles {
		if title.level < 1 || title.level > 6 {
			return Err(ParseError::DocumentInvariant(
				format!("Invalid title level {}, must be between 1 and 6", title.level)
			));
		}
		
		// Получаем уровень последнего узла
		let last_level = if path_stack.is_empty() {
			0 // root level
		} else {
			get_node_level(&root, &path_stack).unwrap_or(0)
		};
		
		// Определяем сценарий и находим правильного родителя
		let target_level = title.level;
		let level_diff = target_level as i16 - last_level as i16;
		
		if level_diff == 1 {
			// Сценарий 1: Заголовок на один уровень больше предыдущего
			// Добавляем как ребенка последнему узлу
			// path_stack уже указывает на правильного родителя
		} else if level_diff == 0 {
			// Сценарий 2: Заголовок того же уровня
			// Добавляем как ребенка родителю последнего узла
			// Нужно подняться на один уровень вверх
			if !path_stack.is_empty() {
				path_stack.pop();
			}
		} else if level_diff > 1 {
			// Сценарий 3: Заголовок больше чем на один уровень
			// Добавляем пустые узлы пока не достигнем уровня на один меньше
			// Затем добавляем ему ребенка
			let mut current_level = last_level;
			while current_level + 1 < target_level {
				// Получаем id родителя для пустого узла
				let parent_id = if path_stack.is_empty() {
					0 // root
				} else {
					// Находим id последнего узла в пути
					let mut node = &root;
					for &idx in &path_stack {
						node = &node.children[idx];
					}
					node.id
				};
				
				// Создаем пустой узел
				let empty_node = HeadingTree::new(
					next_id,
					NodeTitle::None,
					current_level + 1,
					Some(parent_id)
				);
				
				// Добавляем пустой узел
				if let Some(parent) = get_node_mut(&mut root, &path_stack) {
					parent.add_child(empty_node);
					path_stack.push(parent.children.len() - 1);
					current_level += 1;
					next_id += 1;
				} else {
					return Err(ParseError::DocumentInvariant("Failed to add empty node".to_string()));
				}
			}
		} else {
			// Сценарий 4: Заголовок меньшего уровня (level_diff < 0)
			// Идем по родителям пока не получим уровень на один меньше
			let target_parent_level = target_level - 1;
			
			// Поднимаемся по пути, пока не найдем узел с нужным уровнем
			while !path_stack.is_empty() {
				let current_path = path_stack.clone();
				if let Some(level) = get_node_level(&root, &current_path) {
					if level <= target_parent_level {
						break;
					}
				}
				path_stack.pop();
			}
			
			// Если мы на root (путь пуст), но нужен уровень > 0, это ошибка
			if path_stack.is_empty() && target_parent_level > 0 {
				return Err(ParseError::DocumentInvariant(
					format!("Cannot create level {} node as child of root (level 0)", target_level)
				));
			}
		}
		
		// Создаем новый узел для заголовка
		let parent_id = if path_stack.is_empty() {
			0 // root
		} else {
			// Получаем id родителя
			let mut node = &root;
			for &idx in &path_stack {
				node = &node.children[idx];
			}
			node.id
		};
		
		let mut new_node = HeadingTree::new(
			next_id,
			NodeTitle::Span(title.header.clone()),
			target_level,
			Some(parent_id)
		);

		// Обрабатываем директивы заголовка
		for directive in &title.directives {
			match directive.kind {
				DirectiveType::Alias => {
					// Извлекаем alias из payload_span
					let alias_text = doc.slice(&directive.payload_span).trim().to_string();
					new_node.add_alias(alias_text);
				},
				DirectiveType::Code => {
					// Сохраняем span кода
					new_node.add_code_span(directive.payload_span.clone());
				},
				DirectiveType::Dependencies => {
					// TODO: обработать зависимости
					// Пока не обрабатываем
				},
			}
		}
		
		// Добавляем узел в дерево
		if let Some(parent) = get_node_mut(&mut root, &path_stack) {
			parent.add_child(new_node);
			// Обновляем путь - теперь указывает на только что добавленный узел
			path_stack.push(parent.children.len() - 1);
			next_id += 1;
		} else {
			return Err(ParseError::DocumentInvariant("Failed to add title node".to_string()));
		}
	}

	Ok(root)
}


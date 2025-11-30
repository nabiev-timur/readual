use std::collections::{HashMap, HashSet};
use readual_md::{HeadingTree, NodeTitle, Document};

/// Представляет один путь к узлу в дереве заголовков
#[derive(Debug, Clone)]
pub struct PathEntry {
    /// Компоненты пути: ["readual", "readual", "env", "Linux-x64-rpm"]
    pub segments: Vec<String>,
    /// ID узла в дереве заголовков
    pub node_id: usize,
}

/// Индекс всех возможных путей к узлам в дереве заголовков
/// Поддерживает поиск по полным, частичным путям и путям с alias-ами
pub struct PathIndex {
    /// Все записи путей
    pub entries: Vec<PathEntry>,
    /// Индекс: полный путь -> индексы в entries (один путь может иметь несколько вариантов)
    pub path_to_indices: HashMap<String, Vec<usize>>,
    /// Индекс: node_id -> индексы в entries (один узел может иметь несколько путей через alias-ы)
    pub node_id_to_indices: HashMap<usize, Vec<usize>>,
}

impl PathIndex {
    /// Создать пустой индекс
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            path_to_indices: HashMap::new(),
            node_id_to_indices: HashMap::new(),
        }
    }

    /// Построить индекс из дерева заголовков
    pub fn from_tree(tree: &HeadingTree, doc: &Document) -> Self {
        let mut index = Self::new();
        // Начинаем с пустого пути (корневой узел)
        let initial_paths = vec![Vec::new()];
        Self::build_index_recursive(tree, doc, &initial_paths, &mut index);
        index
    }

    /// Рекурсивно строит индекс, обходя дерево
    /// current_path_combinations - все возможные комбинации путей до текущего узла
    fn build_index_recursive(
        node: &HeadingTree,
        doc: &Document,
        current_path_combinations: &[Vec<String>],
        index: &mut PathIndex,
    ) {
        // Получаем все возможные сегменты для текущего узла
        let node_segments = Self::get_node_segments(node, doc);

        // Генерируем все комбинации путей с учетом текущего узла
        let new_path_combinations = if !node_segments.is_empty() {
            // Если у узла есть сегменты, комбинируем все пути с всеми сегментами
            let mut combinations = Vec::new();
            for path in current_path_combinations {
                for segment in &node_segments {
                    let mut new_path = path.clone();
                    new_path.push(segment.clone());
                    combinations.push(new_path);
                }
            }
            combinations
        } else {
            // Если у узла нет сегментов (NodeTitle::None), пропускаем его в пути
            current_path_combinations.to_vec()
        };

        // Добавляем все варианты путей для текущего узла в индекс
        for segments in &new_path_combinations {
            let path_str = segments.join("::");
            let entry_index = index.entries.len();
            
            let entry = PathEntry {
                segments: segments.clone(),
                node_id: node.id,
            };
            
            index.entries.push(entry);
            
            // Обновляем индексы
            index.path_to_indices
                .entry(path_str)
                .or_insert_with(Vec::new)
                .push(entry_index);
            
            index.node_id_to_indices
                .entry(node.id)
                .or_insert_with(Vec::new)
                .push(entry_index);
        }
        
        // Рекурсивно обрабатываем детей со всеми комбинациями путей
        for child in &node.children {
            Self::build_index_recursive(child, doc, &new_path_combinations, index);
        }
    }

    /// Получить все возможные сегменты для узла (заголовок + alias-ы)
    fn get_node_segments(node: &HeadingTree, doc: &Document) -> Vec<String> {
        let mut segments = Vec::new();
        
        // Добавляем заголовок, если он есть
        match &node.title {
            NodeTitle::None => {
                // Пустой узел - не добавляем сегмент
            }
            NodeTitle::Span(span) => {
                let title_text = doc.slice(span).trim().to_string();
                if !title_text.is_empty() {
                    segments.push(title_text);
                }
            }
            NodeTitle::Text(text) => {
                segments.push(text.clone());
            }
        }
        
        // Добавляем alias-ы
        for alias in &node.aliases {
            if !segments.contains(alias) {
                segments.push(alias.clone());
            }
        }
        
        segments
    }

    /// Парсит входной путь на компоненты
    /// Правильно обрабатывает пустые сегменты: "test::::x86" -> ["test", "", "", "x86"]
    /// Логика: каждая пара "::" разделяет сегменты, несколько пар подряд создают пустые сегменты
    fn parse_path(input: &str) -> Vec<String> {
        // Пустая строка -> один пустой элемент
        if input.is_empty() {
            return vec![String::new()];
        }

        // Разбиваем по подстроке "::", сохраняя пустые сегменты
        input
            .split("::")
            .map(|s| s.to_string())
            .collect()
    }

    /// Найти узлы по пути запроса
    /// Возвращает уникальные node_id всех подходящих узлов
    /// 
    /// Логика поиска: "прикладываем трафарет" запроса к каждому пути
    /// - Пустые сегменты в запросе (`::`) означают "любой сегмент"
    /// - Если запрос начинается с `::`, то до первого непустого сегмента может быть любая вложенность
    /// - Если запрос заканчивается на `::`, то после последнего непустого сегмента может быть любая вложенность
    pub fn find_nodes(&self, query: &str) -> Vec<usize> {
        let query_segments = Self::parse_path(query);
        
        debug!("Searching for path: \"{}\"", query);
        debug!("  Parsed segments: {:?}", query_segments);
        
        let mut found_node_ids = HashSet::new();
        
        // Прикладываем "трафарет" к каждому пути
        for entry in &self.entries {
            if Self::match_path_template(&entry.segments, &query_segments) {
                found_node_ids.insert(entry.node_id);
                let path_str = entry.segments.join("::");
                debug!("  Match found: path=\"{}\", node_id={}", path_str, entry.node_id);
            }
        }
        
        // Преобразуем в отсортированный вектор для детерминированного порядка
        let mut result: Vec<usize> = found_node_ids.into_iter().collect();
        result.sort();
        
        debug!("  Found {} unique node(s): {:?}", result.len(), result);
        
        result
    }

	/// Проверяет, совпадают ли два элемента (пустой элемент считается "любым")
	fn elem_match(x: &str, y: &str) -> bool {
		x.is_empty() || y.is_empty() || x == y
	}

	/// Проверяет, совпадает ли шаблон needle с частью haystack, начиная с позиции start
	fn matches_at(haystack: &[String], needle: &[String], start: usize) -> bool {
		if start + needle.len() > haystack.len() {
			return false;
		}
		for i in 0..needle.len() {
			if !Self::elem_match(&haystack[start + i], &needle[i]) {
				return false;
			}
		}
		true
	}

    /// Проверяет, соответствует ли путь шаблону запроса
    /// 
    /// # Параметры
    /// - `path_segments` - сегменты пути для проверки
    /// - `template_segments` - непустые сегменты шаблона (трафарета)
    /// - `allow_any_prefix` - если true, до первого сегмента шаблона может быть любая вложенность
    /// - `allow_any_suffix` - если true, после последнего сегмента шаблона может быть любая вложенность
    fn match_path_template(
        path_segments: &[String],
        template_segments: &[String]
    ) -> bool {
        let n = path_segments.len();
		let m = template_segments.len();

		if m == 0 {
			return true; // пустой шаблон всегда матчится
		}

		if m > n {
			return false;
		}

		for start in 0..=n - m {
			if Self::matches_at(path_segments, template_segments, start) {
				return true;
			}
		}

		false
    }

    /// Вывести debug информацию об индексе
    pub fn debug_print(&self) {
        debug!("PathIndex contains {} entries", self.entries.len());
        debug!("  path_to_indices: {} paths", self.path_to_indices.len());
        debug!("  node_id_to_indices: {} nodes", self.node_id_to_indices.len());
        
        debug!("All paths in index:");
        for (i, entry) in self.entries.iter().enumerate() {
            let path_str = entry.segments.join("::");
            debug!("  [{}] path=\"{}\", segments={:?}, node_id={}", 
                i, path_str, entry.segments, entry.node_id);
        }
    }

}

impl Default for PathIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::PathIndex;

    #[test]
    fn test_parse_path_simple() {
        let result = PathIndex::parse_path("test::x86");
        assert_eq!(result, vec!["test".to_string(), "x86".to_string()]);
    }

    #[test]
    fn test_parse_path_with_empty_segments() {
        let result = PathIndex::parse_path("test::::x86");
        assert_eq!(result, vec!["test".to_string(), "".to_string(), "x86".to_string()]);
    }

    #[test]
    fn test_parse_path_starts_with_colon() {
        let result = PathIndex::parse_path("::test");
        assert_eq!(result, vec!["".to_string(), "test".to_string()]);
    }

    #[test]
    fn test_parse_path_ends_with_colon() {
        let result = PathIndex::parse_path("test::");
        assert_eq!(result, vec!["test".to_string(), "".to_string()]);
    }

    #[test]
    fn test_parse_path_only_colons() {
        let result = PathIndex::parse_path("::");
        assert_eq!(result, vec!["".to_string(), "".to_string()]);
    }

    #[test]
    fn test_parse_path_empty_string() {
        let result = PathIndex::parse_path("");
        assert_eq!(result, vec!["".to_string()]);
    }

    #[test]
    fn test_parse_path_no_separators() {
        let result = PathIndex::parse_path("test");
        assert_eq!(result, vec!["test".to_string()]);
    }

    #[test]
    fn test_parse_path_multiple_empty_segments() {
        let result = PathIndex::parse_path("a::::::b");
        assert_eq!(result, vec!["a".to_string(), "".to_string(), "".to_string(), "b".to_string()]);
    }

    #[test]
    fn test_parse_path_complex() {
        let result = PathIndex::parse_path("readual::readual::env::Linux-x64-rpm");
        assert_eq!(result, vec![
            "readual".to_string(),
            "readual".to_string(),
            "env".to_string(),
            "Linux-x64-rpm".to_string()
        ]);
    }

    #[test]
    fn test_parse_path_with_empty_in_middle() {
        let result = PathIndex::parse_path("test::::x86::release");
        assert_eq!(result, vec![
            "test".to_string(),
            "".to_string(),
            "x86".to_string(),
            "release".to_string()
        ]);
    }
}


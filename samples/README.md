# Readual Project

Это основной проект для работы с текстом и Markdown файлами.

## Описание

Readual - это набор утилит для обработки и анализа текстовых файлов.

### Основные возможности

- Парсинг Markdown файлов
- Построение иерархии заголовков
- Анализ структуры документов

## Установка

### Требования

- Rust 1.70+
- Cargo

### Сборка

```bash
cargo build
```

## Использование

### CLI утилита

```bash
readual --help
```

### MD парсер

```bash
readual-md --info
```

## Структура проекта

### Компоненты

- `readual-cli` - основная CLI утилита
- `readual-md` - парсер Markdown файлов

### Архитектура

#### Модули

- `parser` - парсинг файлов
- `analyzer` - анализ структуры
- `formatter` - форматирование вывода

#### Конфигурация

- Настройки парсера
- Опции форматирования
- Параметры вывода

## Примеры

### Базовое использование

```rust
use readual_md::parser::MarkdownParser;

let parser = MarkdownParser::new();
let result = parser.parse_file("README.md");
```

### Расширенные возможности

```rust
use readual_md::analyzer::DocumentAnalyzer;

let analyzer = DocumentAnalyzer::new();
let hierarchy = analyzer.build_hierarchy(&content);
```

## Лицензия

MIT License

## Вклад в проект

### Как помочь

1. Форкните репозиторий
2. Создайте ветку для новой функции
3. Внесите изменения
4. Создайте Pull Request

### Стандарты кода

- Используйте `cargo fmt`
- Запускайте `cargo clippy`
- Покрывайте тестами

## Changelog

### v0.1.0

- Первоначальный релиз
- Базовая функциональность CLI
- Парсинг Markdown заголовков

## Контакты

- Автор: Nabiev Timur
- Email: nabievtimurprogrammer@gmail.com

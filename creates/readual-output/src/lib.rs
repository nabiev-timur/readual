use std::io::{self, Write};
use std::sync::{OnceLock, Mutex};
use colored::*;

/// Уровни сообщений для форматированного вывода
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Success,
    Warning,
    Error,
    Info,
    Debug,
    None,
}

/// Уровни детализации вывода (глобальное состояние)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputVerbosity {
    /// Silent - блокирует весь вывод
    Silent = 0,
    /// Info - блокирует Debug, показывает остальное
    Info = 1,
    /// Debug - показывает весь вывод
    Debug = 2,
}

impl Default for OutputVerbosity {
    fn default() -> Self {
        OutputVerbosity::Info
    }
}

/// Глобальное состояние уровня детализации вывода
static OUTPUT_VERBOSITY: OnceLock<Mutex<OutputVerbosity>> = OnceLock::new();

/// Инициализация глобального состояния (по умолчанию Info)
fn get_verbosity() -> OutputVerbosity {
    let verbosity = OUTPUT_VERBOSITY.get_or_init(|| Mutex::new(OutputVerbosity::Info));
    *verbosity.lock().unwrap()
}

/// Установить уровень детализации вывода
/// 
/// # Пример
/// ```
/// use readual_output::{set_verbosity, OutputVerbosity};
/// 
/// set_verbosity(OutputVerbosity::Debug);
/// ```
pub fn set_verbosity(level: OutputVerbosity) {
    let verbosity = OUTPUT_VERBOSITY.get_or_init(|| Mutex::new(OutputVerbosity::Info));
    *verbosity.lock().unwrap() = level;
}

/// Получить текущий уровень детализации вывода
pub fn get_current_verbosity() -> OutputVerbosity {
    get_verbosity()
}

/// Проверить, должен ли быть показан вывод для данного уровня сообщения
fn should_output(message_level: MessageLevel) -> bool {
    let verbosity = get_verbosity();
    
    match verbosity {
        OutputVerbosity::Silent => false, // Блокируем весь вывод
        OutputVerbosity::Info => {
            message_level != MessageLevel::Debug
        },
        OutputVerbosity::Debug => true, // Показываем всё
    }
}

impl MessageLevel {
    /// Получить название уровня в верхнем регистре
    fn name(&self) -> &'static str {
        match self {
            MessageLevel::Success => "SUCCESS",
            MessageLevel::Warning => "WARNING",
            MessageLevel::Error => "ERROR",
            MessageLevel::Info => "INFO",
            MessageLevel::Debug => "DEBUG",
            MessageLevel::None => "NONE",
        }
    }

    /// Получить префикс для уровня сообщения
    fn prefix(&self) -> String {
		if *self == MessageLevel::None {
			return String::new();
		}

		format!("[{}]", self.name())
    }
}

/// Внутренняя функция для вывода сообщения с уровнем
/// Используется макросами для фактического вывода
#[doc(hidden)]
pub fn _output_impl(level: MessageLevel, args: std::fmt::Arguments<'_>) {
    // Проверяем, должен ли быть показан вывод
    if !should_output(level) {
        return;
    }
    
    let prefix = level.prefix();
    let message = format!("{}", args);
    
    // Выводим в формате: [LEVEL] message с цветами
    if prefix.is_empty() {
        // Обычный вывод без уровня (None)
        println!("{}", message);
    } else {
        // Цветной вывод в зависимости от уровня
        let colored_prefix = match level {
            MessageLevel::Success => prefix.green().bold(),
            MessageLevel::Warning => prefix.yellow().bold(),
            MessageLevel::Error => prefix.red().bold(),
            MessageLevel::Info => prefix.truecolor(100, 100, 100), // Более бледный серый
            MessageLevel::Debug => prefix.bright_black(), // Серый
            MessageLevel::None => prefix.normal(),
        };
        
        let colored_message = match level {
            MessageLevel::Success => message.green(),
            MessageLevel::Warning => message.yellow(),
            MessageLevel::Error => message.red(),
            MessageLevel::Info => message.truecolor(120, 120, 120), // Более бледный серый
            MessageLevel::Debug => message.bright_black(), // Серый
            MessageLevel::None => message.normal(),
        };
        
        println!("{} {}", colored_prefix, colored_message);
    }
    
    // Принудительно сбрасываем буфер вывода
    let _ = io::stdout().flush();
}

/// Макрос для вывода сообщения с типом сообщения по принципу println!
/// 
/// # Параметры
/// - `level` (опционально) - уровень сообщения (MessageLevel), по умолчанию None
/// - `$($arg:tt)*` - аргументы форматирования (как в println!)
#[macro_export]
macro_rules! output {
    // Вариант с уровнем: output!(MessageLevel::Success, "message")
    // Проверяем, что первый токен начинается с MessageLevel::
    ($crate::MessageLevel::$variant:ident, $($arg:tt)*) => {
        $crate::_output_impl($crate::MessageLevel::$variant, format_args!($($arg)*));
    };
    // Вариант без уровня: output!("message") - использует None по умолчанию
    // Это правило должно быть перед общим правилом с expr, чтобы не перехватывать строковые литералы
    ($fmt:literal $(, $($arg:tt)*)?) => {
        $crate::_output_impl($crate::MessageLevel::None, format_args!($fmt $(, $($arg)*)?));
    };
    // Вариант с уровнем через переменную: output!(level, "message")
    // Это правило должно быть последним, так как оно самое общее
    ($level:expr, $($arg:tt)*) => {
        $crate::_output_impl($level, format_args!($($arg)*));
    };
}

/// Макрос alias для output! для сообщения типа SUCCESS
/// 
/// # Параметры
/// - `$($arg:tt)*` - аргументы форматирования (как в println!)
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        $crate::output!($crate::MessageLevel::Success, $($arg)*);
    };
}

/// Макрос alias для output! для сообщения типа WARNING
/// 
/// # Параметры
/// - `$($arg:tt)*` - аргументы форматирования (как в println!)
#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {
        $crate::output!($crate::MessageLevel::Warning, $($arg)*);
    };
}

/// Макрос alias для output! для сообщения типа ERROR
/// 
/// # Параметры
/// - `$($arg:tt)*` - аргументы форматирования (как в println!)
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::output!($crate::MessageLevel::Error, $($arg)*);
    };
}

/// Макрос alias для output! для сообщения типа INFO
/// 
/// # Параметры
/// - `$($arg:tt)*` - аргументы форматирования (как в println!)
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::output!($crate::MessageLevel::Info, $($arg)*);
    };
}

/// Макрос alias для output! для сообщения типа DEBUG
/// 
/// # Параметры
/// - `$($arg:tt)*` - аргументы форматирования (как в println!)
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::output!($crate::MessageLevel::Debug, $($arg)*);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macros_with_formatting() {
        // Тестируем, что макросы работают с форматированием
        let count = 5;
        let name = "test";
        
        // Эти макросы должны компилироваться без ошибок
        // Мы не проверяем вывод, так как это тестовая среда
        output!(MessageLevel::Success, "Count: {}", count);
        output!(MessageLevel::Info, "Name: {}", name);
        
        success!("Test {}", count);
        warning!("Warning {}", name);
        error!("Error {}", count);
        info!("Info {}", name);
        debug!("Debug {}", count);
    }
}


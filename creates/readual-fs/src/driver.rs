use crate::config::FsConfig;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::ExitStatus;

/// Драйвер файловой системы, создающий сессии
pub trait FsDriver {
    /// Создать новую сессию файловой системы
    fn enter_session(&self, cfg: &FsConfig, cwd: &Path) -> io::Result<Box<dyn FsSession>>;
}

/// Сессия файловой системы для выполнения команд
pub trait FsSession {
    /// Выполнить команду в изолированном окружении
    /// 
    /// `env` - вектор переменных окружения в формате (key, value)
    fn run_command(&mut self, cmd: &str, env: &[(OsString, OsString)]) -> io::Result<ExitStatus>;

    /// Выполнить команду напрямую через bash -c без дополнительной обработки
    /// 
    /// Удобный метод для выполнения готовых команд из code блоков.
    /// Команда передаётся напрямую в `bash -c` без парсинга.
    fn run_command_raw(&mut self, cmd: &str) -> io::Result<ExitStatus> {
        self.run_command(cmd, &[])
    }

    /// Завершить сессию и очистить ресурсы
    fn leave(self: Box<Self>) -> io::Result<()>;
}


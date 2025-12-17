use std::path::PathBuf;

/// Режим работы файловой системы
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsBackend {
    /// Прозрачный режим - без изоляции
    Transparent,
    /// Лёгкая песочница с tmpfs для home/ и tmp/
    TmpFs,
    /// Тяжёлая песочница с обычной FS (без tmpfs)
    TmpMount,
}

/// Тип монтирования
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountKind {
    /// Bind mount - монтирование существующего пути
    Bind,
    /// Tmpfs - временная файловая система в памяти
    TmpFs,
}

/// Режим доступа к монтированию
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountMode {
    /// Только чтение
    ReadOnly,
    /// Чтение и запись
    ReadWrite,
}

/// Спецификация монтирования
#[derive(Debug, Clone)]
pub struct MountSpec {
    /// Исходный путь (None для tmpfs)
    pub src: Option<PathBuf>,
    /// Путь назначения внутри песочницы
    pub dst: PathBuf,
    /// Тип монтирования
    pub kind: MountKind,
    /// Режим доступа
    pub mode: MountMode,
    /// Дополнительные опции mount (например, "size=256M")
    pub options: Option<String>,
}

/// Конфигурация файловой системы
#[derive(Debug, Clone)]
pub struct FsConfig {
    /// Бэкенд файловой системы
    pub backend: FsBackend,
    /// Дополнительные монтирования
    pub extra_mounts: Vec<MountSpec>,
}

impl Default for FsConfig {
    fn default() -> Self {
        Self {
            backend: FsBackend::Transparent,
            extra_mounts: Vec::new(),
        }
    }
}


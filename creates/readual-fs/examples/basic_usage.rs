#[cfg(unix)]
use readual_fs::{FsBackend, FsConfig, FsDriver, SandboxFsDriver};

#[cfg(unix)]
fn main() -> std::io::Result<()> {
    // Создаём конфигурацию с TmpFs backend
    let cfg = FsConfig {
        backend: FsBackend::TmpFs,
        extra_mounts: vec![],
    };

    // Получаем текущую директорию
    let cwd = std::env::current_dir()?;

    // Создаём драйвер и входим в сессию
    let driver = SandboxFsDriver;
    let mut session = driver.enter_session(&cfg, &cwd)?;

    // Выполняем команды в изолированном окружении
    // Используем run_command_raw для простых команд из code блоков
    println!("Выполняем команду: ls -la");
    let status = session.run_command_raw("ls -la")?;
    println!("Exit code: {}", status.code().unwrap_or(-1));

    println!("\nВыполняем команду: pwd");
    let status = session.run_command_raw("pwd")?;
    println!("Exit code: {}", status.code().unwrap_or(-1));

    // Можно также использовать run_command с переменными окружения
    println!("\nВыполняем команду с переменными окружения");
    let env = vec![
        (std::ffi::OsString::from("TEST_VAR"), std::ffi::OsString::from("test_value")),
    ];
    let status = session.run_command("echo $TEST_VAR", &env)?;
    println!("Exit code: {}", status.code().unwrap_or(-1));

    // Завершаем сессию
    session.leave()?;

    println!("Сессия завершена успешно!");
    Ok(())
}

#[cfg(not(unix))]
fn main() {
    println!("Этот пример работает только на Unix-системах");
}


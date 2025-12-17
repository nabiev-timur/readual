#[cfg(unix)]
use crate::config::{FsBackend, FsConfig, MountKind, MountMode};
#[cfg(unix)]
use crate::driver::{FsDriver, FsSession};
#[cfg(unix)]
use crate::transparent::TransparentFsDriver;
#[cfg(unix)]
use libc::{self, c_char, c_int};
#[cfg(unix)]
use std::ffi::CString;
#[cfg(unix)]
use std::fs::{self, File};
#[cfg(unix)]
use std::io::{self, BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::io::{FromRawFd, RawFd};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(unix)]
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::ExitStatus;
#[cfg(unix)]
use std::ptr;

/// Основной драйвер песочницы, поддерживающий все режимы
#[cfg(unix)]
pub struct SandboxFsDriver;

#[cfg(unix)]
impl FsDriver for SandboxFsDriver {
    fn enter_session(&self, cfg: &FsConfig, cwd: &Path) -> io::Result<Box<dyn FsSession>> {
        match cfg.backend {
            FsBackend::Transparent => {
                let driver = TransparentFsDriver;
                driver.enter_session(cfg, cwd)
            }
            FsBackend::TmpFs | FsBackend::TmpMount => {
                create_sandbox_session(cfg, cwd)
            }
        }
    }
}

/// Сессия песочницы
#[cfg(unix)]
struct SandboxSession {
    ipc_in: File,
    ipc_out: File,
    child_pid: libc::pid_t,
    temp_root: PathBuf,
}

#[cfg(unix)]
impl FsSession for SandboxSession {
    fn run_command(&mut self, cmd: &str, _env: &[(std::ffi::OsString, std::ffi::OsString)]) -> io::Result<ExitStatus> {
        // Отправляем команду в helper через IPC
        // Протокол: команда\n
        writeln!(self.ipc_in, "{}", cmd)?;
        self.ipc_in.flush()?;

        // Читаем ответ от helper
        // Протокол: EXIT:<code>\nSTDOUT_START\n<вывод>\nSTDOUT_END\nSTDERR_START\n<вывод>\nSTDERR_END\n
        let mut reader = BufReader::new(&self.ipc_out);
        let mut line = String::new();
        
        // Читаем exit code
        reader.read_line(&mut line)?;
        let exit_code = if line.starts_with("EXIT:") {
            line[5..].trim().parse::<i32>().unwrap_or(1)
        } else {
            // Старый формат для обратной совместимости
            line.trim().parse::<i32>().unwrap_or(1)
        };

        // Читаем stdout
        let mut stdout = String::new();
        line.clear();
        reader.read_line(&mut line)?;
        if line.trim() == "STDOUT_START" {
            loop {
                line.clear();
                reader.read_line(&mut line)?;
                if line.trim() == "STDOUT_END" {
                    break;
                }
                stdout.push_str(&line);
            }
        } else {
            // Старый формат
            stdout = line;
        }

        // Читаем stderr
        let mut stderr = String::new();
        line.clear();
        reader.read_line(&mut line)?;
        if line.trim() == "STDERR_START" {
            loop {
                line.clear();
                reader.read_line(&mut line)?;
                if line.trim() == "STDERR_END" {
                    break;
                }
                stderr.push_str(&line);
            }
        } else {
            // Старый формат
            stderr = line;
        }

        // Выводим stdout и stderr в реальные потоки
        if !stdout.is_empty() {
            print!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprint!("{}", stderr);
        }

        Ok(ExitStatus::from_raw(exit_code as u32))
    }

    fn leave(mut self: Box<Self>) -> io::Result<()> {
        // Закрываем IPC
        drop(self.ipc_in);
        drop(self.ipc_out);

        // Отправляем SIGTERM helper'у
        unsafe {
            if libc::kill(self.child_pid, libc::SIGTERM) != 0 {
                // Игнорируем ошибку, если процесс уже завершился
            }
        }

        // Ждём завершения процесса
        let mut status: c_int = 0;
        unsafe {
            libc::waitpid(self.child_pid, &mut status, 0);
        }

        // Удаляем временный root (tempdir сам удалит при drop)
        let _ = fs::remove_dir_all(&self.temp_root);

        Ok(())
    }
}

/// Создать сессию песочницы
#[cfg(unix)]
fn create_sandbox_session(cfg: &FsConfig, cwd: &Path) -> io::Result<Box<dyn FsSession>> {
    // Создаём временный корень
    let temp_root = create_temp_root()?;

    // Создаём IPC канал (pipe)
    let (ipc_read, ipc_write) = create_pipe()?;
    let (helper_read, helper_write) = create_pipe()?;

    // Fork процесса
    let child_pid = unsafe { libc::fork() };

    match child_pid {
        -1 => Err(io::Error::last_os_error()),
        0 => {
            // Дочерний процесс - настраиваем песочницу
            setup_sandbox_child(cfg, cwd, &temp_root, helper_read, helper_write)?;
            // Не должны сюда попасть
            std::process::exit(1);
        }
        pid => {
            // Родительский процесс
            drop(helper_read);
            drop(helper_write);

            Ok(Box::new(SandboxSession {
                ipc_in: unsafe { File::from_raw_fd(ipc_write) },
                ipc_out: unsafe { File::from_raw_fd(ipc_read) },
                child_pid: pid,
                temp_root,
            }))
        }
    }
}

/// Создать временный корневой каталог
#[cfg(unix)]
fn create_temp_root() -> io::Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let mut counter = 0;
    loop {
        let root = temp_dir.join(format!("readual-sbox-{:04x}", counter));
        match fs::create_dir(&root) {
            Ok(_) => return Ok(root),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                counter += 1;
                if counter > 10000 {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Failed to create unique temp directory",
                    ));
                }
            }
            Err(e) => return Err(e),
        }
    }
}

/// Создать pipe для IPC
#[cfg(unix)]
fn create_pipe() -> io::Result<(RawFd, RawFd)> {
    let mut fds: [c_int; 2] = [0, 0];
    let result = unsafe { libc::pipe(fds.as_mut_ptr()) };
    if result == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok((fds[0], fds[1]))
}

/// Настроить песочницу в дочернем процессе
#[cfg(unix)]
fn setup_sandbox_child(
    cfg: &FsConfig,
    cwd: &Path,
    root: &Path,
    helper_read: RawFd,
    helper_write: RawFd,
) -> io::Result<()> {
    // Создаём новый mount namespace
    unsafe {
        if libc::unshare(libc::CLONE_NEWNS) != 0 {
            return Err(io::Error::last_os_error());
        }
    }

    // Создаём структуру каталогов
    create_sandbox_dirs(root)?;

    // Монтируем системные каталоги (read-only)
    mount_system_dirs(root)?;

    // Монтируем репозиторий
    mount_repository(cwd, root)?;

    // Монтируем tmpfs или обычные каталоги в зависимости от backend
    match cfg.backend {
        FsBackend::TmpFs => {
            mount_tmpfs(root.join("tmp"), Some("size=256M"))?;
            mount_tmpfs(root.join("home/sandbox"), Some("size=64M"))?;
        }
        FsBackend::TmpMount => {
            // Просто оставляем каталоги как есть
        }
        FsBackend::Transparent => unreachable!(),
    }

    // Обрабатываем дополнительные монтирования
    for mount in &cfg.extra_mounts {
        mount_extra(root, mount)?;
    }

    // Монтируем proc
    mount_proc(root)?;

    // Выполняем chroot
    unsafe {
        let root_cstr = CString::new(root.as_os_str().as_bytes())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        if libc::chroot(root_cstr.as_ptr()) != 0 {
            return Err(io::Error::last_os_error());
        }
    }

    // Меняем рабочую директорию
    unsafe {
        if libc::chdir(b"/work\0".as_ptr() as *const c_char) != 0 {
            return Err(io::Error::last_os_error());
        }
    }

    // Запускаем helper-процесс
    run_helper(helper_read, helper_write)?;

    Ok(())
}

/// Создать структуру каталогов в корне песочницы
#[cfg(unix)]
fn create_sandbox_dirs(root: &Path) -> io::Result<()> {
    let dirs = [
        "usr", "bin", "lib", "lib64", "home/sandbox", "tmp", "dev", "proc", "work",
    ];

    for dir in &dirs {
        fs::create_dir_all(root.join(dir))?;
    }

    Ok(())
}

/// Монтировать системные каталоги (read-only)
#[cfg(unix)]
fn mount_system_dirs(root: &Path) -> io::Result<()> {
    let system_dirs = [
        ("/usr", "usr"),
        ("/bin", "bin"),
        ("/lib", "lib"),
        ("/lib64", "lib64"),
    ];

    for (src, dst) in &system_dirs {
        let src_cstr = CString::new(*src)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let dst_path = root.join(dst);
        let dst_cstr = CString::new(dst_path.as_os_str().as_bytes())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        // Bind mount
        unsafe {
            if libc::mount(
                src_cstr.as_ptr(),
                dst_cstr.as_ptr(),
                ptr::null(),
                libc::MS_BIND | libc::MS_REC,
                ptr::null(),
            ) != 0
            {
                return Err(io::Error::last_os_error());
            }

            // Remount read-only
            if libc::mount(
                src_cstr.as_ptr(),
                dst_cstr.as_ptr(),
                ptr::null(),
                libc::MS_BIND | libc::MS_REMOUNT | libc::MS_RDONLY | libc::MS_REC,
                ptr::null(),
            ) != 0
            {
                return Err(io::Error::last_os_error());
            }
        }
    }

    Ok(())
}

/// Монтировать репозиторий как /work
#[cfg(unix)]
fn mount_repository(cwd: &Path, root: &Path) -> io::Result<()> {
    let src_cstr = CString::new(cwd.as_os_str().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let dst_path = root.join("work");
    let dst_cstr = CString::new(dst_path.as_os_str().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    unsafe {
        if libc::mount(
            src_cstr.as_ptr(),
            dst_cstr.as_ptr(),
            ptr::null(),
            libc::MS_BIND,
            ptr::null(),
        ) != 0
        {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(())
}

/// Монтировать tmpfs
#[cfg(unix)]
fn mount_tmpfs(dst: PathBuf, options: Option<&str>) -> io::Result<()> {
    let dst_cstr = CString::new(dst.as_os_str().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let fstype = CString::new("tmpfs")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let data = options
        .map(|s| CString::new(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e)))
        .transpose()?;

    unsafe {
        if libc::mount(
            ptr::null(),
            dst_cstr.as_ptr(),
            fstype.as_ptr(),
            0,
            data.as_ref()
                .map(|s| s.as_ptr())
                .unwrap_or(ptr::null()) as *const c_char,
        ) != 0
        {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(())
}

/// Монтировать дополнительные пути
#[cfg(unix)]
fn mount_extra(root: &Path, mount: &crate::config::MountSpec) -> io::Result<()> {
    let dst_path = root.join(&mount.dst);
    let dst_cstr = CString::new(dst_path.as_os_str().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    match mount.kind {
        MountKind::Bind => {
            let src = mount.src.as_ref().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Bind mount requires src")
            })?;
            let src_cstr = CString::new(src.as_os_str().as_bytes())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

            unsafe {
                if libc::mount(
                    src_cstr.as_ptr(),
                    dst_cstr.as_ptr(),
                    ptr::null(),
                    libc::MS_BIND,
                    ptr::null(),
                ) != 0
                {
                    return Err(io::Error::last_os_error());
                }

                if mount.mode == MountMode::ReadOnly {
                    if libc::mount(
                        src_cstr.as_ptr(),
                        dst_cstr.as_ptr(),
                        ptr::null(),
                        libc::MS_BIND | libc::MS_REMOUNT | libc::MS_RDONLY,
                        ptr::null(),
                    ) != 0
                    {
                        return Err(io::Error::last_os_error());
                    }
                }
            }
        }
        MountKind::TmpFs => {
            mount_tmpfs(dst_path.clone(), mount.options.as_deref())?;
        }
    }

    Ok(())
}

/// Монтировать proc
#[cfg(unix)]
fn mount_proc(root: &Path) -> io::Result<()> {
    let proc_path = root.join("proc");
    let proc_cstr = CString::new(proc_path.as_os_str().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let fstype = CString::new("proc")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    unsafe {
        if libc::mount(
            ptr::null(),
            proc_cstr.as_ptr(),
            fstype.as_ptr(),
            0,
            ptr::null(),
        ) != 0
        {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(())
}

/// Запустить helper-процесс для выполнения команд
#[cfg(unix)]
fn run_helper(helper_read: RawFd, helper_write: RawFd) -> io::Result<()> {
    // Перенаправляем stdin/stdout/stderr
    unsafe {
        libc::dup2(helper_read, libc::STDIN_FILENO);
        libc::dup2(helper_write, libc::STDOUT_FILENO);
        libc::dup2(helper_write, libc::STDERR_FILENO);
        libc::close(helper_read);
        libc::close(helper_write);
    }

    // Запускаем простой shell helper
    // Используем /bin/bash для выполнения команд через bash -c
    let sh_path = CString::new("/bin/bash")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let arg0 = CString::new("bash")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let arg1 = CString::new("-c")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    // Создаём простой скрипт helper'а
    // Читаем команду из stdin, выполняем её через bash -c, выводим exit code, stdout и stderr
    // Используем /tmp вместо mktemp, так как мы в chroot
    let helper_script = r#"
counter=0
while IFS= read -r cmd; do
    if [ "$cmd" = "exit" ]; then
        exit 0
    fi
    # Выполняем команду через bash -c, захватывая stdout и stderr
    stdout_file="/tmp/readual_stdout_$$_$counter"
    stderr_file="/tmp/readual_stderr_$$_$counter"
    bash -c "$cmd" > "$stdout_file" 2> "$stderr_file"
    exit_code=$?
    
    # Выводим результат по протоколу
    echo "EXIT:$exit_code"
    echo "STDOUT_START"
    [ -f "$stdout_file" ] && cat "$stdout_file"
    echo "STDOUT_END"
    echo "STDERR_START"
    [ -f "$stderr_file" ] && cat "$stderr_file"
    echo "STDERR_END"
    
    rm -f "$stdout_file" "$stderr_file"
    counter=$((counter + 1))
done
"#;

    let script_cstr = CString::new(helper_script)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    // Запускаем sh с скриптом
    unsafe {
        let args = [
            arg0.as_ptr(),
            arg1.as_ptr(),
            script_cstr.as_ptr(),
            ptr::null(),
        ];
        libc::execve(sh_path.as_ptr(), args.as_ptr() as *mut *mut c_char, ptr::null());
    }

    Err(io::Error::last_os_error())
}


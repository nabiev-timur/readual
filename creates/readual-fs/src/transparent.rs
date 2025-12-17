use crate::config::FsConfig;
use crate::driver::{FsDriver, FsSession};
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus};

/// Прозрачный драйвер - без изоляции файловой системы
pub struct TransparentFsDriver;

impl FsDriver for TransparentFsDriver {
    fn enter_session(&self, _cfg: &FsConfig, cwd: &Path) -> io::Result<Box<dyn FsSession>> {
        Ok(Box::new(TransparentSession {
            cwd: cwd.to_path_buf(),
        }))
    }
}

struct TransparentSession {
    cwd: std::path::PathBuf,
}

impl FsSession for TransparentSession {
    fn run_command(&mut self, cmd: &str, env: &[(OsString, OsString)]) -> io::Result<ExitStatus> {
        let mut command = Command::new("bash");
        command.arg("-c").arg(cmd);
        command.current_dir(&self.cwd);

        for (key, value) in env {
            command.env(key, value);
        }

        command.status()
    }

    fn leave(self: Box<Self>) -> io::Result<()> {
        // Ничего не нужно делать в прозрачном режиме
        Ok(())
    }
}


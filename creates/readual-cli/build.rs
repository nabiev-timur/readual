use std::process::Command;

fn set_project_version() {
    // Проверяем, задана ли версия через переменную окружения
    if let Ok(env_version) = std::env::var("VERSION") {
        println!("cargo:rustc-env=VERSION={}", env_version);
        return;
    }

    // Если переменная не задана, пытаемся получить версию из git
    let git_output = Command::new("git")
        .args(&["describe", "--tags", "--long"])
        .output();

    let version = match git_output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => {
            // Если git describe не работает, используем значение по умолчанию
            "0.1-0-dev".to_string()
        }
    };

    // Парсим версию в одной функции
    let version = version.trim();
    
    // Убираем префикс 'v' если есть
    let version = if version.starts_with('v') {
        &version[1..]
    } else {
        version
    };
    
    // Разделяем по дефису
    let parts: Vec<&str> = version.split('-').collect();
    
    let parsed_version = if parts.len() >= 2 {
        let base_version = parts[0];
        if let Ok(commits) = parts[1].parse::<u32>() {
            let version_parts: Vec<&str> = base_version.split('.').collect();
            if version_parts.len() >= 2 {
                let major = version_parts[0];
                let minor = version_parts[1];
                format!("{}.{}.{}", major, minor, commits)
            } else {
                base_version.to_string()
            }
        } else {
            base_version.to_string()
        }
    } else {
        version.to_string()
    };

    println!("cargo:rustc-env=VERSION={}", parsed_version);
}

fn main() {
    set_project_version();
}

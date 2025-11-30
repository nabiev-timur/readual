use readual_output::{set_verbosity, OutputVerbosity};

#[macro_use]
extern crate readual_output;

fn main() {
    println!("=== Тест уровня Silent (блокирует весь вывод) ===");
    set_verbosity(OutputVerbosity::Silent);
    success!("Это сообщение SUCCESS не должно быть видно");
    warning!("Это сообщение WARNING не должно быть видно");
    error!("Это сообщение ERROR не должно быть видно");
    info!("Это сообщение INFO не должно быть видно");
    debug!("Это сообщение DEBUG не должно быть видно");
    output!("Это обычное сообщение не должно быть видно");
    
    println!("\n=== Тест уровня Info (блокирует только Debug) ===");
    set_verbosity(OutputVerbosity::Info);
    success!("Это сообщение SUCCESS должно быть видно");
    warning!("Это сообщение WARNING должно быть видно");
    error!("Это сообщение ERROR должно быть видно");
    info!("Это сообщение INFO должно быть видно");
    debug!("Это сообщение DEBUG НЕ должно быть видно");
    output!("Это обычное сообщение должно быть видно");
    
    println!("\n=== Тест уровня Debug (показывает весь вывод) ===");
    set_verbosity(OutputVerbosity::Debug);
    success!("Это сообщение SUCCESS должно быть видно");
    warning!("Это сообщение WARNING должно быть видно");
    error!("Это сообщение ERROR должно быть видно");
    info!("Это сообщение INFO должно быть видно");
    debug!("Это сообщение DEBUG должно быть видно");
    output!("Это обычное сообщение должно быть видно");
}


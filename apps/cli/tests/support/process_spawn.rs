use std::{
    io,
    process::{Child, Command},
    sync::Mutex,
};

// macOS std pipes set CLOEXEC after creation. Serialize test-parent spawning so
// another parallel child cannot inherit a captured pipe in that interval.
static SPAWN: Mutex<()> = Mutex::new(());

pub fn spawn(command: &mut Command) -> io::Result<Child> {
    let _guard = SPAWN.lock().unwrap_or_else(|error| error.into_inner());

    command.spawn()
}

use crate::language::Language;

pub fn status(pid: Option<u32>, language: Language) -> String {
    let (running, stopped) = match language {
        Language::English => (
            include_str!("../locales/en/daemon-running.txt"),
            include_str!("../locales/en/daemon-stopped.txt"),
        ),
        Language::Russian => (
            include_str!("../locales/ru/daemon-running.txt"),
            include_str!("../locales/ru/daemon-stopped.txt"),
        ),
    };

    match pid {
        Some(pid) => format!("{} (pid {pid})", running.trim_end()),
        None => stopped.trim_end().into(),
    }
}

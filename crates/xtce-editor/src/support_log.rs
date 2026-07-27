use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

static LOG_PATH: OnceLock<Result<PathBuf, String>> = OnceLock::new();
static LOG_LOCK: Mutex<()> = Mutex::new(());
static PANIC_HOOK_INSTALLED: OnceLock<()> = OnceLock::new();

pub(crate) fn init() {
    _ = log_path();
    event(
        "INFO",
        &format!(
            "application started version={} os={} arch={}",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH
        ),
    );
    PANIC_HOOK_INSTALLED.get_or_init(|| {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            event("ERROR", &format!("panic: {panic_info}"));
            default_hook(panic_info);
        }));
    });
}

pub(crate) fn event(level: &str, message: &str) {
    let Ok(path) = log_path() else {
        return;
    };
    let Ok(_guard) = LOG_LOCK.try_lock() else {
        return;
    };
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    _ = writeln!(
        file,
        "{}",
        format_record(timestamp_millis(), level, message)
    );
}

pub(crate) fn export_to(destination: &Path) -> Result<(), String> {
    let source = log_path()?;
    fs::copy(source, destination)
        .map(|_| ())
        .map_err(|error| format!("Could not save the application log: {error}"))
}

fn log_path() -> Result<&'static Path, String> {
    match LOG_PATH.get_or_init(create_log_path) {
        Ok(path) => Ok(path.as_path()),
        Err(error) => Err(error.clone()),
    }
}

fn create_log_path() -> Result<PathBuf, String> {
    let state_directory = std::env::var_os("XDG_STATE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(|home| PathBuf::from(home).join(".local/state"))
        })
        .unwrap_or_else(std::env::temp_dir)
        .join("xtce-editor");
    fs::create_dir_all(&state_directory).map_err(|error| {
        format!(
            "Could not create the application log directory {}: {error}",
            state_directory.display()
        )
    })?;
    Ok(state_directory.join("xtce-editor.log"))
}

fn timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn format_record(timestamp: u128, level: &str, message: &str) -> String {
    let message = message.replace(['\r', '\n'], "\\n");
    format!("{timestamp} {level} {message}")
}

#[cfg(test)]
mod tests {
    use super::format_record;

    #[test]
    fn log_records_stay_on_one_line() {
        assert_eq!(
            format_record(123, "ERROR", "first\nsecond\r\nthird"),
            "123 ERROR first\\nsecond\\n\\nthird"
        );
    }
}

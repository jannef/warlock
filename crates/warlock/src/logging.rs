//! Desktop logging and crash reports (spec §10).
//!
//! Every run writes `warlock-<UTC timestamp>.log` into `<platform data dir>/logs`, keeping the
//! most recent few. A panic hook writes `crash-<UTC timestamp>.txt` next to them (version, OS,
//! panic message, backtrace, last log lines) and tells the player where it is. The web build has
//! no filesystem, so none of this exists there: logs go to the browser console.

use bevy::log::LogPlugin;

/// The log plugin to put in `DefaultPlugins`: file logging on desktop, defaults on the web.
pub fn log_plugin() -> LogPlugin {
    #[cfg(not(target_arch = "wasm32"))]
    {
        desktop::log_plugin()
    }
    #[cfg(target_arch = "wasm32")]
    {
        LogPlugin::default()
    }
}

/// Creates the log file and installs the panic hook. Call once, first thing in `main`.
pub fn init() {
    #[cfg(not(target_arch = "wasm32"))]
    desktop::init();
}

/// Dev aid: panics on purpose when `WARLOCK_CRASH_TEST` is set, to exercise the crash report.
pub fn maybe_crash_test() {
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var_os("WARLOCK_CRASH_TEST").is_some() {
        panic!("WARLOCK_CRASH_TEST requested a test panic");
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod desktop {
    use super::LogPlugin;
    use bevy::log::BoxedLayer;
    use bevy::log::tracing_subscriber::{self, Layer};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    /// How many `warlock-*.log` files to keep (including the current run's).
    const KEEP_LOGS: usize = 5;
    /// How many `crash-*.txt` reports to keep.
    const KEEP_CRASHES: usize = 10;
    /// Log lines included in a crash report.
    const TAIL_LINES: usize = 60;

    struct Paths {
        dir: PathBuf,
        log: PathBuf,
    }

    static PATHS: OnceLock<Paths> = OnceLock::new();

    pub fn init() {
        let Some(dir) = log_dir() else { return };
        if fs::create_dir_all(&dir).is_err() {
            return;
        }
        let log = dir.join(format!("warlock-{}.log", timestamp()));
        prune(&dir, "warlock-", ".log", KEEP_LOGS.saturating_sub(1));
        prune(&dir, "crash-", ".txt", KEEP_CRASHES);
        let _ = PATHS.set(Paths { dir, log });
        install_panic_hook();
    }

    pub fn log_plugin() -> LogPlugin {
        LogPlugin {
            custom_layer: file_layer,
            ..Default::default()
        }
    }

    fn file_layer(_app: &mut bevy::app::App) -> Option<BoxedLayer> {
        let file = fs::File::create(&PATHS.get()?.log).ok()?;
        // The file is unbuffered, so the tail is complete when a panic reads it back.
        Some(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(Mutex::new(file))
                .boxed(),
        )
    }

    fn log_dir() -> Option<PathBuf> {
        let dirs = crate::paths::project_dirs()?;
        Some(dirs.data_local_dir().join("logs"))
    }

    fn install_panic_hook() {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            previous(info);
            let Some(paths) = PATHS.get() else { return };
            let tail = read_tail(&paths.log, TAIL_LINES);
            let backtrace = std::backtrace::Backtrace::force_capture();
            let report = build_report(&info.to_string(), &backtrace.to_string(), &tail);
            match write_report(&paths.dir, &report) {
                Ok(path) => eprintln!(
                    "\nWarlock crashed. A crash report was saved to:\n  {}\nThe log is at:\n  {}",
                    path.display(),
                    paths.log.display()
                ),
                Err(err) => {
                    eprintln!("\nWarlock crashed; the crash report could not be saved: {err}");
                }
            }
        }));
    }

    fn build_report(panic: &str, backtrace: &str, log_tail: &str) -> String {
        format!(
            "Warlock crash report\n\
             version: {}\n\
             os: {} {}\n\
             time (UTC): {}\n\n\
             panic: {panic}\n\n\
             backtrace:\n{backtrace}\n\n\
             last log lines:\n{log_tail}\n",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            timestamp(),
        )
    }

    fn write_report(dir: &Path, report: &str) -> std::io::Result<PathBuf> {
        let path = dir.join(format!("crash-{}.txt", timestamp()));
        fs::write(&path, report)?;
        Ok(path)
    }

    fn read_tail(path: &Path, lines: usize) -> String {
        let text = fs::read_to_string(path).unwrap_or_default();
        let all: Vec<&str> = text.lines().collect();
        all[all.len().saturating_sub(lines)..].join("\n")
    }

    /// Deletes all but the `keep` newest files named `<prefix>*<suffix>` (names sort by time).
    fn prune(dir: &Path, prefix: &str, suffix: &str, keep: usize) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        let mut names: Vec<String> = entries
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .filter(|n| n.starts_with(prefix) && n.ends_with(suffix))
            .collect();
        names.sort();
        let excess = names.len().saturating_sub(keep);
        for name in &names[..excess] {
            let _ = fs::remove_file(dir.join(name));
        }
    }

    /// `YYYYMMDD-HHMMSS` in UTC.
    fn timestamp() -> String {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        format_utc(secs)
    }

    fn format_utc(secs: u64) -> String {
        let (days, rem) = ((secs / 86_400) as i64, secs % 86_400);
        // Civil-from-days (Howard Hinnant).
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + i64::from(month <= 2);
        format!(
            "{year:04}{month:02}{day:02}-{:02}{:02}{:02}",
            rem / 3600,
            rem % 3600 / 60,
            rem % 60
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn formats_utc() {
            assert_eq!(format_utc(0), "19700101-000000");
            assert_eq!(format_utc(1_781_654_400 + 3661), "20260617-010101");
        }

        #[test]
        fn prune_keeps_newest() {
            let dir = std::env::temp_dir().join(format!("warlock-prune-{}", std::process::id()));
            fs::create_dir_all(&dir).unwrap();
            for n in [
                "warlock-1.log",
                "warlock-2.log",
                "warlock-3.log",
                "other.txt",
            ] {
                fs::write(dir.join(n), "x").unwrap();
            }
            prune(&dir, "warlock-", ".log", 2);
            assert!(!dir.join("warlock-1.log").exists());
            assert!(dir.join("warlock-2.log").exists());
            assert!(dir.join("warlock-3.log").exists());
            assert!(dir.join("other.txt").exists());
            fs::remove_dir_all(&dir).unwrap();
        }

        #[test]
        fn crash_report_is_written_with_log_tail() {
            let dir = std::env::temp_dir().join(format!("warlock-crash-{}", std::process::id()));
            fs::create_dir_all(&dir).unwrap();
            let log = dir.join("warlock-test.log");
            fs::write(&log, "a\nb\nc\nd\n").unwrap();
            let tail = read_tail(&log, 2);
            assert_eq!(tail, "c\nd");
            let report = build_report("panicked at x.rs:1: boom", "frame 0", &tail);
            let path = write_report(&dir, &report).unwrap();
            let text = fs::read_to_string(path).unwrap();
            assert!(text.contains("version: "));
            assert!(text.contains("boom"));
            assert!(text.contains("frame 0"));
            assert!(text.ends_with("c\nd\n"));
            fs::remove_dir_all(&dir).unwrap();
        }
    }
}

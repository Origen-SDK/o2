extern crate time;

use crate::core::term;
use crate::{Result, STATUS};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::cell::RefCell;
use std::thread;

// Thread specific files, a log request will write to the last log in this vector, if
// none are present then it will fall back to the global/shared files in Logger.inner.files
thread_local!(static FILES: RefCell<Vec<fs::File>> = RefCell::new(vec![]));

#[macro_export]
macro_rules! backend_fail {
    ($message:expr) => {
        origen::LOGGER.error(&format!(
            "A problem occurred in the Origen backend: '{}'",
            $message
        ));
        std::process::exit(1);
    };
}

#[macro_export]
macro_rules! backend_expect {
    ($obj:expr, $message:expr) => {{
        match $obj {
            Some(o) => o,
            None => {
                origen::LOGGER.error(&format!("A problem occurred in the Origen backend as an Error or None value was unwrapped: '{}'", $message));
                std::process::exit(1);
            }
        }
    }};
}

#[macro_export]
macro_rules! fail {
    ($message:expr) => {
        origen::LOGGER.error($message);
        std::process::exit(1);
    };
}

pub struct Logger {
    // All attributes that could be mutated are in here
    inner: RwLock<Inner>
}

#[derive(Default)]
pub struct Inner {
    level: u8,
    // The currently open log files, the last one is the one that will be written to
    // (unless there is an open thread-specific log file)
    files: Vec<fs::File>,
    // Paths to all log files that have been created by the current run
    logs: Vec<PathBuf>,
    // A counter used to generate unique default log file names
    counter: u32,
}

impl Logger {
    pub fn verbosity(&self) -> u8 {
        self.inner.read().unwrap().level
    }

    pub fn set_verbosity(&self, level: u8) -> Result<()> {
        let mut inner = self.inner.write().unwrap();
        inner.level = level;
        Ok(())
    }

    pub fn debug(&self, message: &str) {
        self._log(2, "DEBUG", message, &term::yellowln);
    }

    pub fn debug_block(&self, messages: &Vec<&str>) {
        self._log_block(2, "DEBUG", messages, &(term::yellowln));
    }

    pub fn deprecated(&self, message: &str) {
        self._log(1, "DEPRECATED", message, &term::yellowln);
    }

    pub fn deprecated_block(&self, messages: &Vec<&str>) {
        self._log_block(1, "DEPRECATED", messages, &(term::yellowln));
    }

    pub fn error(&self, message: &str) {
        self._log(0, "ERROR", message, &term::redln);
    }

    pub fn error_block(&self, messages: &Vec<&str>) {
        self._log_block(0, "ERROR", messages, &(term::redln));
    }

    pub fn info(&self, message: &str) {
        self._log(1, "INFO", message, &term::cyanln);
    }

    pub fn info_block(&self, messages: &Vec<&str>) {
        self._log_block(1, "INFO", messages, &(term::cyanln));
    }

    pub fn log(&self, message: &str) {
        self._log(1, "LOG", message, &term::standardln);
    }

    pub fn log_block(&self, messages: &Vec<&str>) {
        self._log_block(1, "LOG", messages, &(term::standardln));
    }

    pub fn success(&self, message: &str) {
        self._log(0, "SUCCESS", message, &term::greenln);
    }

    pub fn success_block(&self, messages: &Vec<&str>) {
        self._log_block(0, "SUCCESS", messages, &(term::greenln));
    }

    pub fn warning(&self, message: &str) {
        self._log(1, "WARNING", message, &term::yellowln);
    }

    pub fn warning_block(&self, messages: &Vec<&str>) {
        self._log_block(1, "WARNING", messages, &(term::yellowln));
    }

    fn _log(&self, level: u8, prefix: &str, message: &str, ref_to_print_func: &dyn Fn(&str)) {
        self._out(
            level,
            &format!("{}{}", self._prefix(prefix), message),
            ref_to_print_func,
        );
    }

    fn _log_block(
        &self,
        level: u8,
        prefix: &str,
        messages: &Vec<&str>,
        ref_to_print_func: &dyn Fn(&str),
    ) {
        if messages.len() == 0 {
            self._out(level, &self._prefix(prefix), ref_to_print_func);
        } else if messages.len() == 1 {
            self._out(
                level,
                &format!("{}{}", self._prefix(prefix), messages[0]),
                ref_to_print_func,
            );
        } else {
            let l = self._prefix(prefix);
            self._out(level, &format!("{}{}", l, messages[0]), ref_to_print_func);
            for m in &messages[1..] {
                self._out(
                    level,
                    &format!("{:>1$}", m, l.len() + m.len()),
                    ref_to_print_func,
                );
            }
        }
    }

    fn _out(&self, level: u8, s: &str, print_func: &dyn Fn(&str)) {
        if self.verbosity() >= level {
            print_func(&s);
        }
        self.write(&format!("{}\n", s));
    }

    fn _prefix(&self, prefix: &str) -> String {
        return String::from(format!("[{}] ({}): ", prefix, self._timestamp()));
    }

    fn _timestamp(&self) -> String {
        let dur = time::now() - STATUS.start_time;
        return format!(
            "{:0>2}:{:0>2}:{:0>2}.{:0>3}",
            // This will take the whole hours, leaving off the minutes, seconds, and milliseconds. Totals hours will still be displayed,
            // just won't be cleaned up. For example, a script that runs 2 days exactly, will display 48:00:00.000.
            dur.num_hours(),
            // Quite verbose way to do this, but the only way I could find without just doing the math  myself. Granted, that's what
            // the functions are doing underneath, but seems safer to have Rust do it.
            dur.num_minutes() - time::Duration::hours(dur.num_hours()).num_minutes(),
            dur.num_seconds() - time::Duration::minutes(dur.num_minutes()).num_seconds(),
            dur.num_milliseconds() - time::Duration::seconds(dur.num_seconds()).num_milliseconds(),
        );
    }

    fn write_header(&self) {
        let mut out = String::from("### Origen Log File\n");
        out += &format!("### Version: {}\n", STATUS.origen_version);
        out += &format!(
            "### Date:    {}\n",
            time::strftime("%F %T", &time::now()).unwrap()
        );
        out += &format!(
            "### Exec:    {}\n",
            env::current_exe()
                .expect("Could not get the current EXE path!")
                .to_string_lossy()
        );
        out += &format!(
            "### Command: {}\n\n",
            env::args().collect::<Vec<_>>().join(" ")
        );
        self.write(&out);
    }

    fn write(&self, msg: &str) {
        FILES.with(|files| {
            let files = files.borrow();
            match files.last() {
                Some(mut f) => {
                    write!(f, "{}", msg).expect("Unable to write data to thread log file!");
                },
                None => {
                    // Took a write lock here to ensure exclusivity, not sure if really required or
                    // not, is the write! macro threadsafe?
                    // No concern from the above write since that is to a thread-specific log file,
                    // so no worries about multiple writes at the same time.
                    let inner = self.inner.write().unwrap();
                    let mut f = inner.files.last().unwrap();
                    write!(f, "{}", msg).expect("Unable to write data to global log file!");
                },
            }
        });
    }

    pub fn open(&self, path: Option<&Path>, thread_local: bool) -> Result<()> {
        let p = match path {
            None => {
                let mut inner = self.inner.write().unwrap();
                inner.counter += 1;
                default_log_file(Some(inner.counter))
            },
            Some(p) => match p.is_absolute() {
                true => p.to_path_buf(),
                false => log_dir().join(p),
            },
        };

        // create all missing directories to avoid panics
        fs::create_dir_all(&p.parent().unwrap())?;

        // Open the file for write
        let f = fs::File::create(&p)?;

        {
            let mut inner = self.inner.write().unwrap();
            inner.logs.push(p);
            if !thread_local {
                inner.files.push(f);
            } else {
                FILES.with(|files| {
                    let mut files = files.borrow_mut();
                    files.push(f);
                });
            }
        }
        self.write_header();
        Ok(())
    }

    pub fn default() -> Logger {
        let logger = Logger {
            inner: RwLock::new(Inner::default())
        };

        logger.open(Some(&default_log_file(None)), false)
            .expect(&format!("Couldn't open default log file '{}'", default_log_file(None).display()));
        logger
    }
}

fn default_log_file(index: Option<u32>) -> PathBuf {
    match index {
        None => log_dir().join("out.log"),
        Some(i) => log_dir().join(&format!("out.log.{}", i)),
    }
}


pub fn log_dir() -> PathBuf {
    match STATUS.is_app_present {
        true => STATUS.root.clone().join("log"),
        false => STATUS.home.clone().join(".origen").join("log"),
    }
}


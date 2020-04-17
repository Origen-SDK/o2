extern crate time;

use crate::{STATUS, Result};
use crate::core::term;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::RwLock;

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
    pub output_file: PathBuf,
    file_handler: fs::File,
    level: RwLock<u8>,
}

impl Logger {
    pub fn verbosity(&self) -> u8 {
        *self.level.read().unwrap()
    }

    pub fn set_verbosity(&self, level: u8) -> Result<()> {
        let mut lvl = self.level.write().unwrap();
        *lvl = level;
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
        self._out(level,
            &format!("{}{}", self._prefix(prefix), message),
            ref_to_print_func,
        );
    }

    fn _log_block(&self, level: u8, prefix: &str, messages: &Vec<&str>, ref_to_print_func: &dyn Fn(&str)) {
        if messages.len() == 0 {
            self._out(level, &self._prefix(prefix), ref_to_print_func);
        } else if messages.len() == 1 {
            self._out(level,
                &format!("{}{}", self._prefix(prefix), messages[0]),
                ref_to_print_func,
            );
        } else {
            let l = self._prefix(prefix);
            self._out(level, &format!("{}{}", l, messages[0]), ref_to_print_func);
            for m in &messages[1..] {
                self._out(level, &format!("{:>1$}", m, l.len() + m.len()), ref_to_print_func);
            }
        }
    }

    fn _out(&self, level: u8, s: &str, print_func: &dyn Fn(&str)) {
        if self.verbosity() >= level {
            print_func(&s);
        }
        write!(&self.file_handler, "{}\n", s).expect("Could not write log file!");
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

    fn _write_header(&self) {
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
        write!(&self.file_handler, "{}", out).expect("Unable to write data to log file!");
    }

    pub fn default_output_dir() -> PathBuf {
        let pb;
        if STATUS.is_app_present {
            pb = (STATUS.root).to_path_buf().join("log");
        } else {
            pb = (STATUS.home).to_path_buf().join(".origen").join("log");
        }
        // create all missing directories to avoid panics
        fs::create_dir_all(pb.as_path())
            .expect(&(format!("Could not create the log directory {}", pb.display())));
        pb
    }

    pub fn default_output_file() -> PathBuf {
        return Logger::default_output_dir().join("out.log");
    }

    pub fn default() -> Logger {
        let ref f = Logger::default_output_file();
        let l = Logger {
            output_file: f.to_path_buf(),
            file_handler: match fs::File::create(f) {
                Ok(f) => f,
                Err(_e) => panic!(
                    "Could not open log file at {}",
                    format!("{}", f.to_string_lossy())
                ),
            },
            level: RwLock::new(0),
        };
        l._write_header();

        return l;
    }
}

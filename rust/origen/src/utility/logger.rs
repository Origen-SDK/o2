//! The Origen logger is implemented as a singleton and data is logged via globally available macros.
//!
//! All log macros operate similar to the println! macro, accepting a string argument and optionally
//! some variables to be substituted into the message.
//!
//! ## Examples
//!
//! Display is like a log aware print, the data will always appear in the console without any timestamp
//! or other decoration and will be written to the log file with timestamp info.
//!
//! ```no_run
//! # #[macro_use] extern crate origen;
//! # fn main() {
//! display!("Some msg");       // This one is equivalent to print! but appears in the log
//! displayln!("Some msg");     // Like println!
//!
//! let x = 10;
//! displayln!("The value of x is {}", x);
//!
//! // There are also red/green versions of display:
//! display_green!("The value of x is {}", x);
//! display_redln!("The value of x is {}", x);
//!
//! # }
//! ```
//!
//! Similar macros exist to display messages at different log levels, all of these appear with timestamps
//! in both the console and the log file:
//!
//! ```no_run
//! # #[macro_use] extern crate origen;
//! # fn main() {
//! let x = 10;
//!
//! log_error!("The value of x is {}", x);
//! log_info!("The value of x is {}", x);
//! log_success!("The value of x is {}", x);
//! log_warning!("The value of x is {}", x);
//! log_deprecated!("The value of x is {}", x);
//! log_debug!("The value of x is {}", x);
//! log_trace!("The value of x is {}", x);
//! # }
//! ```
//!
//! ## Log Levels
//!
//! The log verbosity mapping is shown below, level 0 is equivalent to running with no verbosity switch,
//! level 1 is -v, level 2 is -vv, etc.
//!
//! * Level 0 - display, log_error
//! * Level 1 - log_info, log_success, log_warning, log_deprecated
//! * Level 2 - log_debug
//! * Level 3 - log_trace
//!
extern crate time;

use crate::core::term;
use crate::{Result, STATUS};
use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

// Thread specific files, a log request will write to the last log in this vector, if
// none are present then it will fall back to the global/shared files in Logger.inner.files
thread_local!(static FILES: RefCell<Vec<(PathBuf, fs::File)>> = RefCell::new(vec![]));

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

pub struct Logger {
    // All attributes that could be mutated are in here
    inner: RwLock<Inner>,
}

#[derive(Default)]
pub struct Inner {
    level: u8,
    // The currently open log files, the last one is the one that will be written to
    // (unless there is an open thread-specific log file)
    files: Vec<(PathBuf, fs::File)>,
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
        log_debug!("Setting logger verbosity to '{}'", level);
        let mut inner = self.inner.write().unwrap();
        inner.level = level;
        Ok(())
    }

    /// This is the same as calling 'print!' but with it also being captured to the log.
    /// Use for displaying output to the terminal when creating CLI tools. The given message will always
    /// be output to the console and without a timestamp.
    /// It will also appear in the log file with a timestamp.
    pub fn display(&self, message: &str) {
        self._log(0, "DISPLAY", message, &|_msg| {
            print!("{}", message);
            std::io::stdout().flush().unwrap();
        });
    }

    /// Like display!, but appends a newline, this is like calling println! but it also appears in the log.
    pub fn displayln(&self, message: &str) {
        self._log(0, "DISPLAY", message, &|_msg| {
            println!("{}", message);
            std::io::stdout().flush().unwrap();
        });
    }

    /// See display
    pub fn display_green(&self, message: &str) {
        self._log(0, "DISPLAY", message, &|_msg| {
            term::green(message);
            std::io::stdout().flush().unwrap();
        });
    }

    /// See displayln
    pub fn display_greenln(&self, message: &str) {
        self._log(0, "DISPLAY", message, &|_msg| {
            term::greenln(message);
            std::io::stdout().flush().unwrap();
        });
    }

    /// See display
    pub fn display_yellow(&self, message: &str) {
        self._log(0, "DISPLAY", message, &|_msg| {
            term::yellow(message);
            std::io::stdout().flush().unwrap();
        });
    }

    /// See displayln
    pub fn display_yellowln(&self, message: &str) {
        self._log(0, "DISPLAY", message, &|_msg| {
            term::yellowln(message);
            std::io::stdout().flush().unwrap();
        });
    }

    /// See display
    pub fn display_red(&self, message: &str) {
        self._log(0, "DISPLAY", message, &|_msg| {
            term::red(message);
            std::io::stdout().flush().unwrap();
        });
    }

    /// See displayln
    pub fn display_redln(&self, message: &str) {
        self._log(0, "DISPLAY", message, &|_msg| {
            term::redln(message);
            std::io::stdout().flush().unwrap();
        });
    }

    /// See display
    pub fn display_block(&self, messages: &Vec<&str>) {
        self._log_block(0, "DISPLAY", messages, &(term::yellowln));
    }

    /// Log a debug message, this will be displayed in the terminal when running with -vv
    pub fn debug(&self, message: &str) {
        self._log(2, "DEBUG", message, &term::yellowln);
    }

    /// Log a debug message, this will be displayed in the terminal when running with -vv
    pub fn debug_block(&self, messages: &Vec<&str>) {
        self._log_block(2, "DEBUG", messages, &(term::yellowln));
    }

    /// Log a trace (very low level) debug message, this will be displayed in the terminal when running with -vvv
    pub fn trace(&self, message: &str) {
        self._log(3, "TRACE", message, &term::yellowln);
    }

    /// Log a trace (very low level) debug message, this will be displayed in the terminal when running with -vvv
    pub fn trace_block(&self, messages: &Vec<&str>) {
        self._log_block(3, "TRACE", messages, &(term::yellowln));
    }

    /// Log a deprecation warning message, this will be displayed in the terminal when running with -v
    pub fn deprecated(&self, message: &str) {
        self._log(1, "DEPRECATED", message, &term::yellowln);
    }

    /// Log a deprecation warning message, this will be displayed in the terminal when running with -v
    pub fn deprecated_block(&self, messages: &Vec<&str>) {
        self._log_block(1, "DEPRECATED", messages, &(term::yellowln));
    }

    /// Log an error message, this will always be displayed in the terminal
    pub fn error(&self, message: &str) {
        self._log(0, "ERROR", message, &term::redln);
    }

    /// Log an error message, this will always be displayed in the terminal
    pub fn error_block(&self, messages: &Vec<&str>) {
        self._log_block(0, "ERROR", messages, &(term::redln));
    }

    /// Log an info message, this will be displayed in the terminal when running with -v
    pub fn info(&self, message: &str) {
        self._log(1, "INFO", message, &term::cyanln);
    }

    /// Log an info message, this will be displayed in the terminal when running with -v
    pub fn info_block(&self, messages: &Vec<&str>) {
        self._log_block(1, "INFO", messages, &(term::cyanln));
    }

    /// Log a success message, this will be displayed in the terminal when running with -v
    pub fn success(&self, message: &str) {
        self._log(1, "SUCCESS", message, &term::greenln);
    }

    /// Log a success message, this will be displayed in the terminal when running with -v
    pub fn success_block(&self, messages: &Vec<&str>) {
        self._log_block(1, "SUCCESS", messages, &(term::greenln));
    }

    /// Log a warning message, this will be displayed in the terminal when running with -v
    pub fn warning(&self, message: &str) {
        self._log(1, "WARNING", message, &term::yellowln);
    }

    /// Log a warning message, this will be displayed in the terminal when running with -v
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
            let mut files = files.borrow_mut();
            match files.last_mut() {
                Some((_path, f)) => {
                    write!(f, "{}", msg).expect("Unable to write data to thread log file!");
                }
                None => {
                    // Took a write lock here to ensure exclusivity, not sure if really required or
                    // not, is the write! macro threadsafe?
                    // No concern from the above write since that is to a thread-specific log file,
                    // so no worries about multiple writes at the same time.
                    let mut inner = self.inner.write().unwrap();
                    let (_path, f) = inner.files.last_mut().unwrap();
                    write!(f, "{}", msg).expect("Unable to write data to global log file!");
                }
            }
        });
    }

    /// Returns the path to the current log file
    pub fn output_file(&self) -> PathBuf {
        FILES.with(|files| {
            let files = files.borrow();
            match files.last() {
                Some((path, _f)) => path.clone(),
                None => {
                    let inner = self.inner.read().unwrap();
                    let (path, _f) = inner.files.last().unwrap();
                    path.clone()
                }
            }
        })
    }

    /// See with_log which is the equivalent to calling open_logfile followed by close_logfile
    /// manually.
    pub fn open_logfile(&self, path: Option<&Path>, thread_local: bool) -> Result<PathBuf> {
        let p = match path {
            None => {
                let mut inner = self.inner.write().unwrap();
                inner.counter += 1;
                default_log_file(Some(inner.counter))
            }
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
            inner.logs.push(p.clone());
            if !thread_local {
                inner.files.push((p.clone(), f));
            } else {
                FILES.with(|files| {
                    let mut files = files.borrow_mut();
                    files.push((p.clone(), f));
                });
            }
        }
        self.write_header();
        Ok(p)
    }

    pub fn close_logfile(&self) {
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            if files.len() > 0 {
                files.pop();
            } else {
                let mut inner = self.inner.write().unwrap();
                // Never pop the last log file
                if inner.files.len() > 1 {
                    inner.files.pop();
                }
            }
        });
    }

    /// Send all logging to the given log file for the duration of the given function,
    /// returning all logging to the previous log file at the end.
    ///
    /// If thread_local is set to true this will apply only to the current thread.
    ///
    /// If no path is given for the new log file (normally just the name of it), then
    /// a new logfile will be created with a unique index number appended to the current
    /// logfile name, e.g. out.log.1, out.log.2, etc.
    ///
    /// An error will be returned if there is a problem creating the given log file,
    /// otherwise the result from the given function is returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[macro_use] extern crate origen;
    /// # fn main() {
    /// use origen::LOGGER;
    /// use std::path::Path;
    ///
    /// let result = LOGGER.with_logfile(Some(Path::new("my_log")), false, || {
    ///   log_debug!("This will appear in 'my_log'");
    ///   Ok(())
    /// });
    /// # }
    /// ```
    pub fn with_logfile<F>(
        &self,
        path: Option<&Path>,
        thread_local: bool,
        mut f: F,
    ) -> Result<PathBuf>
    where
        F: FnMut() -> Result<()>,
    {
        let p = self.open_logfile(path, thread_local)?;
        f()?;
        self.close_logfile();
        Ok(p)
    }

    pub fn default() -> Logger {
        let logger = Logger {
            inner: RwLock::new(Inner::default()),
        };

        logger
            .open_logfile(Some(&default_log_file(None)), false)
            .expect(&format!(
                "Couldn't open default log file '{}'",
                default_log_file(None).display()
            ));
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

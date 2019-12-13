extern crate time;

use super::super::super::STATUS;
use super::super::term;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub struct Logger {
    pub output_file: PathBuf,
    file_handler: fs::File,
}

impl Logger {
    pub fn debug(&self, message: &str) {
        self._log("DEBUG", message, &term::yellowln);
    }

    pub fn debug_block(&self, messages: &Vec<&str>) {
        self._log_block("DEBUG", messages, &(term::yellowln));
    }

    pub fn deprecated(&self, message: &str) {
        self._log("DEPRECATED", message, &term::yellowln);
    }

    pub fn deprecated_block(&self, messages: &Vec<&str>) {
        self._log_block("DEPRECATED", messages, &(term::yellowln));
    }

    pub fn error(&self, message: &str) {
        self._log("ERROR", message, &term::redln);
    }

    pub fn error_block(&self, messages: &Vec<&str>) {
        self._log_block("ERROR", messages, &(term::redln));
    }

    pub fn info(&self, message: &str) {
        self._log("INFO", message, &term::cyanln);
    }

    pub fn info_block(&self, messages: &Vec<&str>) {
        self._log_block("INFO", messages, &(term::cyanln));
    }

    pub fn log(&self, message: &str) {
        self._log("LOG", message, &term::standardln);
    }

    pub fn log_block(&self, messages: &Vec<&str>) {
        self._log_block("LOG", messages, &(term::standardln));
    }

    pub fn success(&self, message: &str) {
        self._log("SUCCESS", message, &term::greenln);
    }

    pub fn success_block(&self, messages: &Vec<&str>) {
        self._log_block("SUCCESS", messages, &(term::greenln));
    }

    pub fn warning(&self, message: &str) {
        self._log("WARNING", message, &term::yellowln);
    }

    pub fn warning_block(&self, messages: &Vec<&str>) {
        self._log_block("WARNING", messages, &(term::yellowln));
    }

    fn _log(&self, prefix: &str, message: &str, ref_to_print_func: &dyn Fn(&str)) {
        self._out(
            &format!("{}{}", self._prefix(prefix), message),
            ref_to_print_func,
        );
    }

    fn _log_block(&self, prefix: &str, messages: &Vec<&str>, ref_to_print_func: &dyn Fn(&str)) {
        if messages.len() == 0 {
            self._out(&self._prefix(prefix), ref_to_print_func);
        } else if messages.len() == 1 {
            self._out(
                &format!("{}{}", self._prefix(prefix), messages[0]),
                ref_to_print_func,
            );
        } else {
            let l = self._prefix(prefix);
            self._out(&format!("{}{}", l, messages[0]), ref_to_print_func);
            for m in &messages[1..] {
                self._out(&format!("{:>1$}", m, l.len() + m.len()), ref_to_print_func);
            }
        }
    }

    fn _out(&self, s: &str, print_func: &dyn Fn(&str)) {
        print_func(&s);
        write!(&self.file_handler, "{}\n", s).expect("Could not write log file!");
    }

    fn _prefix(&self, prefix: &str) -> String {
        return String::from(format!("Origen: {} ({}): ", prefix, self._timestamp()));
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

    fn _filename(&self) -> PathBuf {
        return self._output_dir().join("out.log");
    }

    fn _output_dir(&self) -> PathBuf {
        if STATUS.is_app_present {
            return (STATUS.root).to_path_buf().join("log");
        } else {
            match env::var("HOME") {
                Ok(var) => PathBuf::from(var).join(".origen").join("log"),
                Err(_e) => panic!("No environment variable for HOME!"),
            }
        }
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
        if STATUS.is_app_present {
            return (STATUS.root).to_path_buf().join("log");
        } else {
            // at least 2 things to consider here
            // 1) The env var holding the appropriate base path depends on the OS
            // 2) on a machine or account that has never run origen the .origen dir will not exist yet
            let mut pb;
            if cfg!(windows) {
                pb = PathBuf::from(env::var("USERPROFILE").expect("No environment variable for USERPROFILE"));
            }
            else {
                pb = PathBuf::from(env::var("HOME").expect("No environment variable for HOME"));
            }

            // create the .origen and log directory if missing
            pb = pb.join(".origen").join("log");
            fs::create_dir_all(pb.as_path()).expect("Could not create the log directory");
            pb
        }
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
                // This directory creation code is no longer needed (I think) -- handled in pub fn default_output_dir
                Err(_e) => match fs::create_dir(f.parent().unwrap()) {
                    Ok(_d) => match fs::File::create(f) {
                        Ok(f) => f,
                        Err(_e) => panic!(
                            "Could not open log file at {}",
                            format!("{}", f.to_string_lossy())
                        ),
                    },
                    Err(_e) => panic!(
                        "Could not open log file at {}",
                        format!("{}", f.to_string_lossy())
                    ),
                },
            },
        };
        l._write_header();

        return l;
    }
}

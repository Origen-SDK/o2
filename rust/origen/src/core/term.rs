// Responsible for writing to the terminal

use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn green(msg: &str) {
    print(msg, Color::Green);
}

pub fn greenln(msg: &str) {
    println(msg, Color::Green);
}

pub fn red(msg: &str) {
    print(msg, Color::Red);
}

pub fn redln(msg: &str) {
    println(msg, Color::Red);
}

pub fn yellow(msg: &str) {
    print(msg, Color::Rgb(215, 135, 0));
}

pub fn yellowln(msg: &str) {
    println(msg, Color::Rgb(215, 135, 0));
}

pub fn cyan(msg: &str) {
    print(msg, Color::Cyan);
}

pub fn cyanln(msg: &str) {
    println(msg, Color::Cyan);
}

// Prints a standard line without any colorizing, but retains a the same prototype as the other <x>ln functions.
pub fn standardln(msg: &str) {
    println!("{}", msg);
}

fn println(msg: &str, color: Color) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let status = stdout.set_color(ColorSpec::new().set_fg(Some(color)));
    if status.is_ok() {
        let status = writeln!(&mut stdout, "{}", msg);
        if status.is_ok() {
            let _ = stdout.reset();
            // TODO: This flush added to stop the coloring hanging over into the console, perhaps
            // for performance this should only be done when running in interactive mode
            let _ = stdout.flush();
            return;
        }
    }
    let _ = stdout.reset();
    println!("{}", msg);
}

fn print(msg: &str, color: Color) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let status = stdout.set_color(ColorSpec::new().set_fg(Some(color)));
    if status.is_ok() {
        let status = write!(&mut stdout, "{}", msg);
        if status.is_ok() {
            let _ = stdout.reset();
            // TODO: This flush added to stop the coloring hanging over into the console, perhaps
            // for performance this should only be done when running in interactive mode
            let _ = stdout.flush();
            return;
        }
    }
    let _ = stdout.reset();
    print!("{}", msg);
}

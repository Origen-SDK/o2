use std::process::Command;

pub fn on_windows() -> bool {
    if cfg!(windows) {
        return true;
    } else {
        return false;
    }
}

pub fn on_linux() -> bool {
    if cfg!(linux) {
        return true;
    } else {
        return false;
    };
}

// Due to OS differences, the basic std::process::Command doesn't cooperate very well in a Windows environment.
// Instead, wrap the cmd function here, which will ensures some semblance between Windows and Linux commands.
#[allow(unused_mut)]
pub fn cmd(cmd: &str) -> std::process::Command {
    if on_windows() {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(&cmd);
        return c;
    } else {
        let mut c = Command::new(&cmd);
        return c;
    }
}

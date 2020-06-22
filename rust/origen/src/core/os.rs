pub fn on_windows() -> bool {
    if cfg!(windows) {
        return true;
    } else {
        return false;
    }
}

pub fn on_linux() -> bool {
    if cfg!(unix) {
        return true;
    } else {
        return false;
    };
}

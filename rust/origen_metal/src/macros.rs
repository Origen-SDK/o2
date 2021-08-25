#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err($crate::error!($msg));
    };
    ($err:expr $(,)?) => {
        return Err($crate::error!($err));
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::error!($fmt, $($arg)*));
    };
}

#[macro_export]
macro_rules! error {
    ($msg:literal $(,)?) => {
        // Handle $:literal as a special case to make cargo-expanded code more
        // concise in the common case.
        $crate::Error::new($msg)
    };
    ($err:expr $(,)?) => ({
        $crate::Error::new($err)
    });
    ($fmt:expr, $($arg:tt)*) => {
        $crate::Error(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! display {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.display(&formatted);
    }}
}

#[macro_export]
macro_rules! displayln {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.displayln(&formatted);
    }}
}

#[macro_export]
macro_rules! display_green {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.display_green(&formatted);
    }}
}

#[macro_export]
macro_rules! display_greenln {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.display_greenln(&formatted);
    }}
}

#[macro_export]
macro_rules! display_yellow {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.display_yellow(&formatted);
    }}
}

#[macro_export]
macro_rules! display_yellowln {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.display_yellowln(&formatted);
    }}
}

#[macro_export]
macro_rules! display_cyan {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.display_cyan(&formatted);
    }}
}

#[macro_export]
macro_rules! display_cyanln {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.display_cyanln(&formatted);
    }}
}

#[macro_export]
macro_rules! display_red {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.display_red(&formatted);
    }}
}

#[macro_export]
macro_rules! display_redln {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.display_redln(&formatted);
    }}
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.debug(&formatted);
    }}
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.info(&formatted);
    }}
}

#[macro_export]
macro_rules! log_deprecated {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.deprecated(&formatted);
    }}
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.error(&formatted);
    }}
}

#[macro_export]
macro_rules! log_success {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.success(&formatted);
    }}
}

#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.warning(&formatted);
    }}
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        $crate::LOGGER.trace(&formatted);
    }}
}

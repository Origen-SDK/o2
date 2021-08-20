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

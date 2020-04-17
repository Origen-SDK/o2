

#[macro_export]
macro_rules! node {
    ( $attr:ident, $( $x:expr ),* ) => {
        {
            crate::generator::ast::Node::new(crate::generator::ast::Attrs::$attr($( $x ),*))
        }
    };
    ( $attr:ident ) => {
        {
            crate::generator::ast::Node::new(crate::generator::ast::Attrs::$attr)
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        crate::LOGGER.debug(&formatted);
    }}
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        crate::LOGGER.info(&formatted);
    }}
}

#[macro_export]
macro_rules! log_deprecated {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        crate::LOGGER.deprecated(&formatted);
    }}
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        crate::LOGGER.error(&formatted);
    }}
}

#[macro_export]
macro_rules! log_success {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        crate::LOGGER.success(&formatted);
    }}
}

#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        crate::LOGGER.warning(&formatted);
    }}
}

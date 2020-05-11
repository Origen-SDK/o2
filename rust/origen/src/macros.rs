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

/// Exit the origen process with a passing exit code and a big SUCCESS banner
#[macro_export]
macro_rules! exit_success {
    () => {
        display_greenln!(
            r#"     _______. __    __    ______   ______  _______      _______.     _______."#
        );
        display_greenln!(
            r#"    /       ||  |  |  |  /      | /      ||   ____|    /       |    /       |"#
        );
        display_greenln!(
            r#"   |   (----`|  |  |  | |  ,----'|  ,----'|  |__      |   (----`   |   (----`"#
        );
        display_greenln!(
            r#"    \   \    |  |  |  | |  |     |  |     |   __|      \   \        \   \    "#
        );
        display_greenln!(
            r#".----)   |   |  `--'  | |  `----.|  `----.|  |____ .----)   |   .----)   |   "#
        );
        display_greenln!(
            r#"|_______/     \______/   \______| \______||_______||_______/    |_______/    "#
        );
        std::process::exit(0);
    };
}

/// Exit the origen process with a passing exit code and a big PASS banner
#[macro_export]
macro_rules! exit_pass {
    () => {
        display_greenln!(r#".______      ___           _______.     _______."#);
        display_greenln!(r#"|   _  \    /   \         /       |    /       |"#);
        display_greenln!(r#"|  |_)  |  /  ^  \       |   (----`   |   (----`"#);
        display_greenln!(r#"|   ___/  /  /_\  \       \   \        \   \    "#);
        display_greenln!(r#"|  |     /  _____  \  .----)   |   .----)   |   "#);
        display_greenln!(r#"| _|    /__/     \__\ |_______/    |_______/    "#);
        std::process::exit(0);
    };
}

/// Exit the origen process with a failing exit code and a big FAIL banner
#[macro_export]
macro_rules! exit_fail {
    () => {
        display_redln!(r#" _______    ___       __   __      "#);
        display_redln!(r#"|   ____|  /   \     |  | |  |     "#);
        display_redln!(r#"|  |__    /  ^  \    |  | |  |     "#);
        display_redln!(r#"|   __|  /  /_\  \   |  | |  |     "#);
        display_redln!(r#"|  |    /  _____  \  |  | |  `----."#);
        display_redln!(r#"|__|   /__/     \__\ |__| |_______|"#);
        std::process::exit(1);
    };
}

/// Exit the origen process with a failing exit code and a big ERROR banner
#[macro_export]
macro_rules! exit_error {
    () => {
        display_redln!(r#" _______ .______      .______        ______   .______      "#);
        display_redln!(r#"|   ____||   _  \     |   _  \      /  __  \  |   _  \     "#);
        display_redln!(r#"|  |__   |  |_)  |    |  |_)  |    |  |  |  | |  |_)  |    "#);
        display_redln!(r#"|   __|  |      /     |      /     |  |  |  | |      /     "#);
        display_redln!(r#"|  |____ |  |\  \----.|  |\  \----.|  `--'  | |  |\  \----."#);
        display_redln!(r#"|_______|| _| `._____|| _| `._____| \______/  | _| `._____|"#);
        std::process::exit(1);
    };
}

/// Returns an Err<OrigenError> with the given error message
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        Err(crate::Error::new(&formatted))
    }}
}

#[macro_export]
macro_rules! display {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        crate::LOGGER.display(&formatted);
    }}
}

#[macro_export]
macro_rules! displayln {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        crate::LOGGER.displayln(&formatted);
    }}
}

#[macro_export]
macro_rules! display_green {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        crate::LOGGER.display_green(&formatted);
    }}
}

#[macro_export]
macro_rules! display_greenln {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        crate::LOGGER.display_greenln(&formatted);
    }}
}

#[macro_export]
macro_rules! display_red {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        crate::LOGGER.display_red(&formatted);
    }}
}

#[macro_export]
macro_rules! display_redln {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        crate::LOGGER.display_redln(&formatted);
    }}
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

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {{
        let formatted = std::fmt::format(format_args!($($arg)*));
        crate::LOGGER.trace(&formatted);
    }}
}

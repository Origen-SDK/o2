#[macro_export]
macro_rules! node {
    ( $attr:path, $( $x:expr ),* => $( $c:expr ),* $(,)?) => {
        {
            $crate::ast::Node::new_with_children($attr($( $x ),*), vec![$( $c ),*])
        }
    };
    ( $attr:path, $( $x:expr ),* ; $m:expr) => {
        {
            $crate::ast::Node::new_with_meta($attr($( $x ),*), $m)
        }
    };
    ( $attr:path, $( $x:expr ),* ) => {
        {
            $crate::ast::Node::new($attr($( $x ),*))
        }
    };
    ( $attr:path ; $m:expr) => {
        {
            $crate::ast::Node::new_with_meta($attr, $m)
        }
    };
    ( $attr:path => $( $c:expr ),* $(,)?) => {
        {
            $crate::ast::Node::new_with_children($attr, vec![$( $c ),*])
        }
    };
    ( $attr:path ) => {
        {
            $crate::ast::Node::new($attr)
        }
    };
}

#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err($crate::error!($msg))
    };
    ($err:expr $(,)?) => {
        return Err($crate::error!($err))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::error!($fmt, $($arg)*))
    };
}

// "An error condition thought unreachable has occurred:\n {}\n

// Please open an issue at {} to have this addressed
// "
// #[macro_export]
// macro_rules! bail_as_unreachable {
//     ($msg:literal $(,)?) => {
//         return Err($crate::error!($msg))
//     };
//     ($err:expr $(,)?) => {
//         return Err($crate::error!($err))
//     };
//     ($fmt:expr, $($arg:tt)*) => {
//         return Err($crate::error!($fmt, $($arg)*))
//     };
// }

#[macro_export]
macro_rules! trace {
    ( $e:expr , $n:expr ) => {{
        $e.or_else(|e| return crate::prog_gen::trace_error($n, e))?
    }};
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
        $crate::Error { msg: format!($fmt, $($arg)*) }
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

#[macro_export]
macro_rules! backend_fail {
    ($message:expr) => {
        $crate::LOGGER.error(&format!(
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
                $crate::LOGGER.error(&format!("A problem occurred in the Origen backend as an Error or None value was unwrapped: '{}'", $message));
                std::process::exit(1);
            }
        }
    }};
}

/// Get the caller name. Taken from this SO answer:
/// https://stackoverflow.com/a/63904992/8533619
#[macro_export]
macro_rules! current_func {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);

        // Find and cut the rest of the path
        match &name[..name.len() - 3].rfind(':') {
            Some(pos) => &name[pos + 1..name.len() - 3],
            None => &name[..name.len() - 3],
        }
    }};
}

#[macro_export]
macro_rules! hashmap {
    ( $( $name:expr => $value:expr ),+ ) => {{
        let mut h = std::collections::HashMap::new();
        $( h.insert($name, $value); )+
        h
    }};
}

/// Create a new pathbuf, rebuilt from components to resolve OS path differences
/// when displaying paths possibly built from "/", "\", or "\\" used interchangeably
#[macro_export]
macro_rules! scrub_path {
    ($path:expr) => {{
        std::path::PathBuf::from_iter($path.components())
    }}
}
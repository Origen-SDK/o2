#[macro_export]
macro_rules! cycle {
    ( $repeat:expr ) => {{
        crate::node!(Cycle, $repeat, true)
    }};
}

#[macro_export]
macro_rules! set_drive_high {
    ( $pin_name:expr ) => {{
        let mut h: std::collections::HashMap<String, (crate::core::model::pins::pin::PinActions, u8)> = std::collections::HashMap::new();
        h.insert($pin_name.to_string(), (crate::core::model::pins::pin::PinActions::DriveHigh, 1));
        crate::node!(PinAction, h)
    }};
}

#[macro_export]
macro_rules! drive_high {
    ( $pin_name:expr ) => {{
        vec!(crate::set_drive_high!($pin_name), crate::cycle!(1))
    }};
}

#[macro_export]
macro_rules! set_drive_low {
    ( $pin_name:expr ) => {{
        let mut h: std::collections::HashMap<String, (crate::core::model::pins::pin::PinActions, u8)> = std::collections::HashMap::new();
        h.insert($pin_name.to_string(), (crate::core::model::pins::pin::PinActions::DriveLow, 0));
        crate::node!(PinAction, h)
    }};
}

#[macro_export]
macro_rules! drive_low {
    ( $pin_name:expr ) => {{
        vec!(crate::set_drive_low!($pin_name), crate::cycle!(1))
    }};
}

#[macro_export]
macro_rules! set_highz {
    ( $pin_name: expr ) => {{
        let mut h: std::collections::HashMap<String, (crate::core::model::pins::pin::PinActions, u8)> = std::collections::HashMap::new();
        h.insert($pin_name.to_string(), (crate::core::model::pins::pin::PinActions::HighZ, 0));
        crate::node!(PinAction, h)
    }};
}

#[macro_export]
macro_rules! highz {
    ( $pin_name:expr ) => {{
        vec!(crate::set_highz!($pin_name), crate::cycle!(1))
    }};
}

#[macro_export]
macro_rules! set_drive {
    ( $pin_name:expr, $data:expr ) => {{
        let mut h: std::collections::HashMap<String, (crate::core::model::pins::pin::PinActions, u8)> = std::collections::HashMap::new();
        if $data {
            h.insert($pin_name.to_string(), (crate::core::model::pins::pin::PinActions::DriveHigh, 1));
        } else {
            h.insert($pin_name.to_string(), (crate::core::model::pins::pin::PinActions::DriveLow, 0));
        }
        crate::node!(PinAction, h)
    }};
}

#[macro_export]
macro_rules! drive_pin {
    ( $pin_name:expr, $data:expr ) => {{
        vec!(crate::set_drive!($pin_name, $data), crate::cycle!(1))
    }};
}

#[macro_export]
macro_rules! set_verify_high {
    ( $pin_name:expr ) => {{
        let mut h: std::collections::HashMap<String, (crate::core::model::pins::pin::PinActions, u8)> = std::collections::HashMap::new();
        h.insert($pin_name.to_string(), (crate::core::model::pins::pin::PinActions::VerifyHigh, 1));
        crate::node!(PinAction, h)
    }};
}

#[macro_export]
macro_rules! verify_high {
    ( $pin_name:expr ) => {{
        vec!(crate::set_verify_high!($pin_name), crate::cycle!(1))
    }};
}

#[macro_export]
macro_rules! set_verify_low {
    ( $pin_name:expr ) => {{
        let mut h: std::collections::HashMap<String, (crate::core::model::pins::pin::PinActions, u8)> = std::collections::HashMap::new();
        h.insert($pin_name.to_string(), (crate::core::model::pins::pin::PinActions::VerifyLow, 0));
        crate::node!(PinAction, h)
    }};
}

#[macro_export]
macro_rules! verify_low {
    ( $pin_name:expr ) => {{
        vec!(crate::set_verify_low!($pin_name), crate::cycle!(1))
    }};
}

#[macro_export]
macro_rules! set_verify {
    ( $pin_name:expr, $data:expr ) => {{
        let mut h: std::collections::HashMap<String, (crate::core::model::pins::pin::PinActions, u8)> = std::collections::HashMap::new();
        if $data {
            h.insert($pin_name.to_string(), (crate::core::model::pins::pin::PinActions::VerifyHigh, 1));
        } else {
            h.insert($pin_name.to_string(), (crate::core::model::pins::pin::PinActions::VerifyLow, 0));
        }
        crate::node!(PinAction, h)
    }};
}

#[macro_export]
macro_rules! verify_pin {
    ( $pin_name:expr, $data:expr ) => {{
        vec!(crate::set_verify!($pin_name, $data), crate::cycle!(1))
    }};
}

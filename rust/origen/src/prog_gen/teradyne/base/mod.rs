use super::super::test::Definition;

// This is called at the start of a test program generation run to define
// all of the built-in Teradyne test templates
pub fn _define_test_lib() {
    Definition {
        test_def_id: None,
        params: indexmap! {
            "arg0".to_string() => None,
            "arg1".to_string() => None,
            "arg2".to_string() => None,
            "arg3".to_string() => None,
            "arg4".to_string() => None,
            "arg5".to_string() => None,
        },
        aliases: indexmap! {},
        defaults: indexmap! {},
    };
}

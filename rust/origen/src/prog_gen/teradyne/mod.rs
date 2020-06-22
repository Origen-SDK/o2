mod base;
mod j750;
mod ultraflex;

// This is called at the start of a test program generation run to define
// all of the built-in Teradyne test templates
pub fn _define_test_lib() {
    base::_define_test_lib();
}

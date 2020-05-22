mod base;
mod j750;
mod ultraflex;

// This is called at the start of a test program generation run to define
// all of the built-in Teradyne test templates
pub fn define_test_lib() {
    base::define_test_lib();
}

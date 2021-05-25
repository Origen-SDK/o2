use crate::Result;
use super::Frontend;

pub static BEFORE_TESTER_RESET: &str = "before_tester_reset";
pub static AFTER_TESTER_RESET: &str = "after_tester_reset";

pub fn register_callbacks(f: &dyn Frontend) -> Result<()> {
    f.register_callback(
        BEFORE_TESTER_RESET,
        "Emitted just before the global tester are reset. This would be the last chance do gleam anything from the global tester.",
    )?;
    f.register_callback(
        AFTER_TESTER_RESET,
        "Emitted just after the global tester is reset but before any targets are (re)loaded.",
    )?;
    Ok(())
}

pub trait FrontendHandler {
    fn emit(&self, callback: &str) -> Result<()>;
    fn register_new(&self, callback: &str) -> Result<()>;
}

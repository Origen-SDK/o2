use crate::commands::_prelude::*;

pub const BASE_CMD: &'static str = "publish";

/// Temporary compatibility surface for the retired core-development
/// publisher. Release behavior now belongs to ``origen rc tag``.
pub(crate) fn publish_cmd<'a>() -> SubCmd<'a> {
    core_subcmd__no_exts__no_app_opts!(
        BASE_CMD,
        "Deprecated compatibility command; use 'origen rc tag'",
        { |cmd: App| cmd }
    )
}

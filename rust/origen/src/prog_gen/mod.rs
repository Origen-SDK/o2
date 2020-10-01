mod advantest;
mod teradyne;
mod test_program;
mod tests;

use indexmap::IndexMap;
use phf::phf_map;
pub use test_program::TestProgram;

// This includes a map of all test template files, it is built by build.rs at compile time.
// All files in each sub-directory of prog_gen/test_templates are accessible via a map with the
// following structure:
//
//      TEST_TEMPLATES = {
//        "advantest/smt7/dc_tml/continuity.json" => "...",
//        "advantest/smt7/dc_tml/dps_connectivity.json" => "...",
//        ...
//        "advantest/smt7/ac_tml/..." => "...",
//        ...
//        "teradyne/j750/apmu_powersupply.json" => "...",
//        "teradyne/j750/board_pmu.json" => "...",
//      }
//
// Doing it this way means that we can just drop new files into the templates dirs and they will
// automatically be picked up and included in the new build.
include!(concat!(env!("OUT_DIR"), "/test_templates.rs"));

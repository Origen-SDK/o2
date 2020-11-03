mod processors;

use crate::core::tester::TesterAPI;
use crate::prog_gen::Database;
use crate::testers::smt::V93K_SMT7;
use crate::Result;
use crate::FLOW;
use std::path::PathBuf;

/// Main entry point to render the current test program, paths to all files generated are returned
pub fn render_test_program(tester: &V93K_SMT7, _database: &Database) -> Result<Vec<PathBuf>> {
    let mut files = vec![];

    let output_dir = tester.output_dir()?.join("test_program");
    let flow_dir = output_dir.join("testflow");
    if !flow_dir.exists() {
        std::fs::create_dir_all(&flow_dir)?;
    }

    FLOW.with_all_flows(|flows| {
        for (name, flow) in flows {
            log_debug!("Rendering flow '{}' for V93k SMT7", name);
            files.push(processors::write_to_file::run(&flow.to_node(), &flow_dir)?);
        }
        Ok(())
    })?;

    Ok(files)
}

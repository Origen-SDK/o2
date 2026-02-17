pub(crate) mod processors;

use crate::prog_gen::supported_testers::SupportedTester;
use crate::prog_gen::{Model, process_flow};
use crate::{Result, FLOW};
use std::path::{Path, PathBuf};

/// Main entry point to render the current test program, paths to all files generated are returned
pub fn render(output_dir: &Path) -> Result<(Vec<PathBuf>, Model)> {
    let mut files = vec![];

    let testflow_dir = output_dir.join("flows");
    if !testflow_dir.exists() {
        std::fs::create_dir_all(&testflow_dir)?;
    }

    let model = FLOW.with_all_flows(|flows| {
        let mut model = Model::new(SupportedTester::V93KSMT8);

        for (name, flow) in flows {
            log_debug!("Preparing flow '{}' for V93k SMT8", name);
            let (ast, m) = process_flow(flow, model, SupportedTester::V93KSMT8, true)?;

            ////////////////////////////////////////////////////////////////////////////////////////////////////////
            // Generate the flow and limits files
            ////////////////////////////////////////////////////////////////////////////////////////////////////////
            log_debug!("Rendering the main flow file '{}' for V93k SMT8", name);
            let (m, mut new_files) = processors::flow_generator::run(&ast, &testflow_dir, m)?;
            model = m;
            files.append(&mut new_files);
        }

        Ok(model)
    })?;

    Ok((files, model))
}
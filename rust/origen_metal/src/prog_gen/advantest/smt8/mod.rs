mod processors;

use super::super::validators as generic_validators;
use crate::prog_gen::processors as generic_processors;
use crate::prog_gen::{Model, PatternReferenceType, PatternType, VariableType};
use crate::prog_gen::supported_testers::SupportedTester;
use crate::{Result, FLOW};
use std::io::Write;
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
            log_debug!("Rendering flow '{}' for V93k SMT8", name);
            let ast = flow.process(&mut |n| {
                generic_processors::target_tester::run(n, SupportedTester::V93KSMT8)
            })?;
            generic_validators::duplicate_ids::run(&ast)?;
            generic_validators::missing_ids::run(&ast)?;
            generic_validators::jobs::run(&ast)?;
            generic_validators::flags::run(&ast)?;

            // This should be run at the very start after the AST has been validated, it removes all define test
            // and attribute nodes which allows the optimizations to
            let (ast, m) = generic_processors::initial_model_extract::run(
                &ast,
                SupportedTester::V93KSMT8,
                model,
            )?;
            let ast = generic_processors::nest_on_result_nodes::run(&ast)?;
            let ast = generic_processors::relationship::run(&ast)?;
            let ast = generic_processors::condition::run(&ast)?;
            let ast = generic_processors::continue_implementer::run(&ast)?;
            let ast = generic_processors::flag_optimizer::run(&ast, None)?;
            let ast = generic_processors::adjacent_if_combiner::run(&ast)?;

            // Some V93K-specific model and AST processing
            //let (ast, m) = processors::clean_names_and_add_sig::run(&ast, m)?;

            ////////////////////////////////////////////////////////////////////////////////////////////////////////
            // Generate the main flow file
            ////////////////////////////////////////////////////////////////////////////////////////////////////////
            // Do a final model extract for things which may have been optimized away if done earlier, e.g. flag variables
            let (ast, m) = generic_processors::final_model_extract::run(&ast, m)?;
            let (m, mut new_files) = processors::flow_generator::run(&ast, &testflow_dir, m)?;
            model = m;
            files.append(&mut new_files);
        }

        Ok(model)
    })?;

    Ok((files, model))
}
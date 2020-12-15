mod processors;

use super::super::validators as generic_validators;
use crate::core::tester::TesterAPI;
use crate::generator::processors::TargetTester;
use crate::prog_gen::processors as generic_processors;
use crate::prog_gen::Model;
use crate::testers::smt::V93K_SMT7;
use crate::testers::SupportedTester;
use crate::{Result, FLOW};
use std::io::Write;
use std::path::PathBuf;

/// Main entry point to render the current test program, paths to all files generated are returned
pub fn render_test_program(tester: &V93K_SMT7) -> Result<Vec<PathBuf>> {
    let mut files = vec![];

    let output_dir = tester.output_dir()?.join("test_program");
    let testflow_dir = output_dir.join("testflow");
    if !testflow_dir.exists() {
        std::fs::create_dir_all(&testflow_dir)?;
    }
    let vectors_dir = output_dir.join("vectors");
    if !vectors_dir.exists() {
        std::fs::create_dir_all(&vectors_dir)?;
    }

    FLOW.with_all_flows(|flows| {
        let mut model = Model::new();

        for (name, flow) in flows {
            log_debug!("Rendering flow '{}' for V93k SMT7", name);
            let ast = flow.process(&mut |n| TargetTester::run(n, SupportedTester::V93KSMT7))?;
            generic_validators::duplicate_ids::run(&ast)?;
            generic_validators::missing_ids::run(&ast)?;
            generic_validators::jobs::run(&ast)?;
            generic_validators::flags::run(&ast)?;
            // This initially populates the model, however it is not considered finalized until after the flow generator
            // has run, at that point any ATE-specific information will have been entered into the model
            let (ast, m) =
                generic_processors::extract_to_model::run(&ast, SupportedTester::V93KSMT7, model)?;
            model = processors::clean_names::run(&ast, m)?;
            let ast = generic_processors::nest_on_result_nodes::run(&ast)?;
            let ast = generic_processors::relationship::run(&ast)?;
            let ast = generic_processors::condition::run(&ast)?;
            let ast = generic_processors::continue_implementer::run(&ast)?;
            let ast = generic_processors::flag_optimizer::run(&ast, None)?;
            let ast = generic_processors::adjacent_if_combiner::run(&ast)?;
            //dbg!(&ast);
            ////////////////////////////////////////////////////////////////////////////////////////////////////////
            // Generate the main flow file
            ////////////////////////////////////////////////////////////////////////////////////////////////////////
            let (m, mut new_files) = processors::flow_generator::run(&ast, &testflow_dir, model)?;
            model = m;
            files.append(&mut new_files);
        }

        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        // Pattern master files (pmfl)
        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        for (name, pat_ids) in &model.pattern_collections {
            if !pat_ids.is_empty() {
                let pmfl = vectors_dir.join(&format!("{}.pmfl", name));
                let mut f = std::fs::File::create(&pmfl)?;
                files.push(pmfl);
                writeln!(&mut f, "hp93000,pattern_master_file,0.1")?;
                writeln!(&mut f, "")?;
                writeln!(&mut f, "path:")?;
                writeln!(&mut f, "../vectors")?;
                writeln!(&mut f, "")?;
                writeln!(&mut f, "files:")?;
                for p in &model.patterns_from_ids(pat_ids, true, true) {
                    writeln!(&mut f, "{}.binl.gz", p.path)?;
                }
                writeln!(&mut f, "")?;
            }
        }
        Ok(())
    })?;

    Ok(files)
}

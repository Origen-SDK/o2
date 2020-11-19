mod processors;

use super::super::validators;
use crate::core::tester::TesterAPI;
use crate::generator::processors::TargetTester;
use crate::prog_gen::processors::{ExtractToModel, NestOnResultNodes};
use crate::testers::smt::V93K_SMT7;
use crate::testers::SupportedTester;
use crate::{Result, FLOW};
use std::path::PathBuf;

/// Main entry point to render the current test program, paths to all files generated are returned
pub fn render_test_program(tester: &V93K_SMT7) -> Result<Vec<PathBuf>> {
    let mut files = vec![];

    let output_dir = tester.output_dir()?.join("test_program");
    let flow_dir = output_dir.join("testflow");
    if !flow_dir.exists() {
        std::fs::create_dir_all(&flow_dir)?;
    }

    FLOW.with_all_flows(|flows| {
        for (name, flow) in flows {
            log_debug!("Rendering flow '{}' for V93k SMT7", name);
            let ast = flow.process(&mut |n| TargetTester::run(n, SupportedTester::V93KSMT7))?;
            validators::duplicate_ids::run(&ast)?;
            validators::missing_ids::run(&ast)?;
            validators::jobs::run(&ast)?;
            validators::flags::run(&ast)?;
            let (ast, model) = ExtractToModel::run(&ast, SupportedTester::V93KSMT7, name)?;
            let mut model = processors::clean_names::run(&ast, model)?;
            let ast = NestOnResultNodes::run(&ast)?;
            //dbg!(&ast);
            files.push(processors::flow_generator::run(
                &ast, &flow_dir, &mut model,
            )?);
        }
        Ok(())
    })?;

    Ok(files)
}

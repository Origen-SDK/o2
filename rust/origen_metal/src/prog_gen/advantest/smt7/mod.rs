mod processors;

use super::super::validators as generic_validators;
use crate::prog_gen::processors as generic_processors;
use crate::prog_gen::{Model, PatternReferenceType, PatternType, VariableType};
use crate::prog_gen::supported_testers::SupportedTester;
use crate::{Result, FLOW};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Main entry point to render the current test program, paths to all files generated are returned
pub fn render_test_program(output_dir: &Path) -> Result<(Vec<PathBuf>, Model)> {
    let mut files = vec![];

    let testflow_dir = output_dir.join("testflow");
    if !testflow_dir.exists() {
        std::fs::create_dir_all(&testflow_dir)?;
    }
    let testflow_setup_dir = testflow_dir.join("setup");
    if !testflow_setup_dir.exists() {
        std::fs::create_dir_all(&testflow_setup_dir)?;
    }
    let vectors_dir = output_dir.join("vectors");
    if !vectors_dir.exists() {
        std::fs::create_dir_all(&vectors_dir)?;
    }

    let model = FLOW.with_all_flows(|flows| {
        let mut model = Model::new(SupportedTester::V93KSMT7);

        for (name, flow) in flows {
            log_debug!("Rendering flow '{}' for V93k SMT7", name);
            let ast = flow.process(&mut |n| {
                generic_processors::target_tester::run(n, SupportedTester::V93KSMT7)
            })?;
            generic_validators::duplicate_ids::run(&ast)?;
            generic_validators::missing_ids::run(&ast)?;
            generic_validators::jobs::run(&ast)?;
            generic_validators::flags::run(&ast)?;

            // This should be run at the very start after the AST has been validated, it removes all define test
            // and attribute nodes which allows the optimizations to
            let (ast, m) = generic_processors::initial_model_extract::run(
                &ast,
                SupportedTester::V93KSMT7,
                model,
            )?;
            let ast = generic_processors::nest_on_result_nodes::run(&ast)?;
            let ast = generic_processors::relationship::run(&ast)?;
            let ast = generic_processors::condition::run(&ast)?;
            let ast = generic_processors::continue_implementer::run(&ast)?;
            let ast = generic_processors::flag_optimizer::run(&ast, None)?;
            let ast = generic_processors::adjacent_if_combiner::run(&ast)?;

            // Some V93K-specific model and AST processing
            let (ast, m) = processors::clean_names_and_add_sig::run(&ast, m)?;

            ////////////////////////////////////////////////////////////////////////////////////////////////////////
            // Generate the main flow file
            ////////////////////////////////////////////////////////////////////////////////////////////////////////
            // Do a final model extract for things which may have been optimized away if done earlier, e.g. flag variables
            let (ast, m) = generic_processors::final_model_extract::run(&ast, m)?;
            let (m, mut new_files) = processors::flow_generator::run(&ast, &testflow_dir, m)?;
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
                for p in model.patterns_from_ids(pat_ids, true, true) {
                    if p.reference_type != PatternReferenceType::Origen {
                        writeln!(&mut f, "{}.binl.gz", p.path)?;
                    }
                }
                writeln!(&mut f, "")?;
            }
        }

        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        // Pattern compiler files (aiv)
        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        for (name, pat_ids) in &model.pattern_collections {
            if !pat_ids.is_empty() {
                let aiv = output_dir.join(&format!("{}.aiv", name));
                let mut f = std::fs::File::create(&aiv)?;
                files.push(aiv);
                writeln!(&mut f, "AI_DIR_FILE")?;
                writeln!(&mut f, "tmp_dir                         ./tmp")?;
                writeln!(&mut f, "tmf_dir                         ./")?;
                writeln!(&mut f, "vbc_dir                         ./")?;
                writeln!(&mut f, "avc_dir                         ./AVC/")?;
                let dut_name = "";
                writeln!(&mut f, "pinconfig_file                  ./{}.cfg", dut_name)?;
                writeln!(&mut f, "single_binary_pattern_dir       ./BINL/")?;
                writeln!(&mut f, "")?;
                writeln!(
                    &mut f,
                    "AI_V2B_OPTIONS  -ALT -c {}.vbc -k -z PS800",
                    dut_name
                )?;
                writeln!(&mut f, "")?;
                //% if $tester.multiport
                //PATTERNS name tmf_file port v2b_options
                //%   port = !!$tester.multiport ? " #{$tester.multiport}" : ''
                //% else
                writeln!(&mut f, "PATTERNS name tmf_file v2b_options")?;
                let pats = model.patterns_from_ids(pat_ids, true, true);
                let port = "";
                for p in &pats {
                    if p.reference_type != PatternReferenceType::Origen
                        && p.pattern_type == PatternType::Subroutine
                    {
                        writeln!(&mut f, "{} {}.tmf{} -s", p.path, dut_name, port)?;
                    }
                }
                for p in &pats {
                    if p.reference_type != PatternReferenceType::Origen
                        && p.pattern_type == PatternType::Main
                    {
                        writeln!(&mut f, "{} {}.tmf{}", p.path, dut_name, port)?;
                    }
                }
                //% if $tester.multiport
                //%   patterns.each do |pattern|
                //
                //MULTI_PORT_BURST "<%= "#{$tester.multiport_name(pattern)}" %>"
                //PORTS     <%= $tester.multiport %>
                //SEQ_GRPS  grp1
                //          <%= pattern %>
                //%   end
                //% end
            }
        }

        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        // Flow setup files
        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        for (name, flow) in &model.flows {
            if !flow.variables.is_empty() {
                let tf = testflow_setup_dir.join(&format!("{}_vars.tf", name));
                let mut f = std::fs::File::create(&tf)?;
                files.push(tf);
                let vars = model.variables_from_ids(&flow.variables, true, true);
                writeln!(&mut f, "hp93000,testflow,0.1")?;
                writeln!(&mut f, "language_revision = 1;")?;
                writeln!(&mut f, "")?;
                writeln!(&mut f, "declarations")?;
                writeln!(&mut f, "")?;
                let mut done_vars: Vec<&str> = vec![];
                for v in &vars {
                    if v.variable_type == VariableType::Job {
                        writeln!(&mut f, "@JOB = \"\";")?;
                    } else if v.variable_type == VariableType::Flag {
                        if !done_vars.contains(&v.name.as_str()) {
                            done_vars.push(&v.name);
                            writeln!(&mut f, "@{} = 0;", v.name)?;
                        }
                    }
                }
                writeln!(&mut f, "")?;
                writeln!(&mut f, "end")?;
                writeln!(
                    &mut f,
                    "-----------------------------------------------------------------"
                )?;
                writeln!(&mut f, "flags")?;
                writeln!(&mut f, "")?;
                for v in &vars {
                    if v.variable_type == VariableType::Enable {
                        writeln!(&mut f, "user {} = 0;", v.name)?;
                    }
                }
                writeln!(&mut f, "")?;
                writeln!(&mut f, "end")?;
                writeln!(
                    &mut f,
                    "-----------------------------------------------------------------"
                )?;
                writeln!(&mut f, "testmethodparameters")?;
                writeln!(&mut f, "end")?;
                writeln!(
                    &mut f,
                    "-----------------------------------------------------------------"
                )?;
                writeln!(&mut f, "testmethodlimits")?;
                writeln!(&mut f, "end")?;
                writeln!(
                    &mut f,
                    "-----------------------------------------------------------------"
                )?;
                writeln!(&mut f, "test_flow")?;
                writeln!(&mut f, "end")?;
                writeln!(
                    &mut f,
                    "-----------------------------------------------------------------"
                )?;
                writeln!(&mut f, "binning")?;
                writeln!(&mut f, "end")?;
                writeln!(
                    &mut f,
                    "-----------------------------------------------------------------"
                )?;
                writeln!(&mut f, "oocrule")?;
                writeln!(&mut f, "end")?;
                writeln!(
                    &mut f,
                    "-----------------------------------------------------------------"
                )?;
                writeln!(&mut f, "context")?;
                writeln!(&mut f, "end")?;
                writeln!(
                    &mut f,
                    "-----------------------------------------------------------------"
                )?;
                writeln!(&mut f, "hardware_bin_descriptions")?;
                writeln!(&mut f, "end")?;
                writeln!(
                    &mut f,
                    "-----------------------------------------------------------------"
                )?;
            }
        }

        Ok(model)
    })?;

    Ok((files, model))
}

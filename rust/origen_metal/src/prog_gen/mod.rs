pub mod advantest;
pub mod flow_api;
mod flow_manager;
mod model;
mod nodes;
mod processors;
pub mod teradyne;
mod validators;
pub mod config;
mod supported_testers;
pub mod test_ids;

use std::path::Path;
use std::path::PathBuf;

pub use flow_manager::FlowManager;
pub use model::Bin;
pub use model::BinType;
pub use model::FlowCondition;
pub use model::FlowID;
pub use model::GroupType;
pub use model::Limit;
pub use model::LimitSelector;
pub use model::LimitType;
pub use model::Model;
pub use model::ParamType;
pub use model::ParamValue;
pub use model::Pattern;
pub use model::PatternGroupType;
pub use model::PatternReferenceType;
pub use model::PatternType;
pub use model::Test;
pub use model::Variable;
pub use model::VariableOperation;
pub use model::VariableType;
pub use nodes::PGM;
pub use supported_testers::SupportedTester;
pub use model::{TestTemplate, TestTemplateParameter};
use model::load_test_from_lib;

use crate::ast::AST;
use crate::ast::{Attrs, Node};

#[derive(Debug, PartialEq, EnumString, Clone, Serialize)]
pub enum ResourcesType {
    #[strum(serialize = "All", serialize = "all", serialize = "ALL")]
    All,
    #[strum(
        serialize = "Pattern",
        serialize = "pattern",
        serialize = "PATTERN",
        serialize = "Patterns",
        serialize = "patterns",
        serialize = "PATTERNS"
    )]
    Patterns,
    #[strum(
        serialize = "Variable",
        serialize = "variable",
        serialize = "VARIABLE",
        serialize = "Variables",
        serialize = "variables",
        serialize = "VARIABLES"
    )]
    Variables,
}

/// The type of unique signature to append to test names and similar
#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum UniquenessOption {
    /// No unique identitier
    None,
    /// Add an automatically generated signature
    Signature,
    /// Add the flow name
    Flowname,
    /// Add the given string
    String(String),
}

// Implement from_str for UniquenessOption
impl std::str::FromStr for UniquenessOption {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(UniquenessOption::None),
            "signature" => Ok(UniquenessOption::Signature),
            "flowname" => Ok(UniquenessOption::Flowname),
            _ => Ok(UniquenessOption::String(s.to_string())),
        }
    }
}

pub trait ProgramGenerator {
    
}

pub fn trace_error<T: Attrs>(node: &Node<T>, error: crate::Error) -> crate::Result<()> {
    let help = {
        let s = node.meta_string();
        if s != "" {
            s
        } else {
            if crate::PROG_GEN_CONFIG.debug_enabled() {
                // Don't display children since it's potentially huge
                let n = node.replace_children(vec![]);
                format!("Sorry, no flow source information was found, here is the flow node that failed if it helps:\n{}", n)
            } else {
                "Run again with the --debug switch to try and trace this back to a flow source file location".to_string()
            }
        }
    };
    bail!("{}\n{}", error, &help)
}

pub fn render_program(tester: SupportedTester, output_dir: &Path) -> crate::Result<(Vec<PathBuf>, Model)> {
    match tester {
        SupportedTester::V93KSMT7 => advantest::smt7::render(output_dir),
        SupportedTester::V93KSMT8 => advantest::smt8::render(output_dir),
        _ => Ok((vec![], Model::new(tester))),
    }
}

/// Returns a list of accepted test invocation options for the given tester, for example on V93KSMT7 this
/// would return a list of all test suite attributes
pub fn test_invocation_options(tester: SupportedTester) -> crate::Result<Vec<String>> {
    match tester {
        SupportedTester::V93KSMT7 | SupportedTester::V93KSMT8 => {
            let t = load_test_from_lib(
                &tester,
                "_internal",
                "test_suite",
            )?;
            let mut options = vec![];
            if let Some(params) = t.parameter_list {
                for param in params.keys() {
                    options.push(param.to_owned());
                }
            }
            if let Some(params) = t.aliases {
                for param in params.keys() {
                    options.push(param.to_owned());
                }
            }
            Ok(options)
        }
        _ => Ok(vec![]),
    }
}

/// Processes the given flow AST so that it is ready to generate the flow for the given tester,
/// optionally validating it first
/// 
/// ```no_run
/// use origen_metal::FLOW;
/// use origen_metal::prog_gen::Model;
/// use origen_metal::prog_gen::SupportedTester;
/// use origen_metal::prog_gen::process_flow;
/// 
/// FLOW.with_all_flows(|flows| {
///     let mut model = Model::new(SupportedTester::V93KSMT7);
///     for (name, flow) in flows {
///         let ast;
///         (ast, model) = process_flow(flow, model, SupportedTester::V93KSMT7, true)?;
///     }
///     Ok(())
/// });
/// ```
pub fn process_flow(flow: &AST<PGM>, model: Model, tester: SupportedTester, validate: bool) -> crate::Result<(Node<PGM>, Model)> {
    log_debug!("Screening flow for tester {:?}", tester);
    //flow.to_file("unprocessed_ast.txt")?;
    let mut ast = flow.process(&mut |n| {
        processors::target_tester::run(n, tester)
    })?;

    if validate {
        log_debug!("Validating flow for tester {:?}", tester);
        validators::duplicate_ids::run(&ast)?;
        validators::missing_ids::run(&ast)?;
        validators::jobs::run(&ast)?;
        validators::flags::run(&ast)?;
        log_debug!("Flow validation completed successfully");
    }

    // This should be run at the very start after the AST has been validated, it removes all define test
    // and attribute nodes
    let mut m;
    log_debug!("Extracting initial model from flow for tester {:?}", tester);
    //ast.to_file("pre_initial_model_extract_ast.txt")?;
    (ast, m) = processors::initial_model_extract::run(
        &ast,
        tester,
        model,
    )?;

    //ast.to_file("unprocesed_ast.txt")?;
    log_debug!("Processing flow for tester {:?}", tester);
    ast = processors::clean_resources::run(&ast)?;
    ast = processors::nest_on_result_nodes::run(&ast)?;
    ast = processors::relationship::run(&ast)?;
    ast = processors::condition::run(&ast)?;
    ast = processors::continue_implementer::run(&ast)?;
    ast = processors::flag_optimizer::run(&ast, None)?;
    ast = processors::adjacent_if_combiner::run(&ast)?;

    // Any tester-specific processing
    match tester {
        SupportedTester::V93KSMT7 => {
            (ast, m) = advantest::smt7::processors::clean_names_and_add_sig::run(&ast, m)?;
        }
        SupportedTester::V93KSMT8 => {
            ast = advantest::smt8::processors::create_flow_data::run(&ast)?;
        }
        _ => { }
    }

    //ast.to_file("ast.txt")?;

    // Do a final model extract for things which may have been optimized away if done earlier, e.g. flag variables
    Ok(processors::final_model_extract::run(&ast, m)?)
}
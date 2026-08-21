use super::*;
use crate::prog_gen::{
    BinType, FlowCondition, FlowID, IGXLResource, IGXLResourceKind, Limit, LimitSelector,
    LimitType, Model, ParamValue, PatternGroupType, ResourcesType, SupportedTester, PGM,
};
use indexmap::IndexMap;
use std::path::PathBuf;
use tempfile::tempdir;

fn find_flow_row<'a>(flow: &'a str, opcode: &str, parameter: &str) -> Vec<&'a str> {
    flow.lines()
        .map(|line| line.split('\t').skip(1).collect::<Vec<_>>())
        .find(|columns| columns.len() == 32 && columns[5] == opcode && columns[6] == parameter)
        .unwrap_or_else(|| panic!("expected flow row {} {}", opcode, parameter))
}

#[test]
fn renders_flow_conditions_and_core_worksheets() -> Result<()> {
    let reference_values = IndexMap::from([("comment".to_string(), vec!["Block1".to_string()])]);
    let job_values = IndexMap::from([
        ("pinmap".to_string(), vec!["pinmap_test".to_string()]),
        (
            "instances".to_string(),
            vec!["prb1_instances".to_string(), "global_instances".to_string()],
        ),
        ("flows".to_string(), vec!["prb1_flow".to_string()]),
    ]);
    let global_spec_values = IndexMap::from([
        ("value".to_string(), vec!["=17".to_string()]),
        ("job".to_string(), vec!["FT".to_string()]),
        ("comment".to_string(), vec!["entering spec1".to_string()]),
    ]);
    let ac_spec_values = IndexMap::from([
        ("specset".to_string(), vec!["func_100MHz".to_string()]),
        ("selector".to_string(), vec!["nom".to_string()]),
        ("typ".to_string(), vec!["=10*ns".to_string()]),
        ("min".to_string(), vec!["=9*ns".to_string()]),
        ("max".to_string(), vec!["=11*ns".to_string()]),
    ]);
    let dc_spec_values = IndexMap::from([
        ("specset".to_string(), vec!["power_down_levels".to_string()]),
        ("selector".to_string(), vec!["nom".to_string()]),
        ("typ".to_string(), vec!["=0.2*V".to_string()]),
        ("min".to_string(), vec!["=0.1*V".to_string()]),
        ("max".to_string(), vec!["=0.3*V".to_string()]),
    ]);
    let pin_values = IndexMap::from([
        ("kind".to_string(), vec!["power".to_string()]),
        ("group".to_string(), vec![String::new()]),
        ("type".to_string(), vec!["Power".to_string()]),
        ("comment".to_string(), vec!["# vdd1".to_string()]),
    ]);
    let level_values = IndexMap::from([
        ("parameter".to_string(), vec!["VMain".to_string()]),
        ("value".to_string(), vec!["=_vdd_main_val".to_string()]),
        ("comment".to_string(), vec![String::new()]),
    ]);
    let edgeset_values = IndexMap::from([
        ("edgeset".to_string(), vec!["es1".to_string()]),
        ("src".to_string(), vec!["PAT".to_string()]),
        ("format".to_string(), vec!["NR".to_string()]),
        ("drive_on".to_string(), vec!["=0*ns".to_string()]),
        ("drive_data".to_string(), vec!["=1*ns".to_string()]),
        ("drive_return".to_string(), vec![String::new()]),
        ("drive_off".to_string(), vec![String::new()]),
        ("compare_mode".to_string(), vec!["Edge".to_string()]),
        ("compare_open".to_string(), vec!["=2*ns".to_string()]),
        ("compare_close".to_string(), vec!["=3*ns".to_string()]),
        ("resolution".to_string(), vec![String::new()]),
        ("timing_mode".to_string(), vec!["Machine".to_string()]),
    ]);
    let timeset_values = IndexMap::from([
        ("period".to_string(), vec!["=10*ns".to_string()]),
        ("pin".to_string(), vec!["tclk".to_string()]),
        ("edgeset".to_string(), vec!["es1".to_string()]),
        ("clock_period".to_string(), vec!["=10*ns".to_string()]),
        ("setup".to_string(), vec!["clock".to_string()]),
        ("timing_mode".to_string(), vec!["Machine".to_string()]),
    ]);
    let timeset_basic_values = IndexMap::from([
        ("period".to_string(), vec!["=10*ns".to_string()]),
        ("pin".to_string(), vec!["tclk".to_string()]),
        ("clock_period".to_string(), vec!["=10*ns".to_string()]),
        ("setup".to_string(), vec!["clock".to_string()]),
        ("src".to_string(), vec!["PAT".to_string()]),
        ("format".to_string(), vec!["NR".to_string()]),
        ("drive_on".to_string(), vec!["=0*ns".to_string()]),
        ("drive_data".to_string(), vec!["=1*ns".to_string()]),
        ("drive_return".to_string(), vec![String::new()]),
        ("drive_off".to_string(), vec![String::new()]),
        ("compare_mode".to_string(), vec!["Edge".to_string()]),
        ("compare_open".to_string(), vec!["=2*ns".to_string()]),
        ("compare_close".to_string(), vec!["=3*ns".to_string()]),
        ("resolution".to_string(), vec![String::new()]),
        ("timing_mode".to_string(), vec!["Machine".to_string()]),
    ]);
    let flow = node!(PGM::Flow, "prb1".to_string() =>
        node!(PGM::Group, "func_group".to_string(), Some(SupportedTester::ULTRAFLEX), crate::prog_gen::GroupType::Test, None =>
            node!(PGM::DefTest, 1, "func_ins".to_string(), SupportedTester::ULTRAFLEX, "std".to_string(), "functional".to_string()),
            node!(PGM::DefTest, 7, "func_ins_duplicate".to_string(), SupportedTester::ULTRAFLEX, "std".to_string(), "functional".to_string()),
            node!(PGM::DefTest, 8, "func_ins_variant".to_string(), SupportedTester::ULTRAFLEX, "std".to_string(), "functional".to_string())
        ),
        node!(PGM::DefTestInv, 2, "func".to_string(), SupportedTester::ULTRAFLEX),
        node!(PGM::AssignTestToInv, 2, 1),
        node!(PGM::SetAttr, 1, "pattern".to_string(), Some(ParamValue::Any("func_pset".to_string())), false),
        node!(PGM::SetAttr, 7, "pattern".to_string(), Some(ParamValue::Any("func_pset".to_string())), false),
        node!(PGM::SetAttr, 8, "pattern".to_string(), Some(ParamValue::Any("func_variant_pset".to_string())), false),
        node!(PGM::SetAttr, 2, "bin".to_string(), Some(ParamValue::Any("3".to_string())), false),
        node!(PGM::SetAttr, 2, "softbin".to_string(), Some(ParamValue::Any("100".to_string())), false),
        node!(PGM::SetLimit, None, Some(2), LimitSelector::Lo, Some(Limit { kind: LimitType::GTE, value: ParamValue::Float(-2.0), unit: Some("V".to_string()) })),
        node!(PGM::SetLimit, None, Some(2), LimitSelector::Hi, Some(Limit { kind: LimitType::LTE, value: ParamValue::Float(2.0), unit: Some("V".to_string()) })),
        node!(PGM::DefSubTest, 1, "lim1".to_string(), Some(1001),
            Some(Limit { kind: LimitType::GTE, value: ParamValue::Float(-1.0), unit: Some("V".to_string()) }),
            Some(Limit { kind: LimitType::LTE, value: ParamValue::Float(1.0), unit: Some("V".to_string()) })),
        node!(PGM::DefTest, 6, "custom_ins".to_string(), SupportedTester::ULTRAFLEX, "std".to_string(), "custom".to_string()),
        node!(PGM::SetAttr, 6, "test_name".to_string(), Some(ParamValue::Any("custom_ins".to_string())), false),
        node!(PGM::SetAttr, 6, "proc_name".to_string(), Some(ParamValue::Any("MyCustomProcedure".to_string())), false),
        node!(PGM::PatternGroup, 3, "func_pset".to_string(), SupportedTester::ULTRAFLEX, Some(PatternGroupType::Patset)),
        node!(PGM::PushPattern, 3, "func.PAT".to_string(), None),
        node!(PGM::PatternGroup, 4, "legacy_group".to_string(), SupportedTester::ULTRAFLEX, Some(PatternGroupType::Patgroup)),
        node!(PGM::PushPattern, 4, "legacy.PAT".to_string(), None),
        node!(PGM::PatternGroup, 5, "subroutines".to_string(), SupportedTester::ULTRAFLEX, Some(PatternGroupType::Patsubr)),
        node!(PGM::PushPattern, 5, "nvm_global_subs.PAT".to_string(), None),
        node!(PGM::Condition, FlowCondition::IfJob(vec!["prb1".to_string()]) =>
            node!(PGM::Test, 2, FlowID::from_str("func"))
        ),
        node!(PGM::TestStr, "guarded".to_string(), FlowID::from_str("guarded"), None, None, Some(10) =>
            node!(PGM::OnFailed, FlowID::from_str("guarded") =>
                node!(PGM::Bin, 5, Some(50), BinType::Bad)
            )
        ),
        node!(PGM::Condition, FlowCondition::UnlessEnable(vec!["quick".to_string()]) =>
            node!(PGM::Log, "slow path".to_string())
        ),
        node!(PGM::Group, "group1".to_string(), None, crate::prog_gen::GroupType::Flow, Some(FlowID::from_str("group1")) =>
            node!(PGM::TestStr, "group_test1".to_string(), FlowID::from_str("group_test1"), None, None, Some(20)),
            node!(PGM::TestStr, "group_test2".to_string(), FlowID::from_str("group_test2"), None, None, Some(21))
        ),
        node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_str("group1")]) =>
            node!(PGM::Log, "group failed".to_string())
        ),
        node!(PGM::ResourcesFilename, "shared".to_string(), ResourcesType::All),
        node!(PGM::IGXLResource, IGXLResource::new("references", ".\\inc\\file1.xla".to_string(), reference_values)?),
        node!(PGM::IGXLResource, IGXLResource::new("jobs", "FT".to_string(), job_values)?),
        node!(PGM::IGXLResource, IGXLResource::new("global_specs", "spec1".to_string(), global_spec_values)?),
        node!(PGM::IGXLResource, IGXLResource::new("ac_specs", "cycle".to_string(), ac_spec_values)?),
        node!(PGM::IGXLResource, IGXLResource::new("dc_specs", "vdd_main_val".to_string(), dc_spec_values)?),
        node!(PGM::IGXLResource, IGXLResource::new("pinmap", "vdd1".to_string(), pin_values)?),
        node!(PGM::IGXLResource, IGXLResource::new("levels", "vdd1".to_string(), level_values)?),
        node!(PGM::IGXLResource, IGXLResource::new("edgesets", "tclk".to_string(), edgeset_values)?),
        node!(PGM::IGXLResource, IGXLResource::new("timesets", "t1".to_string(), timeset_values)?),
        node!(PGM::IGXLResource, IGXLResource::new("timesets_basic", "t1".to_string(), timeset_basic_values)?),
        node!(PGM::Log, "done".to_string())
    );
    let mut ast = crate::ast::AST::new();
    ast.start(flow);
    let (ast, model) = process_flow(
        &ast,
        Model::new(SupportedTester::ULTRAFLEX),
        SupportedTester::ULTRAFLEX,
        true,
    )?;
    let dir = tempdir()?;
    let (_model, mut files, rows, patterns) = render_flow(&ast, dir.path(), model, "prb1")?;
    files.append(&mut ResourceGenerator::new(rows).render(dir.path())?);
    files.push(write_referenced_list(dir.path(), patterns)?.unwrap());
    assert_eq!(files.len(), 7);
    let flow = std::fs::read_to_string(dir.path().join("prb1_flow.txt"))?;
    let columns = find_flow_row(&flow, "Test-defer-limits", "func_group_v1");
    assert_eq!(columns.len(), 32);
    assert_eq!(columns[2], "PRB1");
    assert_eq!(columns[7], "func");
    assert_eq!(columns[9], "-2");
    assert_eq!(columns[10], "2");
    assert_eq!(columns[12], "V");
    assert_eq!(columns[15], "3");
    assert_eq!(columns[17], "100");
    assert_eq!(columns[18], "Fail");
    let columns = find_flow_row(&flow, "Use-Limit", "func_group_v1");
    assert_eq!(columns[7], "lim1");
    assert_eq!(columns[8], "1001");
    assert_eq!(columns[9], "-1");
    assert_eq!(columns[10], "1");
    let columns = find_flow_row(&flow, "Test", "guarded");
    assert_eq!(columns[7], "guarded");
    assert_eq!(columns[8], "10");
    assert_eq!(columns[20], "guarded_FAILED");
    let columns = find_flow_row(&flow, "goto", "ORIGEN_SKIP_1");
    assert_eq!(columns[1], "quick");
    let columns = find_flow_row(&flow, "nop", "");
    assert_eq!(columns[0], "ORIGEN_SKIP_1");
    let columns = find_flow_row(&flow, "flag-false", "group1_FAILED");
    assert_eq!(columns[5], "flag-false");
    let columns = find_flow_row(&flow, "flag-true", "group1_PASSED");
    assert_eq!(columns[5], "flag-true");
    let instances = std::fs::read_to_string(dir.path().join("prb1_instances.txt"))?;
    assert!(instances.contains("\tfunc_group_v1\tVBT\tFunctional_T\tExcel Macro"));
    assert!(instances.contains("\tfunc_group_v2\tVBT\tFunctional_T\tExcel Macro"));
    assert_eq!(instances.matches("\tfunc_group_v1\t").count(), 1);
    assert!(instances.contains("\tcustom_ins\tOther\tMyCustomProcedure\tVB DLL"));
    let patsets = std::fs::read_to_string(dir.path().join("prb1_patsets.txt"))?;
    assert!(patsets.contains("\tfunc_pset\t\t\tfunc.PAT\tYes"));
    let shared = std::fs::read_to_string(dir.path().join("shared.txt"))?;
    assert!(shared.contains("\t.\\inc\\file1.xla\tBlock1"));
    assert!(shared.contains("\tFT\tpinmap_test\tprb1_instances,global_instances\tprb1_flow"));
    let patgroups = std::fs::read_to_string(dir.path().join("prb1_patgroups.txt"))?;
    assert!(patgroups.contains("ULTRAFLEX DOES NOT SUPPORT PATTERN GROUP SHEETS!!"));
    let patsubrs = std::fs::read_to_string(dir.path().join("prb1_patsubrs.txt"))?;
    assert!(patsubrs.contains("\tnvm_global_subs.PAT\t"));
    assert!(shared.contains("\tspec1\tFT\t=17\tentering spec1"));
    assert!(shared.contains("\tcycle\t\tnom\tMax\t=10*ns\t=9*ns\t=11*ns"));
    assert!(shared.contains("\tvdd_main_val\t\tnom\tMax\t=0.2*V\t=0.1*V\t=0.3*V"));
    assert!(shared.contains("\t\tvdd1\tPower\t# vdd1"));
    assert!(shared.contains("\tvdd1\t\tVMain\t=_vdd_main_val\t"));
    assert!(shared.contains("\ttclk\tes1\tPAT\tNR\t=0*ns\t=1*ns"));
    assert!(shared.contains("\tt1\t=10*ns\ttclk\t=10*ns\tclock\tes1"));
    assert!(shared.contains("\tt1\t=10*ns\ttclk\t=10*ns\tclock\tPAT\tNR"));

    if let Ok(capture_dir) = std::env::var("O2_UFLEX_CAPTURE_DIR") {
        let capture_dir = PathBuf::from(capture_dir);
        std::fs::create_dir_all(&capture_dir)?;
        for file in &files {
            if let Some(name) = file.file_name() {
                std::fs::copy(file, capture_dir.join(name))?;
            }
        }
    }
    let approved_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test_apps/python_app/approved/ultraflex/test_program");
    if std::env::var_os("O2_UFLEX_UPDATE_GOLDENS").is_some() {
        std::fs::create_dir_all(&approved_dir)?;
        for generated_path in &files {
            let name = generated_path.file_name().expect("generated file name");
            std::fs::copy(generated_path, approved_dir.join(name))?;
        }
    } else {
        for generated_path in &files {
            let name = generated_path.file_name().expect("generated file name");
            let generated = std::fs::read_to_string(generated_path)?;
            let approved = std::fs::read_to_string(approved_dir.join(name))?;
            assert_eq!(
                normalize_line_endings(&generated),
                normalize_line_endings(&approved),
                "UltraFLEX golden mismatch for {:?}",
                name
            );
        }
    }
    Ok(())
}

#[test]
fn aggregates_shared_resources_from_multiple_flows() -> Result<()> {
    let rows = vec![
        (
            "shared".to_string(),
            IGXLResourceKind::References,
            "flow1.xla".to_string(),
            IndexMap::from([("comment".to_string(), vec!["flow 1".to_string()])]),
        ),
        (
            "shared".to_string(),
            IGXLResourceKind::References,
            "flow2.xla".to_string(),
            IndexMap::from([("comment".to_string(), vec!["flow 2".to_string()])]),
        ),
    ];
    let dir = tempdir()?;
    let files = ResourceGenerator::new(rows).render(dir.path())?;
    assert_eq!(files, vec![dir.path().join("shared.txt")]);
    let references = std::fs::read_to_string(&files[0])?;
    assert!(references.contains("\tflow1.xla\tflow 1"));
    assert!(references.contains("\tflow2.xla\tflow 2"));
    Ok(())
}

#[test]
fn one_neutral_flow_targets_ultraflex_and_v93k() -> Result<()> {
    let source = node!(PGM::Flow, "multi_target".to_string() =>
        node!(PGM::Log, "shared step".to_string()),
        node!(PGM::TesterEq, vec![SupportedTester::ULTRAFLEX] =>
            node!(PGM::TestStr, "uflex_only".to_string(), FlowID::from_str("uflex_only"), None, None, Some(10))
        ),
        node!(PGM::TesterEq, vec![SupportedTester::V93KSMT8] =>
            node!(PGM::TestStr, "v93k_only".to_string(), FlowID::from_str("v93k_only"), None, None, Some(20))
        )
    );
    let mut source_ast = crate::ast::AST::new();
    source_ast.start(source);

    let (uflex_ast, uflex_model) = process_flow(
        &source_ast,
        Model::new(SupportedTester::ULTRAFLEX),
        SupportedTester::ULTRAFLEX,
        true,
    )?;
    let uflex_dir = tempdir()?;
    let (_, _, _, _) = render_flow(&uflex_ast, uflex_dir.path(), uflex_model, "multi_target")?;
    let uflex = std::fs::read_to_string(uflex_dir.path().join("multi_target_flow.txt"))?;
    assert!(uflex.contains("shared step"));
    assert!(uflex.contains("uflex_only"));
    assert!(!uflex.contains("v93k_only"));

    let (v93k_ast, v93k_model) = process_flow(
        &source_ast,
        Model::new(SupportedTester::V93KSMT8),
        SupportedTester::V93KSMT8,
        true,
    )?;
    let v93k_dir = tempdir()?;
    let (_, v93k_files) = crate::prog_gen::advantest::smt8::processors::flow_generator::run(
        &v93k_ast,
        v93k_dir.path(),
        v93k_model,
    )?;
    let v93k_path = v93k_files
        .iter()
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("flow"))
        .expect("expected SMT8 flow output");
    let v93k = std::fs::read_to_string(v93k_path)?;
    assert!(v93k.contains("shared step"));
    assert!(v93k.contains("v93k_only"));
    assert!(!v93k.contains("uflex_only"));
    Ok(())
}

#[test]
fn renders_supported_control_opcodes_and_rejects_unsupported_ones() -> Result<()> {
    assert_eq!(resources::uflex_expression("10*ns", false), "=10*ns");
    assert_eq!(resources::uflex_expression("=10*ns", false), "=10*ns");
    assert_eq!(resources::uflex_expression("", true), "disable");
    assert_eq!(resources::uflex_expression("", false), "");
    assert_eq!(resources::uflex_spec_expression(""), "0");
    assert_eq!(normalize_line_endings("a\r\nb\r\n"), "a\nb\n");
    let mut model = Model::new(SupportedTester::ULTRAFLEX);
    model.create_flow("control")?;
    let mut generator = FlowGenerator::new(model);
    let supported = node!(PGM::Flow, "control".to_string() =>
        node!(PGM::Label, "RETRY".to_string()),
        node!(PGM::Goto, "RETRY".to_string()),
        node!(PGM::Comment, "retry loop".to_string()),
        node!(PGM::Condition, FlowCondition::IfFlag(vec!["FLAG1".to_string(), "FLAG2".to_string()]) =>
            node!(PGM::Log, "multi flag".to_string())
        ),
        node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["FLAG1".to_string(), "FLAG2".to_string()]) =>
            node!(PGM::Log, "unless multi flag".to_string())
        )
    );
    supported.process(&mut generator)?;
    assert!(generator
        .rows
        .iter()
        .any(|row| row.starts_with("\tRETRY\t") && row.contains("\tnop\t")));
    assert!(generator
        .rows
        .iter()
        .any(|row| row.contains("\tgoto\tRETRY\t")));
    assert!(generator.rows.iter().any(|row| row.ends_with("retry loop")));
    let multi = generator
        .rows
        .iter()
        .find(|row| row.contains("multi flag"))
        .unwrap();
    let columns = multi.split('\t').skip(1).collect::<Vec<_>>();
    assert_eq!(columns[22], "any-active");
    assert_eq!(columns[24], "flag-true");
    assert_eq!(columns[25], "FLAG1,FLAG2");
    let unless_multi = generator
        .rows
        .iter()
        .find(|row| row.contains("unless multi flag"))
        .unwrap();
    let columns = unless_multi.split('\t').skip(1).collect::<Vec<_>>();
    assert_eq!(columns[22], "any-active");
    assert_eq!(columns[23], "not");
    assert_eq!(columns[24], "flag-true");
    assert_eq!(columns[25], "FLAG1,FLAG2");

    let unsupported = node!(PGM::Wait, "1ms".to_string());
    let error = unsupported.process(&mut generator).unwrap_err();
    assert!(error.to_string().contains("no UltraFLEX IG-XL equivalent"));
    Ok(())
}

#[test]
fn supports_independent_resource_sheet_names() -> Result<()> {
    let mut model = Model::new(SupportedTester::ULTRAFLEX);
    model.create_flow("resources")?;
    let reference = IGXLResource::new(
        "references",
        "library.xla".to_string(),
        IndexMap::from([("comment".to_string(), vec!["shared library".to_string()])]),
    )?;
    let node = node!(PGM::Flow, "resources".to_string() =>
        node!(PGM::IGXLResourcesFilename, crate::prog_gen::IGXLResourceKind::References, "Refs".to_string()),
        node!(PGM::IGXLResource, reference)
    );
    let mut generator = FlowGenerator::new(model);
    node.process(&mut generator)?;
    assert_eq!(generator.resources_rows[0].0, "Refs");
    let dir = tempdir()?;
    let files = ResourceGenerator::new(generator.resources_rows).render(dir.path())?;
    assert_eq!(files, vec![dir.path().join("Refs.txt")]);
    Ok(())
}

#[test]
fn rejects_ambiguous_specs_and_limit_units() -> Result<()> {
    let lo = Limit {
        kind: LimitType::GTE,
        value: ParamValue::Float(1.0),
        unit: Some("A".to_string()),
    };
    let hi = Limit {
        kind: LimitType::LTE,
        value: ParamValue::Float(2.0),
        unit: Some("V".to_string()),
    };
    assert_eq!(flow::resolve_limit_units("lo_only", Some(&lo), None)?, "A");
    let matching_hi = Limit {
        kind: LimitType::LTE,
        value: ParamValue::Float(2.0),
        unit: Some("A".to_string()),
    };
    assert_eq!(
        flow::resolve_limit_units("matching", Some(&lo), Some(&matching_hi))?,
        "A"
    );
    let error = flow::resolve_limit_units("mixed_units", Some(&lo), Some(&hi)).unwrap_err();
    assert!(error.to_string().contains("incompatible limit units"));

    let mut generator = FlowGenerator::new(Model::new(SupportedTester::ULTRAFLEX));
    generator.resources_rows = vec![
        (
            "SpecsAC".to_string(),
            IGXLResourceKind::References,
            "library.xla".to_string(),
            IndexMap::from([("comment".to_string(), vec!["library".to_string()])]),
        ),
        (
            "SpecsAC".to_string(),
            IGXLResourceKind::ACSpecs,
            "cycle".to_string(),
            IndexMap::from([
                ("specset".to_string(), vec!["functional".to_string()]),
                ("selector".to_string(), vec!["nom".to_string()]),
                ("typ".to_string(), vec!["10*ns".to_string()]),
                ("min".to_string(), vec![String::new()]),
                ("max".to_string(), vec![String::new()]),
            ]),
        ),
        (
            "SpecsAC".to_string(),
            IGXLResourceKind::ACSpecs,
            "cycle".to_string(),
            IndexMap::from([
                ("specset".to_string(), vec!["scan".to_string()]),
                ("selector".to_string(), vec!["nom".to_string()]),
                ("typ".to_string(), vec!["10*ns".to_string()]),
                ("min".to_string(), vec!["9*ns".to_string()]),
                ("max".to_string(), vec!["11*ns".to_string()]),
            ]),
        ),
    ];
    let dir = tempdir()?;
    let error = ResourceGenerator::write_specs(
        &generator.resources_rows,
        &dir.path().join("SpecsAC.txt"),
        IGXLResourceKind::ACSpecs,
        "AC",
    )
    .unwrap_err();
    assert!(error.to_string().contains("inconsistent categories"));

    let error = ResourceGenerator::new(generator.resources_rows.clone())
        .render(dir.path())
        .unwrap_err();
    assert!(error.to_string().contains("inconsistent categories"));
    let leftover_parts = std::fs::read_dir(dir.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".origen_uflex_")
        })
        .collect::<Vec<_>>();
    assert!(leftover_parts.is_empty());
    Ok(())
}

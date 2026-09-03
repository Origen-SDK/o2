use super::nodes::{AccessLinkStandard, MuxType, PortType, SignalType, ICL};
use super::ParseOptions;
use crate::ast::{Node, AST};
use crate::{Error, Result};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[grammar = "ijtag/icl/icl.pest"]
pub(super) struct ICLParser;

pub(crate) fn parse_file_with_options(path: &Path, options: ParseOptions) -> Result<Node<ICL>> {
    if !path.exists() {
        return Err(Error::new(&format!(
            "File does not exist: {}",
            path.display()
        )));
    }

    let contents = fs::read_to_string(path)?;
    parse_str_with_options(&contents, Some(&path.display().to_string()), options).map_err(|e| {
        let display_path = path
            .canonicalize()
            .unwrap_or_else(|_| path.to_path_buf())
            .display()
            .to_string();
        Error::new(&format!("Error parsing file {}:\n{}", display_path, e.msg))
    })
}

pub(crate) fn parse_str_with_options(
    icl: &str,
    source_file: Option<&str>,
    options: ParseOptions,
) -> Result<Node<ICL>> {
    match ICLParser::parse(Rule::icl_source, icl) {
        Err(e) => Err(Error::new(&e.to_string())),
        Ok(mut parsed) => to_ast(parsed.next().unwrap(), source_file, options),
    }
}

enum Action {
    Open(ICL),
    Leaf(ICL),
    Transparent,
}

fn action(pair: &Pair<'_, Rule>, options: ParseOptions) -> Action {
    let text = pair.as_str();
    let open = match pair.as_rule() {
        Rule::namespace_def => ICL::NameSpace,
        Rule::use_namespace_def => ICL::UseNameSpace,
        Rule::module_def => ICL::Module,

        Rule::scan_in_port_def => ICL::Port(PortType::ScanIn),
        Rule::scan_out_port_def => ICL::Port(PortType::ScanOut),
        Rule::shift_en_port_def => ICL::Port(PortType::ShiftEn),
        Rule::capture_en_port_def => ICL::Port(PortType::CaptureEn),
        Rule::update_en_port_def => ICL::Port(PortType::UpdateEn),
        Rule::data_in_port_def => ICL::Port(PortType::DataIn),
        Rule::data_out_port_def => ICL::Port(PortType::DataOut),
        Rule::to_shift_en_port_def => ICL::Port(PortType::ToShiftEn),
        Rule::to_update_en_port_def => ICL::Port(PortType::ToUpdateEn),
        Rule::to_capture_en_port_def => ICL::Port(PortType::ToCaptureEn),
        Rule::select_port_def => ICL::Port(PortType::Select),
        Rule::to_select_port_def => ICL::Port(PortType::ToSelect),
        Rule::reset_port_def => ICL::Port(PortType::Reset),
        Rule::to_reset_port_def => ICL::Port(PortType::ToReset),
        Rule::tms_port_def => ICL::Port(PortType::Tms),
        Rule::to_tms_port_def => ICL::Port(PortType::ToTms),
        Rule::tck_port_def => ICL::Port(PortType::Tck),
        Rule::to_tck_port_def => ICL::Port(PortType::ToTck),
        Rule::clock_port_def => ICL::Port(PortType::Clock),
        Rule::to_clock_port_def => ICL::Port(PortType::ToClock),
        Rule::trst_port_def => ICL::Port(PortType::Trst),
        Rule::to_trst_port_def => ICL::Port(PortType::ToTrst),
        Rule::to_ir_select_port_def => ICL::Port(PortType::ToIrSelect),
        Rule::address_port_def => ICL::Port(PortType::Address),
        Rule::write_en_port_def => ICL::Port(PortType::WriteEn),
        Rule::read_en_port_def => ICL::Port(PortType::ReadEn),

        Rule::instance_def => ICL::Instance,
        Rule::module_reference => ICL::ModuleReference,
        Rule::scan_register_def => ICL::ScanRegister,
        Rule::data_register_def => ICL::DataRegister,
        Rule::logic_signal_def => ICL::LogicSignal,
        Rule::scan_mux_def => ICL::Mux(MuxType::Scan),
        Rule::data_mux_def => ICL::Mux(MuxType::Data),
        Rule::clock_mux_def => ICL::Mux(MuxType::Clock),
        Rule::scan_mux_selection => ICL::MuxSelection(MuxType::Scan),
        Rule::data_mux_selection => ICL::MuxSelection(MuxType::Data),
        Rule::clock_mux_selection => ICL::MuxSelection(MuxType::Clock),
        Rule::one_hot_scan_group_def => ICL::OneHotScanGroup,
        Rule::one_hot_data_group_def => ICL::OneHotDataGroup,
        Rule::scan_interface_def => ICL::ScanInterface,
        Rule::scan_interface_chain_def => ICL::ScanInterfaceChain,
        Rule::access_link_1149_def => {
            let standard = pair
                .clone()
                .into_inner()
                .find(|child| child.as_rule() == Rule::access_link_standard)
                .expect("validated access link must contain its standard");
            if standard.as_str() == "STD_1149_1_2001" {
                ICL::AccessLink(AccessLinkStandard::Std1149_1_2001)
            } else {
                ICL::AccessLink(AccessLinkStandard::Std1149_1_2013)
            }
        }
        Rule::access_link_generic_def => ICL::GenericAccessLink,
        Rule::bsdl_instruction => ICL::BsdlInstruction,
        Rule::bsdl_entity_def => ICL::BsdlEntity,
        Rule::bsdl_selection => {
            if text.trim_start().starts_with("ScanInterface") {
                ICL::ScanInterfaces
            } else {
                ICL::ActiveSignals
            }
        }
        Rule::access_link_scan_interface_name => ICL::ScanInterfaceReference,
        Rule::access_link_active_signal_name => ICL::PortReference,
        Rule::alias_def => ICL::Alias,
        Rule::enum_def => ICL::Enumeration,
        Rule::enum_item => ICL::EnumerationItem,
        Rule::parameter_def => ICL::Parameter,
        Rule::local_parameter_def => ICL::LocalParameter,
        Rule::attribute_def => ICL::Attribute,

        Rule::source_def => ICL::Source,
        Rule::enable_def => ICL::Enable,
        Rule::ref_enum_def => ICL::RefEnum,
        Rule::default_load_value_def => ICL::DefaultLoadValue,
        Rule::active_polarity_def => {
            let polarity = pair
                .clone()
                .into_inner()
                .find(|child| child.as_rule() == Rule::polarity_value)
                .expect("validated polarity must contain a value");
            ICL::ActivePolarity(polarity.as_str() == "1")
        }
        Rule::differential_inv_of_def => ICL::DifferentialInvOf,
        Rule::frequency_multiplier_def => ICL::FrequencyMultiplier,
        Rule::frequency_divider_def => ICL::FrequencyDivider,
        Rule::period_def => ICL::Period,
        Rule::input_port_connection => ICL::InputPortConnection,
        Rule::allow_broadcast_def => ICL::AllowBroadcastOnScanInterface,
        Rule::instance_address_value | Rule::data_register_address_value => ICL::AddressValue,
        Rule::scan_in_source_def => ICL::ScanInSource,
        Rule::capture_source_def => ICL::CaptureSource,
        Rule::reset_value_def => ICL::ResetValue,
        Rule::write_en_source_def => ICL::WriteEnSource,
        Rule::write_data_source_def => ICL::WriteDataSource,
        Rule::read_callback_def => ICL::ReadCallBack,
        Rule::read_data_source_def => ICL::ReadDataSource,
        Rule::write_callback_def => ICL::WriteCallBack,
        Rule::iproc_reference => ICL::IProcReference,
        Rule::one_hot_scan_group_item | Rule::one_hot_data_group_port_source => ICL::PortReference,
        Rule::scan_interface_port_def => ICL::PortReference,
        Rule::access_together_def => ICL::AccessTogether,
        Rule::apply_end_state_def => ICL::ApplyEndState,

        Rule::vector_id => ICL::VectorIdentifier,
        Rule::hier_port => ICL::HierarchicalIdentifier,
        Rule::index => ICL::Index,
        Rule::range => ICL::Range,
        Rule::hier_data_signal => ICL::Signal(SignalType::HierarchicalData),
        Rule::reset_signal => ICL::Signal(SignalType::Reset),
        Rule::scan_signal => ICL::Signal(SignalType::Scan),
        Rule::data_signal => ICL::Signal(SignalType::Data),
        Rule::clock_signal => ICL::Signal(SignalType::Clock),
        Rule::tck_signal => ICL::Signal(SignalType::Tck),
        Rule::tms_signal => ICL::Signal(SignalType::Tms),
        Rule::trst_signal => ICL::Signal(SignalType::Trst),
        Rule::shift_en_signal => ICL::Signal(SignalType::ShiftEn),
        Rule::capture_en_signal => ICL::Signal(SignalType::CaptureEn),
        Rule::update_en_signal => ICL::Signal(SignalType::UpdateEn),
        Rule::concat_signal
        | Rule::concat_hier_data_signal
        | Rule::concat_number
        | Rule::concat_string => ICL::Concatenation,
        Rule::concat_number_list => ICL::Alternatives,

        Rule::integer_expr => ICL::IntegerExpression,
        Rule::integer_term => ICL::IntegerTerm,
        Rule::integer_paren | Rule::logic_paren => ICL::Parentheses,
        Rule::logic_expr | Rule::logic_bool_expr => ICL::LogicExpression,
        Rule::logic_bitwise_expr => ICL::LogicBitwiseExpression,
        Rule::logic_equality_expr => ICL::LogicEqualityExpression,
        Rule::logic_concat_expr => ICL::LogicConcatenation,
        _ => return leaf_or_transparent(pair, options),
    };
    Action::Open(open)
}

fn leaf_or_transparent(pair: &Pair<'_, Rule>, options: ParseOptions) -> Action {
    let text = pair.as_str();
    match pair.as_rule() {
        Rule::scalar_id => Action::Leaf(ICL::Identifier(text.to_string())),
        Rule::parameter_ref => Action::Leaf(ICL::ParameterReference(text.to_string())),
        Rule::string => Action::Leaf(ICL::StringLiteral(text.to_string())),
        Rule::sized_bin_number
        | Rule::sized_dec_number
        | Rule::sized_hex_number
        | Rule::unsized_bin_number
        | Rule::unsized_dec_number
        | Rule::unsized_hex_number
        | Rule::pos_int => Action::Leaf(ICL::Number(text.to_string())),
        Rule::time_unit => Action::Leaf(ICL::TimeUnit(text.to_string())),
        Rule::iproc_argument => Action::Leaf(ICL::IProcArgument(text.to_string())),
        Rule::generic_access_link_block => {
            Action::Leaf(ICL::GenericAccessLinkBody(text.to_string()))
        }
        Rule::invert => Action::Leaf(ICL::Invert),
        Rule::integer_add_op => {
            if text == "+" {
                Action::Leaf(ICL::Add)
            } else {
                Action::Leaf(ICL::Subtract)
            }
        }
        Rule::integer_mul_op => match text {
            "*" => Action::Leaf(ICL::Multiply),
            "/" => Action::Leaf(ICL::Divide),
            _ => Action::Leaf(ICL::Modulo),
        },
        Rule::logic_bool_op => {
            if text == "&&" {
                Action::Leaf(ICL::BooleanAnd)
            } else {
                Action::Leaf(ICL::BooleanOr)
            }
        }
        Rule::logic_bitwise_op => match text {
            "&" => Action::Leaf(ICL::BitwiseAnd),
            "|" => Action::Leaf(ICL::BitwiseOr),
            _ => Action::Leaf(ICL::BitwiseXor),
        },
        Rule::logic_equality_op => {
            if text == "==" {
                Action::Leaf(ICL::Equal)
            } else {
                Action::Leaf(ICL::NotEqual)
            }
        }
        Rule::logic_unary_op => {
            if text == "!" {
                Action::Leaf(ICL::BooleanNot)
            } else {
                Action::Leaf(ICL::Invert)
            }
        }
        Rule::logic_reduction_op => match text {
            "&" => Action::Leaf(ICL::BitwiseAnd),
            "|" => Action::Leaf(ICL::BitwiseOr),
            _ => Action::Leaf(ICL::BitwiseXor),
        },
        Rule::COMMENT if options.preserve_comments => Action::Leaf(ICL::Comment(text.to_string())),
        _ => Action::Transparent,
    }
}

fn to_ast(
    pair: Pair<'_, Rule>,
    source_file: Option<&str>,
    options: ParseOptions,
) -> Result<Node<ICL>> {
    debug_assert_eq!(pair.as_rule(), Rule::icl_source);
    let mut ast = AST::new();
    ast.push_and_open(node!(ICL::Root));
    if let Some(file) = source_file {
        ast.push(node!(ICL::SourceFile, file.to_string()));
    }

    let mut frames: Vec<(Pairs<'_, Rule>, Option<usize>)> = vec![(pair.into_inner(), None)];
    while let Some((pairs, close_id)) = frames.last_mut() {
        if let Some(next) = pairs.next() {
            match action(&next, options) {
                Action::Leaf(attrs) => ast.push(Node::new(attrs)),
                Action::Open(attrs) => {
                    let id = ast.push_and_open(Node::new(attrs));
                    frames.push((next.into_inner(), Some(id)));
                }
                Action::Transparent => {
                    let inner = next.into_inner();
                    if inner.peek().is_some() {
                        frames.push((inner, None));
                    }
                }
            }
        } else {
            let close_id = *close_id;
            frames.pop();
            if let Some(id) = close_id {
                ast.close(id)?;
            }
        }
    }

    Ok(ast.unwrap())
}

#[cfg(test)]
mod tests {
    use super::super::{from_str, Parser as ConfigurableParser};
    use super::*;

    fn count(node: &Node<ICL>, predicate: &dyn Fn(&ICL) -> bool) -> usize {
        usize::from(predicate(&node.attrs))
            + node
                .children
                .iter()
                .map(|child| count(child, predicate))
                .sum::<usize>()
    }

    fn contains(node: &Node<ICL>, predicate: &dyn Fn(&ICL) -> bool) -> bool {
        predicate(&node.attrs) || node.children.iter().any(|child| contains(child, predicate))
    }

    #[test]
    fn parses_representative_network_and_builds_typed_ast() {
        let source = r#"
            NameSpace Demo;
            UseNameSpace Demo;
            Module Module {
                Parameter WIDTH = 8;
                LocalParameter RESET = 8'b0000_xx11;
                ScanInPort ScanInPort;
                ScanOutPort SO { Source shift_reg; Enable enabled; }
                DataInPort DIN[7:0] { DefaultLoadValue 'h 3f; RefEnum States; }
                DataOutPort DOUT[7:0] { Source shift_reg[7:0]; RefEnum States; }
                ResetPort reset_n { ActivePolarity 0; }
                ClockPort clock;
                ScanRegister shift_reg[$WIDTH-1:0] {
                    ScanInSource ScanInPort;
                    CaptureSource DIN[7:0];
                    ResetValue $WIDTH'h xx;
                    DefaultLoadValue 8'b0;
                    RefEnum States;
                    Attribute Purpose = "dummy \"value\"";
                }
                LogicSignal enabled { !(shift_reg[0] == 1'b0) || DIN[0] & DIN[1]; }
                ScanMux scan_path SelectedBy enabled {
                    Attribute VerificationHint = "dummy";
                    1'b0: ScanInPort;
                    1'b1: shift_reg;
                }
                Enum States { Idle = 8'h00; Busy = 8'hA5; }
                Alias status = child.DOUT[0] { AccessTogether; iApplyEndState 1'b0; RefEnum States; }
                Instance child Of Demo::Peripheral {
                    InputPort input = DIN[7:0];
                    Parameter WIDTH = 8;
                    AddressValue 'h 2a;
                }
                ScanInterface client {
                    Port ScanInPort;
                    Port SO;
                    Chain data { Port SO; DefaultLoadValue 1'b0; }
                }
            }
        "#;

        let ast = from_str(source).expect("representative dummy ICL should parse");
        assert_eq!(count(&ast, &|n| matches!(n, ICL::Module)), 1);
        assert_eq!(count(&ast, &|n| matches!(n, ICL::ScanRegister)), 1);
        assert_eq!(count(&ast, &|n| matches!(n, ICL::Mux(MuxType::Scan))), 1);
        assert!(contains(&ast, &|n| matches!(n, ICL::Range)));
        assert!(contains(&ast, &|n| matches!(n, ICL::BooleanOr)));
        assert!(contains(
            &ast,
            &|n| matches!(n, ICL::Number(v) if v == "$WIDTH'h xx")
        ));
        assert!(contains(
            &ast,
            &|n| matches!(n, ICL::StringLiteral(v) if v.contains("\\\"value\\\""))
        ));
    }

    #[test]
    fn parses_every_port_function() {
        let source = r#"
            Module Ports {
                ScanInPort a; ScanOutPort b; ShiftEnPort c; CaptureEnPort d;
                UpdateEnPort e; DataInPort f; DataOutPort g; ToShiftEnPort h;
                ToUpdateEnPort i; ToCaptureEnPort j; SelectPort k; ToSelectPort l;
                ResetPort m; ToResetPort n; TMSPort o; ToTMSPort p;
                TCKPort q; ToTCKPort r; ClockPort s; ToClockPort t;
                TRSTPort u; ToTRSTPort v; ToIRSelectPort w; AddressPort x;
                WriteEnPort y; ReadEnPort z;
            }
        "#;
        let ast = from_str(source).expect("all standard port functions should parse");
        assert_eq!(count(&ast, &|n| matches!(n, ICL::Port(_))), 26);
    }

    #[test]
    fn parses_port_properties_and_empty_block_terminators() {
        let source = r#"
            Module PortProperties {
                ResetPort reset { ActivePolarity 1; Attribute Kind; }
                ToResetPort reset_out { Source ~reset, reset_data; ActivePolarity /* dummy 1 */ 0; }
                ToCaptureEnPort capture_out { Source capture; }
                ToUpdateEnPort update_out { Source update; }
                ClockPort clock_n { DifferentialInvOf clock_p; }
                ToClockPort generated {
                    Source clock_p;
                    FreqMultiplier 2;
                    FreqDivider 4;
                    DifferentialInvOf clock_n;
                    Period 10ns;
                }
                ToTCKPort forwarded_tck { Source tck; }
                Attribute GeneratedPath = "dummy/\\GENERATED_BLOCK[0]";
                ToSelectPort empty_select {}
                DataRegister empty_register {}
                Alias empty_alias = generated {}
            }
        "#;

        let ast = from_str(source).expect("dummy port properties should parse");
        assert!(contains(&ast, &|n| matches!(n, ICL::ActivePolarity(true))));
        assert!(contains(&ast, &|n| matches!(n, ICL::ActivePolarity(false))));
        assert!(contains(&ast, &|n| matches!(n, ICL::FrequencyMultiplier)));
        assert!(contains(
            &ast,
            &|n| matches!(n, ICL::TimeUnit(unit) if unit == "ns")
        ));
        assert_eq!(count(&ast, &|n| matches!(n, ICL::Source)), 5);
    }

    #[test]
    fn parses_register_callbacks_groups_and_access_links() {
        let source = r#"
            Module Features {
                DataInPort data[3:0]; DataOutPort result[3:0];
                AddressPort address; WriteEnPort write; ReadEnPort read;
                DataRegister register[3:0] {
                    WriteEnSource write;
                    WriteDataSource data[3:0];
                    AddressValue 4'h3;
                    ReadCallBack ::Features::Callbacks read_value <R> 4'h1 "arg";
                    ReadDataSource result[3:0];
                    WriteCallBack Vendor::Features write_value <D> $ARG;
                    ResetValue 4'b0;
                }
                DataMux selected SelectedBy address { 1'b0: data; 1'b1: result; }
                ClockMux clocks SelectedBy address { 1'b0: clock_a; 1'b1: clock_b; }
                OneHotScanGroup scans { Port scan_a; Port scan_b; }
                OneHotDataGroup values { Port data; DataRegister local; }
            }
            Module Links {
                AccessLink boundary Of STD_1149_1_2013 {
                    BSDLEntity DummyEntity;
                    SAMPLE { ScanInterface { target.client; } ActiveSignals { enable; } }
                }
                AccessLink vendor Of DUMMY_PROTOCOL { command "dummy string" { nested body } }
            }
        "#;
        let ast = from_str(source).expect("advanced dummy constructs should parse");
        assert!(contains(&ast, &|n| matches!(n, ICL::ReadCallBack)));
        assert!(contains(&ast, &|n| matches!(n, ICL::OneHotDataGroup)));
        assert!(contains(&ast, &|n| matches!(
            n,
            ICL::AccessLink(AccessLinkStandard::Std1149_1_2013)
        )));
        assert!(contains(
            &ast,
            &|n| matches!(n, ICL::GenericAccessLinkBody(v) if v.contains("nested body"))
        ));
    }

    #[test]
    fn comments_are_optional_and_lossless() {
        let source =
            "// leading dummy comment\nModule Dummy { /* inner dummy comment */ ScanInPort si; }\n";
        let without = from_str(source).unwrap();
        assert_eq!(count(&without, &|n| matches!(n, ICL::Comment(_))), 0);

        let with = ConfigurableParser::new()
            .preserve_comments()
            .from_str(source)
            .unwrap();
        assert_eq!(count(&with, &|n| matches!(n, ICL::Comment(_))), 2);
        assert!(contains(
            &with,
            &|n| matches!(n, ICL::Comment(v) if v == "// leading dummy comment\n")
        ));
        assert!(contains(
            &with,
            &|n| matches!(n, ICL::Comment(v) if v == "/* inner dummy comment */")
        ));
    }

    #[test]
    fn rejects_non_standard_or_malformed_input() {
        for source in [
            "#include \"dummy.icl\"",
            "iProcsForModule Dummy",
            "module WrongCase {}",
            "Module _Bad {}",
            "Module Bad { ScanRegister r { ResetValue 2'b2; } }",
            "Module Bad { ScanInPort missing_terminator }",
            "iProcsForModule Dummy",
        ] {
            assert!(from_str(source).is_err(), "unexpectedly parsed: {source}");
        }
    }

    #[test]
    fn file_api_records_the_source_and_reports_missing_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dummy.icl");
        fs::write(&path, "Module Dummy { ScanInPort si; }").unwrap();

        let ast = super::super::from_file(&path).unwrap();
        assert!(matches!(
            ast.children.first().map(|n| &n.attrs),
            Some(ICL::SourceFile(file)) if file == &path.display().to_string()
        ));
        assert!(super::super::from_file(&dir.path().join("missing.icl")).is_err());
    }

    #[test]
    fn parses_generated_dummy_scale_sample() {
        let mut source = String::from("Module Scale { ScanInPort si;\n");
        for i in 0..2_000 {
            source.push_str(&format!(
                "Instance child_{i} Of Dummy {{ InputPort input = si; Parameter WIDTH = 8; }}\n"
            ));
        }
        source.push_str("}\nModule Dummy { ScanInPort input; Parameter WIDTH = 8; }\n");

        let ast = from_str(&source).expect("generated dummy scale input should parse");
        assert_eq!(count(&ast, &|n| matches!(n, ICL::Instance)), 2_000);
    }
}

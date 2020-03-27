use crate::generator::ast::*;
use crate::node;
use crate::{Error, Result};
use pest::iterators::Pair;
use pest::Parser;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[grammar = "generator/stil/stil.pest"]
pub struct STILParser;

pub fn parse_file(path: &Path) -> Result<Node> {
    if path.exists() {
        match parse_str(&fs::read_to_string(path)?) {
            Ok(n) => Ok(n),
            Err(e) => Err(Error::new(&format!(
                "Error parsing file {}:\n{}",
                path.canonicalize()?.display(),
                e.msg
            ))),
        }
    } else {
        Err(Error::new(&format!(
            "File does not exist: {}",
            path.display()
        )))
    }
}

pub fn parse_str(stil: &str) -> Result<Node> {
    match STILParser::parse(Rule::stil_source, stil) {
        Err(e) => Err(Error::new(&format!("{}", e))),
        Ok(mut stil) => Ok(to_ast(stil.next().unwrap())?.unwrap()),
    }
}

fn inner_strs(pair: Pair<Rule>) -> Vec<&str> {
    pair.into_inner().map(|v| v.as_str()).collect()
}

fn unquote(text: &str) -> String {
    let first = text.chars().next().unwrap();

    if first != '"' && first != '\'' && first != '’' {
        text.to_string()
    } else if text.chars().last().unwrap() != first {
        text.to_string()
    } else {
        text[1..text.len() - 1].to_string()
    }
}

// This is the main function responsible for transforming the parsed strings into an AST
fn to_ast(pair: Pair<Rule>) -> Result<AST> {
    let mut ast = AST::new(node!(STIL));
    let mut ids: Vec<usize> = vec![];
    let mut pairs = vec![pair.into_inner()];
    loop {
        let pair;
        loop {
            match pairs.last_mut().unwrap().next() {
                Some(p) => {
                    pair = p;
                    break;
                }
                None => {
                    pairs.pop();
                    if pairs.len() > 0 {
                        let id = ids.pop().unwrap();
                        if id != 0 {
                            ast.close(id)?;
                        }
                    } else {
                        return Ok(ast);
                    }
                }
            }
        }

        match pair.as_rule() {
            Rule::stil_source => {
                ids.push(ast.push_and_open(node!(STIL)));
                pairs.push(pair.into_inner());
            }
            Rule::stil_version => {
                let vals = inner_strs(pair);
                ast.push(node!(
                    STILVersion,
                    vals[0].parse().unwrap(),
                    vals[1].parse().unwrap()
                ));
            }
            Rule::label => ast.push(node!(STILLabel, unquote(inner_strs(pair)[0]))),
            Rule::header_block => {
                ids.push(ast.push_and_open(node!(STILHeader)));
                pairs.push(pair.into_inner());
            }
            Rule::title => ast.push(node!(STILTitle, unquote(inner_strs(pair)[0]))),
            Rule::date => ast.push(node!(STILDate, unquote(inner_strs(pair)[0]))),
            Rule::source => ast.push(node!(STILSource, unquote(inner_strs(pair)[0]))),
            Rule::history => {
                ids.push(ast.push_and_open(node!(STILHistory)));
                pairs.push(pair.into_inner());
            }
            Rule::annotation => ast.push(node!(STILAnnotation, inner_strs(pair)[0].to_string())),
            Rule::include => {
                let vals = inner_strs(pair);
                if vals.len() == 1 {
                    ast.push(node!(STILInclude, unquote(vals[0]), None))
                } else {
                    ast.push(node!(
                        STILInclude,
                        unquote(vals[0]),
                        Some(vals[1].to_string())
                    ))
                }
            }
            Rule::signals_block => {
                ids.push(ast.push_and_open(node!(STILSignals)));
                pairs.push(pair.into_inner());
            }
            Rule::signal => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                let v2 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    STILSignal,
                    v1.parse().unwrap(),
                    v2.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::termination => {
                ast.push(node!(STILTermination, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::default_state => ast.push(node!(
                STILDefaultState,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::base => {
                let vals = inner_strs(pair);
                ast.push(node!(
                    STILBase,
                    vals[0].parse().unwrap(),
                    vals[1].parse().unwrap()
                ))
            }
            Rule::alignment => ast.push(node!(STILAlignment, inner_strs(pair)[0].parse().unwrap())),
            Rule::scan_in => ast.push(node!(STILScanIn, inner_strs(pair)[0].parse().unwrap())),
            Rule::scan_out => ast.push(node!(STILScanOut, inner_strs(pair)[0].parse().unwrap())),
            Rule::data_bit_count => ast.push(node!(
                STILDataBitCount,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::signal_groups_block => {
                let mut p = pair.into_inner();
                let n;
                if let Some(nxt) = p.peek() {
                    n = match nxt.as_rule() {
                        Rule::name => node!(
                            STILSignalGroups,
                            Some(p.next().unwrap().as_str().to_string())
                        ),
                        _ => node!(STILSignalGroups, None),
                    };
                } else {
                    n = node!(STILSignalGroups, None);
                }
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::signal_group => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STILSignalGroup, unquote(p.next().unwrap().as_str()))),
                );
                pairs.push(p);
            }
            Rule::sigref_expr => {
                ids.push(ast.push_and_open(node!(STILSigRefExpr)));
                pairs.push(pair.into_inner());
            }
            Rule::name => ast.push(node!(STILName, unquote(pair.as_str()))),
            Rule::expression | Rule::expression_subset => {
                ids.push(ast.push_and_open(node!(STILExpr)));
                pairs.push(pair.into_inner());
            }
            Rule::time_expr => {
                ids.push(ast.push_and_open(node!(STILTimeExpr)));
                pairs.push(pair.into_inner());
            }
            Rule::add => {
                ids.push(ast.push_and_open(node!(STILAdd)));
                pairs.push(pair.into_inner());
            }
            Rule::subtract => {
                ids.push(ast.push_and_open(node!(STILSubtract)));
                pairs.push(pair.into_inner());
            }
            Rule::multiply => {
                ids.push(ast.push_and_open(node!(STILMultiply)));
                pairs.push(pair.into_inner());
            }
            Rule::divide => {
                ids.push(ast.push_and_open(node!(STILDivide)));
                pairs.push(pair.into_inner());
            }
            Rule::paren_expression => {
                ids.push(ast.push_and_open(node!(STILParens)));
                pairs.push(pair.into_inner());
            }
            Rule::terminal => {
                ids.push(0);
                pairs.push(pair.into_inner());
            }
            Rule::number => {
                ids.push(ast.push_and_open(node!(STILNumber)));
                pairs.push(pair.into_inner());
            }
            Rule::number_with_unit => {
                ids.push(ast.push_and_open(node!(STILNumberWithUnit)));
                pairs.push(pair.into_inner());
            }
            Rule::si_unit => ast.push(node!(STILSIUnit, pair.as_str().parse().unwrap())),
            Rule::engineering_prefix => {
                ast.push(node!(STILEngPrefix, pair.as_str().parse().unwrap()))
            }
            Rule::integer => ast.push(node!(STILInteger, pair.as_str().parse().unwrap())),
            Rule::signed_integer => {
                ast.push(node!(STILSignedInteger, pair.as_str().parse().unwrap()))
            }
            Rule::point => ast.push(node!(STILPoint)),
            Rule::exponential => ast.push(node!(STILExp)),
            Rule::minus => ast.push(node!(STILMinus)),
            Rule::pattern_exec_block => {
                let mut p = pair.into_inner();
                let n;
                if let Some(nxt) = p.peek() {
                    n = match nxt.as_rule() {
                        Rule::name => {
                            node!(STILPatternExec, Some(unquote(p.next().unwrap().as_str())))
                        }
                        _ => node!(STILPatternExec, None),
                    };
                } else {
                    n = node!(STILPatternExec, None);
                }
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::category => {
                ast.push(node!(STILCategoryRef, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::selector => {
                ast.push(node!(STILSelectorRef, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::timing => ast.push(node!(STILTimingRef, unquote(inner_strs(pair)[0]))),
            Rule::pattern_burst => {
                ast.push(node!(STILPatternBurstRef, unquote(inner_strs(pair)[0])))
            }
            Rule::pattern_burst_block => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STILPatternBurst, unquote(p.next().unwrap().as_str()))),
                );
                pairs.push(p);
            }
            Rule::signal_groups => ast.push(node!(
                STILSignalGroupsRef,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::macro_defs => {
                ast.push(node!(STILMacroDefs, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::procedures => {
                ast.push(node!(STILProcedures, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::scan_structures => ast.push(node!(
                STILScanStructuresRef,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::start => ast.push(node!(STILStart, inner_strs(pair)[0].parse().unwrap())),
            Rule::stop => ast.push(node!(STILStop, inner_strs(pair)[0].parse().unwrap())),
            Rule::termination_block => {
                ids.push(ast.push_and_open(node!(STILTerminations)));
                pairs.push(pair.into_inner());
            }
            Rule::termination_item => {
                ids.push(ast.push_and_open(node!(STILTerminationItem)));
                pairs.push(pair.into_inner());
            }
            Rule::pat_list => {
                ids.push(ast.push_and_open(node!(STILPatList)));
                pairs.push(pair.into_inner());
            }
            Rule::pat_list_item => {
                let mut p = pair.into_inner();
                ids.push(ast.push_and_open(node!(STILPat, unquote(p.next().unwrap().as_str()))));
                pairs.push(p);
            }
            Rule::timing_block => {
                let mut p = pair.into_inner();
                let n;
                if let Some(nxt) = p.peek() {
                    n = match nxt.as_rule() {
                        Rule::name => node!(STILTiming, Some(unquote(p.next().unwrap().as_str()))),
                        _ => node!(STILTiming, None),
                    };
                } else {
                    n = node!(STILTiming, None);
                }
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::waveform_table => {
                let mut p = pair.into_inner();
                ids.push(ast.push_and_open(node!(
                    STILWaveformTable,
                    unquote(p.next().unwrap().as_str())
                )));
                pairs.push(p);
            }
            Rule::period => {
                ids.push(ast.push_and_open(node!(STILPeriod)));
                pairs.push(pair.into_inner());
            }
            Rule::inherit_waveform_table | Rule::inherit_waveform | Rule::inherit_waveform_wfc => {
                ast.push(node!(STILInherit, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::sub_waveforms => {
                ids.push(ast.push_and_open(node!(STILSubWaveforms)));
                pairs.push(pair.into_inner());
            }
            Rule::sub_waveform => {
                ids.push(ast.push_and_open(node!(STILSubWaveform)));
                pairs.push(pair.into_inner());
            }
            Rule::waveforms => {
                ids.push(ast.push_and_open(node!(STILWaveforms)));
                pairs.push(pair.into_inner());
            }
            Rule::waveform => {
                ids.push(ast.push_and_open(node!(STILWaveform)));
                pairs.push(pair.into_inner());
            }
            Rule::wfc_definition => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STILWFChar, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::event => {
                ids.push(ast.push_and_open(node!(STILEvent)));
                pairs.push(pair.into_inner());
            }
            Rule::event_list => {
                let mut vals: Vec<char> = Vec::new();
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::event_char => vals.push(inner_pair.as_str().parse().unwrap()),
                        _ => unreachable!(),
                    };
                }
                ast.push(node!(STILEventList, vals))
            }
            Rule::spec_block => {
                let mut p = pair.into_inner();
                let n;
                if let Some(nxt) = p.peek() {
                    n = match nxt.as_rule() {
                        Rule::name => node!(STILSpec, Some(unquote(p.next().unwrap().as_str()))),
                        _ => node!(STILSpec, None),
                    };
                } else {
                    n = node!(STILSpec, None);
                }
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::category_block => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STILCategory, unquote(p.next().unwrap().as_str()))),
                );
                pairs.push(p);
            }
            Rule::spec_item => {
                ids.push(ast.push_and_open(node!(STILSpecItem)));
                pairs.push(pair.into_inner());
            }
            Rule::typical_var => {
                let mut p = pair.into_inner();
                ids.push(ast.push_and_open(node!(
                    STILTypicalVar,
                    p.next().unwrap().as_str().to_string()
                )));
                pairs.push(p);
            }
            Rule::spec_var => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STILSpecVar, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::spec_var_item => {
                let mut p = pair.into_inner();
                ids.push(ast.push_and_open(node!(
                    STILSpecVarItem,
                    p.next().unwrap().as_str().parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::variable_block => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STILVariable, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::selector_block => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STILSelector, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::selector_item => {
                let strs: Vec<&str> = pair.into_inner().map(|v| v.as_str()).collect();
                ast.push(node!(
                    STILSelectorItem,
                    strs[0].parse().unwrap(),
                    strs[1].parse().unwrap()
                ))
            }
            Rule::scan_structures_block => {
                let mut p = pair.into_inner();
                let n;
                if let Some(nxt) = p.peek() {
                    n = match nxt.as_rule() {
                        Rule::name => node!(
                            STILScanStructures,
                            Some(unquote(p.next().unwrap().as_str()))
                        ),
                        _ => node!(STILScanStructures, None),
                    };
                } else {
                    n = node!(STILScanStructures, None);
                }
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::scan_chain => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STILScanChain, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::scan_in_name => ast.push(node!(STILScanInName, unquote(inner_strs(pair)[0]))),
            Rule::scan_out_name => ast.push(node!(STILScanOutName, unquote(inner_strs(pair)[0]))),
            Rule::scan_length => {
                ast.push(node!(STILScanLength, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::scan_out_length => ast.push(node!(
                STILScanOutLength,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::not => ast.push(node!(STILNot)),
            Rule::scan_cells => {
                ids.push(ast.push_and_open(node!(STILScanCells)));
                pairs.push(pair.into_inner());
            }
            Rule::scan_master_clock => {
                ids.push(ast.push_and_open(node!(STILScanMasterClock)));
                pairs.push(pair.into_inner());
            }
            Rule::scan_slave_clock => {
                ids.push(ast.push_and_open(node!(STILScanSlaveClock)));
                pairs.push(pair.into_inner());
            }
            Rule::scan_inversion => ast.push(node!(
                STILScanInversion,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::pattern_block => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STILPattern, unquote(p.next().unwrap().as_str()))),
                );
                pairs.push(p);
            }
            Rule::time_unit => {
                ids.push(ast.push_and_open(node!(STILTimeUnit)));
                pairs.push(pair.into_inner());
            }
            Rule::vector => {
                ids.push(ast.push_and_open(node!(STILVector)));
                pairs.push(pair.into_inner());
            }
            Rule::cyclized_data => {
                ids.push(ast.push_and_open(node!(STILCyclizedData)));
                pairs.push(pair.into_inner());
            }
            Rule::non_cyclized_data => {
                ids.push(ast.push_and_open(node!(STILNonCyclizedData)));
                pairs.push(pair.into_inner());
            }
            Rule::repeat => ast.push(node!(STILRepeat, inner_strs(pair)[0].parse().unwrap())),
            Rule::waveform_format => ast.push(node!(STILWaveformFormat)),
            Rule::pattern_statement => {
                ids.push(0);
                pairs.push(pair.into_inner());
            }
            Rule::hex_format => {
                let vals = inner_strs(pair);
                ast.push(if vals.len() == 0 {
                    node!(STILHexFormat, None)
                } else {
                    node!(STILHexFormat, Some(vals[0].to_string()))
                })
            }
            Rule::dec_format => {
                let vals = inner_strs(pair);
                ast.push(if vals.len() == 0 {
                    node!(STILDecFormat, None)
                } else {
                    node!(STILDecFormat, Some(vals[0].to_string()))
                })
            }
            Rule::data_string => ast.push(node!(STILData, pair.as_str().to_string())),
            Rule::vec_data => {
                ids.push(0);
                pairs.push(pair.into_inner());
            }
            Rule::time_value => {
                ast.push(node!(STILTimeValue, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::waveform_statement => {
                ast.push(node!(STILWaveformRef, unquote(inner_strs(pair)[0])))
            }
            Rule::condition => {
                ids.push(ast.push_and_open(node!(STILCondition)));
                pairs.push(pair.into_inner());
            }
            Rule::call => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STILCall, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::macro_statement => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STILMacro, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::loop_statement => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STILLoop, p.next().unwrap().as_str().parse().unwrap())),
                );
                pairs.push(p);
            }
            Rule::match_loop => {
                let mut p = pair.into_inner();
                let timeout = p.next().unwrap();
                let n = match timeout.as_rule() {
                    Rule::integer => node!(STILMatchLoop, Some(timeout.as_str().parse().unwrap())),
                    Rule::infinite => node!(STILMatchLoop, None),
                    _ => unreachable!(),
                };
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::goto => ast.push(node!(STILGoto, inner_strs(pair)[0].parse().unwrap())),
            Rule::breakpoint => {
                ids.push(ast.push_and_open(node!(STILBreakPoint)));
                pairs.push(pair.into_inner());
            }
            Rule::iddq => ast.push(node!(STILIDDQ)),
            Rule::stop_statement => ast.push(node!(STILStopStatement)),

            ////println!("********************* {:?}", pair);
            Rule::EOI => {}
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::STIL;
    use super::*;
    use std::path::Path;

    fn read(example: &str) -> String {
        fs::read_to_string(format!("../../example/vendor/stil/{}.stil", example))
            .expect("cannot read file")
    }

    #[test]
    fn test_example1_to_ast() {
        let _stil = STIL::from_file(Path::new("../../example/vendor/stil/example1.stil"))
            .expect("Imported example1");
        //println!("{:?}", _stil.ast);
    }

    #[test]
    fn test_example2_to_ast() {
        let _stil = STIL::from_file(Path::new("../../example/vendor/stil/example2.stil"))
            .expect("Imported example2");
        //println!("{:?}", _stil.ast);
    }

    #[test]
    fn test_example3_to_ast() {
        let _stil = STIL::from_file(Path::new("../../example/vendor/stil/example3.stil"))
            .expect("Imported example3");
        //println!("{:?}", _stil.ast);
    }

    #[test]
    fn test_example4_to_ast() {
        let _stil = STIL::from_file(Path::new("../../example/vendor/stil/example4.stil"))
            .expect("Imported example4");
        //println!("{:?}", _stil.ast);
    }

    #[test]
    fn test_example1_can_parse() {
        let txt = read("example1");
        match STILParser::parse(Rule::stil_source, &txt) {
            Ok(_res) => {} //println!("{:?}", res),
            Err(e) => {
                println!("{}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_example2_can_parse() {
        let txt = read("example2");
        match STILParser::parse(Rule::stil_source, &txt) {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_example3_can_parse() {
        let txt = read("example3");
        match STILParser::parse(Rule::stil_source, &txt) {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                assert_eq!(1, 0);
            }
        }
    }
}

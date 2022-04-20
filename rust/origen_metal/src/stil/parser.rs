use crate::{Error, Result};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::fs;
use std::path::Path;
use crate::ast::node::Node;
use crate::ast::ast::AST;
use super::nodes::STIL;

#[derive(Parser)]
#[grammar = "stil/stil.pest"]
pub struct STILParser;

pub fn parse_file(path: &Path) -> Result<Node<STIL>> {
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

pub fn parse_str(stil: &str) -> Result<Node<STIL>> {
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

fn build_expression(pair: Pair<Rule>) -> Result<Node<STIL>> {
    let mut pairs = pair.into_inner();
    let p2 = pairs.next().unwrap();
    let mut term = to_ast(p2)?.unwrap();
    let mut done = false;
    while !done {
        if let Some(next) = pairs.peek() {
            match next.as_rule() {
                Rule::add => {
                    pairs.next();
                    let next_term = to_ast(pairs.next().unwrap())?.unwrap();
                    let mut n = node!(STIL::Add);
                    n.add_child(term);
                    n.add_child(next_term);
                    term = n;
                }
                Rule::subtract => {
                    pairs.next();
                    let next_term = to_ast(pairs.next().unwrap())?.unwrap();
                    let mut n = node!(STIL::Subtract);
                    n.add_child(term);
                    n.add_child(next_term);
                    term = n;
                }
                _ => done = true,
            }
        } else {
            done = true;
        }
    }
    Ok(term)
}

fn build_term(pair: Pair<Rule>) -> Result<Node<STIL>> {
    let mut pairs = pair.into_inner();
    let mut term = to_ast(pairs.next().unwrap())?.unwrap();
    let mut done = false;
    while !done {
        if let Some(next) = pairs.peek() {
            match next.as_rule() {
                Rule::multiply => {
                    pairs.next();
                    let next_term = to_ast(pairs.next().unwrap())?.unwrap();
                    let mut n = node!(STIL::Multiply);
                    n.add_child(term);
                    n.add_child(next_term);
                    term = n;
                }
                Rule::divide => {
                    pairs.next();
                    let next_term = to_ast(pairs.next().unwrap())?.unwrap();
                    let mut n = node!(STIL::Divide);
                    n.add_child(term);
                    n.add_child(next_term);
                    term = n;
                }
                _ => done = true,
            }
        } else {
            done = true;
        }
    }
    Ok(term)
}

// This is the main function responsible for transforming the parsed strings into an AST
pub fn to_ast(mut pair: Pair<Rule>) -> Result<AST<STIL>> {
    let mut ast = AST::new();
    let mut ids: Vec<usize> = vec![];
    let mut pairs: Vec<Pairs<Rule>> = vec![];

    loop {
        match pair.as_rule() {
            Rule::stil_source => {
                ids.push(ast.push_and_open(node!(STIL::Root)));
                pairs.push(pair.into_inner());
            }
            Rule::stil_version => {
                let vals = inner_strs(pair);
                ast.push(node!(
                    STIL::Version,
                    vals[0].parse().unwrap(),
                    vals[1].parse().unwrap()
                ));
            }
            Rule::label => ast.push(node!(STIL::Label, unquote(inner_strs(pair)[0]))),
            Rule::header_block => {
                ids.push(ast.push_and_open(node!(STIL::Header)));
                pairs.push(pair.into_inner());
            }
            Rule::title => ast.push(node!(STIL::Title, unquote(inner_strs(pair)[0]))),
            Rule::date => ast.push(node!(STIL::Date, unquote(inner_strs(pair)[0]))),
            Rule::source => ast.push(node!(STIL::Source, unquote(inner_strs(pair)[0]))),
            Rule::history => {
                ids.push(ast.push_and_open(node!(STIL::History)));
                pairs.push(pair.into_inner());
            }
            Rule::annotation => ast.push(node!(STIL::Annotation, inner_strs(pair)[0].to_string())),
            Rule::include => {
                let vals = inner_strs(pair);
                if vals.len() == 1 {
                    ast.push(node!(STIL::Include, unquote(vals[0]), None))
                } else {
                    ast.push(node!(
                        STIL::Include,
                        unquote(vals[0]),
                        Some(vals[1].to_string())
                    ))
                }
            }
            Rule::signals_block => {
                ids.push(ast.push_and_open(node!(STIL::Signals)));
                pairs.push(pair.into_inner());
            }
            Rule::signal => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                let v2 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    STIL::Signal,
                    v1.parse().unwrap(),
                    v2.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::termination => {
                ast.push(node!(STIL::Termination, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::default_state => ast.push(node!(
                STIL::DefaultState,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::base => {
                let vals = inner_strs(pair);
                ast.push(node!(
                    STIL::Base,
                    vals[0].parse().unwrap(),
                    vals[1].parse().unwrap()
                ))
            }
            Rule::alignment => ast.push(node!(STIL::Alignment, inner_strs(pair)[0].parse().unwrap())),
            Rule::scan_in => ast.push(node!(STIL::ScanIn, inner_strs(pair)[0].parse().unwrap())),
            Rule::scan_out => ast.push(node!(STIL::ScanOut, inner_strs(pair)[0].parse().unwrap())),
            Rule::data_bit_count => ast.push(node!(
                STIL::DataBitCount,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::signal_groups_block => {
                let mut p = pair.into_inner();
                let n;
                if let Some(nxt) = p.peek() {
                    n = match nxt.as_rule() {
                        Rule::name => node!(
                            STIL::SignalGroups,
                            Some(p.next().unwrap().as_str().to_string())
                        ),
                        _ => node!(STIL::SignalGroups, None),
                    };
                } else {
                    n = node!(STIL::SignalGroups, None);
                }
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::signal_group => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STIL::SignalGroup, unquote(p.next().unwrap().as_str()))),
                );
                pairs.push(p);
            }
            Rule::sigref_expr => {
                ids.push(ast.push_and_open(node!(STIL::SigRefExpr)));
                pairs.push(pair.into_inner());
            }
            Rule::name => ast.push(node!(STIL::String, unquote(pair.as_str()))),
            Rule::expression | Rule::expression_ => {
                ast.push(build_expression(pair)?);
            }
            Rule::time_expr => {
                ids.push(ast.push_and_open(node!(STIL::TimeExpr)));
                pairs.push(pair.into_inner());
            }
            Rule::paren_expression | Rule::paren_expression_ => {
                ids.push(ast.push_and_open(node!(STIL::Parens)));
                pairs.push(pair.into_inner());
            }
            Rule::number => {
                ids.push(0);
                pairs.push(pair.into_inner());
            }
            Rule::number_with_unit => {
                ids.push(ast.push_and_open(node!(STIL::NumberWithUnit)));
                pairs.push(pair.into_inner());
            }
            Rule::si_unit => ast.push(node!(STIL::SIUnit, pair.as_str().parse().unwrap())),
            Rule::engineering_prefix => {
                ast.push(node!(STIL::EngPrefix, pair.as_str().parse().unwrap()))
            }
            Rule::integer | Rule::signed_integer => {
                ast.push(node!(STIL::Integer, pair.as_str().parse().unwrap()))
            }
            Rule::float_number => ast.push(node!(STIL::Float, pair.as_str().parse().unwrap())),
            Rule::pattern_exec_block => {
                let mut p = pair.into_inner();
                let n;
                if let Some(nxt) = p.peek() {
                    n = match nxt.as_rule() {
                        Rule::name => {
                            node!(STIL::PatternExec, Some(unquote(p.next().unwrap().as_str())))
                        }
                        _ => node!(STIL::PatternExec, None),
                    };
                } else {
                    n = node!(STIL::PatternExec, None);
                }
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::category => {
                ast.push(node!(STIL::CategoryRef, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::selector => {
                ast.push(node!(STIL::SelectorRef, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::timing => ast.push(node!(STIL::TimingRef, unquote(inner_strs(pair)[0]))),
            Rule::pattern_burst => {
                ast.push(node!(STIL::PatternBurstRef, unquote(inner_strs(pair)[0])))
            }
            Rule::pattern_burst_block => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STIL::PatternBurst, unquote(p.next().unwrap().as_str()))),
                );
                pairs.push(p);
            }
            Rule::signal_groups => ast.push(node!(
                STIL::SignalGroupsRef,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::macro_defs => {
                ast.push(node!(STIL::MacroDefs, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::procedures => {
                ast.push(node!(STIL::Procedures, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::scan_structures => ast.push(node!(
                STIL::ScanStructuresRef,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::start => ast.push(node!(STIL::Start, inner_strs(pair)[0].parse().unwrap())),
            Rule::stop => ast.push(node!(STIL::Stop, inner_strs(pair)[0].parse().unwrap())),
            Rule::termination_block => {
                ids.push(ast.push_and_open(node!(STIL::Terminations)));
                pairs.push(pair.into_inner());
            }
            Rule::termination_item => {
                ids.push(ast.push_and_open(node!(STIL::TerminationItem)));
                pairs.push(pair.into_inner());
            }
            Rule::pat_list => {
                ids.push(ast.push_and_open(node!(STIL::PatList)));
                pairs.push(pair.into_inner());
            }
            Rule::pat_list_item => {
                let mut p = pair.into_inner();
                ids.push(ast.push_and_open(node!(STIL::Pat, unquote(p.next().unwrap().as_str()))));
                pairs.push(p);
            }
            Rule::timing_block => {
                let mut p = pair.into_inner();
                let n;
                if let Some(nxt) = p.peek() {
                    n = match nxt.as_rule() {
                        Rule::name => node!(STIL::Timing, Some(unquote(p.next().unwrap().as_str()))),
                        _ => node!(STIL::Timing, None),
                    };
                } else {
                    n = node!(STIL::Timing, None);
                }
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::waveform_table => {
                let mut p = pair.into_inner();
                ids.push(ast.push_and_open(node!(
                    STIL::WaveformTable,
                    unquote(p.next().unwrap().as_str())
                )));
                pairs.push(p);
            }
            Rule::period => {
                ids.push(ast.push_and_open(node!(STIL::Period)));
                pairs.push(pair.into_inner());
            }
            Rule::inherit_waveform_table | Rule::inherit_waveform | Rule::inherit_waveform_wfc => {
                ast.push(node!(STIL::Inherit, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::sub_waveforms => {
                ids.push(ast.push_and_open(node!(STIL::SubWaveforms)));
                pairs.push(pair.into_inner());
            }
            Rule::sub_waveform => {
                ids.push(ast.push_and_open(node!(STIL::SubWaveform)));
                pairs.push(pair.into_inner());
            }
            Rule::waveforms => {
                ids.push(ast.push_and_open(node!(STIL::Waveforms)));
                pairs.push(pair.into_inner());
            }
            Rule::waveform => {
                ids.push(ast.push_and_open(node!(STIL::Waveform)));
                pairs.push(pair.into_inner());
            }
            Rule::wfc_definition => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STIL::WFChar, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::event => {
                ids.push(ast.push_and_open(node!(STIL::Event)));
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
                ast.push(node!(STIL::EventList, vals))
            }
            Rule::spec_block => {
                let mut p = pair.into_inner();
                let n;
                if let Some(nxt) = p.peek() {
                    n = match nxt.as_rule() {
                        Rule::name => node!(STIL::Spec, Some(unquote(p.next().unwrap().as_str()))),
                        _ => node!(STIL::Spec, None),
                    };
                } else {
                    n = node!(STIL::Spec, None);
                }
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::category_block => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STIL::Category, unquote(p.next().unwrap().as_str()))),
                );
                pairs.push(p);
            }
            Rule::spec_item => {
                ids.push(ast.push_and_open(node!(STIL::SpecItem)));
                pairs.push(pair.into_inner());
            }
            Rule::typical_var => {
                let mut p = pair.into_inner();
                ids.push(ast.push_and_open(node!(
                    STIL::TypicalVar,
                    p.next().unwrap().as_str().to_string()
                )));
                pairs.push(p);
            }
            Rule::spec_var => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STIL::SpecVar, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::spec_var_item => {
                let mut p = pair.into_inner();
                ids.push(ast.push_and_open(node!(
                    STIL::SpecVarItem,
                    p.next().unwrap().as_str().parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::variable_block => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STIL::Variable, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::selector_block => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STIL::Selector, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::selector_item => {
                let strs: Vec<&str> = pair.into_inner().map(|v| v.as_str()).collect();
                ast.push(node!(
                    STIL::SelectorItem,
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
                            STIL::ScanStructures,
                            Some(unquote(p.next().unwrap().as_str()))
                        ),
                        _ => node!(STIL::ScanStructures, None),
                    };
                } else {
                    n = node!(STIL::ScanStructures, None);
                }
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::scan_chain => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STIL::ScanChain, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::scan_in_name => ast.push(node!(STIL::ScanInName, unquote(inner_strs(pair)[0]))),
            Rule::scan_out_name => ast.push(node!(STIL::ScanOutName, unquote(inner_strs(pair)[0]))),
            Rule::scan_length => {
                ast.push(node!(STIL::ScanLength, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::scan_out_length => ast.push(node!(
                STIL::ScanOutLength,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::not => ast.push(node!(STIL::Not)),
            Rule::scan_cells => {
                ids.push(ast.push_and_open(node!(STIL::ScanCells)));
                pairs.push(pair.into_inner());
            }
            Rule::scan_master_clock => {
                ids.push(ast.push_and_open(node!(STIL::ScanMasterClock)));
                pairs.push(pair.into_inner());
            }
            Rule::scan_slave_clock => {
                ids.push(ast.push_and_open(node!(STIL::ScanSlaveClock)));
                pairs.push(pair.into_inner());
            }
            Rule::scan_inversion => ast.push(node!(
                STIL::ScanInversion,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::pattern_block => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STIL::Pattern, unquote(p.next().unwrap().as_str()))),
                );
                pairs.push(p);
            }
            Rule::time_unit => {
                ids.push(ast.push_and_open(node!(STIL::TimeUnit)));
                pairs.push(pair.into_inner());
            }
            Rule::vector => {
                ids.push(ast.push_and_open(node!(STIL::Vector)));
                pairs.push(pair.into_inner());
            }
            Rule::cyclized_data => {
                ids.push(ast.push_and_open(node!(STIL::CyclizedData)));
                pairs.push(pair.into_inner());
            }
            Rule::non_cyclized_data => {
                ids.push(ast.push_and_open(node!(STIL::NonCyclizedData)));
                pairs.push(pair.into_inner());
            }
            Rule::repeat => ast.push(node!(STIL::Repeat, inner_strs(pair)[0].parse().unwrap())),
            Rule::waveform_format => ast.push(node!(STIL::WaveformFormat)),
            Rule::pattern_statement => {
                ids.push(0);
                pairs.push(pair.into_inner());
            }
            Rule::hex_format => {
                let vals = inner_strs(pair);
                ast.push(if vals.len() == 0 {
                    node!(STIL::HexFormat, None)
                } else {
                    node!(STIL::HexFormat, Some(vals[0].to_string()))
                })
            }
            Rule::dec_format => {
                let vals = inner_strs(pair);
                ast.push(if vals.len() == 0 {
                    node!(STIL::DecFormat, None)
                } else {
                    node!(STIL::DecFormat, Some(vals[0].to_string()))
                })
            }
            Rule::data_string => ast.push(node!(STIL::Data, pair.as_str().to_string())),
            Rule::vec_data => {
                ids.push(0);
                pairs.push(pair.into_inner());
            }
            Rule::time_value => {
                ast.push(node!(STIL::TimeValue, inner_strs(pair)[0].parse().unwrap()))
            }
            Rule::waveform_statement => {
                ast.push(node!(STIL::WaveformRef, unquote(inner_strs(pair)[0])))
            }
            Rule::condition => {
                ids.push(ast.push_and_open(node!(STIL::Condition)));
                pairs.push(pair.into_inner());
            }
            Rule::call => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STIL::Call, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::macro_statement => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STIL::Macro, p.next().unwrap().as_str().to_string())),
                );
                pairs.push(p);
            }
            Rule::loop_statement => {
                let mut p = pair.into_inner();
                ids.push(
                    ast.push_and_open(node!(STIL::Loop, p.next().unwrap().as_str().parse().unwrap())),
                );
                pairs.push(p);
            }
            Rule::match_loop => {
                let mut p = pair.into_inner();
                let timeout = p.next().unwrap();
                let n = match timeout.as_rule() {
                    Rule::integer => node!(STIL::MatchLoop, Some(timeout.as_str().parse().unwrap())),
                    Rule::infinite => node!(STIL::MatchLoop, None),
                    _ => unreachable!(),
                };
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::goto => ast.push(node!(STIL::Goto, inner_strs(pair)[0].parse().unwrap())),
            Rule::breakpoint => {
                ids.push(ast.push_and_open(node!(STIL::BreakPoint)));
                pairs.push(pair.into_inner());
            }
            Rule::iddq => ast.push(node!(STIL::IDDQ)),
            Rule::stop_statement => ast.push(node!(STIL::StopStatement)),
            Rule::term => {
                ast.push(build_term(pair)?);
            }
            Rule::factor | Rule::factor_ => {
                ids.push(0);
                pairs.push(pair.into_inner());
            }

            ////println!("********************* {:?}", pair);
            Rule::EOI => {}
            _ => {
                println!("********************* {:?}", pair);
                unreachable!()
            }
        }

        loop {
            match pairs.last_mut() {
                Some(x) => match x.next() {
                    Some(p) => {
                        pair = p;
                        break;
                    }
                    None => {
                        pairs.pop();
                        if pairs.len() > 0 {
                            let id = ids.pop().unwrap();
                            if id != 0 {
                                if id == 1 {
                                    return Ok(ast);
                                } else {
                                    ast.close(id)?;
                                }
                            }
                        } else {
                            return Ok(ast);
                        }
                    }
                },
                None => return Ok(ast),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::STIL;
    use super::*;
    use std::path::Path;

    fn read(example: &str) -> String {
        fs::read_to_string(format!(
            "../../test_apps/python_app/vendor/stil/{}.stil",
            example
        ))
        .expect("cannot read file")
    }

    #[test]
    fn test_example1_to_ast() {
        let _stil = STIL::from_file(Path::new(
            "../../test_apps/python_app/vendor/stil/example1.stil",
        ))
        .expect("Imported example1");
        //println!("{:?}", _stil.ast);
    }

    #[test]
    fn test_example2_to_ast() {
        let _stil = STIL::from_file(Path::new(
            "../../test_apps/python_app/vendor/stil/example2.stil",
        ))
        .expect("Imported example2");
        //println!("{:?}", _stil.ast);
    }

    #[test]
    fn test_example3_to_ast() {
        let _stil = STIL::from_file(Path::new(
            "../../test_apps/python_app/vendor/stil/example3.stil",
        ))
        .expect("Imported example3");
        //println!("{:?}", _stil.ast);
    }

    #[test]
    fn test_example4_to_ast() {
        let _stil = STIL::from_file(Path::new(
            "../../test_apps/python_app/vendor/stil/example4.stil",
        ))
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

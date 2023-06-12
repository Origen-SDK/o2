use super::nodes::WGL;
use crate::ast::Node;
use crate::ast::AST;
use crate::{Error, Result};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[grammar = "wgl/wgl.pest"]
pub struct WGLParser;

pub fn parse_file(path: &Path) -> Result<Node<WGL>> {
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

pub fn parse_str(wgl: &str) -> Result<Node<WGL>> {
    match WGLParser::parse(Rule::wgl_source, wgl) {
        Err(e) => Err(Error::new(&format!("{}", e))),
        Ok(mut wgl) => Ok(to_ast(wgl.next().unwrap())?.unwrap()),
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

fn build_expression(pair: Pair<Rule>) -> Result<Node<WGL>> {
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
                    let mut n = node!(WGL::Add);
                    n.add_child(term);
                    n.add_child(next_term);
                    term = n;
                }
                Rule::subtract => {
                    pairs.next();
                    let next_term = to_ast(pairs.next().unwrap())?.unwrap();
                    let mut n = node!(WGL::Subtract);
                    n.add_child(term);
                    n.add_child(next_term);
                    term = n;
                }
                Rule::multiply => {
                    pairs.next();
                    let next_term = to_ast(pairs.next().unwrap())?.unwrap();
                    let mut n = node!(WGL::Multiply);
                    n.add_child(term);
                    n.add_child(next_term);
                    term = n;
                }
                Rule::divide => {
                    pairs.next();
                    let next_term = to_ast(pairs.next().unwrap())?.unwrap();
                    let mut n = node!(WGL::Divide);
                    n.add_child(term);
                    n.add_child(next_term);
                    term = n;
                }
                Rule::pow => {
                    pairs.next();
                    let next_term = to_ast(pairs.next().unwrap())?.unwrap();
                    let mut n = node!(WGL::Power);
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
pub fn to_ast(mut pair: Pair<Rule>) -> Result<AST<WGL>> {
    let mut ast = AST::new();
    let mut ids: Vec<usize> = vec![];
    let mut pairs: Vec<Pairs<Rule>> = vec![];

    loop {
        match pair.as_rule() {
            Rule::built_in_func_call => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::BuiltInFuncCall,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::built_in_var => {
                ast.push(node!(
                    WGL::BuiltInVar,
                    pair.as_str().parse().unwrap()
                ));
            }
            Rule::constant => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                
                if let Some(nxt) = p.peek() {
                    match nxt.as_rule() {
                        Rule::scale => {
                            let v2 = p.next().unwrap().as_str();
                            if let Some(_nxt) = p.peek() {
                                let v3 = p.next().unwrap().as_str();
                                ast.push(node!(
                                    WGL::Constant,
                                    v1.parse().unwrap(),
                                    Some(v2.parse().unwrap()),
                                    Some(v3.parse().unwrap())
                                ));
                            } else {
                                ast.push(node!(
                                    WGL::Constant,
                                    v1.parse().unwrap(),
                                    Some(v2.parse().unwrap()),
                                    None
                                ));
                            }
                        }
                        Rule::eq_unit => {
                            let v2 = p.next().unwrap().as_str();
                            ast.push(node!(
                                WGL::Constant,
                                v1.parse().unwrap(),
                                None,
                                Some(v2.parse().unwrap())
                            ));
                        }
                        _ => unreachable!()
                    }
                } else {
                    ast.push(node!(
                        WGL::Constant,
                        v1.parse().unwrap(),
                        None,
                        None
                    ));
                }
            }
            Rule::binary_operation => {
                ast.push(build_expression(pair)?);
            }    
            Rule::positive => {
                ids.push(ast.push_and_open(node!(WGL::Positive)));
                pairs.push(pair.into_inner());
            }
            Rule::negative => {
                ids.push(ast.push_and_open(node!(WGL::Negative)));
                pairs.push(pair.into_inner());
            }
            Rule::pre_increment => {
                ids.push(ast.push_and_open(node!(WGL::PreIncrement)));
                pairs.push(pair.into_inner());
            }
            Rule::pre_decrement => {
                ids.push(ast.push_and_open(node!(WGL::PreDecrement)));
                pairs.push(pair.into_inner());
            }
            Rule::post_increment => {
                ids.push(ast.push_and_open(node!(WGL::PostIncrement)));
                pairs.push(pair.into_inner());
            }
            Rule::post_decrement => {
                ids.push(ast.push_and_open(node!(WGL::PostDecrement)));
                pairs.push(pair.into_inner());
            }
            Rule::paren_expression => {
                ids.push(ast.push_and_open(node!(WGL::Parens)));
                pairs.push(pair.into_inner());
            }
            Rule::wgl_source => {
                ids.push(ast.push_and_open(node!(WGL::Root)));
                pairs.push(pair.into_inner());
            }
            Rule::name => {
                ast.push(node!(WGL::String,unquote(pair.as_str())))
            }
            Rule::identifier => {
                ast.push(node!(WGL::String,unquote(pair.as_str())))
            }
            Rule::quoted_string => {
                ast.push(node!(WGL::String,unquote(pair.as_str())))
            }
            Rule::signals => {
                ids.push(ast.push_and_open(node!(WGL::Signals)));
                pairs.push(pair.into_inner());
            }
            Rule::signal_decl => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::Signal,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::bus_range => {
                let vals = inner_strs(pair);
                ast.push(node!(
                    WGL::Bus,
                    vals[0].parse().unwrap(),
                    vals[1].parse().unwrap()
                ));
            }
            Rule::group_members => {
                ids.push(ast.push_and_open(node!(WGL::Group)));
                pairs.push(pair.into_inner());
            }
            Rule::signal_reference => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                if let Some(_nxt) = p.peek() {
                    ids.push(ast.push_and_open(node!(
                        WGL::SignalRef,
                        v1.parse().unwrap()
                    )));
                    pairs.push(p);
                } else {
                    ast.push(node!(
                        WGL::SignalRef,
                        v1.parse().unwrap()
                    ));
                }
            }
            Rule::range => {
                let vals = inner_strs(pair);
                if vals.len() == 1 {
                    ast.push(node!(
                        WGL::Range,
                        vals[0].parse().unwrap(),
                        None
                    ))
                } else {
                    ast.push(node!(
                        WGL::Range,
                        vals[0].parse().unwrap(),
                        Some(vals[1].parse().unwrap())
                    ))
                }
            }
            Rule::mux_members => {
                ids.push(ast.push_and_open(node!(WGL::MuxMembers)));
                pairs.push(pair.into_inner());
            }
            Rule::mux_part_list => {
                ids.push(ast.push_and_open(node!(WGL::MuxList)));
                pairs.push(pair.into_inner());
            }
            Rule::signal_attributes => {
                ids.push(ast.push_and_open(node!(WGL::SignalAttributes)));
                pairs.push(pair.into_inner());
            }
            Rule::mux => {
                ast.push(node!(WGL::Mux))
            }
            Rule::data_bit_count => ast.push(node!(
                WGL::DataBitCount,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::wide => ast.push(node!(
                WGL::Wide,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::signal_direction => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str().to_lowercase();
                if let Some(_nxt) = p.peek() {
                    let v2 = p.next().unwrap().as_str().to_lowercase();
                    ast.push(node!(
                        WGL::SigDirection,
                        v1.parse().unwrap(),
                        Some(v2.parse().unwrap())
                    ));
                } else {
                    ast.push(node!(
                        WGL::SigDirection,
                        v1.parse().unwrap(),
                        None
                    ));
                }
            }
            Rule::strobe => {
                let vals = inner_strs(pair);
                ast.push(node!(
                    WGL::Strobe,
                    vals[0].parse().unwrap(),
                    vals[1].parse().unwrap(),
                    vals[2].parse().unwrap()
                ))
            }
            Rule::radix => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str().to_lowercase();
                ast.push(node!(
                    WGL::Radix,
                    v1.parse().unwrap()
                ));
            }
            Rule::dut_pins => {
                ids.push(ast.push_and_open(node!(WGL::DutPins)));
                pairs.push(pair.into_inner());
            }            
            Rule::dut_pin_group => {
                ids.push(ast.push_and_open(node!(WGL::DutPinGroup)));
                pairs.push(pair.into_inner());
            }            
            Rule::pin_info => {
                ids.push(ast.push_and_open(node!(WGL::PinInfo)));
                pairs.push(pair.into_inner());
            }            
            Rule::pin_name => {
                ids.push(ast.push_and_open(node!(WGL::PinName)));
                pairs.push(pair.into_inner());
            }            
            Rule::pin_number => {
                ids.push(ast.push_and_open(node!(WGL::PinNumber)));
                pairs.push(pair.into_inner());
            }            
            Rule::ate_pins => {
                ids.push(ast.push_and_open(node!(WGL::AtePins)));
                pairs.push(pair.into_inner());
            }            
            Rule::ate_pin_group => {
                ids.push(ast.push_and_open(node!(WGL::AtePinGroup)));
                pairs.push(pair.into_inner());
            }            
            Rule::ate_pin_info => {
                ids.push(ast.push_and_open(node!(WGL::AtePinInfo)));
                pairs.push(pair.into_inner());
            }            
            Rule::pstate => ast.push(node!(
                WGL::PState,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::integer => {
                ast.push(node!(WGL::Integer, pair.as_str().parse().unwrap()))
            }
            Rule::scan_cells => {
                ids.push(ast.push_and_open(node!(WGL::ScanCells)));
                pairs.push(pair.into_inner());
            }
            Rule::scan_cell_decl => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::ScanCell,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::scan_group => {
                ids.push(ast.push_and_open(node!(WGL::ScanGroup)));
                pairs.push(pair.into_inner());
            }
            Rule::scan_range => {
                let vals = inner_strs(pair);
                ast.push(node!(
                    WGL::ScanRange,
                    vals[0].parse().unwrap(),
                    vals[1].parse().unwrap()
                ));
            }
            Rule::cell_reference => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::CellRef,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::scan_state => {
                ids.push(ast.push_and_open(node!(WGL::ScanStates)));
                pairs.push(pair.into_inner());
            }
            Rule::scan_state_decl => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::ScanState,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::state_vector_element => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::StateVector,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::state_string => {
                ast.push(node!(WGL::StateString,unquote(pair.as_str())))
            }
            Rule::scan_chain => {
                ids.push(ast.push_and_open(node!(WGL::ScanChains)));
                pairs.push(pair.into_inner());
            }
            Rule::chain_decl => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::ScanChain,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::in_edge_signal => ast.push(node!(
                WGL::ScanChainInEdge,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::out_edge_signal => ast.push(node!(
                WGL::ScanChainOutEdge,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::chain_mem_invert => {
                ast.push(node!(WGL::ScanChainMemInvert))
            }
            Rule::time_plate => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::TimePlate,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::time_reference => {
                ids.push(ast.push_and_open(node!(WGL::TimeRef)));
                pairs.push(pair.into_inner());
            }
            Rule::time => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                let v2 = p.next().unwrap().as_str().to_lowercase();
                ast.push(node!(
                    WGL::Time,
                    v1.parse().unwrap(),
                    v2.parse().unwrap()
                ));
            }
            Rule::channel => {
                ids.push(ast.push_and_open(node!(WGL::Channel)));
                pairs.push(pair.into_inner());
            }
            Rule::track => {
                ids.push(ast.push_and_open(node!(WGL::Track)));
                pairs.push(pair.into_inner());
            }
            Rule::first_event => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                let zero_time_ref = "0".to_string();
                let zero_time_ref = zero_time_ref + v1;
                let v2 = p.next().unwrap().as_str();
                if let Some(_nxt) = p.peek() {
                    let v3 = p.next().unwrap();
                    ast.push(node!(
                        WGL::Event,
                        true,
                        zero_time_ref,
                        v2.parse().unwrap(),
                        Some(inner_strs(v3)[0].parse().unwrap())
                    ));
                } else {
                    ast.push(node!(
                        WGL::Event,
                        true,
                        zero_time_ref,
                        v2.parse().unwrap(),
                        None
                    ));
                }
            }
            Rule::event => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                let v2 = p.next().unwrap().as_str();
                if let Some(_nxt) = p.peek() {
                    let v3 = p.next().unwrap();
                    ast.push(node!(
                        WGL::Event,
                        false,
                        v1.parse().unwrap(),
                        v2.parse().unwrap(),
                        Some(inner_strs(v3)[0].parse().unwrap())
                    ));
                } else {
                    ast.push(node!(
                        WGL::Event,
                        false,
                        v1.parse().unwrap(),
                        v2.parse().unwrap(),
                        None
                    ));
                }
            }
            Rule::pattern => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::Pattern,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::subroutine => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::Subroutine,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::pattern_param_dir => {
                ast.push(node!(WGL::PatternParamDir,unquote(pair.as_str())))
            }
            Rule::pattern_param => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::PatternParam,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::pattern_row => {
                let mut p = pair.into_inner();
                let n;
                if let Some(nxt) = p.peek() {
                    n = match nxt.as_rule() {
                        Rule::vector_label => {
                            node!(WGL::PatternRow, Some(unquote(p.next().unwrap().as_str())))
                        }
                        _ => node!(WGL::PatternRow, None),
                    };
                } else {
                    n = node!(WGL::PatternRow, None);
                }
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::loop_stmt => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap();
                let n = match v1.as_rule() {
                    Rule::integer => {
                        node!(
                            WGL::LoopStmt, 
                            v1.as_str().parse().unwrap(),
                            None
                        )
                    }
                    Rule::identifier => {
                        node!(
                            WGL::LoopStmt,
                            p.next().unwrap().as_str().parse().unwrap(),
                            Some(unquote(v1.as_str()))
                        )
                    }
                    _ => unreachable!(),
                };
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::repeat => {
                let mut p = pair.into_inner();
                let nxt = p.peek().unwrap();
                let n = match nxt.as_rule() {
                    Rule::integer => {
                        node!(WGL::Repeat, p.next().unwrap().as_str().parse().unwrap())
                    }
                    _ => node!(WGL::Repeat, 1),
                };
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::vector => {
                ids.push(ast.push_and_open(node!(WGL::Vector)));
                pairs.push(pair.into_inner());
            }
            Rule::address => {
                ids.push(ast.push_and_open(node!(WGL::Address)));
                pairs.push(pair.into_inner());
            }
            Rule::address_increment => {
                ast.push(node!(WGL::AddressIncrement))
            }
            Rule::pattern_expression => {
                ids.push(ast.push_and_open(node!(WGL::PatternExpression)));
                pairs.push(pair.into_inner());
            }
            Rule::state_string_with_selector => {
                ids.push(ast.push_and_open(node!(WGL::StateStringWithSelector)));
                pairs.push(pair.into_inner());
            }
            Rule::time_comment => {
                ids.push(ast.push_and_open(node!(WGL::TimeComment)));
                pairs.push(pair.into_inner());
            }
            Rule::call => ast.push(node!(
                WGL::Call,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::offset => {
                ids.push(ast.push_and_open(node!(WGL::Offset)));
                pairs.push(pair.into_inner());
            }
            Rule::scan_row => {
                ids.push(ast.push_and_open(node!(WGL::ScanRow)));
                pairs.push(pair.into_inner());
            }
            Rule::scan_run => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str().to_lowercase();
                let v2 = p.next().unwrap().as_str();
                let v3 = p.next().unwrap().as_str();
                ast.push(node!(
                    WGL::ScanRun,
                    v1.parse().unwrap(),
                    v2.parse().unwrap(),
                    v3.parse().unwrap()
                ));
            }
            Rule::symbolic => {
                ids.push(ast.push_and_open(node!(WGL::Symbolic)));
                pairs.push(pair.into_inner());
            }
            Rule::sym_direction => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str().to_lowercase();
                if let Some(_nxt) = p.peek() {
                    let v2 = p.next().unwrap().as_str().to_lowercase();
                    ast.push(node!(
                        WGL::SymDirection,
                        v1.parse().unwrap(),
                        Some(v2.parse().unwrap())
                    ));
                } else {
                    ast.push(node!(
                        WGL::SymDirection,
                        v1.parse().unwrap(),
                        None
                    ));
                }
            }
            Rule::symbolic_assignment => {
                let mut p = pair.into_inner();
                let nxt = p.peek().unwrap();
                let n = match nxt.as_rule() {
                    Rule::name => {
                        node!(WGL::SymAssignment, Some(unquote(p.next().unwrap().as_str())))
                    }
                    _ => node!(WGL::SymAssignment, None),
                };
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::equation_sheet => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::EquationSheet,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::expression_decl => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::ExprSet,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::variable_decl => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::Variable,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::equation_defaults => {
                ids.push(ast.push_and_open(node!(WGL::EquationDefaults)));
                pairs.push(pair.into_inner());
            }
            Rule::default_pair => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                let v2 = p.next().unwrap().as_str();
                ast.push(node!(
                    WGL::DefaultPair,
                    v1.parse().unwrap(),
                    v2.parse().unwrap()
                ));
            }
            Rule::formats => {
                ids.push(ast.push_and_open(node!(WGL::Formats)));
                pairs.push(pair.into_inner());
            }
            Rule::format_decl => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::Format,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::tds_state => {
                ast.push(node!(WGL::TdsState,unquote(pair.as_str())))
            }
            Rule::registers => {
                ids.push(ast.push_and_open(node!(WGL::Registers)));
                pairs.push(pair.into_inner());
            }
            Rule::pin_list => {
                ids.push(ast.push_and_open(node!(WGL::PinList)));
                pairs.push(pair.into_inner());
            }
            Rule::register_decl => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::Register,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::format_spec => {
                ast.push(node!(WGL::FormatSpec,unquote(pair.as_str())))
            }
            Rule::pin_groups=> {
                ids.push(ast.push_and_open(node!(WGL::PinGroups)));
                pairs.push(pair.into_inner());
            }
            Rule::pin_group_decl => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::PinGroup,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::time_gens=> {
                ids.push(ast.push_and_open(node!(WGL::TimeGens)));
                pairs.push(pair.into_inner());
            }
            Rule::tg_decl => {
                let vals = inner_strs(pair);
                if vals.len() == 2 {
                    ast.push(node!(
                        WGL::TimeGen,
                        vals[0].parse().unwrap(),
                        None,
                        vals[1].to_lowercase().parse().unwrap()
                    ))
                } else {
                    ast.push(node!(
                        WGL::TimeGen,
                        vals[0].parse().unwrap(),
                        Some(vals[1].parse().unwrap()),
                        vals[2].to_lowercase().parse().unwrap()
                    ))
                }
            }
            Rule::timing_sets => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::TimingSet,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::tg_assign => {
                let mut p = pair.into_inner();
                let nxt = p.peek().unwrap();
                let n = match nxt.as_rule() {
                    Rule::name => {
                        node!(
                            WGL::TimeGenAssign,
                            Some(p.next().unwrap().as_str().parse().unwrap())
                        )
                    }
                    _ => node!(WGL::TimeGenAssign, None),
                };
                ids.push(ast.push_and_open(n));
                pairs.push(p);
            }
            Rule::tg_repeat => ast.push(node!(
                WGL::Repeat,
                inner_strs(pair)[0].parse().unwrap()
            )),
            Rule::macro_definition => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::MacroDef,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::macro_body => {
                ast.push(node!(WGL::MacroBody,unquote(pair.as_str())))
            }
            Rule::macro_invocation => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    WGL::MacroInvocation,
                    v1.parse().unwrap()
                )));
                pairs.push(p);
            }
            Rule::include_invocation => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                ast.push(node!(
                    WGL::Include,
                    v1.parse().unwrap()
                ));
            }
            Rule::annotation => {
                ast.push(node!(WGL::Annotation,unquote(pair.as_str())))
            }
            Rule::global_mode => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str().to_lowercase();
                ast.push(node!(
                    WGL::GlobalMode,
                    v1.parse().unwrap()
                ));
            }
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
    use super::super::from_file;
    use super::*;
    use std::path::Path;

    fn read(example: &str) -> String {
        fs::read_to_string(format!(
            "../../test_apps/python_app/vendor/wgl/{}.wgl",
            example
        ))
        .expect("cannot read file")
    }
 
    #[test]
    fn test_example1_to_ast() {
        let _vcd = from_file(Path::new(
            "../../test_apps/python_app/vendor/wgl/example1.wgl",
        ))
        .expect("Imported example1");
    }

    #[test]
    fn test_example2_to_ast() {
        let _vcd = from_file(Path::new(
            "../../test_apps/python_app/vendor/wgl/example2.wgl",
        ))
        .expect("Imported example2");
    }

    #[test]
    fn test_example3_to_ast() {
        let _vcd = from_file(Path::new(
            "../../test_apps/python_app/vendor/wgl/example3.wgl",
        ))
        .expect("Imported example3");
    }

    #[test]
    fn test_example1_can_parse() {
        let txt = read("example1");
        match WGLParser::parse(Rule::wgl_source, &txt) {
            Ok(_res) => {}
            Err(e) => {
                println!("{}", e);
                assert_eq!(1, 0);
            }
        }
    }
        
    #[test]
    fn test_example2_can_parse() {
        let txt = read("example2");
        match WGLParser::parse(Rule::wgl_source, &txt) {
            Ok(_res) => {}
            Err(e) => {
                println!("{}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_example3_can_parse() {
        let txt = read("example3");
        match WGLParser::parse(Rule::wgl_source, &txt) {
            Ok(_res) => {}
            Err(e) => {
                println!("{}", e);
                assert_eq!(1, 0);
            }
        }
    }
}

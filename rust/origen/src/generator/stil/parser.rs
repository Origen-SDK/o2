use crate::generator::ast::*;
use crate::node;
use crate::{Error, Result};
use pest::iterators::Pair;
use pest::Parser;
use std::fs;

#[derive(Parser)]
#[grammar = "generator/stil/stil.pest"]
pub struct STILParser;

fn import(file: &str) -> Result<Node> {
    let data = fs::read_to_string(file).expect(&format!("Cannot read file: {}", file));

    match STILParser::parse(Rule::stil_source, &data) {
        Err(e) => Err(Error::new(&format!("{}", e))),
        Ok(mut stil) => Ok(to_node(stil.next().unwrap())),
    }
}

fn inner_strs(pair: Pair<Rule>) -> Vec<&str> {
    pair.into_inner().map(|v| v.as_str()).collect()
}

// Adds the child nodes contained in the given parser pair to the given node
fn process_children(mut node: Node, pair: Pair<Rule>) -> Node {
    for c in pair.into_inner() {
        node.add_child(to_node(c));
    }
    node
}

// Returns the child node contained in the given parser pair, the caller is responsible
// for supplying a pair that contains only one node
fn process_child(pair: Pair<Rule>) -> Node {
    let mut nodes: Vec<Node> = pair.into_inner().map(|v| to_node(v)).collect();
    nodes.pop().unwrap()
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
fn to_node(pair: Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::stil_source => process_children(node!(STIL), pair),
        Rule::stil_version => {
            let vals = inner_strs(pair);
            node!(
                STILVersion,
                vals[0].parse().unwrap(),
                vals[1].parse().unwrap()
            )
        }
        Rule::label => node!(STILLabel, unquote(inner_strs(pair)[0])),
        Rule::header_block => process_children(node!(STILHeader), pair),
        Rule::title => node!(STILTitle, unquote(inner_strs(pair)[0])),
        Rule::date => node!(STILDate, unquote(inner_strs(pair)[0])),
        Rule::source => node!(STILSource, unquote(inner_strs(pair)[0])),
        Rule::history => process_children(node!(STILHistory), pair),
        Rule::annotation => node!(STILAnnotation, inner_strs(pair)[0].to_string()),
        Rule::include => {
            let vals = inner_strs(pair);
            if vals.len() == 1 {
                node!(STILInclude, vals[0].to_string(), None)
            } else {
                node!(STILInclude, vals[0].to_string(), Some(vals[1].to_string()))
            }
        }
        Rule::signals_block => process_children(node!(STILSignals), pair),
        Rule::signal => {
            let mut vals: Vec<&str> = Vec::new();
            let mut children: Vec<Node> = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::name | Rule::signal_type => vals.push(inner_pair.as_str()),
                    _ => children.push(to_node(inner_pair)),
                };
            }
            let mut n = node!(
                STILSignal,
                vals[0].parse().unwrap(),
                vals[1].parse().unwrap()
            );
            for child in children {
                n.add_child(child);
            }
            n
        }
        Rule::termination => node!(STILTermination, inner_strs(pair)[0].parse().unwrap()),
        Rule::default_state => node!(STILDefaultState, inner_strs(pair)[0].parse().unwrap()),
        Rule::base => {
            let vals = inner_strs(pair);
            node!(STILBase, vals[0].parse().unwrap(), vals[1].parse().unwrap())
        }
        Rule::alignment => node!(STILAlignment, inner_strs(pair)[0].parse().unwrap()),
        Rule::scan_in => node!(STILScanIn, inner_strs(pair)[0].parse().unwrap()),
        Rule::scan_out => node!(STILScanOut, inner_strs(pair)[0].parse().unwrap()),
        Rule::data_bit_count => node!(STILDataBitCount, inner_strs(pair)[0].parse().unwrap()),
        Rule::signal_groups_block => {
            let mut vals: Vec<&str> = Vec::new();
            let mut children: Vec<Node> = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::name => vals.push(inner_pair.as_str()),
                    _ => children.push(to_node(inner_pair)),
                };
            }
            let mut n;
            if vals.len() == 0 {
                n = node!(STILSignalGroups, None);
            } else {
                n = node!(STILSignalGroups, Some(vals[0].to_string()));
            }
            for child in children {
                n.add_child(child);
            }
            n
        }
        Rule::signal_group => {
            let mut vals: Vec<&str> = Vec::new();
            let mut children: Vec<Node> = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::name => vals.push(inner_pair.as_str()),
                    _ => children.push(to_node(inner_pair)),
                };
            }
            let mut n = node!(STILSignalGroup, vals[0].parse().unwrap());
            for child in children {
                n.add_child(child);
            }
            n
        }
        Rule::sigref_expr => process_children(node!(STILSigRefExpr), pair),
        Rule::name => node!(STILName, pair.as_str().to_string()),
        Rule::expression | Rule::expression_subset => process_children(node!(STILExpr), pair),
        Rule::time_expr => process_child(pair),
        Rule::add => process_children(node!(STILAdd), pair),
        Rule::subtract => process_children(node!(STILSubtract), pair),
        Rule::multiply => process_children(node!(STILMultiply), pair),
        Rule::divide => process_children(node!(STILDivide), pair),
        Rule::paren_expression => process_children(node!(STILParens), pair),
        Rule::terminal => process_child(pair),
        Rule::number => process_children(node!(STILNumber), pair),
        Rule::number_with_unit => process_children(node!(STILNumberWithUnit), pair),
        Rule::si_unit => node!(STILSIUnit, pair.as_str().parse().unwrap()),
        Rule::engineering_prefix => node!(STILEngPrefix, pair.as_str().parse().unwrap()),
        Rule::integer => node!(STILInteger, pair.as_str().parse().unwrap()),
        Rule::signed_integer => node!(STILSignedInteger, pair.as_str().parse().unwrap()),
        Rule::point => node!(STILPoint),
        Rule::exponential => node!(STILExp),
        Rule::minus => node!(STILMinus),
        Rule::pattern_exec_block => {
            let mut vals: Vec<&str> = Vec::new();
            let mut children: Vec<Node> = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::name => vals.push(inner_pair.as_str()),
                    _ => children.push(to_node(inner_pair)),
                };
            }
            let mut n;
            if vals.len() == 0 {
                n = node!(STILPatternExec, None);
            } else {
                n = node!(STILPatternExec, Some(vals[0].to_string()));
            }
            for child in children {
                n.add_child(child);
            }
            n
        }
        Rule::category => node!(STILCategoryRef, inner_strs(pair)[0].parse().unwrap()),
        Rule::selector => node!(STILSelectorRef, inner_strs(pair)[0].parse().unwrap()),
        Rule::timing => node!(STILTimingRef, inner_strs(pair)[0].parse().unwrap()),
        Rule::pattern_burst => node!(STILPatternBurstRef, inner_strs(pair)[0].parse().unwrap()),
        Rule::pattern_burst_block => {
            let mut vals: Vec<&str> = Vec::new();
            let mut children: Vec<Node> = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::name => vals.push(inner_pair.as_str()),
                    _ => children.push(to_node(inner_pair)),
                };
            }
            let mut n = node!(STILPatternBurst, vals[0].parse().unwrap());
            for child in children {
                n.add_child(child);
            }
            n
        }
        Rule::signal_groups => node!(STILSignalGroupsRef, inner_strs(pair)[0].parse().unwrap()),
        Rule::macro_defs => node!(STILMacroDefs, inner_strs(pair)[0].parse().unwrap()),
        Rule::procedures => node!(STILProcedures, inner_strs(pair)[0].parse().unwrap()),
        Rule::scan_structures => node!(STILScanStructuresRef, inner_strs(pair)[0].parse().unwrap()),
        Rule::start => node!(STILStart, inner_strs(pair)[0].parse().unwrap()),
        Rule::stop => node!(STILStop, inner_strs(pair)[0].parse().unwrap()),
        Rule::termination_block => process_children(node!(STILTerminations), pair),
        Rule::termination_item => process_children(node!(STILTerminationItem), pair),
        Rule::pat_list => process_children(node!(STILPatList), pair),
        Rule::pat_list_item => {
            let mut vals: Vec<&str> = Vec::new();
            let mut children: Vec<Node> = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::name => vals.push(inner_pair.as_str()),
                    _ => children.push(to_node(inner_pair)),
                };
            }
            let mut n = node!(STILPat, vals[0].parse().unwrap());
            for child in children {
                n.add_child(child);
            }
            n
        }
        Rule::timing_block => {
            let mut vals: Vec<&str> = Vec::new();
            let mut children: Vec<Node> = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::name => vals.push(inner_pair.as_str()),
                    _ => children.push(to_node(inner_pair)),
                };
            }
            let mut n;
            if vals.len() == 0 {
                n = node!(STILTiming, None);
            } else {
                n = node!(STILTiming, Some(vals[0].to_string()));
            }
            for child in children {
                n.add_child(child);
            }
            n
        }
        Rule::waveform_table => {
            let mut vals: Vec<&str> = Vec::new();
            let mut children: Vec<Node> = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::name => vals.push(inner_pair.as_str()),
                    _ => children.push(to_node(inner_pair)),
                };
            }
            let mut n = node!(STILWaveformTable, vals[0].parse().unwrap());
            for child in children {
                n.add_child(child);
            }
            n
        }
        Rule::period => process_children(node!(STILPeriod), pair),
        Rule::inherit_waveform_table | Rule::inherit_waveform | Rule::inherit_waveform_wfc => {
            node!(STILInherit, inner_strs(pair)[0].parse().unwrap())
        }
        Rule::sub_waveforms => process_children(node!(STILSubWaveforms), pair),
        Rule::sub_waveform => process_children(node!(STILSubWaveform), pair),
        Rule::waveforms => process_children(node!(STILWaveforms), pair),
        Rule::waveform => process_children(node!(STILWaveform), pair),
        Rule::wfc_definition => {
            let mut vals: Vec<&str> = Vec::new();
            let mut children: Vec<Node> = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::wfc_list => vals.push(inner_pair.as_str()),
                    _ => children.push(to_node(inner_pair)),
                };
            }
            let mut n = node!(STILWFChar, vals[0].parse().unwrap());
            for child in children {
                n.add_child(child);
            }
            n
        }
        Rule::event => process_children(node!(STILEvent), pair),
        Rule::event_list => {
            let mut vals: Vec<char> = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::event_char => vals.push(inner_pair.as_str().parse().unwrap()),
                    _ => unreachable!(),
                };
            }
            node!(STILEventList, vals)
        }
        Rule::spec_block => {
            let mut vals: Vec<&str> = Vec::new();
            let mut children: Vec<Node> = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::name => vals.push(inner_pair.as_str()),
                    _ => children.push(to_node(inner_pair)),
                };
            }
            let mut n;
            if vals.len() == 0 {
                n = node!(STILSpec, None);
            } else {
                n = node!(STILSpec, Some(vals[0].to_string()));
            }
            for child in children {
                n.add_child(child);
            }
            n
        }
        Rule::category_block => {
            let mut pairs = pair.into_inner();
            let mut n = node!(
                STILCategory,
                pairs.next().unwrap().as_str().parse().unwrap()
            );
            for p in pairs {
                n.add_child(to_node(p));
            }
            n
        }
        Rule::spec_item => process_children(node!(STILSpecItem), pair),
        Rule::typical_var => {
            let mut pairs = pair.into_inner();
            let mut n = node!(
                STILTypicalVar,
                pairs.next().unwrap().as_str().parse().unwrap()
            );
            for p in pairs {
                n.add_child(to_node(p));
            }
            n
        }
        Rule::spec_var => {
            let mut pairs = pair.into_inner();
            let mut n = node!(STILSpecVar, pairs.next().unwrap().as_str().parse().unwrap());
            for p in pairs {
                n.add_child(to_node(p));
            }
            n
        }
        Rule::spec_var_item => {
            let mut pairs = pair.into_inner();
            let mut n = node!(
                STILSpecVarItem,
                pairs.next().unwrap().as_str().parse().unwrap()
            );
            for p in pairs {
                n.add_child(to_node(p));
            }
            n
        }
        Rule::variable_block => {
            let mut pairs = pair.into_inner();
            let mut n = node!(
                STILVariable,
                pairs.next().unwrap().as_str().parse().unwrap()
            );
            for p in pairs {
                n.add_child(to_node(p));
            }
            n
        }
        Rule::selector_block => {
            let mut pairs = pair.into_inner();
            let mut n = node!(
                STILSelector,
                pairs.next().unwrap().as_str().parse().unwrap()
            );
            for p in pairs {
                n.add_child(to_node(p));
            }
            n
        }
        Rule::selector_item => {
            let strs: Vec<&str> = pair.into_inner().map(|v| v.as_str()).collect();
            node!(
                STILSelectorItem,
                strs[0].parse().unwrap(),
                strs[1].parse().unwrap()
            )
        }
        Rule::scan_structures_block => {
            let mut vals: Vec<&str> = Vec::new();
            let mut children: Vec<Node> = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::name => vals.push(inner_pair.as_str()),
                    _ => children.push(to_node(inner_pair)),
                };
            }
            let mut n;
            if vals.len() == 0 {
                n = node!(STILScanStructures, None);
            } else {
                n = node!(STILScanStructures, Some(vals[0].to_string()));
            }
            for child in children {
                n.add_child(child);
            }
            n
        }
        Rule::scan_chain => {
            let mut pairs = pair.into_inner();
            let mut n = node!(STILScanChain, pairs.next().unwrap().as_str().to_string());
            for p in pairs {
                n.add_child(to_node(p));
            }
            n
        }
        Rule::scan_in_name => node!(STILScanInName, inner_strs(pair)[0].parse().unwrap()),
        Rule::scan_out_name => node!(STILScanOutName, inner_strs(pair)[0].parse().unwrap()),
        Rule::scan_length => node!(STILScanLength, inner_strs(pair)[0].parse().unwrap()),
        Rule::scan_out_length => node!(STILScanOutLength, inner_strs(pair)[0].parse().unwrap()),
        Rule::not => node!(STILNot),
        Rule::scan_cells => process_children(node!(STILScanCells), pair),
        Rule::scan_master_clock => process_children(node!(STILScanMasterClock), pair),
        Rule::scan_slave_clock => process_children(node!(STILScanSlaveClock), pair),
        Rule::scan_inversion => node!(STILScanInversion, inner_strs(pair)[0].parse().unwrap()),

        //println!("********************* {:?}", pair);
        _ => node!(STILUnknown),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(example: &str) -> String {
        fs::read_to_string(format!("../../example/vendor/stil/{}.stil", example))
            .expect("cannot read file")
    }

    #[test]
    fn test_example1_to_ast() {
        let r = import("../../example/vendor/stil/example1.stil");
        println!("{:?}", r);
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

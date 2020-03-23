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

fn process_children(mut node: Node, pair: Pair<Rule>) -> Node {
    for c in pair.into_inner() {
        node.add_child(to_node(c));
    }
    node
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

fn to_node(pair: Pair<Rule>) -> Node {
    //println!("********************* {:?}", pair);
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
        //STILWaveformChar(char),
        //STILAlignment(stil::Alignment),
        //STILScanIn(u32),
        //STILScanOut(u32),
        //STILDataBitCount(u32),
        _ => node!(STIL),
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

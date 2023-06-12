use crate::vcd::ValueChangeType::{Scalar,Vector};
use super::nodes::VCD;
use crate::ast::Node;
use crate::ast::AST;
use crate::{Error, Result};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[grammar = "vcd/vcd.pest"]
pub struct VCDParser;

pub fn parse_file(path: &Path) -> Result<Node<VCD>> {
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

pub fn parse_str(vcd: &str) -> Result<Node<VCD>> {
    match VCDParser::parse(Rule::vcd_source, vcd) {
        Err(e) => Err(Error::new(&format!("{}", e))),
        Ok(mut vcd) => Ok(to_ast(vcd.next().unwrap())?.unwrap()),
    }
}

fn inner_strs(pair: Pair<Rule>) -> Vec<&str> {
    pair.into_inner().map(|v| v.as_str()).collect()
}

// This is the main function responsible for transforming the parsed strings into an AST
pub fn to_ast(mut pair: Pair<Rule>) -> Result<AST<VCD>> {
    let mut ast = AST::new();
    let mut ids: Vec<usize> = vec![];
    let mut pairs: Vec<Pairs<Rule>> = vec![];

    loop {
        match pair.as_rule() {
            Rule::vcd_source => {
                ids.push(ast.push_and_open(node!(VCD::Root)));
                pairs.push(pair.into_inner());
            }
            Rule::vcd_header_section => {
                ids.push(ast.push_and_open(node!(VCD::HeaderSection)));
                pairs.push(pair.into_inner());
            }
            Rule::comment_command => ast.push(node!(VCD::Comment, inner_strs(pair)[0].parse().unwrap())),
            Rule::date_command => ast.push(node!(VCD::Date, inner_strs(pair)[0].parse().unwrap())),
            Rule::version_command => ast.push(node!(VCD::Version, inner_strs(pair)[0].parse().unwrap())),
            Rule::scope_command => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                let v2 = p.next().unwrap().as_str();
                ids.push(ast.push_and_open(node!(
                    VCD::Scope,
                    v1.parse().unwrap(),
                    v2.parse().unwrap()
                )));
                pairs.push(p);               
            }
            Rule::timescale_command => {
                let vals = inner_strs(pair);
                ast.push(node!(
                    VCD::TimeScale,
                    vals[0].parse().unwrap(),
                    vals[1].parse().unwrap()
                ))                
            }
            Rule::var_command => {
                let vals = inner_strs(pair);
                ast.push(node!(
                    VCD::Var,
                    vals[0].parse().unwrap(),
                    vals[1].parse().unwrap(),
                    vals[2].parse().unwrap(),
                    vals[3].parse().unwrap(),
                    None
                ))                
            }
            Rule::upscope_command => ast.push(node!(VCD::UpScope)),
            Rule::enddefinitions_command => ast.push(node!(VCD::EndDefinitions)),
            Rule::vcdclose_command => ast.push(node!(VCD::VcdClose)),
            Rule::vcd_data_section => {
                ids.push(ast.push_and_open(node!(VCD::DataSection)));
                pairs.push(pair.into_inner());
            }
            Rule::dumpall_command => {
                ids.push(ast.push_and_open(node!(VCD::DumpAll)));
                pairs.push(pair.into_inner());
            }
            Rule::dumpoff_command => {
                ids.push(ast.push_and_open(node!(VCD::DumpOff)));
                pairs.push(pair.into_inner());
            }
            Rule::dumpon_command => {
                ids.push(ast.push_and_open(node!(VCD::DumpOn)));
                pairs.push(pair.into_inner());
            }
            Rule::dumpvars_command => {
                ids.push(ast.push_and_open(node!(VCD::DumpVars)));
                pairs.push(pair.into_inner());
            }
            Rule::dumpportsall_command => {
                ids.push(ast.push_and_open(node!(VCD::DumpPortsAll)));
                pairs.push(pair.into_inner());
            }
            Rule::dumpportsoff_command => {
                ids.push(ast.push_and_open(node!(VCD::DumpPortsOff)));
                pairs.push(pair.into_inner());
            }
            Rule::dumpportson_command => {
                ids.push(ast.push_and_open(node!(VCD::DumpPortsOn)));
                pairs.push(pair.into_inner());
            }
            Rule::dumpports_command => {
                ids.push(ast.push_and_open(node!(VCD::DumpPorts)));
                pairs.push(pair.into_inner());
            }
            Rule::simulation_time => ast.push(node!(
                VCD::SimulationTime, 
                inner_strs(pair)[0].parse().unwrap()
            )),         
            Rule::scalar_value_change => {
                let vals = inner_strs(pair);
                ast.push(node!(
                    VCD::ValueChange,
                    Scalar,
                    vals[0].parse().unwrap(),
                    vals[1].parse().unwrap()
                ))                
            }
            Rule::vector_value_change => {
                let vals = inner_strs(pair);
                ast.push(node!(
                    VCD::ValueChange,
                    Vector,
                    vals[0].parse().unwrap(),
                    vals[1].parse().unwrap()
                ))                
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
            "../../test_apps/python_app/vendor/vcd/{}.vcd",
            example
        ))
        .expect("cannot read file")
    }

    #[test]
    fn test_example1_to_ast() {
        let _vcd = from_file(Path::new(
            "../../test_apps/python_app/vendor/vcd/example1.vcd",
        ))
        .expect("Imported example1");
    }

    #[test]
    fn test_example2_to_ast() {
        let _vcd = from_file(Path::new(
            "../../test_apps/python_app/vendor/vcd/example2.vcd",
        ))
        .expect("Imported example2");
    }


    #[test]
    fn test_example3_to_ast() {
        let _vcd = from_file(Path::new(
            "../../test_apps/python_app/vendor/vcd/example3.vcd",
        ))
        .expect("Imported example3");
    }

    #[test]
    fn test_example1_can_parse() {
        let txt = read("example1");
        match VCDParser::parse(Rule::vcd_source, &txt) {
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
        match VCDParser::parse(Rule::vcd_source, &txt) {
            Ok(_res) => {} //println!("{:?}", res),
            Err(e) => {
                println!("{}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_example3_can_parse() {
        let txt = read("example3");
        match VCDParser::parse(Rule::vcd_source, &txt) {
            Ok(_res) => {} //println!("{:?}", res),
            Err(e) => {
                println!("{}", e);
                assert_eq!(1, 0);
            }
        }
    }
}
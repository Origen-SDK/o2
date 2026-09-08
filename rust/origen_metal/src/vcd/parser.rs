use super::nodes::VCD;
use crate::ast::Node;
use crate::ast::AST;
use crate::vcd::ValueChangeType::{Scalar, Vector};
use crate::{Error, Result};
use flate2::read::GzDecoder;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Parser)]
#[grammar = "vcd/vcd.pest"]
pub struct VCDParser;

pub fn parse_file(path: &Path) -> Result<Node<VCD>> {
    if path.exists() {
        let gzip = match path.extension() {
            Some(ext) => ext == "gz",
            None => false,
        };

        let mut reader: Box<dyn Read> = if gzip {
            let f = File::open(path)?;
            Box::new(GzDecoder::new(BufReader::new(f)))
        } else {
            let f = File::open(path)?;
            Box::new(BufReader::new(f))
        };

        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        match parse_str(&contents) {
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
            Rule::comment_command => {
                let text = pair.into_inner().next().unwrap().as_str();
                ast.push(node!(VCD::Comment, text.to_string()));
            }
            Rule::date_command => {
                let text = pair.into_inner().next().unwrap().as_str();
                ast.push(node!(VCD::Date, text.to_string()));
            }
            Rule::version_command => {
                let text = pair.into_inner().next().unwrap().as_str();
                ast.push(node!(VCD::Version, text.to_string()));
            }
            Rule::scope_command => {
                let mut p = pair.into_inner();
                let v1 = p.next().unwrap().as_str();
                let v2 = p.next().unwrap().as_str();
                ast.push(node!(VCD::Scope, v1.parse().unwrap(), v2.parse().unwrap()));
            }
            Rule::timescale_command => {
                let mut p = pair.into_inner();
                let num = p.next().unwrap().as_str();
                let unit = p.next().unwrap().as_str();
                ast.push(node!(
                    VCD::TimeScale,
                    num.parse().unwrap(),
                    unit.parse().unwrap()
                ));
            }
            Rule::var_command => {
                let mut p = pair.into_inner();
                let var_type = p.next().unwrap().as_str();
                let var_size = p.next().unwrap().as_str();
                let var_id = p.next().unwrap().as_str();
                let var_ref_pair = p.next().unwrap();
                // Extract just the signal name from var_reference (ref_name)
                let ref_name = var_ref_pair.into_inner().next().unwrap().as_str();
                ast.push(node!(
                    VCD::Var,
                    var_type.parse().unwrap(),
                    var_size.parse().unwrap(),
                    var_id.parse().unwrap(),
                    ref_name.parse().unwrap(),
                    None
                ));
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
            Rule::simulation_time => {
                let ts = pair.into_inner().next().unwrap().as_str();
                ast.push(node!(VCD::SimulationTime, ts.parse().unwrap()));
            }
            Rule::value_change | Rule::vector_value_change => {
                // Delegate to the inner rule
                ids.push(0);
                pairs.push(pair.into_inner());
            }
            Rule::scalar_value_change => {
                // Atomic rule: first char is value, rest is identifier_code
                let text = pair.as_str();
                let val = &text[..1];
                let id = &text[1..];
                ast.push(node!(
                    VCD::ValueChange,
                    Scalar,
                    val.to_string(),
                    id.to_string()
                ));
            }
            Rule::binary_vector_value => {
                let mut p = pair.into_inner();
                let prefix = p.next().unwrap().as_str(); // atomic: "b101010"
                let id = p.next().unwrap().as_str();
                let val = &prefix[1..]; // skip 'b'/'B' prefix
                ast.push(node!(
                    VCD::ValueChange,
                    Vector,
                    val.to_string(),
                    id.to_string()
                ));
            }
            Rule::real_vector_value => {
                let mut p = pair.into_inner();
                let prefix = p.next().unwrap().as_str(); // atomic: "r3.14"
                let id = p.next().unwrap().as_str();
                let val = &prefix[1..]; // skip 'r'/'R' prefix
                ast.push(node!(
                    VCD::ValueChange,
                    Vector,
                    val.to_string(),
                    id.to_string()
                ));
            }
            Rule::port_value_change => {
                let mut p = pair.into_inner();
                let state = p.next().unwrap().as_str();
                let s0: u8 = p.next().unwrap().as_str().parse().unwrap();
                let s1: u8 = p.next().unwrap().as_str().parse().unwrap();
                let id = p.next().unwrap().as_str();
                ast.push(node!(
                    VCD::PortValueChange,
                    state.to_string(),
                    s0,
                    s1,
                    id.to_string()
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
    use std::fs;
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
        VCDParser::parse(Rule::vcd_source, &txt)
            .unwrap_or_else(|e| panic!("Failed to parse example1: {}", e));
    }

    #[test]
    fn test_example2_can_parse() {
        let txt = read("example2");
        VCDParser::parse(Rule::vcd_source, &txt)
            .unwrap_or_else(|e| panic!("Failed to parse example2: {}", e));
    }

    #[test]
    fn test_example3_can_parse() {
        let txt = read("example3");
        VCDParser::parse(Rule::vcd_source, &txt)
            .unwrap_or_else(|e| panic!("Failed to parse example3: {}", e));
    }

    #[test]
    fn test_header_only_vcd() {
        let vcd = "$timescale 1 ns $end\n$enddefinitions $end\n";
        from_file_str(vcd).expect("Header-only VCD should parse");
    }

    #[test]
    fn test_timescale_10() {
        let vcd = "$timescale 10 ps $end\n$enddefinitions $end\n";
        from_file_str(vcd).expect("Timescale 10 should parse");
    }

    #[test]
    fn test_timescale_100() {
        let vcd = "$timescale 100 us $end\n$enddefinitions $end\n";
        from_file_str(vcd).expect("Timescale 100 should parse");
    }

    #[test]
    fn test_extended_vcd_can_parse() {
        let txt = read("example_extended");
        VCDParser::parse(Rule::vcd_source, &txt)
            .unwrap_or_else(|e| panic!("Failed to parse extended VCD: {}", e));
    }

    #[test]
    fn test_extended_vcd_to_ast() {
        let _vcd = from_file(Path::new(
            "../../test_apps/python_app/vendor/vcd/example_extended.vcd",
        ))
        .expect("Imported extended VCD");
    }

    fn from_file_str(vcd: &str) -> crate::Result<Node<VCD>> {
        super::super::from_str(vcd)
    }
}

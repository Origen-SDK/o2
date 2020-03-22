use pest::Parser;
use std::fs;

#[derive(Parser)]
#[grammar = "stil/stil.pest"]
pub struct STILParser;

#[cfg(test)]
mod tests {
    use super::*;

    fn read(example: &str) -> String {
        fs::read_to_string(format!("../../example/vendor/stil/{}.stil", example))
            .expect("cannot read file")
    }

    #[test]
    fn test_example1_can_parse() {
        let txt = read("example1");
        match STILParser::parse(Rule::stil_source, &txt) {
            Ok(_) => {}
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

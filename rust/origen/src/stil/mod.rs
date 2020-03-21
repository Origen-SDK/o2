use pest::Parser;
use std::fs;

#[derive(Parser)]
#[grammar = "stil/stil.pest"]
pub struct STILParser;

#[cfg(test)]
mod tests {
    use super::*;

    fn read(example: &str) -> String {
        fs::read_to_string(format!("../../example/vendor/stil/{}.stil", example)).expect("cannot read file")
    }
    
    #[test]
    fn test_example1_can_parse() {
        let txt = read("example1");
        match STILParser::parse(Rule::stil_source, &txt) {
            Ok(_) => {},
            Err(e) => {
                println!("{}", e);
                assert_eq!(1, 0);
            }
        }
    }
}

//extern crate nom;
//
//#[derive(Default)]
//pub struct ParseError;
//
//impl std::fmt::Display for ParseError {
//	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//		write!(f, "A parsing error occurred.")
//	}
//}
//impl std::fmt::Debug for ParseError {
//	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//		<ParseError as std::fmt::Display>::fmt(self, f)
//	}
//}
//impl std::error::Error for ParseError { }


///// parser combinators are constructed from the bottom up:
///// first we write parsers for the smallest elements (here a space character),
///// then we'll combine them in larger parsers
//fn sp(i: &str) -> nom::IResult<&str, &str> {
//    let chars = " \t\r\n";
//  
//    // nom combinators like `take_while` return a function. That function is the
//    // parser,to which we can pass the input
//    take_while(move |c| chars.contains(c))(i)
//}



//fn not_whitespace(i: &str) -> nom::IResult<&str, &str> {
//    nom::bytes::complete::is_not(" \t")(i)
//}
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//    
//    #[test]
//    fn test_not_whitespace() {
//        assert_eq!(not_whitespace("abcd efg"), Ok((" efg", "abcd")));
//        assert_eq!(not_whitespace("abcd\tefg"), Ok(("\tefg", "abcd")));
//        assert_eq!(not_whitespace(" abcdefg"), Err(nom::Err::Error((" abcdefg", nom::error::ErrorKind::IsNot))));
//    }
//}


//#[cfg(test)]
//mod tests {
//    
//
//}

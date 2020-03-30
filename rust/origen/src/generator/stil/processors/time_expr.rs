//! Resolves all time expressions in the given AST

use crate::generator::ast::*;
use crate::generator::processor::*;
//use crate::Result;

pub struct TimeExpr {
    process_children: bool,
}

impl TimeExpr {
    #[allow(dead_code)]
    pub fn run(node: &Node) -> Result<Node> {
        let mut p = TimeExpr {
            process_children: false,
        };
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor for TimeExpr {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        let result = match &node.attrs {
            Attrs::STILTimeExpr => {
                self.process_children = true;
                let new_node = node.process_children(self)?;
                self.process_children = false;
                Return::Replace(new_node)
            }
            Attrs::STILNumber => {
                if node.children.len() == 1 {
                    Return::Unwrap
                } else {
                    Return::Unmodified
                }
            }
            Attrs::STILAdd => {
                let lhs = node.children[0].process_children(self)?;
                let rhs = node.children[1].process_children(self)?;

                println!("{} + {}", lhs, rhs);
                Return::Unmodified
            }
            // Only recurse inside time expression nodes
            _ => {
                if self.process_children {
                    Return::ProcessChildren
                } else {
                    Return::Unmodified
                }
            }
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::stil::parser::*;
    use pest::Parser;

    fn parse(expr: &str) -> Node {
        let e = &format!("'{}'", expr);
        //println!("{:?}", STILParser::parse(Rule::time_expr, e));
        let mut p = STILParser::parse(Rule::time_expr, e).unwrap();
        to_ast(p.next().unwrap()).unwrap().unwrap()
    }

    #[test]
    fn it_works() {
        let expr = "1+2+3-3+4";
        //let expr = "3*4";
        //let expr = "1-2--3";
        println!("{}", parse(expr));
        //println!("{}", TimeExpr::run(&parse("1+2+3+4")).unwrap());

        //let mut ast = AST::new(node!(Test, "t1".to_string()));
        //ast.push(node!(Cycle, 1, false));
        //ast.push(node!(Comment, 1, "some comment".to_string()));

        //let mut expect = AST::new(node!(Test, "t1".to_string()));
        //expect.push(node!(Cycle, 1, false));
        //expect.push(node!(Comment, 1, "SOME COMMENT".to_string()));

        //assert_eq!(
        //    UpcaseComments::run(&ast.to_node()).expect("Comments upcased"),
        //    expect
        //);
    }
}

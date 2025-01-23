//! Resolves all time expressions in the given AST

use super::super::nodes::STIL;
use crate::ast::Node;
use crate::ast::{Processor, Return};
use crate::Result;
use std::collections::HashMap;

pub struct TimeExpr {
    time_expr_depth: usize,
    params: Option<HashMap<String, Node<STIL>>>,
}

impl TimeExpr {
    #[allow(dead_code)]
    pub fn run(
        node: &Node<STIL>,
        params: Option<HashMap<String, Node<STIL>>>,
    ) -> Result<Node<STIL>> {
        let mut p = TimeExpr {
            time_expr_depth: 0,
            params: params,
        };
        Ok(node.process(&mut p)?.unwrap())
    }

    fn processing_enabled(&self) -> bool {
        self.time_expr_depth > 0
    }
}

impl Processor<STIL> for TimeExpr {
    fn on_processed_node(&mut self, node: &Node<STIL>) -> Result<Return<STIL>> {
        let result = match &node.attrs {
            STIL::TimeExpr => Return::Unwrap,
            STIL::Parens => Return::Unwrap,
            STIL::Add => Return::Replace(match &node.children[0].attrs {
                STIL::Integer(lhs) => match &node.children[1].attrs {
                    STIL::Integer(rhs) => node!(STIL::Integer, lhs + rhs),
                    STIL::Float(rhs) => node!(STIL::Float, *lhs as f64 + rhs),
                    STIL::String(rhs) => node!(STIL::String, format!("{}+{}", lhs, rhs)),
                    _ => unreachable!("{:?}", node.children[1]),
                },
                STIL::Float(lhs) => match &node.children[1].attrs {
                    STIL::Integer(rhs) => node!(STIL::Float, lhs + *rhs as f64),
                    STIL::Float(rhs) => node!(STIL::Float, lhs + rhs),
                    STIL::String(rhs) => node!(STIL::String, format!("{}+{}", lhs, rhs)),
                    _ => unreachable!("{:?}", node.children[1]),
                },
                STIL::String(lhs) => match &node.children[1].attrs {
                    STIL::Integer(rhs) => node!(STIL::String, format!("{}+{}", lhs, rhs)),
                    STIL::Float(rhs) => node!(STIL::String, format!("{}+{}", lhs, rhs)),
                    STIL::String(rhs) => node!(STIL::String, format!("{}+{}", lhs, rhs)),
                    _ => unreachable!("{:?}", node.children[1]),
                },
                _ => unreachable!("{:?}", node.children[0]),
            }),
            STIL::Subtract => Return::Replace(match &node.children[0].attrs {
                STIL::Integer(lhs) => match &node.children[1].attrs {
                    STIL::Integer(rhs) => node!(STIL::Integer, lhs - rhs),
                    STIL::Float(rhs) => node!(STIL::Float, *lhs as f64 - rhs),
                    STIL::String(rhs) => node!(STIL::String, format!("{}-{}", lhs, rhs)),
                    _ => unreachable!("{:?}", node.children[1]),
                },
                STIL::Float(lhs) => match &node.children[1].attrs {
                    STIL::Integer(rhs) => node!(STIL::Float, lhs - *rhs as f64),
                    STIL::Float(rhs) => node!(STIL::Float, lhs - rhs),
                    STIL::String(rhs) => node!(STIL::String, format!("{}-{}", lhs, rhs)),
                    _ => unreachable!("{:?}", node.children[1]),
                },
                STIL::String(lhs) => match &node.children[1].attrs {
                    STIL::Integer(rhs) => node!(STIL::String, format!("{}-{}", lhs, rhs)),
                    STIL::Float(rhs) => node!(STIL::String, format!("{}-{}", lhs, rhs)),
                    STIL::String(rhs) => node!(STIL::String, format!("{}-{}", lhs, rhs)),
                    _ => unreachable!("{:?}", node.children[1]),
                },
                _ => unreachable!("{:?}", node.children[0]),
            }),
            STIL::Multiply => Return::Replace(match &node.children[0].attrs {
                STIL::Integer(lhs) => match &node.children[1].attrs {
                    STIL::Integer(rhs) => node!(STIL::Integer, lhs * rhs),
                    STIL::Float(rhs) => node!(STIL::Float, *lhs as f64 * rhs),
                    STIL::String(rhs) => node!(STIL::String, format!("{}*{}", lhs, rhs)),
                    _ => unreachable!("{:?}", node.children[1]),
                },
                STIL::Float(lhs) => match &node.children[1].attrs {
                    STIL::Integer(rhs) => node!(STIL::Float, lhs * *rhs as f64),
                    STIL::Float(rhs) => node!(STIL::Float, lhs * rhs),
                    STIL::String(rhs) => node!(STIL::String, format!("{}*{}", lhs, rhs)),
                    _ => unreachable!("{:?}", node.children[1]),
                },
                STIL::String(lhs) => match &node.children[1].attrs {
                    STIL::Integer(rhs) => node!(STIL::String, format!("{}*{}", lhs, rhs)),
                    STIL::Float(rhs) => node!(STIL::String, format!("{}*{}", lhs, rhs)),
                    STIL::String(rhs) => node!(STIL::String, format!("{}*{}", lhs, rhs)),
                    _ => unreachable!("{:?}", node.children[1]),
                },
                _ => unreachable!("{:?}", node.children[0]),
            }),
            STIL::Divide => Return::Replace(match &node.children[0].attrs {
                STIL::Integer(lhs) => match &node.children[1].attrs {
                    STIL::Integer(rhs) => node!(STIL::Integer, lhs / rhs),
                    STIL::Float(rhs) => node!(STIL::Float, *lhs as f64 / rhs),
                    STIL::String(rhs) => node!(STIL::String, format!("{}/{}", lhs, rhs)),
                    _ => unreachable!("{:?}", node.children[1]),
                },
                STIL::Float(lhs) => match &node.children[1].attrs {
                    STIL::Integer(rhs) => node!(STIL::Float, lhs / *rhs as f64),
                    STIL::Float(rhs) => node!(STIL::Float, lhs / rhs),
                    STIL::String(rhs) => node!(STIL::String, format!("{}/{}", lhs, rhs)),
                    _ => unreachable!("{:?}", node.children[1]),
                },
                STIL::String(lhs) => match &node.children[1].attrs {
                    STIL::Integer(rhs) => node!(STIL::String, format!("{}/{}", lhs, rhs)),
                    STIL::Float(rhs) => node!(STIL::String, format!("{}/{}", lhs, rhs)),
                    STIL::String(rhs) => node!(STIL::String, format!("{}/{}", lhs, rhs)),
                    _ => unreachable!("{:?}", node.children[1]),
                },
                _ => unreachable!("{:?}", node.children[0]),
            }),
            STIL::NumberWithUnit => Return::Replace(match &node.children[0].attrs {
                STIL::Integer(val) => match &node.children[1].attrs {
                    STIL::EngPrefix(p) => match p.as_str() {
                        "E" => node!(STIL::Float, *val as f64 * 1_000_000_000_000_000_000_f64),
                        "P" => node!(STIL::Float, *val as f64 * 1_000_000_000_000_000_f64),
                        "T" => node!(STIL::Float, *val as f64 * 1_000_000_000_000_f64),
                        "G" => node!(STIL::Float, *val as f64 * 1_000_000_000_f64),
                        "M" => node!(STIL::Float, *val as f64 * 1_000_000_f64),
                        "k" => node!(STIL::Float, *val as f64 * 1_000_f64),
                        "m" => node!(STIL::Float, *val as f64 / 1_000_f64),
                        "u" => node!(STIL::Float, *val as f64 / 1_000_000_f64),
                        "n" => node!(STIL::Float, *val as f64 / 1_000_000_000_f64),
                        "p" => node!(STIL::Float, *val as f64 / 1_000_000_000_000_f64),
                        "f" => node!(STIL::Float, *val as f64 / 1_000_000_000_000_000_f64),
                        "a" => node!(STIL::Float, *val as f64 / 1_000_000_000_000_000_000_f64),
                        _ => unreachable!("Unknown eng prefix '{}'", p),
                    },
                    _ => unreachable!("{:?}", node.children[1]),
                },
                STIL::Float(val) => match &node.children[1].attrs {
                    STIL::EngPrefix(p) => match p.as_str() {
                        "E" => node!(STIL::Float, val * 1_000_000_000_000_000_000_f64),
                        "P" => node!(STIL::Float, val * 1_000_000_000_000_000_f64),
                        "T" => node!(STIL::Float, val * 1_000_000_000_000_f64),
                        "G" => node!(STIL::Float, val * 1_000_000_000_f64),
                        "M" => node!(STIL::Float, val * 1_000_000_f64),
                        "k" => node!(STIL::Float, val * 1_000_f64),
                        "m" => node!(STIL::Float, val / 1_000_f64),
                        "u" => node!(STIL::Float, val / 1_000_000_f64),
                        "n" => node!(STIL::Float, val / 1_000_000_000_f64),
                        "p" => node!(STIL::Float, val / 1_000_000_000_000_f64),
                        "f" => node!(STIL::Float, val / 1_000_000_000_000_000_f64),
                        "a" => node!(STIL::Float, val / 1_000_000_000_000_000_000_f64),
                        _ => unreachable!("Unknown eng prefix '{}'", p),
                    },
                    _ => unreachable!("{:?}", node.children[1]),
                },
                _ => unreachable!("{:?}", node.children[0]),
            }),
            _ => Return::Unmodified,
        };
        Ok(result)
    }

    fn on_node(&mut self, node: &Node<STIL>) -> Result<Return<STIL>> {
        let result = match &node.attrs {
            STIL::TimeExpr => {
                self.time_expr_depth += 1;
                Return::ProcessChildren
            }
            STIL::String(val) => {
                if self.processing_enabled() {
                    match &self.params {
                        None => Return::Unmodified,
                        Some(params) => {
                            if params.contains_key(val) {
                                Return::Replace(params[val].clone())
                            } else {
                                Return::Unmodified
                            }
                        }
                    }
                } else {
                    Return::Unmodified
                }
            }
            _ => {
                // Only process inside time expression nodes
                if self.processing_enabled() {
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
    use crate::ast::Node;
    use crate::stil::nodes::STIL;
    use crate::stil::parser::*;
    use pest::Parser;

    fn parse(expr: &str) -> Node<STIL> {
        let e = &format!("'{}'", expr);
        //println!("{:?}", STILParser::parse(Rule::time_expr, e));
        let mut stil = STILParser::parse(Rule::time_expr, e).unwrap();
        let p = ParseRunner::default();
        p.to_ast(stil.next().unwrap(), None).unwrap().unwrap()
    }

    #[test]
    fn it_works() -> Result<()> {
        let expr = "1+2+3+4";
        let p = TimeExpr::run(&parse(expr), None)?;
        assert_eq!(p, node!(STIL::Integer, 10));
        let expr = "1+2+3-3+4";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(STIL::Integer, 7)
        );
        let expr = "1+2+3-(3+4)";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(STIL::Integer, -1)
        );
        let expr = "1.0+2";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(STIL::Float, 3.0)
        );
        let expr = "1+2+3-3+4+5.234e3";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(STIL::Float, 5241.0)
        );
        let expr = "10*5";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(STIL::Integer, 50)
        );
        let expr = "5ns";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(STIL::Float, 5e-9)
        );
        let expr = "5ns+35ns";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(STIL::Float, 40e-9)
        );
        let expr = "5ns+35e-9";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(STIL::Float, 40e-9)
        );
        let expr = "period-5ns-5ns";
        //println!("{:?}", parse(expr));
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(STIL::String, "period-0.000000005-0.000000005".to_string())
        );
        let expr = "period-5ns-5ns";
        let mut params: HashMap<String, Node<STIL>> = HashMap::new();
        params.insert("period".to_string(), node!(STIL::Float, 40e-9));
        if let STIL::Float(val) = TimeExpr::run(&parse(expr), Some(params)).unwrap().attrs {
            assert_eq!(true, val > 29e-9 && val < 31e-9);
        } else {
            assert_eq!(true, false);
        }
        Ok(())
    }
}

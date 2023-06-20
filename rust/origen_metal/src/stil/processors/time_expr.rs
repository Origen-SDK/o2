//! Resolves all time expressions in the given AST

use super::super::nodes::STIL;
use crate::ast::Node;
use crate::ast::{Processor, Return};
use crate::Result;
use std::collections::HashMap;

pub struct TimeExpr {
    process_children: bool,
    params: Option<HashMap<String, Node<STIL>>>,
}

impl TimeExpr {
    #[allow(dead_code)]
    pub fn run(
        node: &Node<STIL>,
        params: Option<HashMap<String, Node<STIL>>>,
    ) -> Result<Node<STIL>> {
        let mut p = TimeExpr {
            process_children: false,
            params: params,
        };
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor<STIL> for TimeExpr {
    fn on_node(&mut self, node: &Node<STIL>) -> Result<Return<STIL>> {
        let result = match &node.attrs {
            STIL::TimeExpr => {
                self.process_children = true;
                let mut nodes = node.process_children(self)?;
                self.process_children = false;
                Return::Replace(nodes.pop().unwrap())
            }
            STIL::Parens => {
                let mut nodes = node.process_children(self)?;
                Return::Replace(nodes.pop().unwrap())
            }
            STIL::String(val) => match &self.params {
                None => Return::Unmodified,
                Some(params) => {
                    if params.contains_key(val) {
                        Return::Replace(params[val].clone())
                    } else {
                        Return::Unmodified
                    }
                }
            },
            STIL::Add => {
                let nodes = node.process_children(self)?;
                Return::Replace(match &nodes[0].attrs {
                    STIL::Integer(lhs) => match &nodes[1].attrs {
                        STIL::Integer(rhs) => node!(STIL::Integer, lhs + rhs),
                        STIL::Float(rhs) => node!(STIL::Float, *lhs as f64 + rhs),
                        STIL::String(rhs) => node!(STIL::String, format!("{}+{}", lhs, rhs)),
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    STIL::Float(lhs) => match &nodes[1].attrs {
                        STIL::Integer(rhs) => node!(STIL::Float, lhs + *rhs as f64),
                        STIL::Float(rhs) => node!(STIL::Float, lhs + rhs),
                        STIL::String(rhs) => node!(STIL::String, format!("{}+{}", lhs, rhs)),
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    STIL::String(lhs) => match &nodes[1].attrs {
                        STIL::Integer(rhs) => node!(STIL::String, format!("{}+{}", lhs, rhs)),
                        STIL::Float(rhs) => node!(STIL::String, format!("{}+{}", lhs, rhs)),
                        STIL::String(rhs) => node!(STIL::String, format!("{}+{}", lhs, rhs)),
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    _ => unreachable!("{:?}", nodes[0]),
                })
            }
            STIL::Subtract => {
                let nodes = node.process_children(self)?;
                Return::Replace(match &nodes[0].attrs {
                    STIL::Integer(lhs) => match &nodes[1].attrs {
                        STIL::Integer(rhs) => node!(STIL::Integer, lhs - rhs),
                        STIL::Float(rhs) => node!(STIL::Float, *lhs as f64 - rhs),
                        STIL::String(rhs) => node!(STIL::String, format!("{}-{}", lhs, rhs)),
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    STIL::Float(lhs) => match &nodes[1].attrs {
                        STIL::Integer(rhs) => node!(STIL::Float, lhs - *rhs as f64),
                        STIL::Float(rhs) => node!(STIL::Float, lhs - rhs),
                        STIL::String(rhs) => node!(STIL::String, format!("{}-{}", lhs, rhs)),
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    STIL::String(lhs) => match &nodes[1].attrs {
                        STIL::Integer(rhs) => node!(STIL::String, format!("{}-{}", lhs, rhs)),
                        STIL::Float(rhs) => node!(STIL::String, format!("{}-{}", lhs, rhs)),
                        STIL::String(rhs) => node!(STIL::String, format!("{}-{}", lhs, rhs)),
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    _ => unreachable!("{:?}", nodes[0]),
                })
            }
            STIL::Multiply => {
                let nodes = node.process_children(self)?;
                Return::Replace(match &nodes[0].attrs {
                    STIL::Integer(lhs) => match &nodes[1].attrs {
                        STIL::Integer(rhs) => node!(STIL::Integer, lhs * rhs),
                        STIL::Float(rhs) => node!(STIL::Float, *lhs as f64 * rhs),
                        STIL::String(rhs) => node!(STIL::String, format!("{}*{}", lhs, rhs)),
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    STIL::Float(lhs) => match &nodes[1].attrs {
                        STIL::Integer(rhs) => node!(STIL::Float, lhs * *rhs as f64),
                        STIL::Float(rhs) => node!(STIL::Float, lhs * rhs),
                        STIL::String(rhs) => node!(STIL::String, format!("{}*{}", lhs, rhs)),
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    STIL::String(lhs) => match &nodes[1].attrs {
                        STIL::Integer(rhs) => node!(STIL::String, format!("{}*{}", lhs, rhs)),
                        STIL::Float(rhs) => node!(STIL::String, format!("{}*{}", lhs, rhs)),
                        STIL::String(rhs) => node!(STIL::String, format!("{}*{}", lhs, rhs)),
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    _ => unreachable!("{:?}", nodes[0]),
                })
            }
            STIL::Divide => {
                let nodes = node.process_children(self)?;
                Return::Replace(match &nodes[0].attrs {
                    STIL::Integer(lhs) => match &nodes[1].attrs {
                        STIL::Integer(rhs) => node!(STIL::Integer, lhs / rhs),
                        STIL::Float(rhs) => node!(STIL::Float, *lhs as f64 / rhs),
                        STIL::String(rhs) => node!(STIL::String, format!("{}/{}", lhs, rhs)),
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    STIL::Float(lhs) => match &nodes[1].attrs {
                        STIL::Integer(rhs) => node!(STIL::Float, lhs / *rhs as f64),
                        STIL::Float(rhs) => node!(STIL::Float, lhs / rhs),
                        STIL::String(rhs) => node!(STIL::String, format!("{}/{}", lhs, rhs)),
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    STIL::String(lhs) => match &nodes[1].attrs {
                        STIL::Integer(rhs) => node!(STIL::String, format!("{}/{}", lhs, rhs)),
                        STIL::Float(rhs) => node!(STIL::String, format!("{}/{}", lhs, rhs)),
                        STIL::String(rhs) => node!(STIL::String, format!("{}/{}", lhs, rhs)),
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    _ => unreachable!("{:?}", nodes[0]),
                })
            }
            STIL::NumberWithUnit => {
                let nodes = node.process_children(self)?;
                Return::Replace(match nodes[0].attrs {
                    STIL::Integer(val) => match &nodes[1].attrs {
                        STIL::EngPrefix(p) => match p.as_str() {
                            "E" => node!(STIL::Float, val as f64 * 1_000_000_000_000_000_000_f64),
                            "P" => node!(STIL::Float, val as f64 * 1_000_000_000_000_000_f64),
                            "T" => node!(STIL::Float, val as f64 * 1_000_000_000_000_f64),
                            "G" => node!(STIL::Float, val as f64 * 1_000_000_000_f64),
                            "M" => node!(STIL::Float, val as f64 * 1_000_000_f64),
                            "k" => node!(STIL::Float, val as f64 * 1_000_f64),
                            "m" => node!(STIL::Float, val as f64 / 1_000_f64),
                            "u" => node!(STIL::Float, val as f64 / 1_000_000_f64),
                            "n" => node!(STIL::Float, val as f64 / 1_000_000_000_f64),
                            "p" => node!(STIL::Float, val as f64 / 1_000_000_000_000_f64),
                            "f" => node!(STIL::Float, val as f64 / 1_000_000_000_000_000_f64),
                            "a" => node!(STIL::Float, val as f64 / 1_000_000_000_000_000_000_f64),
                            _ => unreachable!("Unknown eng prefix '{}'", p),
                        },
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    STIL::Float(val) => match &nodes[1].attrs {
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
                        _ => unreachable!("{:?}", nodes[1]),
                    },
                    _ => unreachable!("{:?}", nodes[0]),
                })
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
    use crate::ast::Node;
    use crate::stil::nodes::STIL;
    use crate::stil::parser::*;
    use pest::Parser;

    fn parse(expr: &str) -> Node<STIL> {
        let e = &format!("'{}'", expr);
        //println!("{:?}", STILParser::parse(Rule::time_expr, e));
        let mut p = STILParser::parse(Rule::time_expr, e).unwrap();
        to_ast(p.next().unwrap(), None).unwrap().unwrap()
    }

    #[test]
    fn it_works() {
        let expr = "1+2+3+4";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(STIL::Integer, 10)
        );
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
    }
}

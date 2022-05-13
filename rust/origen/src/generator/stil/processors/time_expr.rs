//! Resolves all time expressions in the given AST

use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::Result;
use std::collections::HashMap;

pub struct TimeExpr {
    process_children: bool,
    params: Option<HashMap<String, Node>>,
}

impl TimeExpr {
    #[allow(dead_code)]
    pub fn run(node: &Node, params: Option<HashMap<String, Node>>) -> Result<Node> {
        let mut p = TimeExpr {
            process_children: false,
            params: params,
        };
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor for TimeExpr {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        let result = match &node.attrs {
            Attrs::STILTimeExpr => {
                self.process_children = true;
                let mut nodes = node.process_children(self)?;
                self.process_children = false;
                Return::Replace(nodes.pop().unwrap())
            }
            Attrs::STILParens => {
                let mut nodes = node.process_children(self)?;
                Return::Replace(nodes.pop().unwrap())
            }
            Attrs::String(val) => match &self.params {
                None => Return::Unmodified,
                Some(params) => {
                    if params.contains_key(val) {
                        Return::Replace(params[val].clone())
                    } else {
                        Return::Unmodified
                    }
                }
            },
            Attrs::STILAdd => {
                let nodes = node.process_children(self)?;
                Return::Replace(match &nodes[0].attrs {
                    Attrs::Integer(lhs) => match &nodes[1].attrs {
                        Attrs::Integer(rhs) => node!(Integer, lhs + rhs),
                        Attrs::Float(rhs) => node!(Float, *lhs as f64 + rhs),
                        Attrs::String(rhs) => node!(String, format!("{}+{}", lhs, rhs)),
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    Attrs::Float(lhs) => match &nodes[1].attrs {
                        Attrs::Integer(rhs) => node!(Float, lhs + *rhs as f64),
                        Attrs::Float(rhs) => node!(Float, lhs + rhs),
                        Attrs::String(rhs) => node!(String, format!("{}+{}", lhs, rhs)),
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    Attrs::String(lhs) => match &nodes[1].attrs {
                        Attrs::Integer(rhs) => node!(String, format!("{}+{}", lhs, rhs)),
                        Attrs::Float(rhs) => node!(String, format!("{}+{}", lhs, rhs)),
                        Attrs::String(rhs) => node!(String, format!("{}+{}", lhs, rhs)),
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    _ => unreachable!("{}", format!("{:?}", nodes[0])),
                })
            }
            Attrs::STILSubtract => {
                let nodes = node.process_children(self)?;
                Return::Replace(match &nodes[0].attrs {
                    Attrs::Integer(lhs) => match &nodes[1].attrs {
                        Attrs::Integer(rhs) => node!(Integer, lhs - rhs),
                        Attrs::Float(rhs) => node!(Float, *lhs as f64 - rhs),
                        Attrs::String(rhs) => node!(String, format!("{}-{}", lhs, rhs)),
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    Attrs::Float(lhs) => match &nodes[1].attrs {
                        Attrs::Integer(rhs) => node!(Float, lhs - *rhs as f64),
                        Attrs::Float(rhs) => node!(Float, lhs - rhs),
                        Attrs::String(rhs) => node!(String, format!("{}-{}", lhs, rhs)),
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    Attrs::String(lhs) => match &nodes[1].attrs {
                        Attrs::Integer(rhs) => node!(String, format!("{}-{}", lhs, rhs)),
                        Attrs::Float(rhs) => node!(String, format!("{}-{}", lhs, rhs)),
                        Attrs::String(rhs) => node!(String, format!("{}-{}", lhs, rhs)),
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    _ => unreachable!("{}", format!("{:?}", nodes[0])),
                })
            }
            Attrs::STILMultiply => {
                let nodes = node.process_children(self)?;
                Return::Replace(match &nodes[0].attrs {
                    Attrs::Integer(lhs) => match &nodes[1].attrs {
                        Attrs::Integer(rhs) => node!(Integer, lhs * rhs),
                        Attrs::Float(rhs) => node!(Float, *lhs as f64 * rhs),
                        Attrs::String(rhs) => node!(String, format!("{}*{}", lhs, rhs)),
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    Attrs::Float(lhs) => match &nodes[1].attrs {
                        Attrs::Integer(rhs) => node!(Float, lhs * *rhs as f64),
                        Attrs::Float(rhs) => node!(Float, lhs * rhs),
                        Attrs::String(rhs) => node!(String, format!("{}*{}", lhs, rhs)),
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    Attrs::String(lhs) => match &nodes[1].attrs {
                        Attrs::Integer(rhs) => node!(String, format!("{}*{}", lhs, rhs)),
                        Attrs::Float(rhs) => node!(String, format!("{}*{}", lhs, rhs)),
                        Attrs::String(rhs) => node!(String, format!("{}*{}", lhs, rhs)),
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    _ => unreachable!("{}", format!("{:?}", nodes[0])),
                })
            }
            Attrs::STILDivide => {
                let nodes = node.process_children(self)?;
                Return::Replace(match &nodes[0].attrs {
                    Attrs::Integer(lhs) => match &nodes[1].attrs {
                        Attrs::Integer(rhs) => node!(Integer, lhs / rhs),
                        Attrs::Float(rhs) => node!(Float, *lhs as f64 / rhs),
                        Attrs::String(rhs) => node!(String, format!("{}/{}", lhs, rhs)),
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    Attrs::Float(lhs) => match &nodes[1].attrs {
                        Attrs::Integer(rhs) => node!(Float, lhs / *rhs as f64),
                        Attrs::Float(rhs) => node!(Float, lhs / rhs),
                        Attrs::String(rhs) => node!(String, format!("{}/{}", lhs, rhs)),
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    Attrs::String(lhs) => match &nodes[1].attrs {
                        Attrs::Integer(rhs) => node!(String, format!("{}/{}", lhs, rhs)),
                        Attrs::Float(rhs) => node!(String, format!("{}/{}", lhs, rhs)),
                        Attrs::String(rhs) => node!(String, format!("{}/{}", lhs, rhs)),
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    _ => unreachable!("{}", format!("{:?}", nodes[0])),
                })
            }
            Attrs::STILNumberWithUnit => {
                let nodes = node.process_children(self)?;
                Return::Replace(match nodes[0].attrs {
                    Attrs::Integer(val) => match &nodes[1].attrs {
                        Attrs::STILEngPrefix(p) => match p.as_str() {
                            "E" => node!(Float, val as f64 * 1_000_000_000_000_000_000_f64),
                            "P" => node!(Float, val as f64 * 1_000_000_000_000_000_f64),
                            "T" => node!(Float, val as f64 * 1_000_000_000_000_f64),
                            "G" => node!(Float, val as f64 * 1_000_000_000_f64),
                            "M" => node!(Float, val as f64 * 1_000_000_f64),
                            "k" => node!(Float, val as f64 * 1_000_f64),
                            "m" => node!(Float, val as f64 / 1_000_f64),
                            "u" => node!(Float, val as f64 / 1_000_000_f64),
                            "n" => node!(Float, val as f64 / 1_000_000_000_f64),
                            "p" => node!(Float, val as f64 / 1_000_000_000_000_f64),
                            "f" => node!(Float, val as f64 / 1_000_000_000_000_000_f64),
                            "a" => node!(Float, val as f64 / 1_000_000_000_000_000_000_f64),
                            _ => unreachable!("{}", format!("Unknown eng prefix '{}'", p)),
                        },
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    Attrs::Float(val) => match &nodes[1].attrs {
                        Attrs::STILEngPrefix(p) => match p.as_str() {
                            "E" => node!(Float, val * 1_000_000_000_000_000_000_f64),
                            "P" => node!(Float, val * 1_000_000_000_000_000_f64),
                            "T" => node!(Float, val * 1_000_000_000_000_f64),
                            "G" => node!(Float, val * 1_000_000_000_f64),
                            "M" => node!(Float, val * 1_000_000_f64),
                            "k" => node!(Float, val * 1_000_f64),
                            "m" => node!(Float, val / 1_000_f64),
                            "u" => node!(Float, val / 1_000_000_f64),
                            "n" => node!(Float, val / 1_000_000_000_f64),
                            "p" => node!(Float, val / 1_000_000_000_000_f64),
                            "f" => node!(Float, val / 1_000_000_000_000_000_f64),
                            "a" => node!(Float, val / 1_000_000_000_000_000_000_f64),
                            _ => unreachable!("{}", format!("Unknown eng prefix '{}'", p)),
                        },
                        _ => unreachable!("{}", format!("{:?}", nodes[1])),
                    },
                    _ => unreachable!("{}", format!("{:?}", nodes[0])),
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
        let expr = "1+2+3+4";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(Integer, 10)
        );
        let expr = "1+2+3-3+4";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(Integer, 7)
        );
        let expr = "1+2+3-(3+4)";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(Integer, -1)
        );
        let expr = "1.0+2";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(Float, 3.0)
        );
        let expr = "1+2+3-3+4+5.234e3";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(Float, 5241.0)
        );
        let expr = "10*5";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(Integer, 50)
        );
        let expr = "5ns";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(Float, 5e-9)
        );
        let expr = "5ns+35ns";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(Float, 40e-9)
        );
        let expr = "5ns+35e-9";
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(Float, 40e-9)
        );
        let expr = "period-5ns-5ns";
        //println!("{:?}", parse(expr));
        assert_eq!(
            TimeExpr::run(&parse(expr), None).unwrap(),
            node!(String, "period-0.000000005-0.000000005".to_string())
        );
        let expr = "period-5ns-5ns";
        let mut params: HashMap<String, Node> = HashMap::new();
        params.insert("period".to_string(), node!(Float, 40e-9));
        if let Attrs::Float(val) = TimeExpr::run(&parse(expr), Some(params)).unwrap().attrs {
            assert_eq!(true, val > 29e-9 && val < 31e-9);
        } else {
            assert_eq!(true, false);
        }
    }
}

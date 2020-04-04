//! Adds info from the STIL to the DUT, e.g. pin, pin groups.

use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::Result;

pub struct DUTBuilder {}

impl DUTBuilder {
    #[allow(dead_code)]
    pub fn run(node: &Node) -> Result<Node> {
        let mut p = DUTBuilder {};
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor for DUTBuilder {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        let result = match &node.attrs {
            Attrs::STILSignal(name, ptype) => {
                //match ptype {
                //    Inout => (),
                //    Out => (),
                //    In => (),
                //    Supply => (),
                //    Pseudo => (),
                //}
                Return::Unmodified
            }
            Attrs::STILSignals => Return::ProcessChildren,
            _ => Return::Unmodified,
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::STIL;
    //use super::*;
    use std::path::Path;

    #[test]
    fn it_works() {
        let _stil = STIL::from_file(Path::new("../../example/vendor/stil/example1.stil"))
            .expect("Imported example1");
        //let expr = "1+2+3+4";
        //assert_eq!(
        //    TimeExpr::run(&parse(expr), None).unwrap(),
        //    node!(Integer, 10)
        //);
    }
}

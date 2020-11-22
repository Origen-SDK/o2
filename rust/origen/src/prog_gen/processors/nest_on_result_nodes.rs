use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::prog_gen::FlowID;
use indexmap::IndexMap;

/// This nests OnFailed and OnPassed nodes into their parent tests (or groups)
///
/// Original:
///     PGMTest(4, FlowID { id: "3" })
///     PGMOnFailed(FlowID { id: "3" })
///         PGMBin(100, Some(1100), Bad)
///     PGMTest(6, FlowID { id: "7" })
///     PGMOnFailed(FlowID { id: "7" })
///         PGMBin(100, Some(1100), Bad)
///
/// Output:
///     PGMTest(4, FlowID { id: "3" })
///         PGMOnFailed(FlowID { id: "3" })
///             PGMBin(100, Some(1100), Bad)
///     PGMTest(6, FlowID { id: "7" })
///         PGMOnFailed(FlowID { id: "7" })
///             PGMBin(100, Some(1100), Bad)
pub struct NestOnResultNodes {
    nodes: IndexMap<FlowID, Node>,
    pass: usize,
}

impl NestOnResultNodes {
    #[allow(dead_code)]
    pub fn run(node: &Node) -> Result<Node> {
        let mut p = NestOnResultNodes {
            nodes: IndexMap::new(),
            pass: 0,
        };
        // The first pass strips out all OnPassed and OnFailed nodes and holds them
        let ast = node.process(&mut p)?.unwrap();
        p.pass = 1;
        // Then the final pass re-inserts them in the right place
        let ast = ast.process(&mut p)?.unwrap();
        Ok(ast)
    }
}

impl Processor for NestOnResultNodes {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::PGMOnFailed(fid) | Attrs::PGMOnPassed(fid) => {
                if self.pass == 0 {
                    let n = { node.process_and_update_children(self)? };
                    self.nodes.insert(fid.to_owned(), n);
                    Ok(Return::None)
                } else {
                    Ok(Return::ProcessChildren)
                }
            }
            Attrs::PGMTest(_, fid) | Attrs::PGMTestStr(_, fid) | Attrs::PGMCz(_, _, fid) => {
                if self.pass == 1 && self.nodes.contains_key(fid) {
                    let n = { self.nodes.remove(fid).unwrap() };
                    let mut nodes = vec![n.process_and_update_children(self)?];
                    nodes.append(&mut node.process_children(self)?);
                    Ok(Return::ReplaceChildren(nodes))
                } else {
                    Ok(Return::ProcessChildren)
                }
            }
            Attrs::PGMGroup(_, _, _, fid) => {
                if let Some(fid) = fid {
                    if self.pass == 1 && self.nodes.contains_key(fid) {
                        let n = { self.nodes.remove(fid).unwrap() };
                        let mut nodes = vec![n.process_and_update_children(self)?];
                        nodes.append(&mut node.process_children(self)?);
                        Ok(Return::ReplaceChildren(nodes))
                    } else {
                        Ok(Return::ProcessChildren)
                    }
                } else {
                    Ok(Return::ProcessChildren)
                }
            }
            _ => Ok(Return::ProcessChildren),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NestOnResultNodes;
    use crate::prog_gen::{BinType, FlowID};
    use crate::Result;

    #[test]
    fn it_works() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1)),
            node!(PGMTest, 2, FlowID::from_int(2)),
            node!(PGMOnFailed, FlowID::from_int(2) =>
                node!(PGMBin, 10, None, BinType::Bad)
            ),
            node!(PGMTest, 3, FlowID::from_int(3)),
            node!(PGMOnPassed, FlowID::from_int(3) =>
                node!(PGMTest, 4, FlowID::from_int(4)),
                node!(PGMOnFailed, FlowID::from_int(4) =>
                    node!(PGMBin, 10, None, BinType::Bad)
                )
            )
        );

        let expected = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1)),
            node!(PGMTest, 2, FlowID::from_int(2) =>
                node!(PGMOnFailed, FlowID::from_int(2) =>
                    node!(PGMBin, 10, None, BinType::Bad)
                )
            ),
            node!(PGMTest, 3, FlowID::from_int(3) =>
                node!(PGMOnPassed, FlowID::from_int(3) =>
                    node!(PGMTest, 4, FlowID::from_int(4) =>
                        node!(PGMOnFailed, FlowID::from_int(4) =>
                            node!(PGMBin, 10, None, BinType::Bad)
                        )
                    )
                )
            )
        );

        assert_eq!(NestOnResultNodes::run(&input)?, expected);
        Ok(())
    }
}

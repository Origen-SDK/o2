use crate::ast::{Node, Processor, Return};
use crate::prog_gen::{FlowID, PGM};
use crate::Result;
use indexmap::IndexMap;

/// This nests OnFailed and OnPassed nodes into their parent tests (or groups). When a test has
/// more than one of a kind, e.g. one from its bin and one from continue_on_fail, they are merged
/// into one node, keeping the order of their contents.
///
/// Input:
///     PGMTest(4, FlowID { id: "3" })
///     PGMOnFailed(FlowID { id: "3" })
///         PGMBin(100, Some(1100), Bad)
///     PGMTest(6, FlowID { id: "7" })
///     PGMOnFailed(FlowID { id: "7" })
///         PGMBin(100, Some(1100), Bad)
///     PGMOnFailed(FlowID { id: "7" })
///         PGMContinue
///     PGMOnPassed(FlowID { id: "7" })
///         PGMTest(8, FlowID { id: "9" })
///
/// Output:
///     PGMTest(4, FlowID { id: "3" })
///         PGMOnFailed(FlowID { id: "3" })
///             PGMBin(100, Some(1100), Bad)
///     PGMTest(6, FlowID { id: "7" })
///         PGMOnFailed(FlowID { id: "7" })
///             PGMBin(100, Some(1100), Bad)
///             PGMContinue
///         PGMOnPassed(FlowID { id: "7" })
///             PGMTest(8, FlowID { id: "9" })
pub struct NestOnResultNodes {
    /// The OnFailed and OnPassed nodes of each test or group, at most one of each kind
    nodes: IndexMap<FlowID, Vec<Node<PGM>>>,
    pass: usize,
}

pub fn run(node: &Node<PGM>) -> Result<Node<PGM>> {
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

impl Processor<PGM> for NestOnResultNodes {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        match &node.attrs {
            PGM::OnFailed(fid) | PGM::OnPassed(fid) => {
                if self.pass == 0 {
                    let n = { node.process_and_update_children(self)? };
                    let nodes = self.nodes.entry(fid.to_owned()).or_default();
                    if let Some(existing) = nodes.iter_mut().find(|e| e.attrs == n.attrs) {
                        existing.children.extend(n.children);
                    } else {
                        nodes.push(n);
                    }
                    Ok(Return::None)
                } else {
                    Ok(Return::ProcessChildren)
                }
            }
            PGM::Test(_, fid) | PGM::TestStr(_, fid, _, _, _) | PGM::Cz(_, _, fid) => {
                if self.pass == 1 && self.nodes.contains_key(fid) {
                    let mut nodes = vec![];
                    for n in self.nodes.shift_remove(fid).unwrap() {
                        nodes.push(n.process_and_update_children(self)?);
                    }
                    nodes.append(&mut node.process_children(self)?);
                    Ok(Return::ReplaceChildren(nodes))
                } else {
                    Ok(Return::ProcessChildren)
                }
            }
            PGM::Group(_, _, _, fid) => {
                if let Some(fid) = fid {
                    if self.pass == 1 && self.nodes.contains_key(fid) {
                        let mut nodes = vec![];
                        for n in self.nodes.shift_remove(fid).unwrap() {
                            nodes.push(n.process_and_update_children(self)?);
                        }
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
    use super::run;
    use crate::prog_gen::{BinType, FlowID, PGM};
    use crate::Result;

    #[test]
    fn it_works() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Test, 2, FlowID::from_int(2)),
            node!(PGM::OnFailed, FlowID::from_int(2) =>
                node!(PGM::Bin, 10, None, BinType::Bad)
            ),
            node!(PGM::Test, 3, FlowID::from_int(3)),
            node!(PGM::OnPassed, FlowID::from_int(3) =>
                node!(PGM::Test, 4, FlowID::from_int(4)),
                node!(PGM::OnFailed, FlowID::from_int(4) =>
                    node!(PGM::Bin, 10, None, BinType::Bad)
                )
            )
        );

        let expected = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Test, 2, FlowID::from_int(2) =>
                node!(PGM::OnFailed, FlowID::from_int(2) =>
                    node!(PGM::Bin, 10, None, BinType::Bad)
                )
            ),
            node!(PGM::Test, 3, FlowID::from_int(3) =>
                node!(PGM::OnPassed, FlowID::from_int(3) =>
                    node!(PGM::Test, 4, FlowID::from_int(4) =>
                        node!(PGM::OnFailed, FlowID::from_int(4) =>
                            node!(PGM::Bin, 10, None, BinType::Bad)
                        )
                    )
                )
            )
        );

        assert_eq!(run(&input)?, expected);
        Ok(())
    }

    #[test]
    fn it_merges_the_result_blocks_of_a_test() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::OnFailed, FlowID::from_int(1) =>
                node!(PGM::Bin, 10, None, BinType::Bad)
            ),
            node!(PGM::OnFailed, FlowID::from_int(1) =>
                node!(PGM::Continue)
            ),
            node!(PGM::OnPassed, FlowID::from_int(1) =>
                node!(PGM::Test, 2, FlowID::from_int(2))
            )
        );

        let expected = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1) =>
                node!(PGM::OnFailed, FlowID::from_int(1) =>
                    node!(PGM::Bin, 10, None, BinType::Bad),
                    node!(PGM::Continue)
                ),
                node!(PGM::OnPassed, FlowID::from_int(1) =>
                    node!(PGM::Test, 2, FlowID::from_int(2))
                )
            )
        );

        assert_eq!(run(&input)?, expected);
        Ok(())
    }
}

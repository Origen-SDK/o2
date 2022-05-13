use crate::prog_gen::{GroupType, PGM};
use crate::Result;
use origen_metal::ast::{Node, Processor, Return};

/// Implements continue on a fail branch for V93K by removing any bin nodes that are
/// siblings of continue nodes. The continue nodes are also removed in the process since
/// they have now served their function.
pub struct ContinueImplementer {
    cont: bool,
}

pub fn run(node: &Node<PGM>) -> Result<Node<PGM>> {
    let mut p = ContinueImplementer { cont: false };
    let ast = node.process(&mut p)?.unwrap();
    Ok(ast)
}

impl Processor<PGM> for ContinueImplementer {
    fn on_node(&mut self, node: &Node<PGM>) -> origen_metal::Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::OnFailed(_) => {
                let orig = self.cont;
                if node
                    .children
                    .iter()
                    .any(|n| matches!(n.attrs, PGM::Continue))
                {
                    self.cont = true;
                }
                let children = node.process_and_box_children(self)?;
                self.cont = orig;
                if children.is_empty() {
                    Return::None
                } else {
                    Return::Replace(node.updated(None, Some(children), None))
                }
            }
            PGM::Continue => Return::None,
            PGM::Bin(_, _, _) => {
                if self.cont {
                    Return::None
                } else {
                    Return::Unmodified
                }
            }
            PGM::Group(_, _, kind, _) => match kind {
                GroupType::Flow => {
                    let on_fail = node
                        .children
                        .iter()
                        .find(|n| matches!(n.attrs, PGM::OnFailed(_)));
                    if let Some(on_fail) = on_fail {
                        let orig = self.cont;
                        if on_fail
                            .children
                            .iter()
                            .any(|n| matches!(n.attrs, PGM::Continue))
                        {
                            self.cont = true;
                        }
                        let children = node.process_and_box_children(self)?;
                        self.cont = orig;
                        Return::Replace(node.updated(None, Some(children), None))
                    } else {
                        Return::ProcessChildren
                    }
                }
                _ => Return::ProcessChildren,
            },
            _ => Return::ProcessChildren,
        })
    }
}

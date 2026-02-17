use crate::prog_gen::{Model, PGM, LimitSelector};
use crate::prog_gen::supported_testers::SupportedTester;
use crate::Result;
use crate::ast::{Node, Processor, Return};

/// This extracts all definitions for tests, test invocations, pattern sets, bins, etc.
/// and converts them into a program model which is returned.
/// The resultant AST has most of the associated nodes removed but is otherwise unchanged.
/// The model is not considered finalized until after the flow generator for the specific ATE
/// target has run, at that point any ATE-specific extraction into the model will be complete,
/// e.g. to extract pattern refernces made by test objects.
pub struct ExtractToModel {
    model: Model,
    tester: SupportedTester,
    pass: usize,
}

pub fn run(node: &Node<PGM>, tester: SupportedTester, model: Model) -> Result<(Node<PGM>, Model)> {
    let mut p = ExtractToModel {
        model: model,
        tester: tester,
        pass: 0,
    };
    let ast = node.process(&mut p)?.unwrap();
    p.pass = 1;
    let ast = ast.process(&mut p)?.unwrap();
    Ok((ast, p.model))
}

impl Processor<PGM> for ExtractToModel {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        // On first pass extract all tests and invocations and link them together. This is to ensure that any attribute
        // assignments can be checked against both the invocation and its assigned test
        if self.pass == 0 {
            Ok(match &node.attrs {
                PGM::ResourcesFilename(name, kind) => {
                    self.model.set_resources_filename(name.to_owned(), kind);
                    Return::Unmodified
                }
                PGM::Flow(name) => {
                    self.model.create_flow(name)?;
                    Return::ProcessChildren
                }
                PGM::DefTest(id, name, _, library_name, template_name) => {
                    trace!(
                        self.model.add_test_from_template(
                            *id,
                            name.to_owned(),
                            &self.tester,
                            template_name,
                            Some(library_name),
                        ),
                        node
                    );
                    Return::None
                }
                PGM::DefTestInv(id, name, _) => {
                    trace!(
                        self.model
                            .add_test_invocation(*id, name.to_owned(), &self.tester),
                        node
                    );
                    Return::None
                }
                PGM::AssignTestToInv(inv_id, test_id) => {
                    trace!(self.model.assign_test_to_inv(*inv_id, *test_id), node);
                    Return::None
                }
                _ => Return::ProcessChildren,
            })
        } else {
            Ok(match &node.attrs {
                PGM::ResourcesFilename(name, kind) => {
                    self.model.set_resources_filename(name.to_owned(), kind);
                    Return::Unmodified
                }
                PGM::SetAttr(id, name, value, allow_missing) => {
                    trace!(self.model.set_test_attr(*id, name, value.to_owned(), *allow_missing), node);
                    Return::None
                }
                PGM::SetLimit(test_id, inv_id, selector, value) => {
                    let t = {
                        if let Some(id) = test_id {
                            self.model.tests.get_mut(id)
                        } else if let Some(id) = inv_id {
                            self.model.test_invocations.get_mut(id)
                        } else {
                            None
                        }
                    };
                    if let Some(t) = t {
                        match selector {
                            LimitSelector::Hi => t.hi_limit = value.to_owned(),
                            LimitSelector::Lo => t.lo_limit = value.to_owned(),
                        }
                    }
                    Return::None
                }
                _ => Return::ProcessChildren,
            })
        }
    }
}

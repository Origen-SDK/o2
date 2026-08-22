use crate::ast::{Node, Processor, Return};
use crate::prog_gen::supported_testers::SupportedTester;
use crate::prog_gen::{LimitSelector, Model, PGM};
use crate::Result;

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
    test_instance_groups: Vec<String>,
}

pub fn run(node: &Node<PGM>, tester: SupportedTester, model: Model) -> Result<(Node<PGM>, Model)> {
    let mut p = ExtractToModel {
        model: model,
        tester: tester,
        pass: 0,
        test_instance_groups: vec![],
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
                PGM::Group(name, _, crate::prog_gen::GroupType::Test, _) => {
                    self.test_instance_groups.push(name.clone());
                    let updated = node.process_and_update_children(self)?;
                    self.test_instance_groups.pop();
                    Return::Replace(updated)
                }
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
                    if let Some(group) = self.test_instance_groups.last() {
                        self.model.add_test_to_instance_group(group, *id);
                    }
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
                PGM::DefSubTest(test_id, name, number, lo_limit, hi_limit) => {
                    trace!(
                        self.model.add_sub_test(
                            *test_id,
                            name.clone(),
                            *number,
                            lo_limit.clone(),
                            hi_limit.clone(),
                        ),
                        node
                    );
                    Return::None
                }
                PGM::DefTestCollectionItem(
                    id,
                    parent_id,
                    collection_name,
                    instance_id,
                    allow_missing,
                ) => {
                    trace!(
                        self.model.add_test_collection_item(
                            *parent_id,
                            *id,
                            collection_name,
                            instance_id,
                            *allow_missing,
                        ),
                        node
                    );
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
                    trace!(
                        self.model
                            .set_test_attr(*id, name, value.to_owned(), *allow_missing),
                        node
                    );
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

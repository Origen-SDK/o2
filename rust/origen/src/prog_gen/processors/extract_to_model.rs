use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::prog_gen::Model;
use crate::testers::SupportedTester;

/// This extracts all definitions for tests, test invocations, pattern sets, bins, etc.
/// and converts them into a program model which is returned.
/// The resultant AST has most of the associated nodes removed but is otherwise unchanged.
pub struct ExtractToModel {
    model: Model,
    tester: SupportedTester,
    pass: usize,
}

pub fn run(node: &Node, tester: SupportedTester, flow_name: &str) -> Result<(Node, Model)> {
    let mut p = ExtractToModel {
        model: Model::new(flow_name.to_string()),
        tester: tester,
        pass: 0,
    };
    let ast = node.process(&mut p)?.unwrap();
    p.pass = 1;
    let ast = ast.process(&mut p)?.unwrap();
    Ok((ast, p.model))
}

impl Processor for ExtractToModel {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        // On first pass extract all tests and invocations and link them together. This is to ensure that any attribute
        // assignments can be checked against both the invocation and its assigned test
        if self.pass == 0 {
            match &node.attrs {
                Attrs::PGMDefTest(id, name, _, library_name, template_name) => {
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
                    Ok(Return::None)
                }
                Attrs::PGMDefTestInv(id, name, _) => {
                    trace!(
                        self.model
                            .add_test_invocation(*id, name.to_owned(), &self.tester),
                        node
                    );
                    Ok(Return::None)
                }
                Attrs::PGMAssignTestToInv(inv_id, test_id) => {
                    trace!(self.model.assign_test_to_inv(*inv_id, *test_id), node);
                    Ok(Return::None)
                }
                //Attrs::PGMPatternGroup(id, name, _, kind) => Ok(Return::None),
                //Attrs::PGMPushPattern(id, name, start_label) => Ok(Return::None),
                //// These will be left in the AST for later consumption by the flow, but they also function
                //// as un-official bin definitions.
                //// Any official bin defs in the AST for the same bin number will be combined with the model
                //// defintion created from here.
                //Attrs::PGMBin(hardbin, softbin, kind) => Ok(Return::Unmodified),
                _ => Ok(Return::ProcessChildren),
            }
        } else {
            match &node.attrs {
                Attrs::PGMSetAttr(id, name, value) => {
                    trace!(self.model.set_test_attr(*id, name, value.to_owned()), node);
                    Ok(Return::None)
                }
                _ => Ok(Return::ProcessChildren),
            }
        }
    }
}

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
}

impl ExtractToModel {
    pub fn run(node: &Node, tester: SupportedTester, flow_name: &str) -> Result<(Node, Model)> {
        let mut p = ExtractToModel {
            model: Model::new(flow_name.to_string()),
            tester: tester,
        };
        let ast = node.process(&mut p)?.unwrap();
        Ok((ast, p.model))
    }
}

impl Processor for ExtractToModel {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::PGMDefTest(id, name, _, library_name, template_name) => {
                self.model.add_test_from_template(
                    *id,
                    name.to_owned(),
                    &self.tester,
                    template_name,
                    Some(library_name),
                )?;
                Ok(Return::None)
            }
            Attrs::PGMDefTestInv(id, name, _) => {
                self.model
                    .add_test_invocation(*id, name.to_owned(), &self.tester)?;
                Ok(Return::None)
            }
            //Attrs::PGMSetAttr(id, name, value) => Ok(Return::None),
            //Attrs::PGMPatternGroup(id, name, _, kind) => Ok(Return::None),
            //Attrs::PGMPushPattern(id, name, start_label) => Ok(Return::None),
            //Attrs::PGMAssignTestToInv(inv_id, test_id) => Ok(Return::None),
            //// These will be left in the AST for later consumption by the flow, but they also function
            //// as un-official bin definitions.
            //// Any official bin defs in the AST for the same bin number will be combined with the model
            //// defintion created from here.
            //Attrs::PGMBin(hardbin, softbin, kind) => Ok(Return::Unmodified),
            _ => Ok(Return::ProcessChildren),
        }
    }
}

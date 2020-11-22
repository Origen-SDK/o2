use crate::core::dut::Dut;
use crate::core::file_handler::File;
use crate::core::model::pins::StateTracker;
use crate::generator::ast::{Attrs, Node};
use crate::generator::processor::{Processor, Return};
use crate::STATUS;
use crate::{Result, DUT};
use std::path::PathBuf;

use crate::generator::processors::{CycleCombiner, FlattenText, PinActionCombiner};

pub trait RendererAPI: std::fmt::Debug + crate::core::tester::TesterAPI {
    fn file_ext(&self) -> &str;
    fn comment_str(&self) -> &str;
    fn print_vector(
        &self,
        renderer: &mut Renderer,
        repeat: u32,
        compressable: bool,
    ) -> Option<Result<String>>;
    fn print_pinlist(&self, renderer: &mut Renderer) -> Option<Result<String>>;

    fn override_node(&self, _renderer: &mut Renderer, _node: &Node) -> Option<Result<Return>> {
        None
    }

    fn print_pattern_end(&self, _renderer: &mut Renderer) -> Option<Result<String>> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct Renderer<'a> {
    pub tester: &'a dyn RendererAPI,
    pub current_timeset_id: Option<usize>,
    pub path: Option<PathBuf>,
    pub output_file: Option<File>,
    pub states: Option<StateTracker>,
    pub pin_header_printed: bool,
    pub pin_header_id: Option<usize>,
}

impl<'a> Renderer<'a> {
    pub fn run(tester: &'a dyn RendererAPI, ast: &Node) -> Result<Vec<PathBuf>> {
        // Optimize the vectors
        let mut n = PinActionCombiner::run(ast)?;
        n = CycleCombiner::run(&n)?;

        // Generate comments
        n = FlattenText::run(&n)?;

        // Finally, generate the output
        let mut p = Self::new(tester);
        // println!("{}", n);
        n.process(&mut p)?;
        Ok(vec![p.path.unwrap()])
    }

    fn new(tester: &'a dyn RendererAPI) -> Self {
        Self {
            tester: tester,
            current_timeset_id: None,
            path: None,
            output_file: None,
            states: None,
            pin_header_printed: false,
            pin_header_id: None,
        }
    }

    pub fn states(&mut self, dut: &Dut) -> &mut StateTracker {
        if self.states.is_none() {
            let model_id;
            if let Some(id) = self.pin_header_id {
                model_id = dut.pin_headers[id].model_id;
            } else {
                model_id = 0;
            }
            self.states = Some(StateTracker::new(model_id, self.pin_header_id, dut));
        }
        self.states.as_mut().unwrap()
    }

    pub fn update_states(
        &mut self,
        grp_id: usize,
        actions: &Vec<String>,
        dut: &Dut,
    ) -> Result<Return> {
        let s = self.states(dut);
        s.update(grp_id, actions, dut)?;
        Ok(Return::Unmodified)
    }

    pub fn render_states(&self) -> Result<String> {
        let dut = DUT.lock().unwrap();
        let t = &dut.timesets[self.current_timeset_id.unwrap()];
        Ok(self
            .states
            .as_ref()
            .unwrap()
            .to_symbols(self.tester.name(), &dut, &t)
            .unwrap()
            .join(" "))
    }

    pub fn timeset_name(&self) -> Result<String> {
        let dut = DUT.lock().unwrap();

        let t = &dut.timesets[self
            .current_timeset_id
            .expect("Attempted to retrieve the current timeset name but no timeset has been set")];
        Ok(t.name.clone())
    }
}

impl<'a> Processor for Renderer<'a> {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match self.tester.override_node(self, &node) {
            Some(retn) => return retn,
            None => {}
        }

        match &node.attrs {
            Attrs::Test(name) => {
                let _ = STATUS.with_output_dir(false, |dir| {
                    let mut p = dir.to_path_buf();
                    p.push(self.tester.name().to_lowercase());
                    p.push(name);
                    p.set_extension(self.tester.file_ext());
                    self.path = Some(p.clone());
                    self.output_file = Some(File::create(p));
                    Ok(())
                });
                Ok(Return::ProcessChildren)
            }
            Attrs::Comment(_level, msg) => {
                self.output_file.as_mut().unwrap().write_ln(&format!(
                    "{} {}",
                    self.tester.comment_str(),
                    msg
                ));
                Ok(Return::Unmodified)
            }
            Attrs::Text(text) => {
                self.output_file.as_mut().unwrap().write_ln(&format!(
                    "{} {}",
                    self.tester.comment_str(),
                    text
                ));
                Ok(Return::Unmodified)
            }
            Attrs::PatternHeader => Ok(Return::ProcessChildren),
            // Attrs::PinGroupAction(grp_id, actions, _metadata) => {
            //     let dut = DUT.lock().unwrap();
            //     return self.update_states(*grp_id, actions, &dut);
            // },
            Attrs::PinAction(pin_id, action, _metadata) => {
                let dut = DUT.lock().unwrap();
                let pin = &dut.pins[*pin_id];
                let grp_id = dut.get_pin_group(pin.model_id, &pin.name).unwrap().id;
                return self.update_states(grp_id, &vec![action.clone()], &dut);
            }
            Attrs::Cycle(repeat, compressable) => {
                if !self.pin_header_printed {
                    match self.tester.print_pinlist(self) {
                        Some(pinlist) => {
                            self.output_file.as_mut().unwrap().write_ln(&pinlist?);
                        }
                        None => {}
                    }
                    self.pin_header_printed = true;
                }

                match self.tester.print_vector(self, *repeat, *compressable) {
                    Some(vector) => {
                        self.output_file.as_mut().unwrap().write_ln(&vector?);
                    }
                    None => {}
                }
                Ok(Return::Unmodified)
            }
            Attrs::SetTimeset(timeset_id) => {
                self.current_timeset_id = Some(*timeset_id);
                Ok(Return::Unmodified)
            }
            Attrs::SetPinHeader(pin_header_id) => {
                self.pin_header_id = Some(*pin_header_id);
                Ok(Return::Unmodified)
            }
            Attrs::PatternEnd => {
                match self.tester.print_pattern_end(self) {
                    Some(end) => {
                        self.output_file.as_mut().unwrap().write_ln(&end?);
                    }
                    None => {}
                }
                Ok(Return::Unmodified)
            }
            _ => Ok(Return::ProcessChildren),
        }
    }
}

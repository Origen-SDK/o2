use crate::core::dut::Dut;
use crate::core::file_handler::File;
use crate::core::model::pins::StateTracker;
use crate::generator::PAT;
use crate::STATUS;
use crate::{Overlay, Result, DUT};
use origen_metal::ast::{Node, Processor, Return};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::generator::processors::{
    CycleCombiner, FlattenText, PinActionCombiner, TargetTester, UnpackCaptures,
};

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

    fn override_node(
        &self,
        _renderer: &mut Renderer,
        _node: &Node<PAT>,
    ) -> Option<Result<Return<PAT>>> {
        None
    }

    fn print_pattern_end(&self, _renderer: &mut Renderer) -> Option<Result<String>> {
        None
    }

    fn start_overlay(
        &self,
        _renderer: &mut Renderer,
        _overlay: &Overlay,
    ) -> Option<Result<String>> {
        None
    }

    fn end_overlay(
        &self,
        _renderer: &mut Renderer,
        _label: &Option<String>,
        _pin_id: &Option<usize>,
    ) -> Option<Result<String>> {
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
    pub least_cycles_remaining: usize,
    pub capturing: HashMap<Option<usize>, Option<String>>,
    pub overlaying: HashMap<Option<usize>, (Option<String>, Option<String>)>,
}

impl<'a> Renderer<'a> {
    pub fn run(tester: &'a dyn RendererAPI, ast: &Node<PAT>) -> Result<Vec<PathBuf>> {
        // Screen out nodes not relevant to this renderer
        let mut n = TargetTester::run(ast, tester.id())?;

        // Optimize the vectors
        n = PinActionCombiner::run(&n)?;
        n = CycleCombiner::run(&n)?;
        n = UnpackCaptures::run(&n)?;

        // Generate comments
        n = FlattenText::run(&n)?;

        // Finally, generate the output
        let mut p = Self::new(tester);
        if crate::LOGGER.has_keyword("vector_based_dump_final_ast") {
            crate::LOGGER.info("Vector Based Tester- Printing Final AST");
            crate::LOGGER.info(&format!("{}", n));
        }
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
            least_cycles_remaining: std::usize::MAX,
            capturing: HashMap::new(),
            overlaying: HashMap::new(),
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
    ) -> Result<Return<PAT>> {
        let s = self.states(dut);
        s.update(grp_id, actions, dut)?;
        Ok(Return::Unmodified)
    }

    pub fn render_states(&self) -> Result<String> {
        let dut = DUT.lock().unwrap();
        let t = &dut.timesets[self.current_timeset_id.unwrap()];
        let mut ppin_overrides: HashMap<usize, String> = HashMap::new();
        for (ppin, symbol) in self.capturing.iter() {
            if ppin.is_some() && symbol.is_some() {
                ppin_overrides.insert(ppin.unwrap(), symbol.as_ref().unwrap().to_string());
            }
        }
        for (ppin, (_label, symbol)) in self.overlaying.iter() {
            if ppin.is_some() && symbol.is_some() {
                ppin_overrides.insert(ppin.unwrap(), symbol.as_ref().unwrap().to_string());
            }
        }
        Ok(self
            .states
            .as_ref()
            .unwrap()
            .to_symbols(self.tester.name(), &dut, &t, Some(&ppin_overrides))
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

impl<'a> Processor<PAT> for Renderer<'a> {
    fn on_node(&mut self, node: &Node<PAT>) -> Result<Return<PAT>> {
        match self.tester.override_node(self, &node) {
            Some(retn) => return retn,
            None => {}
        }

        match &node.attrs {
            PAT::Test(name) => {
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
            PAT::Comment(_level, msg) => {
                self.output_file.as_mut().unwrap().write_ln(&format!(
                    "{} {}",
                    self.tester.comment_str(),
                    msg
                ));
                Ok(Return::Unmodified)
            }
            PAT::Text(text) => {
                self.output_file.as_mut().unwrap().write_ln(&format!(
                    "{} {}",
                    self.tester.comment_str(),
                    text
                ));
                Ok(Return::Unmodified)
            }
            PAT::PatternHeader => Ok(Return::ProcessChildren),
            // Attrs::PinGroupAction(grp_id, actions, _metadata) => {
            //     let dut = DUT.lock().unwrap();
            //     return self.update_states(*grp_id, actions, &dut);
            // },
            PAT::PinAction(pin_id, action, _metadata) => {
                let dut = DUT.lock().unwrap();
                let pin = &dut.pins[*pin_id];
                let grp_id = dut.get_pin_group(pin.model_id, &pin.name).unwrap().id;
                return self.update_states(grp_id, &vec![action.clone()], &dut);
            }
            PAT::Capture(capture, _metadata) => {
                if capture.pin_ids.is_some() {
                    for pin in capture.enabled_capture_pins()? {
                        self.capturing.insert(Some(pin), capture.symbol.clone());
                    }
                } else {
                    self.capturing.insert(None, capture.symbol.clone());
                }
                Ok(Return::Unmodified)
            }
            PAT::Overlay(overlay, _) => {
                if overlay.pin_ids.is_some() {
                    for pin in overlay.enabled_overlay_pins()? {
                        self.overlaying
                            .insert(Some(pin), (overlay.label.clone(), overlay.symbol.clone()));
                    }
                } else {
                    self.overlaying
                        .insert(None, (overlay.label.clone(), overlay.symbol.clone()));
                }

                if let Some(s) = self.tester.start_overlay(self, overlay) {
                    self.output_file.as_mut().unwrap().write_ln(&format!(
                        "{} {}",
                        self.tester.comment_str(),
                        s?
                    ));
                }
                Ok(Return::Unmodified)
            }
            PAT::Cycle(repeat, compressable) => {
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
            PAT::SetTimeset(timeset_id) => {
                self.current_timeset_id = Some(*timeset_id);
                Ok(Return::Unmodified)
            }
            PAT::SetPinHeader(pin_header_id) => {
                self.pin_header_id = Some(*pin_header_id);
                Ok(Return::Unmodified)
            }
            PAT::EndCapture(pin_id) => {
                self.capturing.remove(&pin_id);
                Ok(Return::Unmodified)
            }
            PAT::EndOverlay(label, pin_id) => {
                self.overlaying.remove(&pin_id);
                if let Some(s) = self.tester.end_overlay(self, label, pin_id) {
                    self.output_file.as_mut().unwrap().write_ln(&format!(
                        "{} {}",
                        self.tester.comment_str(),
                        s?
                    ));
                }
                Ok(Return::Unmodified)
            }
            PAT::PatternEnd => {
                // Raise an error is any leftover captures remain
                if !self.capturing.is_empty() {
                    bail!("Pattern end reached but requested captures still remain");
                }
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

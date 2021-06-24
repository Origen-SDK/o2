pub mod api_structs;
pub use api_structs::{Capture, Overlay};

use super::model::pins::pin::Resolver;
use super::model::pins::pin_header::PinHeader;
use super::model::timesets::timeset::Timeset;
use crate::core::dut::Dut;
use crate::core::reference_files;
use crate::generator::ast::{Attrs, Node};
use crate::testers::{instantiate_tester, SupportedTester};
use crate::utility::differ::Differ;
use crate::utility::file_utils::to_relative_path;
use crate::{add_children, node, text, text_line, with_current_job};
use crate::{Error, Result};
use crate::{FLOW, TEST};
use indexmap::IndexMap;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum TesterSource {
    Internal(Box<dyn TesterAPI + std::marker::Send>),
    External(SupportedTester),
}

impl Clone for TesterSource {
    fn clone(&self) -> TesterSource {
        match self {
            TesterSource::Internal(_g) => TesterSource::Internal((*_g).clone()),
            TesterSource::External(_g) => TesterSource::External(_g.clone()),
        }
    }
}

impl PartialEq<TesterSource> for TesterSource {
    fn eq(&self, g: &TesterSource) -> bool {
        match g {
            TesterSource::Internal(_g) => match self {
                TesterSource::Internal(_self) => _g.id() == _self.id(),
                _ => false,
            },
            TesterSource::External(_g) => match self {
                TesterSource::External(_self) => _g == _self,
                _ => false,
            },
        }
    }
}
impl Eq for TesterSource {}

impl std::hash::Hash for TesterSource {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Internal(g) => {
                g.id().hash(state);
            }
            Self::External(g) => {
                g.hash(state);
            }
        }
    }
}

impl TesterSource {
    pub fn name(&self) -> String {
        match self {
            Self::External(g) => g.to_string(),
            Self::Internal(g) => g.name(),
        }
    }

    pub fn id(&self) -> SupportedTester {
        match self {
            Self::External(g) => g.to_owned(),
            Self::Internal(g) => g.id(),
        }
    }
}

#[derive(Debug)]
pub struct Tester {
    /// The current timeset ID, if its set.
    /// This is the direct ID to the timeset object.
    /// The name and model ID can be found on this object.
    current_timeset_id: Option<usize>,
    current_pin_header_id: Option<usize>,
    /// This stores additional testers provided by the Python domain, effectively adding them
    /// to the list of supported_testers. It never needs to be cleared.
    external_testers: IndexMap<String, TesterSource>,
    /// This is the testers that have been selected by the current target, it will be cleared
    /// and new testers will be pushed to it during a target load.
    /// It contains references to both Rust and Python domain testers.
    pub target_testers: Vec<TesterSource>,
    /// Keeps track of some stats, like how many patterns have been generated, how many with
    /// diffs, etc.
    pub stats: Stats,
}

#[derive(Debug, Default, Serialize)]
pub struct Stats {
    pub generated_pattern_files: usize,
    pub changed_pattern_files: usize,
    pub new_pattern_files: usize,
    pub generated_program_files: usize,
    pub changed_program_files: usize,
    pub new_program_files: usize,
}

impl Stats {
    pub fn to_pickle(&self) -> Vec<u8> {
        serde_pickle::to_vec(self, true).unwrap()
    }
}

impl Tester {
    pub fn new() -> Self {
        Tester {
            current_timeset_id: Option::None,
            current_pin_header_id: Option::None,
            external_testers: IndexMap::new(),
            target_testers: vec![],
            stats: Stats::default(),
        }
    }

    /// Starts a new tester-specific section in the current pattern and/or test program.
    /// The returned ID should be kept and given to end_tester_eq_block when the
    /// tester specific section is complete.
    pub fn start_tester_eq_block(&self, testers: Vec<SupportedTester>) -> Result<(usize, usize)> {
        let n = node!(TesterEq, testers.clone());
        let pat_ref_id = TEST.push_and_open(n.clone());
        let prog_ref_id;
        if FLOW.selected().is_some() {
            prog_ref_id = FLOW.push_and_open(n)?;
        } else {
            prog_ref_id = 0;
        }
        // This also verifies that the given tester selection is valid
        crate::STATUS.push_testers_eq(testers)?;
        Ok((pat_ref_id, prog_ref_id))
    }

    /// Ends an open tester-specific section in the current pattern and/or test program.
    /// The ID produced when opening the block (via start_tester_eq_block) should be supplied
    /// as the main argument.
    pub fn end_tester_eq_block(&self, pat_ref_id: usize, prog_ref_id: usize) -> Result<()> {
        TEST.close(pat_ref_id)?;
        if FLOW.selected().is_some() {
            FLOW.close(prog_ref_id)?;
        }
        crate::STATUS.pop_testers_eq()?;
        Ok(())
    }

    /// Like start_tester_eq_block, but the contained block will be included for all testers
    /// except those in the given list
    pub fn start_tester_neq_block(&self, testers: Vec<SupportedTester>) -> Result<(usize, usize)> {
        let n = node!(TesterNeq, testers.clone());
        let pat_ref_id = TEST.push_and_open(n.clone());
        let prog_ref_id;
        if FLOW.selected().is_some() {
            prog_ref_id = FLOW.push_and_open(n)?;
        } else {
            prog_ref_id = 0;
        }
        // This also verifies that the given tester selection is valid
        crate::STATUS.push_testers_neq(testers)?;
        Ok((pat_ref_id, prog_ref_id))
    }

    pub fn end_tester_neq_block(&self, pat_ref_id: usize, prog_ref_id: usize) -> Result<()> {
        TEST.close(pat_ref_id)?;
        if FLOW.selected().is_some() {
            FLOW.close(prog_ref_id)?;
        }
        crate::STATUS.pop_testers_neq()?;
        Ok(())
    }

    pub fn custom_tester_ids(&self) -> Vec<String> {
        self.external_testers
            .keys()
            .map(|n| n.to_string())
            .collect::<Vec<String>>()
    }

    pub fn _current_timeset_id(&self) -> Result<usize> {
        match self.current_timeset_id {
            Some(t_id) => Ok(t_id),
            None => Err(Error::new(&format!("No timeset has been set!"))),
        }
    }

    pub fn _current_pin_header_id(&self) -> Result<usize> {
        match self.current_pin_header_id {
            Some(ph_id) => Ok(ph_id),
            None => Err(Error::new(&format!("No pin header has been set!"))),
        }
    }

    pub fn init(&mut self) -> Result<()> {
        self.target_testers.clear();
        self.current_timeset_id = Option::None;
        Ok(())
    }

    /// This will be called by Origen immediately before it is about to load the target, it unloads
    /// all tester targets and all other state making it ready to accept a new set of targets
    pub fn reset(&mut self) -> Result<()> {
        crate::emit_callback(crate::CALLBACKS::BEFORE_TESTER_RESET, None, None, None)?;

        self.init()?;

        crate::emit_callback(crate::CALLBACKS::AFTER_TESTER_RESET, None, None, None)?;
        Ok(())
    }

    pub fn register_external_tester(&mut self, tester: &str) -> Result<SupportedTester> {
        let t_id = SupportedTester::CUSTOM(tester.to_string());
        self.external_testers
            .insert(tester.to_string(), TesterSource::External(t_id.clone()));
        // Store it in the STATUS so that it's presence is globally readable without having to
        // get a lock on the TESTER
        crate::STATUS.register_custom_tester(tester);
        Ok(t_id)
    }

    pub fn get_timeset(&self, dut: &Dut) -> Option<Timeset> {
        if let Some(t_id) = self.current_timeset_id {
            Some(dut.timesets[t_id].clone())
        } else {
            Option::None
        }
    }

    pub fn _get_timeset(&self, dut: &Dut) -> Result<Timeset> {
        if let Some(t_id) = self.current_timeset_id {
            Ok(dut.timesets[t_id].clone())
        } else {
            Err(Error::new(&format!("No timeset has been set!")))
        }
    }

    pub fn set_timeset(&mut self, dut: &Dut, model_id: usize, timeset_name: &str) -> Result<()> {
        self.current_timeset_id = Some(dut._get_timeset(model_id, timeset_name)?.id);
        TEST.push(node!(SetTimeset, self.current_timeset_id.unwrap()));
        Ok(())
    }

    pub fn clear_timeset(&mut self) -> Result<()> {
        self.current_timeset_id = Option::None;
        TEST.push(node!(ClearTimeset));
        Ok(())
    }

    pub fn get_pin_header(&self, dut: &Dut) -> Option<PinHeader> {
        if let Some(ph_id) = self.current_pin_header_id {
            Some(dut.pin_headers[ph_id].clone())
        } else {
            Option::None
        }
    }

    pub fn _get_pin_header(&self, dut: &Dut) -> Result<PinHeader> {
        if let Some(ph_id) = self.current_pin_header_id {
            Ok(dut.pin_headers[ph_id].clone())
        } else {
            Err(Error::new(&format!("No pin header has been set!")))
        }
    }

    pub fn set_pin_header(
        &mut self,
        dut: &Dut,
        model_id: usize,
        pin_header_name: &str,
    ) -> Result<()> {
        self.current_pin_header_id = Some(dut._get_pin_header(model_id, pin_header_name)?.id);
        TEST.push(node!(SetPinHeader, self.current_pin_header_id.unwrap()));
        Ok(())
    }

    pub fn clear_pin_header(&mut self) -> Result<()> {
        self.current_pin_header_id = Option::None;
        TEST.push(node!(ClearPinHeader));
        Ok(())
    }

    pub fn issue_callback_at(&mut self, idx: usize) -> Result<()> {
        let g = &mut self.target_testers[idx];

        // Grab the last node and immutably pass it to the interceptor
        match g {
            TesterSource::Internal(g_) => {
                let last_node = TEST.get(0).unwrap();
                match &last_node.attrs {
                    Attrs::Cycle(repeat, compressable) => {
                        g_.cycle(*repeat, *compressable, &last_node)?
                    }
                    Attrs::Comment(level, msg) => g_.cc(*level, &msg, &last_node)?,
                    Attrs::SetTimeset(timeset_id) => g_.set_timeset(*timeset_id, &last_node)?,
                    Attrs::ClearTimeset => g_.clear_timeset(&last_node)?,
                    Attrs::SetPinHeader(pin_header_id) => {
                        g_.set_pin_header(*pin_header_id, &last_node)?
                    }
                    Attrs::ClearPinHeader => g_.clear_pin_header(&last_node)?,
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn cc(&mut self, comment: &str) -> Result<()> {
        let comment_node = node!(Comment, 1, comment.to_string());
        TEST.push(comment_node);
        Ok(())
    }

    pub fn cycle(&mut self, repeat: Option<usize>) -> Result<()> {
        let cycle_node = node!(Cycle, repeat.unwrap_or(1) as u32, true);
        TEST.push(cycle_node);
        Ok(())
    }

    pub fn overlay(&self, overlay: &Overlay) -> Result<()> {
        TEST.push(overlay.to_node());
        Ok(())
    }

    pub fn capture(&self, capture: &Capture) -> Result<()> {
        TEST.push(capture.to_node());
        Ok(())
    }

    pub fn generate_pattern_header(
        &self,
        app_comments: Option<Vec<String>>,
        pattern_comments: Option<Vec<String>>,
    ) -> Result<()> {
        let mut header = node!(PatternHeader);
        header.add_child(node!(TextBoundaryLine));
        let mut section = node!(TextSection, Some("Generated".to_string()), None);
        section.add_children(vec![
            text_line!(text!("Time: "), node!(Timestamp)),
            text_line!(text!("By: "), node!(User)),
            with_current_job(|job| {
                Ok(text_line!(
                    text!("Command: "),
                    node!(OrigenCommand, job.command())
                ))
            })
            .unwrap(),
        ]);
        header.add_child(section);

        header.add_child(node!(TextBoundaryLine));
        section = node!(TextSection, Some("Workspace".to_string()), None);
        section.add_children(vec![
            add_children!(
                node!(TextSection, Some("Environment".to_string()), None),
                text_line!(text!("OS: "), node!(OS)),
                text_line!(text!("Mode: "), node!(Mode)),
                add_children!(
                    node!(TextSection, Some("Targets".to_string()), None),
                    node!(TargetsStacked)
                )
            ),
            add_children!(
                node!(TextSection, Some("Application".to_string()), None),
                // To-do: Add this
                //text_line!(text!("Version: "), node!(AppVersion)),
                text_line!(text!("Local Path: "), node!(AppRoot))
            ),
            add_children!(
                node!(TextSection, Some("Origen Core".to_string()), None),
                text_line!(text!("Version: "), node!(OrigenVersion)),
                text_line!(text!("Executable Path: "), node!(OrigenRoot))
            ), // To-do: Add these as well
               // node!(TextSection, Some("Application".to_string()), None).add_children(vec!(
               // )),
               // node!(TextSection, Some("Origen Core".to_string()), None).add_children(vec!(
               // ))
        ]);
        header.add_child(section);

        if app_comments.is_some() || pattern_comments.is_some() {
            header.add_child(node!(TextBoundaryLine));
            section = node!(TextSection, Some("Header Comments".to_string()), None);
            if let Some(comments) = app_comments {
                let mut s = node!(TextSection, Some("From the Application".to_string()), None);
                s.add_children(comments.iter().map(|c| text!(c)).collect::<Vec<Node>>());
                section.add_child(s);
            }
            if let Some(comments) = pattern_comments {
                let mut s = node!(TextSection, Some("From the Pattern".to_string()), None);
                s.add_children(comments.iter().map(|c| text!(c)).collect::<Vec<Node>>());
                section.add_child(s);
            }
            header.add_child(section);
        }
        header.add_child(node!(TextBoundaryLine));

        TEST.push(header);
        Ok(())
    }

    pub fn end_pattern(&self) -> Result<()> {
        TEST.push(node!(PatternEnd));
        Ok(())
    }

    /// Renders the output for the target at index i.
    /// Allows the frontend to call testers in a loop.
    /// When diff_and_display is true the generated files will be displayed to the console
    /// and checked for diffs.
    pub fn render_pattern_for_target_at(
        &mut self,
        idx: usize,
        diff_and_display: bool,
    ) -> Result<Vec<PathBuf>> {
        let g = &mut self.target_testers[idx];
        match g {
            TesterSource::External(gen) => {
                error!("Tester '{}' is Python-based and pattern rendering must be invoked from Python code", &gen)
            }
            TesterSource::Internal(gen) => {
                let paths = TEST.with_ast(|ast| gen.render_pattern(ast))?;
                if !paths.is_empty() {
                    for path in &paths {
                        self.stats.generated_pattern_files += 1;
                        log_debug!("Tester '{}' created file '{}'", gen.name(),  path.display());
                        if diff_and_display {
                            if let Ok(p) = to_relative_path(path, None) {
                                display!("Created: {}", p.display());
                            } else {
                                display!("Created: {}", path.display());
                            }
                            if let Some(ref_dir) = crate::STATUS.reference_dir() {
                                match path.strip_prefix(crate::STATUS.output_dir()) {
                                    Err(e) => log_error!("{}", e),
                                    Ok(stem) => {
                                        let ref_pat = ref_dir.join(&stem);
                                        display!(" - ");
                                        if ref_pat.exists() {
                                            if let Some(mut differ) = gen.pattern_differ(path, &ref_pat) {
                                                if differ.has_diffs()? {
                                                    if let Err(e) = reference_files::create_changed_ref(&stem, &path, &ref_pat) {
                                                        log_error!("{}", e);
                                                    }
                                                    self.stats.changed_pattern_files += 1;
                                                    display_redln!("Diffs found");
                                                    let old = to_relative_path(&ref_pat, None).unwrap_or(ref_pat);
                                                    let new = to_relative_path(&path, None).unwrap_or(path.to_owned());
                                                    let diff_tool = env::var("ORIGEN_DIFF_TOOL").unwrap_or("tkdiff".to_string());
                                                    displayln!("  {} {} {} &", &diff_tool, old.display(), new.display());
                                                    display!("  origen save_ref {}", stem.display());
                                                } else {
                                                    display_green!("No diffs");
                                                }
                                            } else {
                                                log_debug!("No differ defined for tester '{}'", gen.name());
                                                display_yellow!("Diff not checked");
                                            }
                                        } else {
                                            self.stats.new_pattern_files += 1;
                                            if let Err(e) = reference_files::create_new_ref(&stem, &path, &ref_pat) {
                                                log_error!("{}", e);
                                            }
                                            display_cyanln!("New pattern");
                                            display!("  origen save_ref {}", stem.display());

                                        }
                                    }
                                }
                            }
                            displayln!("");
                        }
                    }
                } else {
                    log_debug!("No files generated by tester '{}", gen.name());
                }
                Ok(paths)
            }
        }
    }

    /// Renders the test program for the target at index i.
    /// Allows the frontend to call testers in a (multithreaded) loop.
    /// When diff_and_display is true the generated files will be displayed to the console
    /// and checked for diffs.
    pub fn render_program_for_target_at(
        &mut self,
        idx: usize,
        diff_and_display: bool,
    ) -> Result<Vec<PathBuf>> {
        let g = &mut self.target_testers[idx];
        match g {
            TesterSource::External(gen) => {
                error!("Tester '{}' is Python-based and pattern rendering must be invoked from Python code", &gen)
            }
            TesterSource::Internal(gen) => {
                log_info!("Rendering program for {}", &gen.name());
                let paths = gen.render_program()?;
                if !paths.is_empty() {
                    for path in &paths {
                        self.stats.generated_program_files += 1;
                        log_debug!("Tester '{}' created file '{}'", gen.name(),  path.display());
                        if diff_and_display {
                            if let Ok(p) = to_relative_path(path, None) {
                                display!("Created: {}", p.display());
                            } else {
                                display!("Created: {}", path.display());
                            }
                            if let Some(ref_dir) = crate::STATUS.reference_dir() {
                                match path.strip_prefix(crate::STATUS.output_dir()) {
                                    Err(e) => log_error!("{}", e),
                                    Ok(stem) => {
                                        let ref_pat = ref_dir.join(&stem);
                                        display!(" - ");
                                        if ref_pat.exists() {
                                            if let Some(mut differ) = gen.program_differ(path, &ref_pat) {
                                                if differ.has_diffs()? {
                                                    if let Err(e) = reference_files::create_changed_ref(&stem, &path, &ref_pat) {
                                                        log_error!("{}", e);
                                                    }
                                                    self.stats.changed_program_files += 1;
                                                    display_redln!("Diffs found");
                                                    let old = to_relative_path(&ref_pat, None).unwrap_or(ref_pat);
                                                    let new = to_relative_path(&path, None).unwrap_or(path.to_owned());
                                                    let diff_tool = env::var("ORIGEN_DIFF_TOOL").unwrap_or("tkdiff".to_string());
                                                    displayln!("  {} {} {} &", &diff_tool, old.display(), new.display());
                                                    display!("  origen save_ref {}", stem.display());
                                                } else {
                                                    display_green!("No diffs");
                                                }
                                            } else {
                                                log_debug!("No differ defined for tester '{}'", gen.name());
                                                display_yellow!("Diff not checked");
                                            }
                                        } else {
                                            self.stats.new_program_files += 1;
                                            if let Err(e) = reference_files::create_new_ref(&stem, &path, &ref_pat) {
                                                log_error!("{}", e);
                                            }
                                            display_cyanln!("New file");
                                            display!("  origen save_ref {}", stem.display());

                                        }
                                    }
                                }
                            }
                            displayln!("");
                        }
                    }
                } else {
                    log_debug!("No files generated by tester '{}", gen.name());
                }
                Ok(paths)
            }
        }
    }

    pub fn target(&mut self, tester: SupportedTester) -> Result<&TesterSource> {
        let g;
        if let SupportedTester::CUSTOM(id) = &tester {
            if let Some(_g) = self.external_testers.get(id) {
                g = (*_g).clone();
            } else {
                return error!(
                    "Could not find tester '{}', the available testers are: {}",
                    tester,
                    self.custom_tester_ids()
                        .iter()
                        .map(|id| format!("CUSTOM::{}", id))
                        .collect::<Vec<String>>()
                        .join(", ")
                );
            }
        } else {
            g = TesterSource::Internal(instantiate_tester(&tester)?);
        }

        if self.target_testers.contains(&g) {
            Err(Error::new(&format!(
                "Tester {} has already been targeted!",
                tester
            )))
        } else {
            self.target_testers.push(g);
            Ok(&self.target_testers.last().unwrap())
        }
    }

    pub fn targets(&self) -> &Vec<TesterSource> {
        &self.target_testers
    }

    pub fn targets_as_strs(&self) -> Vec<String> {
        self.target_testers.iter().map(|g| g.name()).collect()
    }

    pub fn focused_tester(&self) -> Option<&TesterSource> {
        match self.target_testers.first() {
            Some(t) => Some(&t),
            None => None,
        }
    }

    pub fn focused_tester_name(&self) -> Option<String> {
        match self.target_testers.first() {
            Some(t) => Some(t.name()),
            None => None,
        }
    }

    /// This is called automatically at the very start of a generate command, it is invoked from Python,
    /// so state can be established here which will persist for the rest of the command
    pub fn prepare_for_generate(&mut self) -> Result<()> {
        let on_lsf = crate::core::lsf::is_running_remotely();
        // Jobs to be done before launching to the LSF
        if !on_lsf {
            reference_files::clear_save_refs()?;
        }
        Ok(())
    }
}

/// Trait which allows Rust-side implemented testers to intercept generic calls
///   from the tester.
/// Each method will be given the resulting node after processing.
/// Note: the node given is only a clone of what will be stored in the AST.
pub trait Interceptor {
    fn cycle(&mut self, _repeat: u32, _compressable: bool, _node: &Node) -> Result<()> {
        Ok(())
    }

    fn set_timeset(&mut self, _timeset_id: usize, _node: &Node) -> Result<()> {
        Ok(())
    }

    fn clear_timeset(&mut self, _node: &Node) -> Result<()> {
        Ok(())
    }

    fn cc(&mut self, _level: u8, _msg: &str, _node: &Node) -> Result<()> {
        Ok(())
    }

    fn set_pin_header(&mut self, _pin_header_id: usize, _node: &Node) -> Result<()> {
        Ok(())
    }

    fn clear_pin_header(&mut self, _node: &Node) -> Result<()> {
        Ok(())
    }
}
impl<'a, T> Interceptor for &'a T where T: TesterAPI {}
impl<'a, T> Interceptor for &'a mut T where T: TesterAPI {}

pub trait TesterID {
    fn id(&self) -> SupportedTester;

    /// Returns the id() as a String in most cases, but may be overridden to something
    /// more friendly (but still unique), e.g. for custom Python-based testers
    fn name(&self) -> String {
        self.id().to_string()
    }
}

pub trait TesterAPI: std::fmt::Debug + Interceptor + TesterID + TesterAPIClone {
    /// Render the given AST to an output, returning the path(s) to the created file(s)
    /// if successful.
    /// A default implementation is given since some testers may only support prog gen
    /// and not patgen and vice versa, in that case they will return an empty vector.
    fn render_pattern(&mut self, ast: &Node) -> crate::Result<Vec<PathBuf>> {
        let _ = ast;
        log_debug!("Tester '{}' does not implement render_pattern", &self.id());
        Ok(vec![])
    }

    /// Render the test program to an output, returning the path(s) to the created file(s)
    /// if successful.
    /// A default implementation is given since some testers may only support prog gen
    /// and not patgen and vice versa, in that case they will return an empty vector.
    fn render_program(&mut self) -> crate::Result<Vec<PathBuf>> {
        log_debug!("Tester '{}' does not implement render_program", &self.id());
        Ok(vec![])
    }

    /// The tester should implement this to return a differ instance which is configured
    /// per the tester's pattern format, e.g. to define the command char(s).
    /// If diff'ing is not applicable to the tester, e.g. the pattern is in binary format,
    /// then the tester does not need to implement this.
    /// If only some patterns can be diffed then then test should return None in the case
    /// where the pattern is one that cannot be diffed.
    fn pattern_differ(&self, pat_a: &Path, pat_b: &Path) -> Option<Box<dyn Differ>> {
        let _ = pat_a;
        let _ = pat_b;
        None
    }

    fn program_differ(&self, pat_a: &Path, pat_b: &Path) -> Option<Box<dyn Differ>> {
        let _ = pat_a;
        let _ = pat_b;
        None
    }

    fn pin_action_resolver(&self) -> Option<Resolver> {
        None
    }

    /// Returns a path to the tester-specific output directory, it is expected to create it
    /// if it doesn't exist so the caller doesn't have to
    fn output_dir(&self) -> Result<PathBuf> {
        let dir = crate::STATUS.output_dir().join(&self.name().to_lowercase());
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }
}

// This stuff derived from here: https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
pub trait TesterAPIClone {
    fn clone_box(&self) -> Box<dyn TesterAPI + Send>;
}

impl<T> TesterAPIClone for T
where
    T: 'static + TesterAPI + Clone + Send,
{
    fn clone_box(&self) -> Box<dyn TesterAPI + Send> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn TesterAPI + Send> {
    fn clone(&self) -> Box<dyn TesterAPI + Send> {
        self.clone_box()
    }
}

impl PartialEq<TesterSource> for dyn TesterAPI {
    fn eq(&self, g: &TesterSource) -> bool {
        self.id() == g.id()
    }
}

impl std::hash::Hash for dyn TesterAPI {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

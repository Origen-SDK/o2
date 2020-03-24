use crate::error::Error;
use super::model::timesets::timeset::{Timeset};
use indexmap::IndexMap;
use crate::testers::{instantiate_tester, available_testers};
use crate::core::dut::{Dut};
use crate::generator::ast::{Attrs, Node};
use crate::TEST;
use crate::node;

#[derive(Debug)]
pub enum TesterSource {
  Internal(Box<dyn TesterAPI + std::marker::Send>),
  External(String),
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
        TesterSource::Internal(_self) => {
          *_g.name() == *_self.name()
        },
        _ => false
      }
      TesterSource::External(_g) => match self {
        TesterSource::External(_self) => {
          _g == _self
        },
        _ => false
      }
    }
  }
}

impl TesterSource {
  pub fn to_string(&self) -> String {
    match self {
      Self::External(g) => g.clone(),
      Self::Internal(g) => g.to_string(),
    }
  }
}

#[derive(Debug)]
pub struct Tester {
  /// The current timeset ID, if its set.
  /// This is the direct ID to the timeset object.
  /// The name and model ID can be found on this object.
  current_timeset_id: Option<usize>,
  external_testers: IndexMap<String, TesterSource>,
  pub target_testers: Vec<TesterSource>,
}

impl Tester {
  pub fn new() -> Self {
    Tester {
      current_timeset_id: Option::None,
      external_testers: IndexMap::new(),
      target_testers: vec!(),
    }
  }

  pub fn _current_timeset_id(&self) -> Result<usize, Error> {
    match self.current_timeset_id {
      Some(t_id) => Ok(t_id),
      None => Err(Error::new(&format!("No timeset has been set!")))
    }
  }

  pub fn reset(&mut self, ast_name: Option<String>) -> Result<(), Error> {
    self.clear_dut_dependencies(ast_name)?;
    self.reset_external_testers()?;
    Ok(())
  }

  /// Clears all members which reference members on the current DUT.
  pub fn clear_dut_dependencies(&mut self, ast_name: Option<String>) -> Result<(), Error> {
    if let Some(ast) = ast_name {
      TEST.start(&ast);
    } else {
      TEST.start("ad-hoc");
    }
    self.current_timeset_id = Option::None;
    Ok(())
  }

  // Resets the external testers.
  // Also clears the targeted testers, as it may point to an external one that will be cleared.
  pub fn reset_external_testers(&mut self) -> Result<(), Error> {
    self.target_testers.clear();
    self.external_testers.clear();
    Ok(())
  }

  pub fn reset_targets(&mut self) -> Result<(), Error> {
    self.target_testers.clear();
    Ok(())
  }

  pub fn register_external_tester(&mut self, tester: &str) -> Result<(), Error> {
    self.external_testers.insert(tester.to_string(), TesterSource::External(tester.to_string()));
    Ok(())
  }

  pub fn get_timeset(&self, dut: &Dut) -> Option<Timeset> {
    if let Some(t_id) = self.current_timeset_id {
      Some(dut.timesets[t_id].clone())
    } else {
      Option::None
    }
  }

  pub fn _get_timeset(&self, dut: &Dut) -> Result<Timeset, Error> {
    if let Some(t_id) = self.current_timeset_id {
      Ok(dut.timesets[t_id].clone())
    } else {
      Err(Error::new(&format!("No timeset has been set!")))
    }
  }

  pub fn set_timeset(&mut self, dut: &Dut, model_id: usize, timeset_name: &str) -> Result<(), Error> {
    self.current_timeset_id = Some(dut._get_timeset(model_id, timeset_name)?.id);
    TEST.push(node!(SetTimeset, self.current_timeset_id.unwrap()));
    Ok(())
  }

  pub fn clear_timeset(&mut self) -> Result<(), Error> {
    self.current_timeset_id = Option::None;
    Ok(())
  }

  pub fn issue_callback_at(&mut self, idx: usize) -> Result<(), Error> {
    let g = &mut self.target_testers[idx];

    // Grab the last node and immutably pass it to the interceptor
    match g {
      TesterSource::Internal(g_) => {
        let last_node = TEST.get(0).unwrap();
        match &last_node.attrs {
          Attrs::Cycle(repeat, compressable) => g_.cycle(*repeat, *compressable, &last_node)?,
          Attrs::Comment(level, msg) => g_.cc(*level, &msg, &last_node)?,
          Attrs::SetTimeset(timeset_id) => g_.set_timeset(*timeset_id, &last_node)?,
          Attrs::ClearTimeset() => g_.clear_timeset(&last_node)?,
          _ => {}
        }
      },
      _ => {}
    }
    Ok(())
  }

  pub fn cc(&mut self, comment: &str) -> Result<(), Error> {
    let comment_node = node!(Comment, 1, comment.to_string());
    TEST.push(comment_node);
    Ok(())
  }

  pub fn cycle(&mut self, repeat: Option<usize>) -> Result<(), Error> {
    let cycle_node = node!(Cycle, repeat.unwrap_or(1) as u32, true);
    TEST.push(cycle_node);
    Ok(())
  }

  /// Renders the output for the target at index i.
  /// Allows the frontend to call testers in a loop.
  pub fn render_target_at(&mut self, idx: usize) -> Result<RenderStatus, Error> {
    let mut stat = RenderStatus::new();
    let g = &mut self.target_testers[idx];
    match g {
      TesterSource::External(gen) => {
        stat.external.push(gen.to_string());
      },
      TesterSource::Internal(gen) => {
        TEST.process(&mut |ast| gen.run(ast));
        stat.completed.push(gen.to_string())
      }
    }
    Ok(stat)
  }

  pub fn target(&mut self, tester: &str) -> Result<&TesterSource, Error> {
    let g;
    if let Some(_g) = instantiate_tester(tester) {
      g = TesterSource::Internal(_g);
    } else if let Some(_g) = self.external_testers.get(tester) {
      g = (*_g).clone();
    } else {
      return Err(Error::new(&format!("Could not find tester '{}'!", tester)));
    }

    if self.target_testers.contains(&g) {
        Err(Error::new(&format!("Tester {} has already been targeted!", tester)))
    } else {
      self.target_testers.push(g);
      Ok(&self.target_testers.last().unwrap())
    }
  }

  pub fn targets(&self) -> &Vec<TesterSource> {
    &self.target_testers
  }

  pub fn targets_as_strs(&self) -> Vec<String> {
    self.target_testers.iter().map( |g| g.to_string()).collect()
  }

  pub fn clear_targets(&mut self) -> Result<(), Error> {
    self.target_testers.clear();
    Ok(())
  }

  pub fn testers(&self) -> Vec<String> {
    let mut gens: Vec<String> = available_testers();
    gens.extend(self.external_testers.iter().map(|(n, _)| n.clone()).collect::<Vec<String>>());
    gens
  }
}

pub struct RenderStatus {
  pub completed: Vec<String>,
  pub external: Vec<String>,
}

impl RenderStatus {
  pub fn new() -> Self {
    Self {
      completed: vec!(),
      external: vec!(),
    }
  }
}

/// Trait which allows Rust-side implemented testers to intercept generic calls
///   from the tester.
/// Each method will be given the resulting node after processing.
/// Note: the node given is only a clone of what will be stored in the AST.
pub trait Interceptor {
  fn cycle(&mut self, _repeat: u32, _compressable: bool, _node: &Node) -> Result<(), Error> {
    Ok(())
  }

  fn set_timeset(&mut self, _timeset_id: usize, _node: &Node) -> Result<(), Error> {
    Ok(())
  }

  fn clear_timeset(&mut self, _node: &Node) -> Result<(), Error> {
    Ok(())
  }

  fn cc(&mut self, _level: u8, _msg: &str, _node: &Node) -> Result<(), Error> {
    Ok(())
  }
}
impl<'a, T> Interceptor for &'a T where T: TesterAPI {}
impl<'a, T> Interceptor for &'a mut T where T: TesterAPI {}

pub trait TesterAPI: std::fmt::Debug + crate::generator::processor::Processor + Interceptor {
  fn name(&self) -> String;
  fn clone(&self) -> Box<dyn TesterAPI + std::marker::Send>;
  fn run(&mut self, node: &Node) -> Node;

  fn to_string(&self) -> String {
    format!("::{}", self.name())
  }
}

impl PartialEq<TesterSource> for dyn TesterAPI {
  fn eq(&self, g: &TesterSource) -> bool {
    self.to_string() == g.to_string()
  }
}

use crate::error::Error;
use super::model::timesets::timeset::{Timeset};
use indexmap::IndexMap;
use crate::testers::v93k::Generator as V93KGen;
use crate::testers::simulator::Generator as Simulator;
use crate::testers::{DummyGeneratorWithInterceptors, DummyGenerator};
use crate::core::dut::{Dut};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Generators {
  DummyGenerator,
  DummyGeneratorWithInterceptors,
  V93kSt7,
  Simulator,
  //J750,
  //Uflex,
  External(String),
}

impl Generators {
  pub fn from_str(s: &str) -> Option<Self> {
    match s {
      "::DummyGenerator" => Some(Self::DummyGenerator),
      "::DummyGeneratorWithInterceptors" => Some(Self::DummyGeneratorWithInterceptors),
      "::V93K::ST7" => Some(Self::V93kSt7),
      "::Simulator" => Some(Self::Simulator),
      _ => None
    }
  }

  pub fn to_string(&self) -> String {
    match self {
      Self::DummyGenerator => "::DummyGenerator".to_string(),
      Self::DummyGeneratorWithInterceptors => "::DummyGeneratorWithInterceptors".to_string(),
      Self::V93kSt7 => "::V93K::ST7".to_string(),
      Self::Simulator => "::Simulator".to_string(),
      Self::External(g) => g.clone(),
    }
  }

  pub fn to_vector_strings(&self) -> Vec<String> {
    vec!("::DummyGenerator".to_string(), "::DummyGeneratorWithInterceptors".to_string(), "::V93K::ST7".to_string(), "::Simulator".to_string())
  }
}

#[derive(Debug)]
pub struct ExternalGenerator {
  name: String,
  source: String,
  generator: Box<dyn core::any::Any + std::marker::Send + 'static>,
}

#[derive(Debug)]
pub enum StubNodes {
  Comment {
    content: String,
    meta: IndexMap<String, usize>,
  },
  Vector {
    //timeset: String,
    timeset_id: usize,
    repeat: usize,
    meta: IndexMap<String, usize>,
  },
  Node {
    meta: IndexMap<String, usize>,
  },
}

impl StubNodes {
  pub fn add_metadata_id(&mut self, identifier: &str, id: usize) -> Result<(), Error> {
    match self {
      Self::Comment {content: _, meta} => {
        meta.insert(identifier.to_string(), id);
      },
      Self::Vector {timeset_id: _, repeat: _, meta} => {
        meta.insert(identifier.to_string(), id);
      },
      Self::Node {meta} => {
        meta.insert(identifier.to_string(), id);
      }
    }
    Ok(())
  }

  pub fn get_metadata_id(&self, identifier: &str) -> Option<usize> {
    match self {
      Self::Comment {content: _, meta} => {
        match meta.get(identifier) {
          Some(id) => Some(*id),
          None => None,
        }
      },
      Self::Vector {timeset_id: _, repeat: _, meta} => {
        match meta.get(identifier) {
          Some(id) => Some(*id),
          None => None,
        }
      },
      Self::Node {meta} => {
        match meta.get(identifier) {
          Some(id) => Some(*id),
          None => None,
        }
      }
    }
  }
}

#[derive(Debug)]
pub struct StubAST {
  pub nodes: Vec<StubNodes>,
  vector_count: usize,
  cycle_count: usize,
}

impl StubAST {
  pub fn new() -> Self {
    Self {
      nodes: vec!(),
      vector_count: 0,
      cycle_count: 0,
    }
  }

  pub fn reset(&mut self) -> () {
    self.nodes.clear();
    self.vector_count = 0;
    self.cycle_count = 0;
  }

  pub fn push_comment(&mut self, comment: &str) -> Result<(), Error> {
    self.nodes.push(StubNodes::Comment {
      content: comment.to_string(),
      meta: IndexMap::new(),
    });
    Ok(())
  }

  pub fn push_vector(&mut self, timeset_id: usize, repeat: usize) -> Result<(), Error> {
    self.nodes.push(StubNodes::Vector {
      timeset_id: timeset_id,
      repeat: repeat,
      meta: IndexMap::new(),
    });
    self.vector_count += 1;
    self.cycle_count += repeat;
    Ok(())
  }

  pub fn cycle_count(&self) -> usize {
    self.cycle_count
  }

  pub fn vector_count(&self) -> usize {
    self.vector_count
  }

  pub fn len(&self) -> usize {
    self.nodes.len()
  }
}

#[derive(Debug)]
pub struct Tester {
  /// The current timeset ID, if its set.
  /// This is the direct ID to the timeset object.
  /// The name and model ID can be found on this object.
  current_timeset_id: Option<usize>,

  /// Stubbed AST. Replace this with something else when it becomes available.
  ast: StubAST,

  external_generators: IndexMap<String, Generators>,
  pub target_generators: Vec<Generators>,
}

impl Tester {
  pub fn new() -> Self {
    Tester {
      current_timeset_id: Option::None,
      ast: StubAST::new(),
      external_generators: IndexMap::new(),
      target_generators: vec!(),
    }
  }

  pub fn _current_timeset_id(&self) -> Result<usize, Error> {
    match self.current_timeset_id {
      Some(t_id) => Ok(t_id),
      None => Err(Error::new(&format!("No timeset has been set!")))
    }
  }

  pub fn reset(&mut self) -> Result<(), Error> {
    self.clear_dut_dependencies()?;
    self.reset_external_generators()?;
    Ok(())
  }

  /// Clears all members which reference members on the current DUT.
  pub fn clear_dut_dependencies(&mut self) -> Result<(), Error> {
    self.ast.reset();
    self.current_timeset_id = Option::None;
    Ok(())
  }

  // Resets the external generators.
  // Also clears the targeted generators, as it may point to an external one that will be cleared.
  pub fn reset_external_generators(&mut self) -> Result<(), Error> {
    self.target_generators.clear();
    self.external_generators.clear();
    Ok(())
  }

  pub fn reset_targets(&mut self) -> Result<(), Error> {
    self.target_generators.clear();
    Ok(())
  }

  pub fn register_external_generator(&mut self, generator: &str) -> Result<(), Error> {
    self.external_generators.insert(generator.to_string(), Generators::External(generator.to_string()));
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
    Ok(())
  }

  pub fn clear_timeset(&mut self) -> Result<(), Error> {
    self.current_timeset_id = Option::None;
    Ok(())
  }

  pub fn issue_callback_at(&mut self, func: &str, idx: usize, dut: &Dut) -> Result<(), Error> {
    let g = &self.target_generators[idx];
    match g {
      Generators::DummyGeneratorWithInterceptors => {
        let g_ = DummyGeneratorWithInterceptors{};
        if func == "cc" {
          g_.cc(&mut self.ast)?;
        } else if func == "cycle" {
          g_.cycle(&mut self.ast)?;
        };
      },
      Generators::Simulator => {
        let g_ = Simulator{};
        match func {
          "cc" => g_.cc(&mut self.ast, dut)?,
          "cycle" => g_.cycle(&mut self.ast, dut)?,
          "set_timeset" => g_.set_timeset(self.current_timeset_id, dut)?,
          "clear_timeset" => g_.clear_timeset(dut)?,
          _ => {}, // Simulator does not have callbacks for other functions
        }
      }
      _ => {}
    }
    Ok(())
  }

  pub fn cc(&mut self, comment: &str) -> Result<(), Error> {
    self.ast.push_comment(comment)?;
    Ok(())
  }

  pub fn cycle(&mut self, repeat: Option<usize>) -> Result<(), Error>{
    self.ast.push_vector(self._current_timeset_id()?, repeat.unwrap_or(1))?;
    Ok(())
  }

  /// Generates the output for the target at index i.
  /// Allows the frontend to call generators in a loop.
  pub fn generate_target_at(&mut self, idx: usize, dut: &Dut) -> Result<GenerateStatus, Error> {
    let mut stat = GenerateStatus::new();
    let g = &self.target_generators[idx];
    match g {
      Generators::DummyGenerator => {
        DummyGenerator{}.generate(&self.ast, dut)?;
        stat.completed.push(Generators::DummyGenerator.to_string())
      },
      Generators::DummyGeneratorWithInterceptors => {
        DummyGeneratorWithInterceptors{}.generate(&self.ast, dut)?;
        stat.completed.push(Generators::DummyGeneratorWithInterceptors.to_string())
      }
      Generators::V93kSt7 => {
        V93KGen{}.generate(&self.ast, dut)?;
        stat.completed.push(Generators::V93kSt7.to_string())
      }
      Generators::Simulator => {
        Simulator{}.generate(&self.ast, dut)?;
        stat.completed.push(Generators::Simulator.to_string())
      }
      Generators::External(gen) => {
        stat.external.push(gen.to_string());
      }
    }
    Ok(stat)
  }

  pub fn target(&mut self, generator: &str) -> Result<(), Error> {
    let g;
    if let Some(_g) = Generators::from_str(generator) {
      g = _g;
    } else if let Some(_g) = self.external_generators.get(generator) {
      g = (*_g).clone();
    } else {
      return Err(Error::new(&format!("Could not find generator '{}'!", generator)));
    }

    if self.target_generators.contains(&g) {
        Err(Error::new(&format!("Generator {} has already been targeted!", generator)))
    } else {
      self.target_generators.push(g);
      Ok(())
    }
  }

  pub fn targets(&self) -> &Vec<Generators> {
    &self.target_generators
  }

  pub fn targets_as_strs(&self) -> Vec<String> {
    self.target_generators.iter().map( |g| g.to_string()).collect()
  }

  pub fn clear_targets(&mut self) -> Result<(), Error> {
    self.target_generators.clear();
    Ok(())
  }

  pub fn get_ast(&self) -> &StubAST {
    &self.ast
  }

  pub fn get_mut_ast(&mut self) -> &mut StubAST {
    &mut self.ast
  }

  pub fn generators(&self) -> Vec<String> {
    let mut gens = Generators::DummyGenerator.to_vector_strings();
    gens.extend(self.external_generators.iter().map(|(n, _)| n.clone()).collect::<Vec<String>>());
    gens
  }
}

pub struct GenerateStatus {
  pub completed: Vec<String>,
  pub external: Vec<String>,
}

impl GenerateStatus {
  pub fn new() -> Self {
    Self {
      completed: vec!(),
      external: vec!(),
    }
  }
}
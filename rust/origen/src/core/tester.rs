use crate::error::Error;
use super::model::timesets::timeset::{Timeset};
use indexmap::IndexMap;
use crate::testers::v93k::Generator as V93KGen;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Generators {
  DummyGenerator,
  DummyGeneratorWithInterceptors,
  V93kSt7,
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
      _ => None
    }
  }

  pub fn to_string(&self) -> String {
    match self {
      Self::DummyGenerator => "::DummyGenerator".to_string(),
      Self::DummyGeneratorWithInterceptors => "::DummyGeneratorWithInterceptors".to_string(),
      Self::V93kSt7 => "::V93K::ST7".to_string(),
      Self::External(g) => g.clone(),
    }
  }

  pub fn to_vector_strings(&self) -> Vec<String> {
    vec!("::DummyGenerator".to_string(), "::DummyGeneratorWithInterceptors".to_string(), "::V93K::ST7".to_string())
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
    meta: Vec<usize>,
  },
  Vector {
    //timeset: String,
    timeset_id: usize,
    repeat: usize,
    meta: Vec<usize>,
  },
  Node {
    meta: Vec<usize>,
  },
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
    //self.nodes.push(comment.to_string());
    self.nodes.push(StubNodes::Comment {
      content: comment.to_string(),
      meta: vec!(),
    });
    Ok(())
  }

  pub fn push_vector(&mut self, timeset_id: usize, repeat: usize) -> Result<(), Error> {
    //self.nodes.push(format!("Vector - Timeset ID: {}, Repeat: {}", timeset_id, repeat).to_string());
    self.nodes.push(StubNodes::Vector {
      timeset_id: timeset_id,
      repeat: repeat,
      meta: vec!(),
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

  // pub fn get_node(&self, idx) {

  // }
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
    self.ast.reset();
    self.current_timeset_id = Option::None;
    self.target_generators.clear();
    self.external_generators.clear();
    Ok(())
  }

  pub fn register_external_generator(&mut self, generator: &str) -> Result<(), Error> {
    self.external_generators.insert(generator.to_string(), Generators::External(generator.to_string()));
    Ok(())
  }

  pub fn get_timeset(&self, dut: &super::dut::Dut) -> Option<Timeset> {
    if let Some(t_id) = self.current_timeset_id {
      Some(dut.timesets[t_id].clone())
    } else {
      Option::None
    }
  }

  pub fn _get_timeset(&self, dut: &super::dut::Dut) -> Result<Timeset, Error> {
    if let Some(t_id) = self.current_timeset_id {
      Ok(dut.timesets[t_id].clone())
    } else {
      Err(Error::new(&format!("No timeset has been set!")))
    }
  }

  pub fn set_timeset(&mut self, dut: &super::dut::Dut, model_id: usize, timeset_name: &str) -> Result<(), Error> {
    self.current_timeset_id = Some(dut._get_timeset(model_id, timeset_name)?.id);
    self.issue_callbacks("set_timeset")?;
    Ok(())
  }

  pub fn clear_timeset(&mut self) -> Result<(), Error> {
    self.current_timeset_id = Option::None;
    self.issue_callbacks("clear_timeset")?;
    Ok(())
  }

  pub fn issue_callbacks(&mut self, func: &str) -> Result<(), Error> {
    for g in self.target_generators.iter() {
      match g {
        Generators::DummyGeneratorWithInterceptors => {
          let g_ = DummyGeneratorWithInterceptors{};
          if func == "cc" {
            g_.cc(&mut self.ast)?;
          } else if func == "cycle" {
            g_.cycle(&mut self.ast)?;
          };
        },
        _ => {}
      }
    }
    Ok(())
  }

  pub fn cc(&mut self, comment: &str) -> Result<(), Error> {
    self.ast.push_comment(comment)?;
    self.issue_callbacks("cc")?;
    Ok(())
  }

  pub fn cycle(&mut self, repeat: Option<usize>) -> Result<(), Error>{
    self.ast.push_vector(self._current_timeset_id()?, repeat.unwrap_or(1));
    self.issue_callbacks("cycle")?;
    Ok(())
  }

  pub fn generate(&mut self) -> Result<GenerateStatus, Error> {
    let mut stat = GenerateStatus::new();
    let num_targets;
    {
      num_targets = self.target_generators.len();
    }
    //for (i, g) in self.target_generators.iter().enumerate() {
    for i in 0..num_targets {
      let stat_ = self.generate_target_at(i)?;
      stat.completed.extend(stat_.completed);
      stat.external.extend(stat_.external);
      // match g {
      //   Generators::DummyGenerator => {
      //     DummyGenerator{}.generate(&self.ast)?;
      //     stat.completed.push(Generators::DummyGenerator.to_string())
      //   },
      //   Generators::DummyGeneratorWithInterceptors => {
      //     DummyGeneratorWithInterceptors{}.generate(&self.ast)?;
      //     stat.completed.push(Generators::DummyGeneratorWithInterceptors.to_string())
      //   }
      //   Generators::V93K_ST7 => {
      //     V93KGen{}.generate(&self.ast)?;
      //     stat.completed.push(Generators::V93K_ST7.to_string())
      //   }
      //   Generators::External(gen) => {
      //     stat.external.push(gen.to_string());
      //   }
      // }
    }
    Ok(stat)
  }

  /// Generates the output for the target at index i.
  /// Allows the frontend to call generators in a loop.
  pub fn generate_target_at(&mut self, idx: usize) -> Result<GenerateStatus, Error> {
    let mut stat = GenerateStatus::new();
    let g = &self.target_generators[idx];
    match g {
      Generators::DummyGenerator => {
        DummyGenerator{}.generate(&self.ast)?;
        stat.completed.push(Generators::DummyGenerator.to_string())
      },
      Generators::DummyGeneratorWithInterceptors => {
        DummyGeneratorWithInterceptors{}.generate(&self.ast)?;
        stat.completed.push(Generators::DummyGeneratorWithInterceptors.to_string())
      }
      Generators::V93kSt7 => {
        V93KGen{}.generate(&self.ast)?;
        stat.completed.push(Generators::V93kSt7.to_string())
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
    //let temp = self.external_generators.iter().map ( |n| n).collect::<Vec<String>>();
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

struct Generator {
  name: String,
  path: String,

}

struct DummyGenerator {}

impl DummyGenerator {

  /// A dummy generator which simply prints everything to the screen.
  pub fn generate(&self, ast: &StubAST) -> Result<(), Error> {
    println!("Printing StubAST to console...");
    for (i, n) in ast.nodes.iter().enumerate() {
      match n {
        StubNodes::Comment {content, meta} => println!("  ::DummyGenerator Node {}: Comment - Content: {}", i, content),
        StubNodes::Vector {timeset_id, repeat, meta} => println!("  ::DummyGenerator Node {}: Vector - Repeat: {}", i, repeat),
        StubNodes::Node {meta} => println!("  ::DummyGenerator Node {}: Node", i),
      }
    }
    Ok(())
  }
}

struct DummyGeneratorWithInterceptors {}

impl DummyGeneratorWithInterceptors {

  /// A dummy generator which simply prints everything to the screen.
  pub fn generate(&self, ast: &StubAST) -> Result<(), Error> {
    println!("Printing StubAST to console...");
    for (i, n) in ast.nodes.iter().enumerate() {
      match n {
        StubNodes::Comment {content, meta} => println!("  ::DummyGeneratorWithInterceptors Node {}: Comment - Content: {}", i, content),
        StubNodes::Vector {timeset_id, repeat, meta} => println!("  ::DummyGeneratorWithInterceptors Node {}: Vector - Repeat: {}", i, repeat),
        StubNodes::Node {meta} => println!("  ::DummyGeneratorWithInterceptors Node {}: Node", i),
      }
    }
    Ok(())
  }

  pub fn cycle(&self, ast: &mut StubAST) -> Result<(), Error> {
    let n = ast.nodes.last_mut().unwrap();
    match n {
      StubNodes::Vector {timeset_id, repeat, meta} => {
        println!("Vector intercepted by DummyGeneratorWithInterceptors!");
        Ok(())
      },
      _ => Err(Error::new(&format!("Error Intercepting Vector! Expected vector node!")))
    }
  }

  pub fn cc(&self, ast: &mut StubAST) -> Result<(), Error> {
    let n = ast.nodes.last().unwrap();
    let n_;
    match n {
      StubNodes::Comment {content, meta} => {
        println!("Comment intercepted by DummyGeneratorWithInterceptors!");
        n_ = StubNodes::Comment {
          content: String::from(format!("  Comment intercepted by DummyGeneratorWithInterceptors! {}", content.clone())),
          meta: meta.clone(),
        };
      },
      _ => return Err(Error::new(&format!("Error Intercepting Comment! Expected comment node!")))
    }
    drop(n);
    let i = ast.len() - 1;
    ast.nodes[i] = n_;
    Ok(())
  }
}

pub mod v93k;
pub mod simulator;
use crate::core::tester::{StubAST, StubNodes};
use crate::error::Error;
use crate::core::dut::{Dut};

pub struct DummyGenerator {}

impl DummyGenerator {

  /// A dummy generator which simply prints everything to the screen.
  pub fn generate(&self, ast: &StubAST, dut: &Dut) -> Result<(), Error> {
    println!("Printing StubAST to console...");
    for (i, n) in ast.nodes.iter().enumerate() {
      match n {
        StubNodes::Comment {content, ..} => println!("  ::DummyGenerator Node {}: Comment - Content: {}", i, content),
        StubNodes::Vector {timeset_id, repeat, ..} => {
          let t = &dut.timesets[*timeset_id];
          println!("  ::DummyGenerator Node {}: Vector - Repeat: {}, Timeset: '{}'", i, repeat, t.name);
        },
        StubNodes::Node { .. } => println!("  ::DummyGenerator Node {}: Node", i),
      }
    }
    Ok(())
  }
}

pub struct DummyGeneratorWithInterceptors {}

impl DummyGeneratorWithInterceptors {

  /// A dummy generator which simply prints everything to the screen.
  pub fn generate(&self, ast: &StubAST, dut: &Dut) -> Result<(), Error> {
    println!("Printing StubAST to console...");
    for (i, n) in ast.nodes.iter().enumerate() {
      match n {
        StubNodes::Comment {content, ..} => println!("  ::DummyGeneratorWithInterceptors Node {}: Comment - Content: {}", i, content),
        StubNodes::Vector {timeset_id, repeat, ..} => {
          let t = &dut.timesets[*timeset_id];
          println!("  ::DummyGeneratorWithInterceptors Node {}: Vector - Repeat: {}, Timeset: '{}'", i, repeat, t.name);
        },
        StubNodes::Node { .. } => println!("  ::DummyGeneratorWithInterceptors Node {}: Node", i),
      }
    }
    Ok(())
  }

  pub fn cycle(&self, ast: &mut StubAST) -> Result<(), Error> {
    let n = ast.nodes.last_mut().unwrap();
    match n {
      StubNodes::Vector { .. } => {
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
          content: String::from(format!("Comment intercepted by DummyGeneratorWithInterceptors! {}", content.clone())),
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

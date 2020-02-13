use crate::core::tester::{StubAST};
use crate::error::Error;
use crate::core::dut::{Dut};

pub struct Generator {}

impl Generator {
  /// The simulator's actions are done through the AST generation. There's actually nothing currently to do during generation.
  pub fn generate(&self, _ast: &StubAST, _dut: &Dut) -> Result<(), Error> {
    Ok(())
  }

  pub fn clear_timeset(&self, _dut: &Dut) -> Result<(), Error> {
    println!("<Issue command to clear the timeset in the simulator...>");
    Ok(())
  }

  pub fn set_timeset(&self, _timeset_id: Option<usize>, _dut: &Dut) -> Result<(), Error> {
    println!("<Issue command to set the timeset in the simulator...>");
    Ok(())
  }

  pub fn cycle(&self, _ast: &mut StubAST, _dut: &Dut) -> Result<(), Error> {
    println!("<Issue command to cycle the simulator...>");
    Ok(())
  }

  pub fn cc(&self, _ast: &mut StubAST, _dut: &Dut) -> Result<(), Error> {
    println!("<Issue command to place a comment in the simulator...>");
    Ok(())
  }
}
use crate::core::tester::{StubAST, StubNodes};
use crate::error::Error;
use crate::core::dut::{Dut};

pub struct Generator {}

impl Generator {
  /// An extremely elementary generator for the 93. Won't actually generate anything of use yet.
  pub fn generate(&self, ast: &StubAST, dut: &Dut) -> Result<(), Error> {
    for (_i, n) in ast.nodes.iter().enumerate() {
      match n {
        StubNodes::Comment {content, ..} => println!("# {}", content),
        StubNodes::Vector {timeset_id, repeat, ..} => {
          let t = &dut.timesets[*timeset_id];
          println!("R{} {} <Pins> # <EoL Comment>;", repeat, t.name);
        },
        StubNodes::Node {..} => return Err(Error::new(&format!("Pure meta nodes are not supported by the V93K yet!"))),
      }
    }
    Ok(())
  }
}
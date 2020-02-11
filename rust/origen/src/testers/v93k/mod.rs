use crate::core::tester::{StubAST, StubNodes};
use crate::error::Error;

pub struct Generator {}

impl Generator {
  /// An extremely elementary generator for the 93. Won't actually generate anything of use yet.
  pub fn generate(&self, ast: &StubAST) -> Result<(), Error> {
    for (i, n) in ast.nodes.iter().enumerate() {
      match n {
        StubNodes::Comment {content, meta} => println!("# {}", content),
        StubNodes::Vector {timeset_id, repeat, meta} => println!("R{} <Timeset> <Pins> <EoL Comment>;", repeat),
        StubNodes::Node {meta} => return Err(Error::new(&format!("Pure meta nodes are not supported by the V93K yet!"))),
      }
    }
    Ok(())
  }
}
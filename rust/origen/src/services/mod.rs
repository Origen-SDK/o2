pub mod jtag;

#[derive(Debug)]
pub enum Service {
    None, // Not used, but removes a compiler warning until we have more service types
    JTAG(jtag::Service),
}

impl Service {}

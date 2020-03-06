use crate::drivers::jtag;

#[derive(Debug)]
pub enum Service {
    JTAG(jtag::API),
}

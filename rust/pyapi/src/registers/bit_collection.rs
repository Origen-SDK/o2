use pyo3::prelude::*;

/// A BitCollection represents either a whole register of a subset of a
/// registers bits (not necessarily contiguous bits) and provides the user
/// with the same API to set and consume register data in both cases.
#[pyclass]
#[derive(Debug)]
pub struct BitCollection {
    /// The ID of the parent register
    reg_id: usize,
    /// When true the BitCollection contains an entire register's worth of bits
    whole: bool,
    /// The index numbers of the bits from the register that are included in this
    /// collection. Typically this will be mapped to the actual register bits by
    /// BitCollection's methods.
    bit_numbers: Vec<u16>,
    /// Iterator index
    i: usize,
}

/// Rust-private methods, i.e. not accessible from Python
impl BitCollection {
    pub fn from_reg_id(id: usize) -> BitCollection {
        BitCollection {
            reg_id: id,
            whole: true,
            bit_numbers: Vec::new(),
            i: 0,
        }
    }
}

/// Methods available from Rust and Python
#[pymethods]
impl BitCollection {}

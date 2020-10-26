//! This module implements classes which expose ATE-specific APIs for use in both pattern and program generation.
mod v93k;

use origen::prog_gen::TestInvocation;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use v93k::TestSuite;

#[pymodule]
fn tester_apis(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<v93k::V93K>()?;
    Ok(())
}

pub fn to_test_invocation(obj: &PyAny) -> Option<TestInvocation> {
    if let Ok(ts) = obj.extract::<&TestSuite>() {
        log_info!("Got V93K test! - {}", ts.name);
        let t = TestInvocation {
            test_id: 0,
            test_inv_id: 0,
        };
        Some(t)
    } else {
        None
    }
}

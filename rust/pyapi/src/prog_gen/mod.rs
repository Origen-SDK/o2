//! Implements Python bindings for program generation data structures and functions

pub mod interface;

use origen::testers::SupportedTester;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Test {
    pub id: usize,
    pub name: String,
    pub tester: SupportedTester,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct TestInvocation {
    pub id: usize,
    pub test_id: Option<usize>,
    pub name: String,
    pub tester: SupportedTester,
}
//! The framework module contains functionality that is closely related to implementing a
//! a particular framework feature. While still generic in nature, they are not so low-level
//! as the functions found in the utils module which could be used in a variety of different
//! contexts.
//! The functionality in this module assumes that it will be used to implement something very
//! similar to Origen's implementation of the given feature.
pub mod logger;
pub mod reference_files;
pub mod sessions;
pub mod typed_value;
pub mod users;

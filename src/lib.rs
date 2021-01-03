#![allow(dead_code)]

use nalgebra::{Dynamic, MatrixMN, U3};
use pyo3::prelude::*;

pub type CoordMat<N> = MatrixMN<N, U3, Dynamic>;

pub mod adjacency;
mod agnr;
pub mod structure;

pub use agnr::AGNR;

/// A Python module implemented in Rust.
#[pymodule]
fn agnr_ml(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<AGNR>()?;

    Ok(())
}

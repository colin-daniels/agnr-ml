#![allow(dead_code)]

use nalgebra::{Dynamic, MatrixMN, U3};

pub type CoordMat<N> = MatrixMN<N, U3, Dynamic>;

pub mod adjacency;
pub mod generate;
pub mod structure;

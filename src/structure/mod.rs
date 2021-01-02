use serde::{Deserialize, Serialize};

pub mod poscar;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Atom {
    pub elem: String,
    pub xyz: [f64; 3],
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AtomicStructure<M = ()> {
    pub lattice_vectors: [[f64; 3]; 3],
    pub atoms: Vec<Atom>,
    pub meta: M,
}

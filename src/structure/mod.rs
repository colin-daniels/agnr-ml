use pyo3::prelude::*;
use std::convert::TryInto;
use vasp_poscar::Poscar;

mod poscar;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Atom {
    pub elem: String,
    pub xyz: [f64; 3],
}

#[pyclass(module = "agnr_ml")]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct AtomicStructure {
    pub lattice_vectors: [[f64; 3]; 3],
    pub atoms: Vec<Atom>,
}

#[pymethods]
impl AtomicStructure {
    fn lattice(&self) -> Vec<Vec<f64>> {
        self.lattice_vectors.iter().map(|v| v.to_vec()).collect()
    }

    fn types(&self) -> Vec<String> {
        self.atoms.iter().map(|a| a.elem.clone()).collect()
    }

    fn coords(&self) -> Vec<Vec<f64>> {
        self.atoms.iter().map(|a| a.xyz.to_vec()).collect()
    }

    fn to_poscar_string(&self) -> String {
        let poscar: Poscar = self.try_into().unwrap();
        format!("{}", poscar)
    }
}

use crate::adjacency::add_hydrogen;
use crate::agnr::generation::generate_all_agnrs;
use crate::agnr::spec::AGNRSpec;
use crate::structure::AtomicStructure;
use itertools::Itertools;
use pyo3::prelude::*;
use std::convert::TryInto;
use vasp_poscar::Poscar;

mod generation;
mod spec;

#[pyclass(module = "agnr_ml")]
pub struct AGNR {
    spec: AGNRSpec,
}

#[pymethods]
impl AGNR {
    #[new]
    pub fn new(spec: Vec<(i32, i32)>) -> Self {
        assert!(!spec.is_empty());

        // TODO: modify to this converts from the paper's version of the spec
        AGNR {
            spec: AGNRSpec(spec),
        }
    }

    #[staticmethod]
    pub fn all_possible_agnrs(
        min_len: usize,
        max_len: usize,
        min_width: usize,
        max_width: usize,
        symmetric_only: bool,
    ) -> Vec<AGNR> {
        let all_agnrs = generate_all_agnrs(
            min_len..(max_len + 1),
            min_width..(max_width + 1),
            symmetric_only,
            max_width,
        );

        all_agnrs
            .into_iter()
            .map(|spec| AGNR::new(spec.0))
            .collect_vec()
    }

    /// Number of dimer lines across the AGNR
    pub fn len(&self) -> usize {
        self.spec.len() * 2
    }

    /// Total width if bounding on top/bottom
    pub fn total_width(&self) -> usize {
        self.spec.width().unwrap().try_into().unwrap()
    }

    #[getter]
    pub fn spec(&self) -> Vec<(i32, i32)> {
        // TODO: modify to this converts to the paper's version of the spec
        self.spec.0.clone()
    }
}

impl AGNR {
    /// Build a Poscar from an AGNR
    pub fn to_poscar(
        &self,
        cc_bond: Option<f64>,
        ch_bond: Option<f64>,
        vacuum_sep: Option<f64>,
    ) -> Poscar {
        use std::iter::once;
        use vasp_poscar::{Builder, Coords};

        let cc_bond = cc_bond.unwrap_or(1.42045);
        let ch_bond = ch_bond.unwrap_or(1.09047);
        let vacuum_sep = vacuum_sep.unwrap_or(15.0);

        let dx = (3.0 * cc_bond) / 2.0;
        let dy = (f64::sqrt(3.0) * cc_bond) / 2.0;
        let cc_cutoff = cc_bond * 1.1;

        let spec = &self.spec;
        // compute the coordinates of all of the carbon atoms
        let coords = spec
            .0
            .iter()
            .enumerate()
            .flat_map(|(i, s)| {
                // note: the (constant) adjustments to x/y/z are just to center the structure
                // nicely in the periodic cell
                let x = dx * i as f64 - cc_bond / 2.0;
                (s.0..s.1).step_by(2).flat_map(move |y| {
                    let y = y as f64 * dy + vacuum_sep / 2.0;
                    let z = vacuum_sep / 2.0;
                    let atom_1 = [x, y, z];
                    let atom_2 = [x + cc_bond, y, z];
                    Iterator::chain(once(atom_1), once(atom_2))
                })
            })
            .collect_vec();

        // build a Poscar from just the carbon atoms
        let poscar = Builder::new()
            .group_counts(vec![coords.len()])
            .group_symbols(vec!["C"])
            .lattice_vectors(&[
                [dx * spec.len() as f64, 0.0, 0.0],
                [0.0, dy * spec.width().unwrap() as f64 + vacuum_sep, 0.0],
                [0.0, 0.0, vacuum_sep],
            ])
            .positions(Coords::Cart(coords))
            .build()
            .unwrap();

        // add hydrogen and that's it
        add_hydrogen(poscar, ch_bond, cc_cutoff)
    }

    /// Build an AtomicStructure from an AGNR
    pub fn to_structure(
        &self,
        cc_bond: Option<f64>,
        ch_bond: Option<f64>,
        vacuum_sep: Option<f64>,
    ) -> AtomicStructure {
        self.to_poscar(cc_bond, ch_bond, vacuum_sep).into()
    }
}

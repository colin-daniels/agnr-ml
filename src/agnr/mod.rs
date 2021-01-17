use crate::adjacency::add_hydrogen;
use crate::structure::AtomicStructure;
use itertools::Itertools;
use pyo3::prelude::*;
use std::convert::TryInto;
use vasp_poscar::Poscar;

mod generation;

#[pyclass(module = "agnr_ml")]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct AGNR {
    spec: Vec<(i32, i32)>,
}

#[pymethods]
impl AGNR {
    #[new]
    pub fn new(spec: Vec<(i32, i32)>) -> Self {
        assert!(!spec.is_empty());
        // TODO: modify to this converts from the paper's version of the spec
        Self { spec }
    }

    #[getter]
    pub fn spec(&self) -> Vec<(i32, i32)> {
        // TODO: modify to this converts to the paper's version of the spec
        self.spec.clone()
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

impl AGNR {
    pub fn len(&self) -> usize {
        self.spec.len()
    }

    pub fn width(&self) -> Option<i32> {
        self.spec.iter().map(|s| s.1).max()
    }

    pub fn possible_extensions(&self) -> Option<[(i32, i32); 4]> {
        self.spec.last().map(|&(l, h)| {
            [
                // grow by 1 (y must shift down)
                (l - 1, h + 1),
                // shrink by 1 (y must shift up)
                (l + 1, h - 1),
                // stay same width (can do either)
                (l + 1, h + 1),
                (l - 1, h - 1),
            ]
        })
    }

    pub fn name(&self) -> Option<String> {
        const CHAR_MAP: &'static [u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
        let mut name = Vec::new();
        for &(low, high) in &self.spec {
            let low: usize = low.try_into().ok()?;
            let high: usize = high.try_into().ok().filter(|&h| h < CHAR_MAP.len())?;

            // dont output first number since it's always 0 according to our convention
            if !name.is_empty() {
                name.push(CHAR_MAP[low]);
            } else {
                assert_eq!(low, 0, "spec should always start with 0");
            }

            name.push(CHAR_MAP[high]);
        }

        Some(String::from_utf8(name).unwrap())
    }

    pub fn is_periodic(&self) -> bool {
        if self.spec.len() == 0 {
            return true;
        }

        let first = self.spec.first().unwrap();
        for next in &self.possible_extensions().unwrap() {
            if next == first {
                return true;
            }
        }
        false
    }

    /// Get the "minimum" spec after applying all possible transformations.
    /// Returns the minimum and whether or not the structure has any symmetries.
    pub fn minimum_image(&self) -> (Self, bool) {
        // if we hit the requisite length and we can properly "connect" back to ourselves
        // across the periodic boundary
        let width = self.width().unwrap();
        let mut temp = self.spec.clone();
        let mut minimum_image = self.spec.clone();
        let mut has_symmetry = false;

        // get the "minimum" spec out of all possible images of the GNR
        for &x_mirror in &[false, true] {
            for &y_mirror in &[false, true] {
                for shift in 0..temp.len() {
                    // check + add symmetries
                    if (x_mirror || y_mirror || shift != 0) && temp == self.spec {
                        has_symmetry = true;
                    }

                    // check for minimum
                    if temp < minimum_image {
                        minimum_image.copy_from_slice(&temp);
                    }

                    // translations
                    temp.rotate_right(1);
                }
                // y mirror plane
                temp.reverse();
            }
            // x mirror plane
            for v in &mut temp {
                *v = (width - v.1, width - v.0);
            }
        }

        (Self::new(minimum_image), has_symmetry)
    }

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

        // horizontal distance between each hexagonal "segment" of the GNR
        let dx = 1.5 * cc_bond;
        // vertical distance between each carbon atom in the GNR
        let dy = (f64::sqrt(3.0) * cc_bond) / 2.0;

        let spec = &self.spec;
        // compute the coordinates of all of the carbon atoms
        let coords = spec
            .iter()
            .enumerate()
            .flat_map(|(i, s)| {
                let x = i as f64 * dx;
                (s.0..s.1).step_by(2).flat_map(move |j| {
                    let y = j as f64 * dy;
                    let atom_1 = [x, y];
                    let atom_2 = [x + cc_bond, y];
                    Iterator::chain(once(atom_1), once(atom_2))
                })
            })
            .map(|[x, y]| {
                // adjust x/y and add z coord to center the structure nicely in the periodic cell
                [x - cc_bond / 2.0, y + vacuum_sep / 2.0, vacuum_sep / 2.0]
            })
            .collect_vec();

        let gnr_width = (self.width().unwrap() - 2) as f64 * dy;

        // build a Poscar from just the carbon atoms
        let poscar = Builder::new()
            .group_counts(vec![coords.len()])
            .group_symbols(vec!["C"])
            .lattice_vectors(&[
                [dx * self.len() as f64, 0.0, 0.0],
                [0.0, gnr_width + vacuum_sep, 0.0],
                [0.0, 0.0, vacuum_sep],
            ])
            .positions(Coords::Cart(coords))
            .build()
            .unwrap();

        // add hydrogen and that's it
        let cutoff_distance = cc_bond * 1.1;
        add_hydrogen(poscar, ch_bond, cutoff_distance)
    }
}

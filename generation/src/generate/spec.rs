use crate::adjacency::add_hydrogen;
use crate::structure::AtomicStructure;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::convert::TryInto;
use std::hash::Hasher;
use vasp_poscar::Poscar;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Symmetry {
    MirrorX,
    MirrorY,
    Shift { amount: i32 },
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct AGNRSpec {
    // assumed ordered
    pub spec: Vec<(i32, i32)>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub symmetries: Vec<Vec<Symmetry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structure: Option<AtomicStructure>,
}

impl std::hash::Hash for AGNRSpec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.spec.hash(state)
    }
}

impl PartialOrd for AGNRSpec {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.spec.partial_cmp(&other.spec)
    }
}

impl Ord for AGNRSpec {
    fn cmp(&self, other: &Self) -> Ordering {
        self.spec.cmp(&other.spec)
    }
}

impl PartialEq for AGNRSpec {
    fn eq(&self, other: &Self) -> bool {
        self.spec.eq(&other.spec)
    }
}

impl Eq for AGNRSpec {}

impl AGNRSpec {
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

    pub fn to_poscar(&self) -> Poscar {
        use std::iter::once;
        use vasp_poscar::{Builder, Coords};

        const DX: f64 = 4.26119 / 2.0;
        const DY: f64 = 2.46051 / 2.0;
        const BOND_LEN: f64 = 1.42045;
        const VACUUM_SEP: f64 = 15.0;
        const HYDROGEN_DIST: f64 = 1.09047;
        const CUTOFF_DISTANCE: f64 = BOND_LEN * 1.1;

        let coords = self
            .spec
            .iter()
            .enumerate()
            .flat_map(|(i, s)| {
                // note: the (constant) adjustments to x/y/z are just to center the structure
                // nicely in the periodic cell
                let x = DX * i as f64 - BOND_LEN / 2.0;
                (s.0..s.1).step_by(2).flat_map(move |y| {
                    let y = y as f64 * DY + VACUUM_SEP / 2.0;
                    let z = VACUUM_SEP / 2.0;
                    Iterator::chain(once([x, y, z]), once([x + BOND_LEN, y, z]))
                })
            })
            .collect_vec();

        let poscar = Builder::new()
            .group_counts(vec![coords.len()])
            .group_symbols(vec!["C"])
            .lattice_vectors(&[
                [DX * self.len() as f64, 0.0, 0.0],
                [0.0, DY * self.width().unwrap() as f64 + VACUUM_SEP, 0.0],
                [0.0, 0.0, VACUUM_SEP],
            ])
            .positions(Coords::Cart(coords))
            .build()
            .unwrap();

        add_hydrogen(poscar, HYDROGEN_DIST, CUTOFF_DISTANCE)
    }

    pub fn generate_all(
        starting_width: usize,
        length: usize,
        db: &mut HashSet<Self>,
        symmetric_only: bool,
        max_width: usize,
    ) {
        const MIN_WIDTH: i32 = 2;
        let starting_width: i32 = starting_width.try_into().unwrap();

        assert_ne!(starting_width, 0);
        assert_ne!(length, 0);
        assert!(starting_width >= MIN_WIDTH);

        fn gen(
            current: &mut AGNRSpec,
            length: usize,
            db: &mut HashSet<AGNRSpec>,
            symmetric_only: bool,
            max_width: usize,
        ) {
            if current.len() != length {
                for &next in &current.possible_extensions().unwrap() {
                    let next_width = next.1 - next.0;
                    if next_width <= 2 * max_width as i32
                        && next_width >= 2 * MIN_WIDTH
                        && next.0 >= 0
                    {
                        current.spec.push(next);
                        gen(current, length, db, symmetric_only, max_width);
                        current.spec.pop();
                    }
                }
            } else if current.is_periodic() {
                // if we hit the requisite length and we can properly "connect" back to ourselves
                // across the periodic boundary
                let width = current.width().unwrap();
                let mut temp = current.clone();
                let mut to_add = current.clone();

                // get the "minimum" spec out of all possible images of the GNR
                for &x_mirror in &[false, true] {
                    for &y_mirror in &[false, true] {
                        for shift in 0..temp.len() {
                            // check + add symmetries
                            if (x_mirror || y_mirror || shift != 0) && temp.spec == current.spec {
                                let mut symm = Vec::new();
                                if x_mirror {
                                    symm.push(Symmetry::MirrorX);
                                }
                                if y_mirror {
                                    symm.push(Symmetry::MirrorY);
                                }
                                if shift != 0 {
                                    symm.push(Symmetry::Shift {
                                        amount: shift as i32,
                                    })
                                }
                                to_add.symmetries.push(symm);
                            }

                            // check for minimum
                            if temp.spec < to_add.spec {
                                to_add.spec.copy_from_slice(&temp.spec);
                            }

                            // translations
                            temp.spec.rotate_right(1);
                        }
                        // y mirror plane
                        temp.spec.reverse();
                    }
                    // x mirror plane
                    for v in &mut temp.spec {
                        *v = (width - v.1, width - v.0);
                    }
                }

                if !symmetric_only || !to_add.symmetries.is_empty() {
                    db.insert(to_add);
                }
            }
        }

        let mut initial = AGNRSpec {
            spec: vec![(0, 2 * starting_width)],
            symmetries: vec![],
            structure: None,
        };
        gen(&mut initial, length, db, symmetric_only, max_width);
    }
}

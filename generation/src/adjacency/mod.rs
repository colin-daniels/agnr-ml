use crate::adjacency::graph::{CompressedGraph, Edge};
use crate::CoordMat;
use itertools::Itertools;
use nalgebra::{Matrix3, Vector3};
use vasp_poscar::Poscar;

pub mod graph;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, PartialEq, Eq)]
pub struct Bond {
    pub from: usize,
    pub to: usize,
    pub image_offset: [i32; 3],
}

impl From<Bond> for Edge {
    fn from(b: Bond) -> Self {
        Self {
            from: b.from,
            to: b.to,
            meta: (),
        }
    }
}

impl Bond {
    pub fn to_delta(&self, lattice: &Matrix3<f64>, coords: &CoordMat<f64>) -> Vector3<f64> {
        let [a, b, c] = self.image_offset;
        (coords.column(self.to)
            + a as f64 * lattice.column(0)
            + b as f64 * lattice.column(1)
            + c as f64 * lattice.column(2))
            - coords.column(self.from)
    }
}

pub fn calc_bonds(
    // columns are lattice vectors
    lattice: &Matrix3<f64>,
    coords: &CoordMat<f64>,
    cutoff_distance: f64,
) -> Vec<Vec<Bond>> {
    assert!(cutoff_distance > 0.0);

    // number of necessary periodic images in each lattice direction
    let (ia, ib, ic) = (0..3)
        .map(|idx| {
            let direction = lattice.column(idx);
            // determine how many images of the unit cell we need for each periodic direction by first
            // getting the distance between the two planes in the periodic cell that go in this lattice
            // direction
            let plane_vec_a = lattice.column((idx + 1) % 3);
            let plane_vec_b = lattice.column((idx + 2) % 3);
            let plane_normal = plane_vec_a.cross(&plane_vec_b);
            let plane_distance = f64::abs(plane_normal.dot(&direction) / plane_normal.norm());
            // and then just comparing that distance to the cutoff
            i32::max(2, f64::ceil(cutoff_distance / plane_distance) as i32)
        })
        .collect_tuple()
        .unwrap();

    #[rustfmt::skip]
        let image_offsets = (0..ia)
        .flat_map(move |a| (0..ib)
            .flat_map(move |b| (0..ic)
                .map(move |c| [a, b, c]
                )));

    let n = coords.ncols();
    let cutoff_squared = cutoff_distance * cutoff_distance;
    let (va, vb, vc) = lattice.column_iter().collect_tuple().unwrap();

    let mut bonds = vec![Vec::new(); n];
    for image_offset in image_offsets {
        #[rustfmt::skip]
            let coord_offset = va * image_offset[0] as f64
            + vb * image_offset[1] as f64
            + vc * image_offset[2] as f64;

        let in_unit_cell = match image_offset {
            [0, 0, 0] => true,
            _ => false,
        };

        for (i, from) in coords.column_iter().enumerate() {
            let from = from + coord_offset;

            for (j, to) in coords.column_iter().enumerate() {
                let dist_squared = (to - from).norm_squared();
                // note: second check is to ensure we don't double count
                if dist_squared <= cutoff_squared && (!in_unit_cell || i < j) {
                    let [a, b, c] = image_offset;
                    bonds[i].push(Bond {
                        from: i,
                        to: j,
                        image_offset: [-a, -b, -c],
                    });
                    bonds[j].push(Bond {
                        from: j,
                        to: i,
                        image_offset: [a, b, c],
                    });
                }
            }
        }
    }

    bonds
}

pub fn calc_graph(
    // columns are lattice vectors
    lattice: &Matrix3<f64>,
    coords: &CoordMat<f64>,
    cutoff_distance: f64,
) -> CompressedGraph {
    let bond_list = calc_bonds(lattice, coords, cutoff_distance);
    bond_list.into_iter().flat_map(|b| b.into_iter()).collect()
}

pub fn add_hydrogen(poscar: Poscar, hydrogen_dist: f64, cutoff_distance: f64) -> Poscar {
    use vasp_poscar::{Builder, Coords};

    macro_rules! elements_of {
        ($it:expr) => {
            $it.iter().flat_map(|c| c.iter().copied())
        };
    }

    let lattice = Matrix3::from_iterator(elements_of!(poscar.scaled_lattice_vectors()));
    let mut new_coords = poscar.scaled_cart_positions().iter().copied().collect_vec();

    let coords = CoordMat::from_iterator(new_coords.len(), elements_of!(new_coords));
    let bond_list = calc_bonds(&lattice, &coords, cutoff_distance);

    let num_carbon = new_coords.len();
    new_coords.extend(bond_list.into_iter().enumerate().filter_map(|(i, bonds)| {
        match bonds.as_slice() {
            [b1, b2] => {
                let b1 = b1.to_delta(&lattice, &coords).normalize();
                let b2 = b2.to_delta(&lattice, &coords).normalize();
                let h_bond = -hydrogen_dist * (b1 + b2).normalize();
                let h = coords.column(i) + h_bond;
                Some([h[0], h[1], h[2]])
            }
            _ => None,
        }
    }));
    let num_hydrogen = new_coords.len() - num_carbon;

    Builder::new()
        .group_counts(vec![num_carbon, num_hydrogen])
        .group_symbols(vec!["C", "H"])
        .lattice_vectors(&poscar.scaled_lattice_vectors())
        .positions(Coords::Cart(new_coords))
        .build()
        .unwrap()
}

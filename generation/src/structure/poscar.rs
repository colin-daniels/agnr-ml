use super::{Atom, AtomicStructure};
use std::convert::TryInto;
use vasp_poscar::{Poscar, ValidationError};

impl<M: Default> From<Poscar> for AtomicStructure<M> {
    fn from(p: Poscar) -> Self {
        let lattice_vectors = p.scaled_lattice_vectors();
        let coords = p.scaled_cart_positions();

        let atoms = if coords.is_empty() {
            Default::default()
        } else {
            let types = p
                .site_symbols()
                .expect("failed to get site symbols from POSCAR");

            types
                .zip(coords.iter().copied())
                .map(|(elem, xyz)| Atom {
                    elem: elem.into(),
                    xyz,
                })
                .collect()
        };

        Self {
            lattice_vectors,
            atoms,
            meta: Default::default(),
        }
    }
}

impl<M> TryInto<Poscar> for AtomicStructure<M> {
    type Error = ValidationError;

    fn try_into(self) -> Result<Poscar, Self::Error> {
        use vasp_poscar::{Builder, Coords};

        // get list of elements and how many of each there are
        let mut elements = Vec::new();
        for Atom { elem, .. } in &self.atoms {
            match elements.last_mut() {
                Some((last, count)) if last == elem => *count += 1,
                _ => elements.push((elem.clone(), 1)),
            }
        }

        let element_counts = elements.iter().map(|(_, count)| *count);
        let element_symbols = elements.iter().map(|(symbol, _)| symbol.clone());

        Builder::new()
            .group_counts(element_counts)
            .group_symbols(element_symbols)
            .lattice_vectors(&self.lattice_vectors)
            .positions(Coords::Cart(self.atoms.iter().map(|a| a.xyz)))
            .build()
    }
}

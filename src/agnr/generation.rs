use crate::agnr::AGNR;
use pyo3::prelude::*;
use std::collections::HashSet;
use std::convert::TryInto;

fn recursive_gen(
    current: &mut AGNR,
    length: usize,
    symmetric_only: bool,
    min_width: usize,
    max_width: usize,
    possible_agnrs: &mut HashSet<AGNR>,
) {
    if current.len() != length {
        for &next in &current.possible_extensions().unwrap() {
            let next_width = next.1 - next.0;
            // TODO: check if we can even reach the end, could save time
            if next_width <= 2 * max_width as i32
                && next_width >= 2 * min_width as i32
                && next.0 >= 0
            {
                // TODO: check python signals
                current.spec.push(next);
                recursive_gen(
                    current,
                    length,
                    symmetric_only,
                    min_width,
                    max_width,
                    possible_agnrs,
                );
                current.spec.pop();
            }
        }
    } else if current.is_periodic() {
        // get the so-called "minimum" agnr so that we don't include duplicates
        let (minimum_image, has_symmetry) = current.minimum_image();
        if !symmetric_only || has_symmetry {
            possible_agnrs.insert(minimum_image);
        }
    }
}

#[pymethods]
impl AGNR {
    /// Generate all possible AGNRs by brute force.
    #[staticmethod]
    pub fn generate_all_agnrs(
        min_len: usize,
        max_len: usize,
        min_width: usize,
        max_width: usize,
        symmetric_only: bool,
    ) -> HashSet<AGNR> {
        assert!(min_len > 0);
        assert!(min_len <= max_len);

        assert!(min_width >= 1);
        assert!(min_width <= max_width);

        let mut all_gnrs = HashSet::<AGNR>::default();
        for length in min_len..=max_len {
            let mut all_gnrs_with_len = HashSet::default();
            for width in min_width..=max_width {
                let width: i32 = width.try_into().unwrap();
                let initial = &mut AGNR::new(vec![(0, 2 * width)]);
                recursive_gen(
                    initial,
                    length * 2,
                    symmetric_only,
                    min_width,
                    max_width,
                    &mut all_gnrs_with_len,
                );
            }

            for section_len in min_len..length {
                if length % section_len == 0 {
                    // only keep GNRs which don't repeat, since they will be
                    // generated for smaller lengths
                    all_gnrs_with_len.retain(|gnr| {
                        let beginning = &gnr.spec[0..(section_len * 2)];
                        let symm = beginning.repeat(length / section_len);
                        gnr.spec != symm
                    });
                }
            }

            all_gnrs.extend(all_gnrs_with_len.into_iter());
        }

        all_gnrs
    }
}

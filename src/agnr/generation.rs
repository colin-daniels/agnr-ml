use crate::agnr::spec::AGNRSpec;
use std::collections::HashSet;
use std::convert::TryInto;
use std::ops::Range;

fn recursive_gen(
    current: &mut AGNRSpec,
    length: usize,
    symmetric_only: bool,
    max_width: usize,
    possible_agnrs: &mut HashSet<AGNRSpec>,
) {
    const MIN_WIDTH: i32 = 2;

    if current.len() != length {
        for &next in &current.possible_extensions().unwrap() {
            let next_width = next.1 - next.0;
            // TODO: check if we can even reach the end, could save time
            if next_width <= 2 * max_width as i32 && next_width >= 2 * MIN_WIDTH && next.0 >= 0 {
                // TODO: check python signals
                current.0.push(next);
                recursive_gen(current, length, symmetric_only, max_width, possible_agnrs);
                current.0.pop();
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

/// Generate all possible AGNRs by brute force.
pub fn generate_all_agnrs(
    lengths: Range<usize>,
    widths: Range<usize>,
    symmetric_only: bool,
    max_width: usize,
) -> HashSet<AGNRSpec> {
    assert!(lengths.start > 0);
    assert!(lengths.start < lengths.end);

    assert!(widths.start > 1);
    assert!(widths.start < widths.end);

    let mut all_gnrs = HashSet::<AGNRSpec>::default();
    for l in lengths.clone() {
        let mut all_gnrs_with_len = HashSet::default();
        for width in widths.clone() {
            let width: i32 = width.try_into().unwrap();
            let initial = &mut AGNRSpec(vec![(0, 2 * width)]);
            recursive_gen(
                initial,
                l * 2,
                symmetric_only,
                max_width,
                &mut all_gnrs_with_len,
            );
        }

        for s in lengths.start..l {
            if l % s == 0 {
                // only keep GNRs which don't repeat, since they will be
                // generated for smaller lengths
                all_gnrs_with_len.retain(|gnr| {
                    let beginning = &gnr.0[0..(s * 2)];
                    let symm = beginning.repeat(l / s);
                    gnr.0 != symm
                });
            }
        }

        all_gnrs.extend(all_gnrs_with_len.into_iter());
    }

    all_gnrs
}

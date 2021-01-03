use std::convert::TryInto;

#[derive(Default, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct AGNRSpec(pub(super) Vec<(i32, i32)>);

impl AGNRSpec {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn width(&self) -> Option<i32> {
        self.0.iter().map(|s| s.1).max()
    }

    pub fn possible_extensions(&self) -> Option<[(i32, i32); 4]> {
        self.0.last().map(|&(l, h)| {
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
        for &(low, high) in &self.0 {
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
        if self.0.len() == 0 {
            return true;
        }

        let first = self.0.first().unwrap();
        for next in &self.possible_extensions().unwrap() {
            if next == first {
                return true;
            }
        }
        false
    }

    /// Get the "minimum" spec after applying all possible transformations.
    /// Returns the minimum and whether or not the structure has any symmetries.
    pub fn minimum_image(&self) -> (AGNRSpec, bool) {
        // if we hit the requisite length and we can properly "connect" back to ourselves
        // across the periodic boundary
        let width = self.width().unwrap();
        let mut temp = self.0.clone();
        let mut minimum_image = self.0.clone();
        let mut has_symmetry = false;

        // get the "minimum" spec out of all possible images of the GNR
        for &x_mirror in &[false, true] {
            for &y_mirror in &[false, true] {
                for shift in 0..temp.len() {
                    // check + add symmetries
                    if (x_mirror || y_mirror || shift != 0) && temp == self.0 {
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

        (AGNRSpec(minimum_image), has_symmetry)
    }
}

use std::collections::HashSet;
use std::convert::TryInto;
use vasp_poscar::failure::_core::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct GNRSlice {
    low: i16,
    high: i16,
}

impl std::hash::Hash for GNRSlice {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.low.hash(state);
        self.high.hash(state);
    }
}

impl Default for GNRSlice {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl GNRSlice {
    #[inline(always)]
    pub fn new(low: i16, high: i16) -> Self {
        debug_assert!(low <= high);
        Self { low, high }
    }

    #[inline(always)]
    pub fn width(&self) -> i16 {
        self.high - self.low
    }

    #[inline(always)]
    pub fn possible_neighbors(&self) -> [Self; 4] {
        let Self { low, high } = self;
        [
            // increase width
            Self::new(low - 1, high + 1),
            // decrease width
            Self::new(low + 1, high - 1),
            // same width (must shift up or down)
            Self::new(low + 1, high + 1),
            Self::new(low - 1, high - 1),
        ]
    }
}

#[derive(Copy, Clone, Eq, Ord)]
pub struct AGNRSpec {
    dimers: [GNRSlice; Self::MAX_LEN],
    len: u32,
}

impl PartialOrd for AGNRSpec {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.slices().partial_cmp(other.slices())
    }
}

impl PartialEq for AGNRSpec {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.slices().eq(other.slices())
    }
}

impl std::hash::Hash for AGNRSpec {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.slices().hash(state)
    }
}

impl AGNRSpec {
    const MAX_LEN: usize = 16;

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    #[inline(always)]
    pub fn slices(&self) -> &[GNRSlice] {
        &self.dimers[0..self.len()]
    }

    pub fn width(&self) -> Option<i16> {
        // assumes min of s.low is zero
        self.dimers.iter().map(|s| s.high).max()
    }

    pub fn is_periodic(&self) -> bool {
        let len = self.len();

        if len == 0 {
            true
        } else {
            let first = self.dimers[0];
            let last = self.dimers[len - 1];

            last.possible_neighbors().iter().any(|d| *d == first)
        }
    }

    fn translate(&mut self) {
        let len = self.len();

        // let last = self.dimers[len - 1];
        // for i in 0..(len - 1) {
        //     self.dimers[i + 1] = self.dimers[i];
        // }
        // self.dimers[0] = last;
        self.dimers[0..len].rotate_right(1);
    }

    fn mirror_x(&mut self, width: i16) {
        debug_assert!(width == self.width().unwrap());
        let len = self.len();
        for dimer in &mut self.dimers[0..len] {
            let temp = dimer.low;
            dimer.low = width - dimer.high;
            dimer.high = width - temp;
        }
    }

    fn mirror_y(&mut self) {
        let len = self.len();
        self.dimers[0..len].reverse();
    }

    fn gen(&mut self, slices_left: usize, db: &mut HashSet<Self>) {
        debug_assert!(self.len() > 0);
        debug_assert!(slices_left < i16::max_value() as usize);

        if slices_left != 0 {
            let first = self.dimers[0];
            let last = self.dimers[self.len() - 1];

            for next in &last.possible_neighbors() {
                let delta_l = i16::abs(first.low - next.low);
                let delta_h = i16::abs(first.high - next.high);
                if next.low >= 0 && delta_h <= slices_left as i16 && delta_l <= slices_left as i16 {
                    self.dimers[self.len()] = *next;
                    self.len += 1;
                    self.gen(slices_left - 1, db);
                    self.len -= 1;
                }
            }
        } else {
            let width = self.width().unwrap();

            let mut temp = *self;
            let mut to_add = *self;

            for &_x_mirror in &[false, true] {
                for &_y_mirror in &[false, true] {
                    for _shift in 0..temp.len() {
                        // check for minimum
                        if temp.dimers < to_add.dimers {
                            to_add = temp;
                        }

                        // translations
                        temp.translate();
                    }
                    // y mirror plane
                    temp.mirror_y();
                }
                // x mirror plane
                temp.mirror_x(width);
            }

            db.insert(to_add);
        }
    }

    pub fn generate_all(starting_width: usize, length: usize, db: &mut HashSet<Self>) {
        const MIN_WIDTH: i32 = 2;
        let starting_width: i32 = starting_width.try_into().unwrap();

        assert_ne!(starting_width, 0);
        assert_ne!(length, 0);
        assert!(starting_width >= MIN_WIDTH);

        let mut initial = Self {
            dimers: Default::default(),
            len: 1,
        };
        initial.dimers[0] = GNRSlice::new(0, 2 * starting_width as i16);

        initial.gen(length - 1, db);
    }
}

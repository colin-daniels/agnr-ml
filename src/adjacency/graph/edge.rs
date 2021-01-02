use std::fmt::{Display, Formatter};
use std::hash::Hash;

#[derive(Default, Debug, Copy, Clone, Hash, PartialOrd, PartialEq, Eq, Ord)]
pub struct Edge<M = ()> {
    pub from: usize,
    pub to: usize,
    pub meta: M,
}

impl Edge {
    pub fn new(from: usize, to: usize) -> Self {
        Self { from, to, meta: () }
    }
}

impl<M> Edge<M> {
    pub fn new_with_meta(from: usize, to: usize, meta: M) -> Self {
        Self { from, to, meta }
    }
}

impl From<(usize, usize)> for Edge {
    fn from(e: (usize, usize)) -> Self {
        Self::new(e.0, e.1)
    }
}

impl<'a> From<&'a (usize, usize)> for Edge {
    fn from(e: &(usize, usize)) -> Self {
        Self::new(e.0, e.1)
    }
}

impl<M> From<(usize, usize, M)> for Edge<M> {
    fn from(e: (usize, usize, M)) -> Self {
        Self::new_with_meta(e.0, e.1, e.2)
    }
}

impl<'a, M: Copy> From<&'a (usize, usize, M)> for Edge<M> {
    fn from(e: &(usize, usize, M)) -> Self {
        Self::new_with_meta(e.0, e.1, e.2)
    }
}

impl<M: Display> Display for Edge<M> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "({}, {}, meta: {})", self.from, self.to, self.meta)
    }
}

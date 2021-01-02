use itertools::Itertools;
use rand::prelude::*;
use std::iter::FromIterator;
use std::ops::Range;

use super::Edge;

/// Note: `CompressedGraph` is not made for efficient modification, only access.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompressedGraph<M = ()> {
    /// graph edges
    edges: Vec<Edge<M>>,
    /// edge index ranges for edges that start at each vertex
    edge_ranges: Vec<Range<usize>>,
}

// explicit implementation because derive will normally require M to be Default
impl<M> Default for CompressedGraph<M> {
    fn default() -> Self {
        Self {
            edges: Default::default(),
            edge_ranges: Default::default(),
        }
    }
}

impl<T, M> Extend<T> for CompressedGraph<M>
where
    T: Into<Edge<M>>,
{
    // note: this is used for CompressedGraph::new as well as FromIterator
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        // convert into edges and sort
        let mut new_edges: Vec<Edge<M>> = iter.into_iter().map(|e| e.into()).collect();
        // check for early exit
        if new_edges.is_empty() {
            return;
        }

        new_edges.sort_unstable_by_key(|e| (e.from, e.to));

        // replace the old edges with the old ones + new ones merged together
        let old_edges = self.edges.drain(..);
        self.edges = Itertools::merge_by(old_edges, new_edges.into_iter(), |a, b| {
            (a.from, a.to).lt(&(b.from, b.to))
        })
        .collect();
        // finally, re-construct edge_ranges from the new edges
        self.update_edge_ranges();
    }
}

impl<T, M> FromIterator<T> for CompressedGraph<M>
where
    T: Into<Edge<M>>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_edges(iter)
    }
}

impl<M> CompressedGraph<M> {
    pub fn new<I, T>(n_vertices: usize, edges: I) -> Self
    where
        I: IntoIterator<Item = T>,
        Self: Extend<T>,
    {
        let mut ret = Self::from_edges(edges);
        ret.resize(n_vertices);
        ret
    }

    pub fn from_edges<I, T>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        Self: Extend<T>,
    {
        let mut ret = Self::default();
        ret.extend(iter);
        ret
    }

    /// Number of vertices, note: not all vertices necessarily have edges.
    #[inline(always)]
    pub fn n_vertices(&self) -> usize {
        self.edge_ranges.len()
    }

    /// Number of edges.
    #[inline(always)]
    pub fn n_edges(&self) -> usize {
        self.edges.len()
    }

    pub fn clear(&mut self) {
        self.resize(0);
    }

    pub fn resize(&mut self, new_n_vertices: usize) {
        let n_vertices = self.n_vertices();

        if n_vertices == 0 {
            self.edges.clear();
            self.edge_ranges.clear();
        } else if new_n_vertices < n_vertices {
            // shrink the edge ranges, then find the new 'last' edge and shrink the edge vector
            self.edge_ranges
                .resize_with(new_n_vertices, || unreachable!("should be shrinking"));
            if let Some(last) = self.edge_ranges.last() {
                let new_n_edges = last.end;
                self.edges
                    .resize_with(new_n_edges, || unreachable!("should be shrinking edges"));
            } else {
                unreachable!("handled zero case earlier, should be unreachable")
            }
        } else {
            // we're _only_ adding vertices (edges don't change), so just update edge_ranges
            let n_edges = self.n_edges();
            self.edge_ranges
                .resize_with(new_n_vertices, || n_edges..n_edges);
        }
    }

    #[inline(always)]
    pub fn vertices(&self) -> impl Iterator<Item = usize> {
        0..self.n_vertices()
    }

    #[inline]
    pub fn edges(&self) -> &[Edge<M>] {
        &self.edges
    }

    #[inline(always)]
    pub fn edges_at(&self, vertex: usize) -> &[Edge<M>] {
        &self.edges[self.edge_ranges[vertex].clone()]
    }

    #[inline(always)]
    pub fn neighbors<'a>(&'a self, vertex: usize) -> impl Iterator<Item = usize> + 'a {
        self.neighbors_meta(vertex).map(|(to, _)| to)
    }

    #[inline(always)]
    pub fn neighbors_meta<'a>(&'a self, vertex: usize) -> impl Iterator<Item = (usize, &M)> + 'a {
        self.edges_at(vertex).iter().map(|e| (e.to, &e.meta))
    }

    /// Perform a random walk on the graph.
    #[inline]
    pub fn random_walk<'a, R: Rng + 'a>(
        &'a self,
        start: usize,
        mut rng: R,
    ) -> impl Iterator<Item = usize> + 'a {
        std::iter::successors(Some(start), move |&last| {
            let range = self.edge_ranges[last].clone();
            self.edges[range].choose(&mut rng).map(|e| e.to)
        })
    }

    pub fn maximum_matching(&self) -> Option<()> {
        // use pathfinding::kuhn_munkres::{kuhn_munkres, Weights};
        let (_left, _right) = self.bipartite_coloring()?;
        // note: kuhn_munkres requires rows <= columns
        todo!()
    }

    pub fn bipartite_coloring(&self) -> Option<(Vec<usize>, Vec<usize>)> {
        let (start, _) = self
            .edge_ranges
            .iter()
            .find_position(|&r| r.end > r.start)?;

        #[derive(Copy, Clone, Eq, PartialEq)]
        enum Color {
            Red,
            Blue,
            None,
        }

        let mut colors = vec![Color::None; self.n_vertices()];
        let mut to_visit = vec![self.edge_ranges[start].clone()];
        colors[start] = Color::Red;

        while let Some(edge_ids) = to_visit.last_mut() {
            match edge_ids.next().map(|id| &self.edges[id]) {
                Some(edge) => {
                    let next_color = match colors[edge.from] {
                        Color::Red => Color::Blue,
                        Color::Blue => Color::Red,
                        Color::None => unreachable!("error"),
                    };

                    if colors[edge.to] == Color::None {
                        // alternate colors for bipartite coloring
                        colors[edge.to] = next_color;
                        to_visit.push(self.edge_ranges[edge.to].clone());
                    } else if colors[edge.to] != next_color {
                        // wrong color, not bipartite
                        dbg!("wrong color, not bipartite");
                        return None;
                    }
                }
                None => {
                    to_visit.pop();
                }
            }
        }

        let mut red = Vec::with_capacity(colors.len());
        let mut blue = Vec::with_capacity(colors.len());

        for (i, color) in colors.into_iter().enumerate() {
            match color {
                Color::Red => red.push(i),
                Color::Blue => blue.push(i),
                Color::None => {
                    // couldn't reach all vertices
                    if !self.edges_at(i).is_empty() {
                        dbg!("couldn't reach all vertices (multiple connected components)");
                        return None;
                    }
                }
            }
        }

        Some((red, blue))
    }

    #[doc(hidden)]
    fn update_edge_ranges(&mut self) {
        let old_num_vertices = self.n_vertices();
        let num_vertices_from_edges = self.edges.last().map(|e| e.from + 1).unwrap_or(0);

        // initial guess for number of vertices
        let mut n_vertices = usize::max(old_num_vertices, num_vertices_from_edges);

        // determine offsets (where edges for a particular vertex are located in the edge vector)
        let mut offsets = Vec::with_capacity(n_vertices + 1);
        for (i, &Edge { from, to, .. }) in self.edges.iter().enumerate() {
            // update total number of vertices
            n_vertices = usize::max(n_vertices, to + 1);
            // update offsets
            if offsets.len() < from + 1 {
                // note: we use extend to deal with the case where vertices are skipped (aka have
                // no edges)
                offsets.resize(from + 1, i);
            }
        }
        offsets.resize(n_vertices + 1, self.edges.len());

        // finally, build and assign edge ranges
        self.edge_ranges = offsets
            .into_iter()
            .tuple_windows()
            .map(|(a, b)| a..b)
            .collect();
    }
}

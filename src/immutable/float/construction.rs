use crate::float::kdtree::Axis;
use crate::float_leaf_simd::leaf_node::BestFromDists;
use crate::immutable::float::kdtree::ImmutableKdTree;
use crate::types::Content;
use az::Cast;
use std::ops::Rem;
use tracing::{event, Level};

impl<A: Axis, T: Content, const K: usize, const B: usize> ImmutableKdTree<A, T, K, B>
where
    A: Axis + BestFromDists<T, B>,
    T: Content,
    usize: Cast<T>,
{
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn add_to_optimized(&mut self, query: &[A; K], item: T) {
        let mut dim = 0;
        let mut idx: usize = 1;
        let mut val: A;

        while idx < self.stems.len() {
            val = *unsafe { self.stems.get_unchecked(idx) };

            let is_right_child = *unsafe { query.get_unchecked(dim) } >= val;
            idx = (idx << 1) + usize::from(is_right_child);
            dim = (dim + 1).rem(K);
        }
        idx -= self.stems.len();

        let node_size = (unsafe { self.leaves.get_unchecked_mut(idx) }).size;
        if node_size == B {
            event!(
                Level::ERROR,
                "Overflowing add to leaf #{}. item={:?}",
                idx,
                query
            );
            event!(Level::TRACE, "Tree Stats: {:?}", self.generate_stats())
        }

        let node = unsafe { self.leaves.get_unchecked_mut(idx) };
        debug_assert!(node.size < B);

        // *unsafe { node.content_points.get_unchecked_mut(node.size) } = *query;
        for (dim, &val) in query.iter().enumerate() {
            *unsafe { node.content_points[dim].get_unchecked_mut(node.size) } = val;
        }

        *unsafe { node.content_items.get_unchecked_mut(node.size) } = item;

        node.size += 1;
        self.size += 1;
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn safe_add_to_optimized(&mut self, query: &[A; K], item: T) {
        let mut dim = 0;
        let mut idx: usize = 1;
        let mut val: A;

        while idx < self.stems.len() {
            val = self.stems[idx];

            let is_right_child = query[dim] >= val;
            idx = (idx << 1) + usize::from(is_right_child);
            dim = (dim + 1).rem(K);
        }
        idx -= self.stems.len();

        let node_size = self.leaves[idx].size;
        if node_size == B {
            event!(
                Level::ERROR,
                "Overflowing add to leaf #{}. item={:?}",
                idx,
                query
            );
            event!(Level::TRACE, "Tree Stats: {:?}", self.generate_stats())
        }

        let node = self.leaves.get_mut(idx).unwrap();
        debug_assert!(node.size < B);

        for (dim, &val) in query.iter().enumerate() {
            *node.content_points[dim].get_mut(node.size).unwrap() = val;
        }

        *node.content_items.get_mut(node.size).unwrap() = item;

        node.size += 1;
        self.size += 1;
    }
}

#[cfg(test)]
mod tests {}

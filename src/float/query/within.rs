use az::{Az, Cast};
use std::collections::BinaryHeap;
use std::ops::Rem;

use crate::float::kdtree::{Axis, KdTree};
use crate::nearest_neighbour::NearestNeighbour;
use crate::types::{is_stem_index, Content, Index};

use crate::generate_within;

macro_rules! generate_float_within {
    ($doctest_build_tree:tt) => {
        generate_within!((
            "Finds all elements within `dist` of `query`, using the specified
distance metric function.

Results are returned sorted nearest-first

# Examples

```rust
    use kiddo::float::kdtree::KdTree;
    use kiddo::distance::squared_euclidean;
    ",
            $doctest_build_tree,
            "

    let within = tree.within(&[1.0, 2.0, 5.0], 10f64, &squared_euclidean);

    assert_eq!(within.len(), 2);
```"
        ));
    };
}

impl<A: Axis, T: Content, const K: usize, const B: usize, IDX: Index<T = IDX>>
    KdTree<A, T, K, B, IDX>
where
    usize: Cast<IDX>,
{
    generate_float_within!(
        "
let mut tree: KdTree<f64, u32, 3, 32, u32> = KdTree::new();
tree.add(&[1.0, 2.0, 5.0], 100);
tree.add(&[2.0, 3.0, 6.0], 101);"
    );
}

#[cfg(feature = "rkyv")]
use crate::float::kdtree::ArchivedKdTree;
#[cfg(feature = "rkyv")]
impl<
        A: Axis + rkyv::Archive<Archived = A>,
        T: Content + rkyv::Archive<Archived = T>,
        const K: usize,
        const B: usize,
        IDX: Index<T = IDX> + rkyv::Archive<Archived = IDX>,
    > ArchivedKdTree<A, T, K, B, IDX>
where
    usize: Cast<IDX>,
{
    generate_float_within!(
        "use std::fs::File;
use memmap::MmapOptions;

let mmap = unsafe { MmapOptions::new().map(&File::open(\"./examples/test-tree.rkyv\").unwrap()).unwrap() };
let tree = unsafe { rkyv::archived_root::<KdTree<f64, u32, 3, 32, u32>>(&mmap) };"
    );
}

#[cfg(test)]
mod tests {
    use crate::float::distance::manhattan;
    use crate::float::kdtree::{Axis, KdTree};
    use crate::nearest_neighbour::NearestNeighbour;
    use rand::Rng;
    use std::cmp::Ordering;

    type AX = f32;

    #[test]
    fn can_query_items_within_radius() {
        let mut tree: KdTree<AX, u32, 4, 5, u32> = KdTree::new();

        let content_to_add: [([AX; 4], u32); 16] = [
            ([0.9f32, 0.0f32, 0.9f32, 0.0f32], 9),
            ([0.4f32, 0.5f32, 0.4f32, 0.5f32], 4),
            ([0.12f32, 0.3f32, 0.12f32, 0.3f32], 12),
            ([0.7f32, 0.2f32, 0.7f32, 0.2f32], 7),
            ([0.13f32, 0.4f32, 0.13f32, 0.4f32], 13),
            ([0.6f32, 0.3f32, 0.6f32, 0.3f32], 6),
            ([0.2f32, 0.7f32, 0.2f32, 0.7f32], 2),
            ([0.14f32, 0.5f32, 0.14f32, 0.5f32], 14),
            ([0.3f32, 0.6f32, 0.3f32, 0.6f32], 3),
            ([0.10f32, 0.1f32, 0.10f32, 0.1f32], 10),
            ([0.16f32, 0.7f32, 0.16f32, 0.7f32], 16),
            ([0.1f32, 0.8f32, 0.1f32, 0.8f32], 1),
            ([0.15f32, 0.6f32, 0.15f32, 0.6f32], 15),
            ([0.5f32, 0.4f32, 0.5f32, 0.4f32], 5),
            ([0.8f32, 0.1f32, 0.8f32, 0.1f32], 8),
            ([0.11f32, 0.2f32, 0.11f32, 0.2f32], 11),
        ];

        for (point, item) in content_to_add {
            tree.add(&point, item);
        }

        assert_eq!(tree.size(), 16);

        let query_point = [0.78f32, 0.55f32, 0.78f32, 0.55f32];

        let radius = 0.2;
        let expected = linear_search(&content_to_add, &query_point, radius);

        let mut result: Vec<_> = tree.within(&query_point, radius, &manhattan);
        stabilize_sort(&mut result);
        assert_eq!(result, expected);

        let mut rng = rand::thread_rng();
        for _i in 0..1000 {
            let query_point = [
                rng.gen_range(0f32..1f32),
                rng.gen_range(0f32..1f32),
                rng.gen_range(0f32..1f32),
                rng.gen_range(0f32..1f32),
            ];
            let radius: f32 = 2.0;
            let expected = linear_search(&content_to_add, &query_point, radius);

            let mut result: Vec<_> = tree.within(&query_point, radius, &manhattan);
            stabilize_sort(&mut result);

            assert_eq!(result, expected);
        }
    }

    #[test]
    fn can_query_items_within_radius_large_scale() {
        const TREE_SIZE: usize = 100_000;
        const NUM_QUERIES: usize = 100;
        const RADIUS: f32 = 0.2;

        let content_to_add: Vec<([f32; 4], u32)> = (0..TREE_SIZE)
            .map(|_| rand::random::<([f32; 4], u32)>())
            .collect();

        let mut tree: KdTree<AX, u32, 4, 32, u32> = KdTree::with_capacity(TREE_SIZE);
        content_to_add
            .iter()
            .for_each(|(point, content)| tree.add(point, *content));
        assert_eq!(tree.size(), TREE_SIZE as u32);

        let query_points: Vec<[f32; 4]> = (0..NUM_QUERIES)
            .map(|_| rand::random::<[f32; 4]>())
            .collect();

        for query_point in query_points {
            let expected = linear_search(&content_to_add, &query_point, RADIUS);

            let mut result: Vec<_> = tree.within(&query_point, RADIUS, &manhattan);

            // TODO: ensure that adjacent results with the same dist are sorted in order of item val
            //       to prevent occasional test failures due to the linear search returning items
            //       with the same dist in a different order to the query
            stabilize_sort(&mut result);

            // let slice = &mut result[..];
            // let slice_of_cells: &[Cell<NearestNeighbour<f32, u32>>] = Cell::from_mut(slice).as_slice_of_cells();
            // for w in slice_of_cells.windows(2) {
            //     if w[0].get().distance == w[1].get().distance && w[0].get().item > w[1].get().item {
            //         Cell::swap(&w[0], &w[1]);
            //     }
            //
            // }
            assert_eq!(result, expected);
        }
    }

    fn linear_search<A: Axis, const K: usize>(
        content: &[([A; K], u32)],
        query_point: &[A; K],
        radius: A,
    ) -> Vec<NearestNeighbour<A, u32>> {
        let mut matching_items = vec![];

        for &(p, item) in content {
            let distance = manhattan(query_point, &p);
            if distance < radius {
                matching_items.push(NearestNeighbour { distance, item });
            }
        }

        stabilize_sort(&mut matching_items);

        // let slice = &mut matching_items[..];
        // let slice_of_cells: &[Cell<NearestNeighbour<A, u32>>] = Cell::from_mut(slice).as_slice_of_cells();
        //
        // for w in slice_of_cells.windows(2) {
        //     if w[0].get().distance == w[1].get().distance && w[0].get().item > w[1].get().item {
        //         Cell::swap(&w[0], &w[1]);
        //     }
        // }

        matching_items
    }

    fn stabilize_sort<A: Axis>(matching_items: &mut Vec<NearestNeighbour<A, u32>>) {
        matching_items.sort_unstable_by(|a, b| {
            let dist_cmp = a.distance.partial_cmp(&b.distance).unwrap();
            if dist_cmp == Ordering::Equal {
                a.item.cmp(&b.item)
            } else {
                dist_cmp
            }
        });
    }
}

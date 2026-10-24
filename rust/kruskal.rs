#[derive(Clone, Copy, Debug)]
pub struct Edge {
    pub u: usize,
    pub v: usize,
    pub weight: i64,
}

#[derive(Clone)]
struct LocalUnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
    size: Vec<usize>,
}

impl LocalUnionFind {

    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
            size: vec![1; size],
        }
    }

    fn root(&mut self, vertex: usize) -> usize {
        if self.parent[vertex] != vertex {
            let parent = self.parent[vertex];
            self.parent[vertex] = self.root(parent);
        }
        self.parent[vertex]
    }

    fn unite(&mut self, u: usize, v: usize) -> bool {
        let mut ru = self.root(u);
        let mut rv = self.root(v);
        if ru == rv {
            return false;
        }

        if self.rank[ru] < self.rank[rv] {
            std::mem::swap(&mut ru, &mut rv);
        }

        self.parent[rv] = ru;
        self.size[ru] += self.size[rv];
        if self.rank[ru] == self.rank[rv] {
            self.rank[ru] += 1;
        }
        true
    }

    fn root_size(&mut self, vertex: usize) -> usize {
        let r = self.root(vertex);
        self.size[r]
    }
}

/// クラスカル法で最小全域木の重みを返す。連結でない場合は None。
pub fn kruskal(vertex_number: usize, edges: &mut [Edge]) -> Option<i64> {

    if vertex_number == 0 {
        return Some(0);
    }

    edges.sort_by_key(|e| e.weight);
    let mut uf = LocalUnionFind::new(vertex_number);
    let mut total = 0i64;

    for edge in edges.iter() {
        if uf.unite(edge.u, edge.v) {
            total += edge.weight;
        }
    }

    if uf.root_size(0) == vertex_number {
        Some(total)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};


    use super::{kruskal, Edge};

    #[test]
    fn kruskal_with_no_vertices_returns_zero_weight() {
        // Arrange
        let vertex_count = 0;
        let mut edges = vec![];
        let expected_weight = Some(0);

        // Act
        let actual_weight = kruskal(vertex_count, &mut edges);

        // Assert
        assert_eq!(actual_weight, expected_weight);
    }

    #[test]
    fn kruskal_on_triangle_returns_weight_three() {
        // Arrange
        let vertex_count = 3;
        let mut edges = vec![
            Edge {
                u: 0,
                v: 1,
                weight: 1,
            },
            Edge {
                u: 1,
                v: 2,
                weight: 2,
            },
            Edge {
                u: 0,
                v: 2,
                weight: 4,
            },
        ];
        let expected_weight = Some(3);

        // Act
        let actual_weight = kruskal(vertex_count, &mut edges);

        // Assert
        assert_eq!(actual_weight, expected_weight);
    }

    #[test]
    fn returns_none_when_graph_disconnected() {
        // Arrange
        let vertex_count = 3;
        let mut edges = vec![Edge {
            u: 0,
            v: 1,
            weight: 1,
        }];
        let expected_weight = None;

        // Act
        let actual_weight = kruskal(vertex_count, &mut edges);

        // Assert
        assert_eq!(actual_weight, expected_weight);
    }

    #[test]
    fn kruskal_on_single_vertex_ignores_self_loop_and_returns_zero() {
        // Arrange
        let vertex_count = 1;
        let mut edges = vec![Edge {
            u: 0,
            v: 0,
            weight: -5,
        }];
        let expected_weight = Some(0);

        // Act
        let actual_weight = kruskal(vertex_count, &mut edges);

        // Assert
        assert_eq!(actual_weight, expected_weight);
    }

    #[test]
    fn parallel_edges_choose_smallest_weight() {
        // Arrange
        let vertex_count = 3;
        let mut edges = vec![
            Edge {
                u: 0,
                v: 1,
                weight: 5,
            },
            Edge {
                u: 0,
                v: 1,
                weight: 1,
            },
            Edge {
                u: 1,
                v: 2,
                weight: 2,
            },
        ];
        let expected_weight = Some(3);

        // Act
        let actual_weight = kruskal(vertex_count, &mut edges);

        // Assert
        assert_eq!(actual_weight, expected_weight);
    }

    #[test]
    fn negative_edges_are_supported_in_connected_graph() {
        // Arrange
        let vertex_count = 3;
        let mut edges = vec![
            Edge {
                u: 0,
                v: 1,
                weight: -2,
            },
            Edge {
                u: 1,
                v: 2,
                weight: 3,
            },
            Edge {
                u: 0,
                v: 2,
                weight: 10,
            },
        ];
        let expected_weight = Some(1);

        // Act
        let actual_weight = kruskal(vertex_count, &mut edges);

        // Assert
        assert_eq!(actual_weight, expected_weight);
    }

    #[test]
    fn unsorted_edges_are_sorted_internally() {
        // Arrange
        let vertex_count = 3;
        let mut edges = vec![
            Edge {
                u: 0,
                v: 2,
                weight: 4,
            },
            Edge {
                u: 1,
                v: 2,
                weight: 1,
            },
            Edge {
                u: 0,
                v: 1,
                weight: 2,
            },
        ];
        let expected_weight = Some(3);

        // Act
        let actual_weight = kruskal(vertex_count, &mut edges);

        // Assert
        assert_eq!(actual_weight, expected_weight);
    }

    #[test]
    fn out_of_range_vertex_panics() {
        // Arrange
        let vertex_count = 3;
        let mut edges = vec![Edge {
            u: 0,
            v: 3,
            weight: 1,
        }];

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| kruskal(vertex_count, &mut edges)));

        // Assert
        assert!(actual_result.is_err());
    }
}

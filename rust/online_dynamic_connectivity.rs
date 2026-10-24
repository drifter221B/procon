use std::collections::{hash_map::Entry, HashMap};

#[derive(Clone, Copy, Debug)]
pub enum Query {
    Add(usize, usize),
    Remove(usize, usize),
    Connected(usize, usize),
}

#[derive(Clone)]
struct RollbackDsu {
    parent: Vec<usize>,
    size: Vec<usize>,
    history: Vec<(usize, usize, usize)>, // child root, parent root, prev_size_parent
}

#[derive(Clone, Copy)]
struct ActiveEdge {
    start: usize,
    count: usize,
}

impl RollbackDsu {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            size: vec![1; size],
            history: Vec::new(),
        }
    }

    fn root(&mut self, vertex: usize) -> usize {
        let mut v = vertex;
        while self.parent[v] != v {
            v = self.parent[v];
        }
        v
    }

    fn merge(&mut self, u: usize, v: usize) -> bool {
        let mut ru = self.root(u);
        let mut rv = self.root(v);
        if ru == rv {
            return false;
        }
        if self.size[ru] < self.size[rv] {
            std::mem::swap(&mut ru, &mut rv);
        }
        self.history.push((rv, ru, self.size[ru]));
        self.parent[rv] = ru;
        self.size[ru] += self.size[rv];
        true
    }

    fn snapshot(&self) -> usize {
        self.history.len()
    }

    fn rollback(&mut self, version: usize) {
        while version < self.history.len() {
            if let Some((child, parent, prev_size_parent)) = self.history.pop() {
                self.parent[child] = child;
                self.size[parent] = prev_size_parent;
            }
        }
    }

    fn is_same_component(&mut self, u: usize, v: usize) -> bool {
        self.root(u) == self.root(v)
    }
}

/// オンラインで加算・削除・連結判定を扱う（オフライン処理）。
pub struct OnlineDynamicConnectivity {
    vertex_number: usize,
    queries: Vec<Query>,
}

impl OnlineDynamicConnectivity {
    pub fn new(vertex_number: usize) -> Self {
        Self {
            vertex_number,
            queries: Vec::new(),
        }
    }

    pub fn add_query(&mut self, query: Query) {
        self.queries.push(query);
    }

    pub fn solve(&self) -> Vec<bool> {
        if self.queries.is_empty() {
            return Vec::new();
        }

        let intervals = self.build_active_intervals();
        let mut segment_edges = vec![Vec::<(usize, usize)>::new(); self.queries.len() * 4 + 4];

        for (l, r, edge) in intervals {
            Self::add_edge(1, 0, self.queries.len(), l, r, edge, &mut segment_edges);
        }

        let mut dsu = RollbackDsu::new(self.vertex_number);
        let mut answers = Vec::with_capacity(self.queries.len());
        self.dfs(
            1,
            0,
            self.queries.len(),
            &segment_edges,
            &mut dsu,
            &mut answers,
        );
        answers
    }

    fn normalize_edge(u: usize, v: usize) -> (usize, usize) {
        if u <= v {
            (u, v)
        } else {
            (v, u)
        }
    }

    fn build_active_intervals(&self) -> Vec<(usize, usize, (usize, usize))> {
        let mut active: HashMap<(usize, usize), ActiveEdge> = HashMap::new();
        let mut intervals = Vec::new();

        for (i, q) in self.queries.iter().enumerate() {
            match *q {
                Query::Add(u, v) => {
                    active
                        .entry(Self::normalize_edge(u, v))
                        .and_modify(|active_edge| active_edge.count += 1)
                        .or_insert(ActiveEdge { start: i, count: 1 });
                }
                Query::Remove(u, v) => {
                    let edge = Self::normalize_edge(u, v);
                    if let Entry::Occupied(mut entry) = active.entry(edge) {
                        let active_edge = entry.get_mut();
                        active_edge.count -= 1;
                        if active_edge.count == 0 {
                            let start = active_edge.start;
                            intervals.push((start, i, edge));
                            entry.remove();
                        }
                    }
                }
                Query::Connected(_, _) => {}
            }
        }

        for (edge, active_edge) in active {
            intervals.push((active_edge.start, self.queries.len(), edge));
        }

        intervals
    }

    fn add_edge(
        node: usize,
        l: usize,
        r: usize,
        ql: usize,
        qr: usize,
        edge: (usize, usize),
        segment_edges: &mut [Vec<(usize, usize)>],
    ) {
        if qr <= l || r <= ql {
            return;
        }
        if ql <= l && r <= qr {
            segment_edges[node].push(edge);
            return;
        }
        let mid = (l + r) >> 1;
        Self::add_edge(node << 1, l, mid, ql, qr, edge, segment_edges);
        Self::add_edge((node << 1) | 1, mid, r, ql, qr, edge, segment_edges);
    }

    fn dfs(
        &self,
        node: usize,
        l: usize,
        r: usize,
        segment_edges: &[Vec<(usize, usize)>],
        dsu: &mut RollbackDsu,
        answers: &mut Vec<bool>,
    ) {
        if r <= l {
            return;
        }

        let snapshot = dsu.snapshot();
        for &(u, v) in segment_edges[node].iter() {
            dsu.merge(u, v);
        }

        if r - l == 1 {
            if let Query::Connected(u, v) = self.queries[l] {
                answers.push(dsu.is_same_component(u, v));
            }
            dsu.rollback(snapshot);
            return;
        }

        let mid = (l + r) >> 1;
        self.dfs(node << 1, l, mid, segment_edges, dsu, answers);
        self.dfs((node << 1) | 1, mid, r, segment_edges, dsu, answers);
        dsu.rollback(snapshot);
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::{OnlineDynamicConnectivity, Query};

    #[test]
    fn returns_empty_when_no_queries() {
        // Arrange
        let vertex_number = 2;
        let expected_answers: Vec<bool> = vec![];
        let odc = OnlineDynamicConnectivity::new(vertex_number);

        // Act
        let actual_answers = odc.solve();

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn out_of_range_vertex_panics_during_solve() {
        // Arrange
        let mut structure = OnlineDynamicConnectivity::new(2);
        structure.add_query(Query::Add(0, 2));
        structure.add_query(Query::Connected(0, 1));

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| structure.solve()));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn handles_add_remove_and_connected_queries() {
        // Arrange
        let vertex_number = 3;
        let queries = [Query::Add(0, 1), Query::Add(1, 2), Query::Connected(0, 2), Query::Remove(1, 2), Query::Connected(0, 2)];
        let expected_answers = vec![true, false];
        let mut odc = OnlineDynamicConnectivity::new(vertex_number);
        for query in queries { odc.add_query(query); }

        // Act
        let actual_answers = odc.solve();

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn remove_without_add_does_not_panic_and_returns_false() {
        // Arrange
        let vertex_number = 2;
        let queries = [Query::Remove(0, 1), Query::Connected(0, 1)];
        let expected_answers = vec![false];
        let mut odc = OnlineDynamicConnectivity::new(vertex_number);
        for query in queries { odc.add_query(query); }

        // Act
        let actual_answers = odc.solve();

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn duplicate_adds_require_same_number_of_removes_to_disconnect() {
        // Arrange
        let vertex_number = 2;
        let queries = [Query::Add(0, 1), Query::Add(0, 1), Query::Remove(0, 1), Query::Connected(0, 1), Query::Remove(0, 1), Query::Connected(0, 1)];
        let expected_answers = vec![true, false];
        let mut odc = OnlineDynamicConnectivity::new(vertex_number);
        for query in queries { odc.add_query(query); }

        // Act
        let actual_answers = odc.solve();

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn duplicate_add_and_single_remove_keeps_edge_active() {
        // Arrange
        let mut structure = OnlineDynamicConnectivity::new(2);
        structure.add_query(Query::Add(0, 1));
        structure.add_query(Query::Add(0, 1));
        structure.add_query(Query::Remove(0, 1));
        structure.add_query(Query::Connected(0, 1));
        let expected_answers = vec![true];

        // Act
        let actual_answers = structure.solve();

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn edge_without_remove_stays_active_until_end() {
        // Arrange
        let mut structure = OnlineDynamicConnectivity::new(3);
        structure.add_query(Query::Add(0, 1));
        structure.add_query(Query::Connected(0, 1));
        structure.add_query(Query::Connected(1, 2));
        let expected_answers = vec![true, false];

        // Act
        let actual_answers = structure.solve();

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn reversed_remove_endpoint_order_removes_undirected_edge() {
        // Arrange
        let vertex_number = 2;
        let queries = [Query::Add(0, 1), Query::Remove(1, 0), Query::Connected(0, 1)];
        let expected_answers = vec![false];
        let mut structure = OnlineDynamicConnectivity::new(vertex_number);
        for query in queries { structure.add_query(query); }

        // Act
        let actual_answers = structure.solve();

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn single_vertex_is_connected_to_itself() {
        // Arrange
        let vertex_number = 1;
        let query = Query::Connected(0, 0);
        let expected_answers = vec![true];
        let mut structure = OnlineDynamicConnectivity::new(vertex_number);
        structure.add_query(query);

        // Act
        let actual_answers = structure.solve();

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }
}

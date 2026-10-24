/// Link-Cut Tree による動的木構造。
/// パス反転（evert）、link / cut、パス和クエリに対応する。
pub struct LinkCutTree {
    nodes: Vec<Node>,
}

#[derive(Clone)]
struct Node {
    parent: Option<usize>,
    child: [Option<usize>; 2],
    rev: bool,
    value: i64,
    sum: i64,
}

impl Node {
    fn new(value: i64) -> Self {
        Self {
            parent: None,
            child: [None, None],
            rev: false,
            value,
            sum: value,
        }
    }
}

impl LinkCutTree {
    /// `n` 個の頂点を値 0 で初期化する。
    pub fn new(n: usize) -> Self {
        Self {
            nodes: (0..n).map(|_| Node::new(0)).collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn set_value(&mut self, x: usize, value: i64) {
        if self.len() <= x {
            return;
        }
        self.access(x);
        self.splay(x);
        self.nodes[x].value = value;
        self.update(x);
    }

    /// 頂点 `x` の値を返す（範囲外は None）。
    pub fn value(&mut self, x: usize) -> Option<i64> {
        if self.len() <= x {
            return None;
        }
        self.access(x);
        self.splay(x);
        Some(self.nodes[x].value)
    }

    /// 2 頂点が同じ木に属するか判定する。
    pub fn connected(&mut self, u: usize, v: usize) -> Option<bool> {
        if self.len() <= u || self.len() <= v {
            return None;
        }
        Some(self.find_root(u) == self.find_root(v))
    }

    /// 頂点 `u` を根とみなしてから `v` に link する。既に同じ木に属する場合は false。
    pub fn link(&mut self, u: usize, v: usize) -> bool {
        if self.len() <= u || self.len() <= v {
            return false;
        }
        if self.connected(u, v).unwrap_or(false) {
            return false;
        }
        self.evert(u);
        self.nodes[u].parent = Some(v);
        true
    }

    /// 辺 (u, v) を切断する。成功時 true。
    pub fn cut(&mut self, u: usize, v: usize) -> bool {
        if self.len() <= u || self.len() <= v {
            return false;
        }
        self.evert(u);
        self.access(v);
        self.splay(v);
        if let Some(left) = self.nodes[v].child[0] {
            if left == u && self.nodes[u].child[1].is_none() && self.nodes[u].child[0].is_none() {
                self.nodes[v].child[0] = None;
                self.nodes[u].parent = None;
                self.update(v);
                return true;
            }
        }
        false
    }

    /// パス (u, v) の値の総和を返す。範囲外なら None。
    pub fn path_sum(&mut self, u: usize, v: usize) -> Option<i64> {
        if self.len() <= u || self.len() <= v {
            return None;
        }
        self.evert(u);
        self.access(v);
        self.splay(v);
        Some(self.nodes[v].sum)
    }

    /// 頂点 `x` を木の根にする。
    fn evert(&mut self, x: usize) {
        self.access(x);
        self.splay(x);
        self.toggle_rev(x);
    }

    fn find_root(&mut self, x: usize) -> Option<usize> {
        self.access(x);
        self.splay(x);
        let mut v = x;
        self.push(v);
        while let Some(left) = self.nodes[v].child[0] {
            v = left;
            self.push(v);
        }
        self.splay(v);
        Some(v)
    }

    fn access(&mut self, mut x: usize) {
        let mut last = None;
        let mut cur = Some(x);
        while let Some(v) = cur {
            self.splay(v);
            self.nodes[v].child[1] = last;
            self.update(v);
            last = Some(v);
            cur = self.nodes[v].parent;
        }
    }

    fn update(&mut self, x: usize) {
        let left_sum = self.nodes[x].child[0].map(|c| self.nodes[c].sum).unwrap_or(0);
        let right_sum = self.nodes[x].child[1].map(|c| self.nodes[c].sum).unwrap_or(0);
        self.nodes[x].sum = self.nodes[x].value + left_sum + right_sum;
    }

    fn push(&mut self, x: usize) {
        if self.nodes[x].rev {
            self.nodes[x].rev = false;
            let (l, r) = (self.nodes[x].child[0], self.nodes[x].child[1]);
            self.nodes[x].child.swap(0, 1);
            if let Some(lc) = l {
                self.toggle_rev(lc);
            }
            if let Some(rc) = r {
                self.toggle_rev(rc);
            }
        }
    }

    fn toggle_rev(&mut self, x: usize) {
        self.nodes[x].rev ^= true;
    }

    fn is_root(&self, x: usize) -> bool {
        if let Some(p) = self.nodes[x].parent {
            let left = self.nodes[p].child[0] == Some(x);
            let right = self.nodes[p].child[1] == Some(x);
            !(left || right)
        } else {
            true
        }
    }

    fn rotate(&mut self, x: usize) {
        let p = self.nodes[x].parent.unwrap();
        let g = self.nodes[p].parent;

        let is_right = self.nodes[p].child[1] == Some(x);
        let b = if is_right {
            self.nodes[x].child[0]
        } else {
            self.nodes[x].child[1]
        };

        if is_right {
            self.nodes[x].child[0] = Some(p);
            self.nodes[p].child[1] = b;
        } else {
            self.nodes[x].child[1] = Some(p);
            self.nodes[p].child[0] = b;
        }

        if let Some(bc) = b {
            self.nodes[bc].parent = Some(p);
        }

        self.nodes[p].parent = Some(x);
        self.nodes[x].parent = g;

        if let Some(gp) = g {
            if self.nodes[gp].child[0] == Some(p) {
                self.nodes[gp].child[0] = Some(x);
            } else if self.nodes[gp].child[1] == Some(p) {
                self.nodes[gp].child[1] = Some(x);
            }
        }

        self.update(p);
        self.update(x);
    }

    fn splay(&mut self, x: usize) {
        self.push_path(x);
        while !self.is_root(x) {
            let p = self.nodes[x].parent.unwrap();
            if !self.is_root(p) {
                let gp = self.nodes[p].parent.unwrap();
                let zigzig = (self.nodes[gp].child[0] == Some(p)) == (self.nodes[p].child[0] == Some(x));
                if zigzig {
                    self.rotate(p);
                } else {
                    self.rotate(x);
                }
            }
            self.rotate(x);
        }
    }

    fn push_path(&mut self, mut x: usize) {
        let mut stack = Vec::new();
        let mut v = Some(x);
        while let Some(cur) = v {
            stack.push(cur);
            v = self.nodes[cur].parent;
        }
        while let Some(cur) = stack.pop() {
            self.push(cur);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LinkCutTree;

    #[test]
    fn non_connected_path_sum_returns_destination_component_value() {
        // Arrange
        let input_value_1 = 2;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 1;
        let input_value_5 = 2;
        let input_value_6 = 0;
        let input_value_7 = 1;
        let mut tree = LinkCutTree::new(input_value_1);
        tree.set_value(input_value_2, input_value_3);
        tree.set_value(input_value_4, input_value_5);
        let expected_value = Some(2);

        // Act
        let actual_value = tree.path_sum(input_value_6, input_value_7);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn link_rejects_vertices_already_in_same_tree() {
        // Arrange
        let size = 2;
        let (first, second) = (0, 1);
        let expected = (true, false);
        let mut tree = LinkCutTree::new(size);

        // Act
        let first_link = tree.link(first, second);
        let repeated_link = tree.link(second, first);
        let actual = (first_link, repeated_link);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cut_separates_trees() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 1;
        let input_value_5 = 2;
        let input_value_6 = 0;
        let input_value_7 = 1;
        let input_value_8 = 0;
        let input_value_9 = 2;
        let mut lct = LinkCutTree::new(input_value_1);
        lct.link(input_value_2, input_value_3);
        lct.link(input_value_4, input_value_5);
        let expected_check_2 = Some(false);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = lct.cut(input_value_6, input_value_7);
        let actual_check_2 = lct.connected(input_value_8, input_value_9);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
        assert_eq!(actual_check_2, expected_check_2);
    }

    #[test]
    fn cut_of_non_existing_edge_returns_false() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 0;
        let input_value_5 = 2;
        let mut tree = LinkCutTree::new(input_value_1);
        tree.link(input_value_2, input_value_3);
        let expected_value = false;

        // Act
        let actual_value = tree.cut(input_value_4, input_value_5);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn cut_then_relink_restores_connectivity_and_sum() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 1;
        let input_value_5 = 2;
        let input_value_6 = 2;
        let input_value_7 = 3;
        let input_value_8 = 0;
        let input_value_9 = 1;
        let input_value_10 = 1;
        let input_value_11 = 2;
        let input_value_12 = 1;
        let input_value_13 = 2;
        let input_value_14 = 0;
        let input_value_15 = 2;
        let input_value_16 = 1;
        let input_value_17 = 2;
        let input_value_18 = 0;
        let input_value_19 = 2;
        let input_value_20 = 0;
        let input_value_21 = 2;
        let mut tree = LinkCutTree::new(input_value_1);
        tree.set_value(input_value_2, input_value_3);
        tree.set_value(input_value_4, input_value_5);
        tree.set_value(input_value_6, input_value_7);
        tree.link(input_value_8, input_value_9);
        tree.link(input_value_10, input_value_11);
        let expected_check_3 = Some(false);
        let expected_check_4 = Some(true);
        let expected_check_5 = Some(6);

        let expected_boolean_1 = true;
        let expected_boolean_2 = true;

        // Act
        let actual_check_1 = tree.cut(input_value_12, input_value_13);
        let disconnected = tree.connected(input_value_14, input_value_15);
        let actual_check_2 = tree.link(input_value_16, input_value_17);
        let connected = tree.connected(input_value_18, input_value_19);
        let path_sum = tree.path_sum(input_value_20, input_value_21);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
        assert_eq!(actual_check_2, expected_boolean_2);
        assert_eq!(disconnected, expected_check_3);
        assert_eq!(connected, expected_check_4);
        assert_eq!(path_sum, expected_check_5);
    }

    #[test]
    fn connects_and_queries_path_sum() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 1;
        let input_value_5 = 2;
        let input_value_6 = 2;
        let input_value_7 = 3;
        let input_value_8 = 3;
        let input_value_9 = 4;
        let input_value_10 = 0;
        let input_value_11 = 1;
        let input_value_12 = 1;
        let input_value_13 = 2;
        let input_value_14 = 2;
        let input_value_15 = 3;
        let input_value_16 = 0;
        let input_value_17 = 3;
        let input_value_18 = 0;
        let input_value_19 = 3;
        let mut lct = LinkCutTree::new(input_value_1);
        lct.set_value(input_value_2, input_value_3);
        lct.set_value(input_value_4, input_value_5);
        lct.set_value(input_value_6, input_value_7);
        lct.set_value(input_value_8, input_value_9);
        let expected_check_4 = Some(10);

        let expected_boolean_1 = true;
        let expected_boolean_2 = true;
        let expected_boolean_3 = true;
        let expected_boolean_4 = true;

        // Act
        let actual_check_1 = lct.link(input_value_10, input_value_11);
        let actual_check_2 = lct.link(input_value_12, input_value_13);
        let actual_check_3 = lct.link(input_value_14, input_value_15);

        let actual_check_4 = lct.path_sum(input_value_16, input_value_17);
        let actual_check_5 = lct.connected(input_value_18, input_value_19).unwrap();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
        assert_eq!(actual_check_2, expected_boolean_2);
        assert_eq!(actual_check_3, expected_boolean_3);
        assert_eq!(actual_check_4, expected_check_4);
        assert_eq!(actual_check_5, expected_boolean_4);
    }

    #[test]
    fn single_vertex_set_and_get_value_work() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 0;
        let input_value_3 = 7;
        let input_value_4 = 0;
        let mut tree = LinkCutTree::new(input_value_1);
        let expected_value = Some(7);

        // Act
        tree.set_value(input_value_2, input_value_3);
        let actual_value = tree.value(input_value_4);

        // Assert
        assert_eq!(actual_value, expected_value);
    }
    #[test]
    fn out_of_range_vertices_return_false_or_none() {
        // Arrange
        let size = 2;
        let valid = 0;
        let invalid = size;
        let expected = (false, false, None, None, None);
        let mut tree = LinkCutTree::new(size);

        // Act
        let actual = (
            tree.link(valid, invalid),
            tree.cut(valid, invalid),
            tree.connected(valid, invalid),
            tree.path_sum(valid, invalid),
            tree.value(invalid),
        );

        // Assert
        assert_eq!(actual, expected);
    }
    #[test]
    fn empty_tree_has_zero_length_and_rejects_vertex_zero() {
        // Arrange
        let size = 0;
        let vertex = 0;
        let expected = (true, 0, false, false, None, None, None);
        let mut tree = LinkCutTree::new(size);

        // Act
        let actual = (
            tree.is_empty(),
            tree.len(),
            tree.link(vertex, vertex),
            tree.cut(vertex, vertex),
            tree.connected(vertex, vertex),
            tree.path_sum(vertex, vertex),
            tree.value(vertex),
        );

        // Assert
        assert_eq!(actual, expected);
    }
}

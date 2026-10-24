pub trait MapMonoid {
    type S: Clone;
    type F: Clone;

    fn id() -> Self::S;
    fn identity_map() -> Self::F;
    fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S;
    fn mapping(f: &Self::F, x: &Self::S) -> Self::S;
    fn composition(f: &Self::F, g: &Self::F) -> Self::F;
}

struct TreapNode<M>
where
    M: MapMonoid,
{
    value: M::S,
    priority: u64,
    size: usize,
    forward: M::S,
    backward: M::S,
    lazy: M::F,
    has_lazy: bool,
    reversed: bool,
    left: Option<Box<TreapNode<M>>>,
    right: Option<Box<TreapNode<M>>>,
}

impl<M> std::fmt::Debug for TreapNode<M>
where
    M: MapMonoid,
    M::S: std::fmt::Debug,
    M::F: std::fmt::Debug,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TreapNode")
            .field("value", &self.value)
            .field("priority", &self.priority)
            .field("size", &self.size)
            .field("forward", &self.forward)
            .field("backward", &self.backward)
            .field("lazy", &self.lazy)
            .field("has_lazy", &self.has_lazy)
            .field("reversed", &self.reversed)
            .field("left", &self.left)
            .field("right", &self.right)
            .finish()
    }
}

impl<M> TreapNode<M>
where
    M: MapMonoid,
{
    fn new(value: M::S, priority: u64) -> Self {
        Self {
            forward: value.clone(),
            backward: value.clone(),
            value,
            priority,
            size: 1,
            lazy: M::identity_map(),
            has_lazy: false,
            reversed: false,
            left: None,
            right: None,
        }
    }

    fn size_of(node: &Option<Box<TreapNode<M>>>) -> usize {
        node.as_ref().map_or(0, |target_node| target_node.size)
    }

    fn forward_of(node: &Option<Box<TreapNode<M>>>) -> M::S {
        node.as_ref()
            .map_or_else(M::id, |target_node| target_node.forward.clone())
    }

    fn backward_of(node: &Option<Box<TreapNode<M>>>) -> M::S {
        node.as_ref()
            .map_or_else(M::id, |target_node| target_node.backward.clone())
    }

    fn apply_mapping(node: &mut Box<TreapNode<M>>, mapping: &M::F) {
        node.value = M::mapping(mapping, &node.value);
        node.forward = M::mapping(mapping, &node.forward);
        node.backward = M::mapping(mapping, &node.backward);
        node.lazy = if node.has_lazy {
            M::composition(mapping, &node.lazy)
        } else {
            mapping.clone()
        };
        node.has_lazy = true;
    }

    fn apply_reverse(node: &mut Box<TreapNode<M>>) {
        std::mem::swap(&mut node.left, &mut node.right);
        std::mem::swap(&mut node.forward, &mut node.backward);
        node.reversed = !node.reversed;
    }

    fn push(node: &mut Box<TreapNode<M>>) {
        if node.has_lazy {
            let pending_mapping = node.lazy.clone();
            if let Some(left_child_node) = node.left.as_mut() {
                Self::apply_mapping(left_child_node, &pending_mapping);
            }
            if let Some(right_child_node) = node.right.as_mut() {
                Self::apply_mapping(right_child_node, &pending_mapping);
            }
            node.lazy = M::identity_map();
            node.has_lazy = false;
        }

        if node.reversed {
            if let Some(left_child_node) = node.left.as_mut() {
                Self::apply_reverse(left_child_node);
            }
            if let Some(right_child_node) = node.right.as_mut() {
                Self::apply_reverse(right_child_node);
            }
            node.reversed = false;
        }
    }

    fn pull(node: &mut Box<TreapNode<M>>) {
        node.size = 1 + Self::size_of(&node.left) + Self::size_of(&node.right);

        let left_forward = Self::forward_of(&node.left);
        let right_forward = Self::forward_of(&node.right);
        let left_with_value = M::binary_operation(&left_forward, &node.value);
        node.forward = M::binary_operation(&left_with_value, &right_forward);

        let right_backward = Self::backward_of(&node.right);
        let left_backward = Self::backward_of(&node.left);
        let right_with_value = M::binary_operation(&right_backward, &node.value);
        node.backward = M::binary_operation(&right_with_value, &left_backward);
    }
}

#[derive(Debug)]
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        let fallback_seed = 0x9E37_79B9_7F4A_7C15_u64;
        let initial_state = if seed == 0 { fallback_seed } else { seed };
        Self {
            state: initial_state,
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut random_value = self.state;
        random_value ^= random_value << 13;
        random_value ^= random_value >> 7;
        random_value ^= random_value << 17;
        self.state = random_value;
        random_value
    }
}

fn merge_treap<M>(
    mut left_root: Option<Box<TreapNode<M>>>,
    mut right_root: Option<Box<TreapNode<M>>>,
) -> Option<Box<TreapNode<M>>>
where
    M: MapMonoid,
{
    match (left_root.take(), right_root.take()) {
        (None, right_tree) => right_tree,
        (left_tree, None) => left_tree,
        (Some(mut left_node), Some(mut right_node)) => {
            if right_node.priority <= left_node.priority {
                TreapNode::push(&mut left_node);
                left_node.right = merge_treap(left_node.right.take(), Some(right_node));
                TreapNode::pull(&mut left_node);
                Some(left_node)
            } else {
                TreapNode::push(&mut right_node);
                right_node.left = merge_treap(Some(left_node), right_node.left.take());
                TreapNode::pull(&mut right_node);
                Some(right_node)
            }
        }
    }
}

fn split_treap<M>(
    mut root_node: Option<Box<TreapNode<M>>>,
    split_index: usize,
) -> (Option<Box<TreapNode<M>>>, Option<Box<TreapNode<M>>>)
where
    M: MapMonoid,
{
    if let Some(mut target_node) = root_node.take() {
        TreapNode::push(&mut target_node);
        let left_subtree_size = TreapNode::size_of(&target_node.left);

        if split_index <= left_subtree_size {
            let (left_tree, right_tree) = split_treap(target_node.left.take(), split_index);
            target_node.left = right_tree;
            TreapNode::pull(&mut target_node);
            (left_tree, Some(target_node))
        } else {
            let next_split_index = split_index - left_subtree_size - 1;
            let (left_tree, right_tree) = split_treap(target_node.right.take(), next_split_index);
            target_node.right = left_tree;
            TreapNode::pull(&mut target_node);
            (Some(target_node), right_tree)
        }
    } else {
        (None, None)
    }
}

pub struct LazyReversibleTreap<M>
where
    M: MapMonoid,
{
    root: Option<Box<TreapNode<M>>>,
    random_generator: XorShift64,
}

impl<M> std::fmt::Debug for LazyReversibleTreap<M>
where
    M: MapMonoid,
    M::S: std::fmt::Debug,
    M::F: std::fmt::Debug,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LazyReversibleTreap")
            .field("root", &self.root)
            .field("random_generator", &self.random_generator)
            .finish()
    }
}

impl<M> LazyReversibleTreap<M>
where
    M: MapMonoid,
{
    pub fn new(seed: u64) -> Self {
        Self {
            root: None,
            random_generator: XorShift64::new(seed),
        }
    }

    pub fn from_slice(seed: u64, values: &[M::S]) -> Self {
        let mut treap = Self::new(seed);
        for (index, value) in values.iter().enumerate() {
            treap.insert(index, value.clone());
        }
        treap
    }

    pub fn len(&self) -> usize {
        TreapNode::size_of(&self.root)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, index: usize, value: M::S) {
        let clamped_index = std::cmp::min(index, self.len());
        let (left_tree, right_tree) = split_treap(self.root.take(), clamped_index);

        let node_priority = self.random_generator.next_u64();
        let new_node = Some(Box::new(TreapNode::new(value, node_priority)));

        let merged_left_tree = merge_treap(left_tree, new_node);
        self.root = merge_treap(merged_left_tree, right_tree);
    }

    pub fn remove(&mut self, index: usize) -> Option<M::S> {
        if self.len() <= index {
            return None;
        }

        let (left_tree, center_right_tree) = split_treap(self.root.take(), index);
        let (center_tree, right_tree) = split_treap(center_right_tree, 1);
        self.root = merge_treap(left_tree, right_tree);
        center_tree.map(|target_node| target_node.value)
    }

    pub fn get(&mut self, index: usize) -> Option<M::S> {
        if self.len() <= index {
            return None;
        }

        let (left_tree, center_right_tree) = split_treap(self.root.take(), index);
        let (center_tree, right_tree) = split_treap(center_right_tree, 1);
        let selected_value = center_tree
            .as_ref()
            .map(|target_node| target_node.value.clone());
        let merged_left_tree = merge_treap(left_tree, center_tree);
        self.root = merge_treap(merged_left_tree, right_tree);
        selected_value
    }

    pub fn reverse(&mut self, left_index: usize, right_index: usize) -> bool {
        if self.len() < right_index || right_index < left_index {
            return false;
        }

        if left_index == right_index {
            return true;
        }

        let (left_tree, center_right_tree) = split_treap(self.root.take(), left_index);
        let (mut center_tree, right_tree) =
            split_treap(center_right_tree, right_index - left_index);

        if let Some(center_root_node) = center_tree.as_mut() {
            TreapNode::apply_reverse(center_root_node);
        }

        let merged_left_tree = merge_treap(left_tree, center_tree);
        self.root = merge_treap(merged_left_tree, right_tree);
        true
    }

    pub fn apply(&mut self, left_index: usize, right_index: usize, mapping: M::F) -> bool {
        if self.len() < right_index || right_index < left_index {
            return false;
        }

        if left_index == right_index {
            return true;
        }

        let (left_tree, center_right_tree) = split_treap(self.root.take(), left_index);
        let (mut center_tree, right_tree) =
            split_treap(center_right_tree, right_index - left_index);

        if let Some(center_root_node) = center_tree.as_mut() {
            TreapNode::apply_mapping(center_root_node, &mapping);
        }

        let merged_left_tree = merge_treap(left_tree, center_tree);
        self.root = merge_treap(merged_left_tree, right_tree);
        true
    }

    pub fn query(&mut self, left_index: usize, right_index: usize) -> Option<M::S> {
        if self.len() < right_index || right_index < left_index {
            return None;
        }

        if left_index == right_index {
            return Some(M::id());
        }

        let (left_tree, center_right_tree) = split_treap(self.root.take(), left_index);
        let (center_tree, right_tree) = split_treap(center_right_tree, right_index - left_index);
        let target_value = center_tree
            .as_ref()
            .map_or_else(M::id, |target_node| target_node.forward.clone());
        let merged_left_tree = merge_treap(left_tree, center_tree);
        self.root = merge_treap(merged_left_tree, right_tree);
        Some(target_value)
    }

    pub fn to_vec(&mut self) -> Vec<M::S> {
        let mut result_values = Vec::with_capacity(self.len());
        Self::collect_values_in_order(&mut self.root, &mut result_values);
        result_values
    }

    fn collect_values_in_order(
        node: &mut Option<Box<TreapNode<M>>>,
        result_values: &mut Vec<M::S>,
    ) {
        if let Some(target_node) = node.as_mut() {
            TreapNode::push(target_node);
            Self::collect_values_in_order(&mut target_node.left, result_values);
            result_values.push(target_node.value.clone());
            Self::collect_values_in_order(&mut target_node.right, result_values);
            TreapNode::pull(target_node);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct T;

impl MapMonoid for T {
    type S = (usize, usize, usize);
    type F = usize;

    fn id() -> Self::S {
        (0, 0, 0)
    }

    fn identity_map() -> Self::F {
        0
    }

    fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
        (lhs.0 + rhs.0, lhs.1 + rhs.1, lhs.2 + rhs.2 + lhs.1 * rhs.0)
    }

    fn mapping(f: &Self::F, x: &Self::S) -> Self::S {
        if *f & 1 == 0 {
            *x
        } else {
            (x.1, x.0, x.1 * x.0 - x.2)
        }
    }

    // f(g(x))
    fn composition(f: &Self::F, g: &Self::F) -> Self::F {
        *f + *g
    }
}

#[cfg(test)]
mod tests {
    use super::{LazyReversibleTreap as GenericLazyReversibleTreap, MapMonoid};

    const TEST_SEED: u64 = 123_456_789;

    const RANDOMIZED_OPERATION_COUNT: usize = 1_200;

    const RANDOMIZED_MAX_LEN: usize = 48;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct SumNode {
        range_sum: i64,
        range_len: i64,
    }

    #[derive(Clone, Copy)]
    struct RangeAddSumMonoid;

    impl MapMonoid for RangeAddSumMonoid {
        type S = SumNode;
        type F = i64;

        fn id() -> Self::S {
            SumNode {
                range_sum: 0,
                range_len: 0,
            }
        }

        fn identity_map() -> Self::F {
            0
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            SumNode {
                range_sum: lhs.range_sum + rhs.range_sum,
                range_len: lhs.range_len + rhs.range_len,
            }
        }

        fn mapping(f: &Self::F, x: &Self::S) -> Self::S {
            SumNode {
                range_sum: x.range_sum + f * x.range_len,
                range_len: x.range_len,
            }
        }

        fn composition(f: &Self::F, g: &Self::F) -> Self::F {
            *f + *g
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct AffineNode {
        range_sum: i64,
        range_len: i64,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct AffineMap {
        mul_value: i64,
        add_value: i64,
    }

    #[derive(Clone, Copy)]
    struct AffineSumMonoid;

    impl MapMonoid for AffineSumMonoid {
        type S = AffineNode;
        type F = AffineMap;

        fn id() -> Self::S {
            AffineNode {
                range_sum: 0,
                range_len: 0,
            }
        }

        fn identity_map() -> Self::F {
            AffineMap {
                mul_value: 1,
                add_value: 0,
            }
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            AffineNode {
                range_sum: lhs.range_sum + rhs.range_sum,
                range_len: lhs.range_len + rhs.range_len,
            }
        }

        fn mapping(f: &Self::F, x: &Self::S) -> Self::S {
            AffineNode {
                range_sum: f.mul_value * x.range_sum + f.add_value * x.range_len,
                range_len: x.range_len,
            }
        }

        fn composition(f: &Self::F, g: &Self::F) -> Self::F {
            AffineMap {
                mul_value: f.mul_value * g.mul_value,
                add_value: f.mul_value * g.add_value + f.add_value,
            }
        }
    }

    #[derive(Clone, Copy)]
    struct StringConcatToggleMonoid;

    impl MapMonoid for StringConcatToggleMonoid {
        type S = String;
        type F = bool;

        fn id() -> Self::S {
            String::new()
        }

        fn identity_map() -> Self::F {
            false
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            let mut concatenated_value = lhs.clone();
            concatenated_value.push_str(rhs);
            concatenated_value
        }

        fn mapping(toggle_case: &Self::F, value: &Self::S) -> Self::S {
            if *toggle_case {
                toggle_ascii_case(value)
            } else {
                value.clone()
            }
        }

        fn composition(new_map: &Self::F, old_map: &Self::F) -> Self::F {
            *new_map ^ *old_map
        }
    }

    struct LazyReversibleTreap {
        treap: GenericLazyReversibleTreap<RangeAddSumMonoid>,
    }

    impl LazyReversibleTreap {
        fn new(seed: u64) -> Self {
            Self {
                treap: GenericLazyReversibleTreap::new(seed),
            }
        }

        fn from_slice(seed: u64, values: &[i64]) -> Self {
            let sum_nodes: Vec<SumNode> = values.iter().copied().map(create_sum_node).collect();
            Self {
                treap: GenericLazyReversibleTreap::from_slice(seed, &sum_nodes),
            }
        }

        fn len(&self) -> usize {
            self.treap.len()
        }

        fn is_empty(&self) -> bool {
            self.treap.is_empty()
        }

        fn insert(&mut self, index: usize, value: i64) {
            self.treap.insert(index, create_sum_node(value));
        }

        fn remove(&mut self, index: usize) -> Option<i64> {
            self.treap.remove(index).map(|node| node.range_sum)
        }

        fn get(&mut self, index: usize) -> Option<i64> {
            self.treap.get(index).map(|node| node.range_sum)
        }

        fn reverse(&mut self, left_index: usize, right_index: usize) -> bool {
            self.treap.reverse(left_index, right_index)
        }

        fn range_add(&mut self, left_index: usize, right_index: usize, added_value: i64) -> bool {
            self.treap.apply(left_index, right_index, added_value)
        }

        fn range_sum(&mut self, left_index: usize, right_index: usize) -> Option<i64> {
            self.treap
                .query(left_index, right_index)
                .map(|node| node.range_sum)
        }

        fn to_vec(&mut self) -> Vec<i64> {
            self.treap
                .to_vec()
                .into_iter()
                .map(|node| node.range_sum)
                .collect()
        }
    }

    #[derive(Clone, Copy)]
    struct ReferenceXorShift64 {
        state: u64,
    }

    impl ReferenceXorShift64 {
        fn new(seed: u64) -> Self {
            let fallback_seed = 0xA076_1D64_78BD_642F_u64;
            Self {
                state: if seed == 0 { fallback_seed } else { seed },
            }
        }

        fn next_u64(&mut self) -> u64 {
            let mut random_value = self.state;
            random_value ^= random_value << 13;
            random_value ^= random_value >> 7;
            random_value ^= random_value << 17;
            self.state = random_value;
            random_value
        }
    }

    fn toggle_ascii_case(value: &str) -> String {
        value
            .chars()
            .map(|character| {
                if character.is_ascii_lowercase() {
                    character.to_ascii_uppercase()
                } else if character.is_ascii_uppercase() {
                    character.to_ascii_lowercase()
                } else {
                    character
                }
            })
            .collect()
    }

    fn create_sum_node(value: i64) -> SumNode {
        SumNode {
            range_sum: value,
            range_len: 1,
        }
    }

    fn create_affine_node(value: i64) -> AffineNode {
        AffineNode {
            range_sum: value,
            range_len: 1,
        }
    }

    fn create_treap_with_values(values: &[i64]) -> LazyReversibleTreap {
        LazyReversibleTreap::from_slice(TEST_SEED, values)
    }

    fn calculate_range_sum(values: &[i64], left_index: usize, right_index: usize) -> i64 {
        values[left_index..right_index].iter().sum()
    }

    fn insert_with_clamp(values: &mut Vec<i64>, index: usize, value: i64) {
        let clamped_index = std::cmp::min(index, values.len());
        values.insert(clamped_index, value);
    }

    fn remove_if_exists(values: &mut Vec<i64>, index: usize) -> Option<i64> {
        if values.len() <= index {
            None
        } else {
            Some(values.remove(index))
        }
    }

    fn get_if_exists(values: &[i64], index: usize) -> Option<i64> {
        values.get(index).copied()
    }

    fn reverse_if_valid(values: &mut [i64], left_index: usize, right_index: usize) -> bool {
        if values.len() < right_index || right_index < left_index {
            return false;
        }

        if left_index == right_index {
            return true;
        }

        values[left_index..right_index].reverse();
        true
    }

    fn range_add_if_valid(
        values: &mut [i64],
        left_index: usize,
        right_index: usize,
        added_value: i64,
    ) -> bool {
        if values.len() < right_index || right_index < left_index {
            return false;
        }

        if left_index == right_index {
            return true;
        }

        for value in &mut values[left_index..right_index] {
            *value += added_value;
        }
        true
    }

    fn range_sum_if_valid(values: &[i64], left_index: usize, right_index: usize) -> Option<i64> {
        if values.len() < right_index || right_index < left_index {
            return None;
        }

        if left_index == right_index {
            return Some(0);
        }

        Some(calculate_range_sum(values, left_index, right_index))
    }

    fn generate_random_index(
        len: usize,
        cycle: usize,
        random_generator: &mut ReferenceXorShift64,
    ) -> usize {
        match cycle % 5 {
            0 => 0,
            1 => len,
            2 => len + 1,
            3 => usize::MAX,
            _ if len == 0 => 0,
            _ => random_generator.next_u64() as usize % len,
        }
    }

    fn generate_random_range(
        len: usize,
        cycle: usize,
        random_generator: &mut ReferenceXorShift64,
    ) -> (usize, usize) {
        match cycle % 6 {
            0 => (0, len),
            1 => {
                let index = random_generator.next_u64() as usize % (len + 1);
                (index, index)
            }
            2 => {
                let first_index = random_generator.next_u64() as usize % (len + 1);
                let second_index = random_generator.next_u64() as usize % (len + 1);
                (
                    std::cmp::min(first_index, second_index),
                    std::cmp::max(first_index, second_index),
                )
            }
            3 => (len + 1, len),
            4 => (0, len + 1),
            _ => (0, usize::MAX),
        }
    }

    fn run_randomized_vec_oracle(treap_seed: u64, operation_count: usize) -> Result<(), String> {
        let initial_values = vec![3_i64, -1, 4, -1, 5, -9, 2, 6];
        let mut expected_values = initial_values.clone();
        let mut treap = LazyReversibleTreap::from_slice(treap_seed, &initial_values);
        let operation_seed = 0xE703_7ED1_A0B4_28DB_u64;
        let mut random_generator = ReferenceXorShift64::new(operation_seed);

        for step in 0..operation_count {
            let cycle = step / 6;

            match step % 6 {
                0 if expected_values.len() < RANDOMIZED_MAX_LEN => {
                    let insert_index =
                        generate_random_index(expected_values.len(), cycle, &mut random_generator);
                    let inserted_value = (random_generator.next_u64() % 101) as i64 - 50;
                    treap.insert(insert_index, inserted_value);
                    insert_with_clamp(&mut expected_values, insert_index, inserted_value);
                }
                0 => {
                    let remove_index = random_generator.next_u64() as usize % expected_values.len();
                    let actual_removed_value = treap.remove(remove_index);
                    let expected_removed_value =
                        remove_if_exists(&mut expected_values, remove_index);

                    if actual_removed_value != expected_removed_value {
                        return Err(format!(
                            "step {step}, capped insert replacement remove({remove_index}): expected {expected_removed_value:?}, actual {actual_removed_value:?}"
                        ));
                    }
                }
                1 => {
                    let remove_index =
                        generate_random_index(expected_values.len(), cycle, &mut random_generator);
                    let actual_removed_value = treap.remove(remove_index);
                    let expected_removed_value =
                        remove_if_exists(&mut expected_values, remove_index);

                    if actual_removed_value != expected_removed_value {
                        return Err(format!(
                            "step {step}, remove({remove_index}): expected {expected_removed_value:?}, actual {actual_removed_value:?}"
                        ));
                    }
                }
                2 => {
                    let get_index =
                        generate_random_index(expected_values.len(), cycle, &mut random_generator);
                    let actual_value = treap.get(get_index);
                    let expected_value = get_if_exists(&expected_values, get_index);

                    if actual_value != expected_value {
                        return Err(format!(
                            "step {step}, get({get_index}): expected {expected_value:?}, actual {actual_value:?}"
                        ));
                    }
                }
                3 => {
                    let (left_index, right_index) =
                        generate_random_range(expected_values.len(), cycle, &mut random_generator);
                    let actual_result = treap.reverse(left_index, right_index);
                    let expected_result =
                        reverse_if_valid(&mut expected_values, left_index, right_index);

                    if actual_result != expected_result {
                        return Err(format!(
                            "step {step}, reverse({left_index}, {right_index}): expected {expected_result}, actual {actual_result}"
                        ));
                    }
                }
                4 => {
                    let (left_index, right_index) =
                        generate_random_range(expected_values.len(), cycle, &mut random_generator);
                    let added_value = (random_generator.next_u64() % 21) as i64 - 10;
                    let actual_result = treap.range_add(left_index, right_index, added_value);
                    let expected_result = range_add_if_valid(
                        &mut expected_values,
                        left_index,
                        right_index,
                        added_value,
                    );

                    if actual_result != expected_result {
                        return Err(format!(
                            "step {step}, range_add({left_index}, {right_index}, {added_value}): expected {expected_result}, actual {actual_result}"
                        ));
                    }
                }
                _ => {
                    let (left_index, right_index) =
                        generate_random_range(expected_values.len(), cycle, &mut random_generator);
                    let actual_sum = treap.range_sum(left_index, right_index);
                    let expected_sum =
                        range_sum_if_valid(&expected_values, left_index, right_index);

                    if actual_sum != expected_sum {
                        return Err(format!(
                            "step {step}, range_sum({left_index}, {right_index}): expected {expected_sum:?}, actual {actual_sum:?}"
                        ));
                    }
                }
            }

            let actual_len = treap.len();
            let expected_len = expected_values.len();
            if actual_len != expected_len {
                return Err(format!(
                    "step {step}, len: expected {expected_len}, actual {actual_len}"
                ));
            }

            let actual_is_empty = treap.is_empty();
            let expected_is_empty = expected_values.is_empty();
            if actual_is_empty != expected_is_empty {
                return Err(format!(
                    "step {step}, is_empty: expected {expected_is_empty}, actual {actual_is_empty}"
                ));
            }

            if step % 29 == 28 {
                let actual_values = treap.to_vec();
                if actual_values != expected_values {
                    return Err(format!(
                        "step {step}, to_vec checkpoint: expected {expected_values:?}, actual {actual_values:?}"
                    ));
                }
            }
        }

        for left_index in 0..=expected_values.len() {
            for right_index in left_index..=expected_values.len() {
                let actual_sum = treap.range_sum(left_index, right_index);
                let expected_sum = range_sum_if_valid(&expected_values, left_index, right_index);

                if actual_sum != expected_sum {
                    return Err(format!(
                        "final range_sum({left_index}, {right_index}): expected {expected_sum:?}, actual {actual_sum:?}"
                    ));
                }
            }
        }

        let actual_values = treap.to_vec();
        if actual_values != expected_values {
            return Err(format!(
                "final to_vec: expected {expected_values:?}, actual {actual_values:?}"
            ));
        }

        Ok(())
    }

    #[test]
    fn new_returns_empty_state() {
        // Arrange
        let seed = TEST_SEED;
        let expected_len = 0_usize;
        let expected_is_empty = true;
        let expected_values: Vec<i64> = vec![];
        let mut treap = LazyReversibleTreap::new(seed);

        // Act
        let actual_len = treap.len();
        let actual_is_empty = treap.is_empty();
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn new_with_zero_seed_allows_insert_and_get() {
        // Arrange
        let seed = 0_u64;
        let insert_index = 0_usize;
        let inserted_value = 42_i64;
        let expected_value = Some(42_i64);
        let expected_values = vec![42_i64];
        let mut treap = LazyReversibleTreap::new(seed);

        // Act
        treap.insert(insert_index, inserted_value);
        let actual_value = treap.get(insert_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn range_add_sum_wrapper_supports_new_and_from_slice() {
        // Arrange
        let seed = TEST_SEED;
        let input_values = vec![1_i64, 2, 3];
        let insert_index = 0_usize;
        let inserted_value = 4_i64;
        let query_left_index = 0_usize;
        let empty_query_right_index = 1_usize;
        let slice_query_right_index = input_values.len();
        let added_value = 2_i64;
        let expected_add_result = true;
        let expected_empty_tree_sum = Some(4_i64);
        let expected_slice_tree_sum = Some(12_i64);
        let expected_slice_tree_values = vec![3_i64, 4, 5];

        // Act
        let mut empty_treap = LazyReversibleTreap::new(seed);
        let mut slice_treap = LazyReversibleTreap::from_slice(seed, &input_values);
        empty_treap.insert(insert_index, inserted_value);
        let actual_add_result =
            slice_treap.range_add(query_left_index, slice_query_right_index, added_value);
        let actual_empty_tree_sum =
            empty_treap.range_sum(query_left_index, empty_query_right_index);
        let actual_slice_tree_sum =
            slice_treap.range_sum(query_left_index, slice_query_right_index);
        let actual_slice_tree_values = slice_treap.to_vec();

        // Assert
        assert_eq!(actual_add_result, expected_add_result);
        assert_eq!(actual_empty_tree_sum, expected_empty_tree_sum);
        assert_eq!(actual_slice_tree_sum, expected_slice_tree_sum);
        assert_eq!(actual_slice_tree_values, expected_slice_tree_values);
    }

    #[test]
    fn from_slice_with_empty_values_returns_empty_state() {
        // Arrange
        let input_values: Vec<i64> = vec![];
        let expected_len = 0_usize;
        let expected_is_empty = true;
        let expected_values: Vec<i64> = vec![];
        let mut treap = create_treap_with_values(&input_values);

        // Act
        let actual_len = treap.len();
        let actual_is_empty = treap.is_empty();
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn from_slice_with_single_value_returns_value_and_sum() {
        // Arrange
        let input_values = vec![7_i64];
        let index = 0_usize;
        let left_index = 0_usize;
        let right_index = 1_usize;
        let expected_value = Some(7_i64);
        let expected_sum = Some(7_i64);
        let mut treap = create_treap_with_values(&input_values);

        // Act
        let actual_value = treap.get(index);
        let actual_sum = treap.range_sum(left_index, right_index);

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn from_slice_with_multiple_values_keeps_order() {
        // Arrange
        let input_values = vec![1_i64, 2_i64, 3_i64];
        let expected_values = input_values.clone();
        let mut treap = create_treap_with_values(&input_values);

        // Act
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn insert_at_zero_inserts_to_head() {
        // Arrange
        let initial_values = vec![20_i64, 30_i64];
        let insert_index = 0_usize;
        let inserted_value = 10_i64;
        let expected_values = vec![10_i64, 20_i64, 30_i64];
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        treap.insert(insert_index, inserted_value);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn insert_at_len_inserts_to_tail() {
        // Arrange
        let initial_values = vec![10_i64, 20_i64];
        let inserted_value = 30_i64;
        let expected_values = vec![10_i64, 20_i64, 30_i64];
        let mut treap = create_treap_with_values(&initial_values);

        let insert_index = initial_values.len();

        // Act
        treap.insert(insert_index, inserted_value);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn insert_beyond_len_clamps_to_tail() {
        // Arrange
        let initial_values = vec![10_i64, 20_i64];
        let insert_index = 3_usize;
        let inserted_value = 30_i64;
        let expected_values = vec![10_i64, 20_i64, 30_i64];
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        treap.insert(insert_index, inserted_value);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn remove_on_empty_returns_none_and_keeps_state() {
        // Arrange
        let remove_index = 0_usize;
        let expected_value = None;
        let expected_values: Vec<i64> = vec![];
        let mut treap = create_treap_with_values(&[]);

        // Act
        let actual_value = treap.remove(remove_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn remove_at_zero_removes_head() {
        // Arrange
        let initial_values = vec![10_i64, 20_i64, 30_i64];
        let remove_index = 0_usize;
        let expected_value = Some(10_i64);
        let expected_values = vec![20_i64, 30_i64];
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_value = treap.remove(remove_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn remove_at_len_minus_one_removes_tail() {
        // Arrange
        let initial_values = vec![10_i64, 20_i64, 30_i64];
        let expected_value = Some(30_i64);
        let expected_values = vec![10_i64, 20_i64];
        let mut treap = create_treap_with_values(&initial_values);

        let remove_index = initial_values.len() - 1;

        // Act
        let actual_value = treap.remove(remove_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn remove_at_len_returns_none_and_keeps_state() {
        // Arrange
        let initial_values = vec![10_i64, 20_i64, 30_i64];
        let expected_value = None;
        let expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        let remove_index = initial_values.len();

        // Act
        let actual_value = treap.remove(remove_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn range_add_then_remove_without_to_vec_returns_updated_removed_value() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64];
        let add_left_index = 1_usize;
        let add_right_index = 3_usize;
        let added_value = 10_i64;
        let remove_index = 2_usize;
        let expected_add_result = true;
        let expected_remove_value = Some(13_i64);
        let expected_values = vec![1_i64, 12_i64];
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_add_result = treap.range_add(add_left_index, add_right_index, added_value);
        let actual_remove_value = treap.remove(remove_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_add_result, expected_add_result);
        assert_eq!(actual_remove_value, expected_remove_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn get_on_empty_returns_none_and_keeps_state() {
        // Arrange
        let get_index = 0_usize;
        let expected_value = None;
        let expected_values: Vec<i64> = vec![];
        let mut treap = create_treap_with_values(&[]);

        // Act
        let actual_value = treap.get(get_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn get_at_zero_returns_head_and_keeps_order() {
        // Arrange
        let initial_values = vec![10_i64, 20_i64, 30_i64];
        let get_index = 0_usize;
        let expected_value = Some(10_i64);
        let expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_value = treap.get(get_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn get_at_len_minus_one_returns_tail_and_keeps_order() {
        // Arrange
        let initial_values = vec![10_i64, 20_i64, 30_i64];
        let expected_value = Some(30_i64);
        let expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        let get_index = initial_values.len() - 1;

        // Act
        let actual_value = treap.get(get_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn get_at_len_returns_none_and_keeps_state() {
        // Arrange
        let initial_values = vec![10_i64, 20_i64, 30_i64];
        let expected_value = None;
        let expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        let get_index = initial_values.len();

        // Act
        let actual_value = treap.get(get_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn reverse_then_get_without_to_vec_returns_value_in_reversed_order() {
        // Arrange
        let initial_values = vec![10_i64, 20_i64, 30_i64];
        let reverse_left_index = 0_usize;
        let reverse_right_index = 3_usize;
        let get_index = 0_usize;
        let expected_reverse_result = true;
        let expected_get_value = Some(30_i64);
        let expected_values = vec![30_i64, 20_i64, 10_i64];
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_reverse_result = treap.reverse(reverse_left_index, reverse_right_index);
        let actual_get_value = treap.get(get_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_reverse_result, expected_reverse_result);
        assert_eq!(actual_get_value, expected_get_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn range_add_then_get_without_to_vec_returns_added_value() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64];
        let add_left_index = 0_usize;
        let add_right_index = 3_usize;
        let added_value = 5_i64;
        let get_index = 1_usize;
        let expected_add_result = true;
        let expected_get_value = Some(7_i64);
        let expected_values = vec![6_i64, 7_i64, 8_i64];
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_add_result = treap.range_add(add_left_index, add_right_index, added_value);
        let actual_get_value = treap.get(get_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_add_result, expected_add_result);
        assert_eq!(actual_get_value, expected_get_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn reverse_with_left_greater_than_right_returns_false_and_keeps_state() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64];
        let left_index = 2_usize;
        let right_index = 1_usize;
        let expected_result = false;
        let expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_result = treap.reverse(left_index, right_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn reverse_on_empty_with_zero_bounds_returns_true_and_keeps_empty() {
        // Arrange
        let initial_values: Vec<i64> = vec![];
        let left_index = 0_usize;
        let right_index = 0_usize;
        let expected_result = true;
        let expected_values: Vec<i64> = vec![];
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_result = treap.reverse(left_index, right_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn reverse_with_right_beyond_len_returns_false_and_keeps_state() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64];
        let left_index = 0_usize;
        let right_index = 4_usize;
        let expected_result = false;
        let expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_result = treap.reverse(left_index, right_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn reverse_with_equal_bounds_returns_true_and_keeps_state() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64];
        let left_index = 1_usize;
        let right_index = 1_usize;
        let expected_result = true;
        let expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_result = treap.reverse(left_index, right_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn reverse_whole_range_reverses_order_and_keeps_total_sum() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64, 4_i64];
        let left_index = 0_usize;
        let right_index = initial_values.len();
        let expected_result = true;
        let expected_values = vec![4_i64, 3_i64, 2_i64, 1_i64];
        let expected_total_sum = Some(10_i64);
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_result = treap.reverse(left_index, right_index);
        let actual_values = treap.to_vec();
        let actual_total_sum = treap.range_sum(left_index, right_index);

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_total_sum, expected_total_sum);
    }

    #[test]
    fn reverse_partial_range_changes_only_target_segment() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
        let left_index = 1_usize;
        let right_index = 4_usize;
        let expected_result = true;
        let expected_values = vec![1_i64, 4_i64, 3_i64, 2_i64, 5_i64];
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_result = treap.reverse(left_index, right_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn range_add_then_reverse_matches_vec_reference_for_values_and_sum() {
        // Arrange
        let initial_values = vec![3_i64, 1_i64, 4_i64, 1_i64, 5_i64, 9_i64];
        let add_left_index = 1_usize;
        let add_right_index = 5_usize;
        let added_value = 2_i64;
        let reverse_left_index = 0_usize;
        let reverse_right_index = 4_usize;
        let sum_left_index = 1_usize;
        let sum_right_index = 5_usize;
        let mut expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        let expected_add_result = range_add_if_valid(
            &mut expected_values,
            add_left_index,
            add_right_index,
            added_value,
        );
        let expected_reverse_result = reverse_if_valid(
            &mut expected_values,
            reverse_left_index,
            reverse_right_index,
        );
        let expected_sum = range_sum_if_valid(&expected_values, sum_left_index, sum_right_index);

        // Act
        let actual_add_result = treap.range_add(add_left_index, add_right_index, added_value);
        let actual_reverse_result = treap.reverse(reverse_left_index, reverse_right_index);
        let actual_values = treap.to_vec();
        let actual_sum = treap.range_sum(sum_left_index, sum_right_index);

        // Assert
        assert_eq!(actual_add_result, expected_add_result);
        assert_eq!(actual_reverse_result, expected_reverse_result);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn lazy_action_and_reverse_update_both_aggregate_directions() {
        // Arrange
        let seed = TEST_SEED;
        let input_values = ["a", "B", "c", "D", "e"].map(str::to_string);
        let apply_left_index = 1_usize;
        let apply_right_index = 4_usize;
        let toggle_case_mapping = true;
        let reverse_left_index = 0_usize;
        let reverse_right_index = 4_usize;
        let full_query_left_index = 0_usize;
        let full_query_right_index = input_values.len();
        let first_index = 0_usize;
        let expected_apply_result = true;
        let expected_reverse_result = true;
        let expected_partial_query = Some("dCba".to_string());
        let expected_full_query = Some("dCbae".to_string());
        let expected_first_value = Some("d".to_string());
        let expected_values = ["d", "C", "b", "a", "e"].map(str::to_string).to_vec();
        let mut treap =
            GenericLazyReversibleTreap::<StringConcatToggleMonoid>::from_slice(seed, &input_values);

        // Act
        let actual_apply_result =
            treap.apply(apply_left_index, apply_right_index, toggle_case_mapping);
        let actual_reverse_result = treap.reverse(reverse_left_index, reverse_right_index);
        let actual_partial_query = treap.query(reverse_left_index, reverse_right_index);
        let actual_full_query = treap.query(full_query_left_index, full_query_right_index);
        let actual_first_value = treap.get(first_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_reverse_result, expected_reverse_result);
        assert_eq!(actual_partial_query, expected_partial_query);
        assert_eq!(actual_full_query, expected_full_query);
        assert_eq!(actual_first_value, expected_first_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn range_add_with_left_greater_than_right_returns_false_and_keeps_state() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64];
        let left_index = 2_usize;
        let right_index = 1_usize;
        let added_value = 5_i64;
        let expected_result = false;
        let expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_result = treap.range_add(left_index, right_index, added_value);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn range_add_on_empty_with_zero_bounds_returns_true_and_keeps_empty() {
        // Arrange
        let initial_values: Vec<i64> = vec![];
        let left_index = 0_usize;
        let right_index = 0_usize;
        let added_value = 10_i64;
        let expected_result = true;
        let expected_values: Vec<i64> = vec![];
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_result = treap.range_add(left_index, right_index, added_value);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn range_add_with_right_beyond_len_returns_false_and_keeps_state() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64];
        let left_index = 0_usize;
        let right_index = 4_usize;
        let added_value = 5_i64;
        let expected_result = false;
        let expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_result = treap.range_add(left_index, right_index, added_value);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn range_add_with_equal_bounds_returns_true_and_keeps_state() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64];
        let left_index = 1_usize;
        let right_index = 1_usize;
        let added_value = 5_i64;
        let expected_result = true;
        let expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_result = treap.range_add(left_index, right_index, added_value);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn range_add_whole_range_with_positive_value_adds_all_elements() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64];
        let left_index = 0_usize;
        let right_index = initial_values.len();
        let added_value = 5_i64;
        let expected_result = true;
        let expected_values = vec![6_i64, 7_i64, 8_i64];
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_result = treap.range_add(left_index, right_index, added_value);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn range_add_whole_range_with_negative_value_subtracts_all_elements() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64];
        let left_index = 0_usize;
        let right_index = initial_values.len();
        let added_value = -2_i64;
        let expected_result = true;
        let expected_values = vec![-1_i64, 0_i64, 1_i64];
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_result = treap.range_add(left_index, right_index, added_value);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn range_add_partial_range_updates_only_target_segment() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
        let left_index = 1_usize;
        let right_index = 4_usize;
        let added_value = 10_i64;
        let expected_result = true;
        let expected_values = vec![1_i64, 12_i64, 13_i64, 14_i64, 5_i64];
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_result = treap.range_add(left_index, right_index, added_value);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn reverse_then_range_add_matches_vec_reference_for_values_and_sum() {
        // Arrange
        let initial_values = vec![8_i64, 6_i64, 7_i64, 5_i64, 3_i64, 0_i64, 9_i64];
        let reverse_left_index = 2_usize;
        let reverse_right_index = 6_usize;
        let add_left_index = 1_usize;
        let add_right_index = 5_usize;
        let added_value = -2_i64;
        let sum_left_index = 0_usize;
        let sum_right_index = initial_values.len();
        let mut expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        let expected_reverse_result = reverse_if_valid(
            &mut expected_values,
            reverse_left_index,
            reverse_right_index,
        );
        let expected_add_result = range_add_if_valid(
            &mut expected_values,
            add_left_index,
            add_right_index,
            added_value,
        );
        let expected_sum = range_sum_if_valid(&expected_values, sum_left_index, sum_right_index);

        // Act
        let actual_reverse_result = treap.reverse(reverse_left_index, reverse_right_index);
        let actual_add_result = treap.range_add(add_left_index, add_right_index, added_value);
        let actual_values = treap.to_vec();
        let actual_sum = treap.range_sum(sum_left_index, sum_right_index);

        // Assert
        assert_eq!(actual_reverse_result, expected_reverse_result);
        assert_eq!(actual_add_result, expected_add_result);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn lazy_action_composition_applies_new_map_after_old_map() {
        // Arrange
        let seed = TEST_SEED;
        let input_values = [1_i64, 2, 3, 4, 5].map(create_affine_node);
        let old_map = AffineMap {
            mul_value: 2,
            add_value: 1,
        };
        let new_map = AffineMap {
            mul_value: 3,
            add_value: 4,
        };
        let apply_left_index = 0_usize;
        let apply_right_index = input_values.len();
        let selected_index = 2_usize;
        let partial_query_left_index = 1_usize;
        let partial_query_right_index = 4_usize;
        let full_query_left_index = 0_usize;
        let full_query_right_index = input_values.len();
        let expected_old_apply_result = true;
        let expected_new_apply_result = true;
        let expected_selected_value = Some(create_affine_node(25));
        let expected_partial_sum = Some(AffineNode {
            range_sum: 75,
            range_len: 3,
        });
        let expected_full_sum = Some(AffineNode {
            range_sum: 125,
            range_len: 5,
        });
        let expected_values = [13_i64, 19, 25, 31, 37].map(create_affine_node).to_vec();
        let mut treap =
            GenericLazyReversibleTreap::<AffineSumMonoid>::from_slice(seed, &input_values);

        // Act
        let actual_old_apply_result = treap.apply(apply_left_index, apply_right_index, old_map);
        let actual_new_apply_result = treap.apply(apply_left_index, apply_right_index, new_map);
        let actual_selected_value = treap.get(selected_index);
        let actual_partial_sum = treap.query(partial_query_left_index, partial_query_right_index);
        let actual_full_sum = treap.query(full_query_left_index, full_query_right_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_old_apply_result, expected_old_apply_result);
        assert_eq!(actual_new_apply_result, expected_new_apply_result);
        assert_eq!(actual_selected_value, expected_selected_value);
        assert_eq!(actual_partial_sum, expected_partial_sum);
        assert_eq!(actual_full_sum, expected_full_sum);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn range_sum_with_left_greater_than_right_returns_none() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64];
        let left_index = 2_usize;
        let right_index = 1_usize;
        let expected_sum = None;
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_sum = treap.range_sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn range_sum_on_empty_with_zero_bounds_returns_some_zero() {
        // Arrange
        let initial_values: Vec<i64> = vec![];
        let left_index = 0_usize;
        let right_index = 0_usize;
        let expected_sum = Some(0_i64);
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_sum = treap.range_sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn range_sum_with_right_beyond_len_returns_none() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64];
        let left_index = 0_usize;
        let right_index = 4_usize;
        let expected_sum = None;
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_sum = treap.range_sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn range_sum_with_equal_bounds_returns_zero() {
        // Arrange
        let initial_values = vec![1_i64, 2_i64, 3_i64];
        let left_index = 1_usize;
        let right_index = 1_usize;
        let expected_sum = Some(0_i64);
        let mut treap = create_treap_with_values(&initial_values);

        // Act
        let actual_sum = treap.range_sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn range_sum_for_boundary_ranges_returns_expected_values() {
        // Arrange
        let initial_values = vec![5_i64, 6_i64, 7_i64];
        let expected_head_sum = Some(5_i64);
        let expected_full_sum = Some(18_i64);
        let expected_tail_sum = Some(7_i64);
        let mut treap = create_treap_with_values(&initial_values);

        let len = initial_values.len();
        let head = (0, 1);
        let full = (0, len);
        let tail = (len - 1, len);

        // Act
        let actual_head_sum = treap.range_sum(head.0, head.1);
        let actual_full_sum = treap.range_sum(full.0, full.1);
        let actual_tail_sum = treap.range_sum(tail.0, tail.1);

        // Assert
        assert_eq!(actual_head_sum, expected_head_sum);
        assert_eq!(actual_full_sum, expected_full_sum);
        assert_eq!(actual_tail_sum, expected_tail_sum);
    }

    #[test]
    fn non_commutative_query_after_partial_reverse_uses_reversed_order() {
        // Arrange
        let seed = TEST_SEED;
        let input_values = ["a", "b", "c", "d", "e"].map(str::to_string);
        let left_index = 1_usize;
        let right_index = 4_usize;
        let full_query_left_index = 0_usize;
        let full_query_right_index = input_values.len();
        let expected_reverse_result = true;
        let expected_full_query = Some("adcbe".to_string());
        let expected_partial_query = Some("dcb".to_string());
        let expected_values = ["a", "d", "c", "b", "e"].map(str::to_string).to_vec();
        let mut treap =
            GenericLazyReversibleTreap::<StringConcatToggleMonoid>::from_slice(seed, &input_values);

        // Act
        let actual_reverse_result = treap.reverse(left_index, right_index);
        let actual_full_query = treap.query(full_query_left_index, full_query_right_index);
        let actual_partial_query = treap.query(left_index, right_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_reverse_result, expected_reverse_result);
        assert_eq!(actual_full_query, expected_full_query);
        assert_eq!(actual_partial_query, expected_partial_query);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn repeated_to_vec_calls_keep_following_operations_consistent() {
        // Arrange
        let initial_values = vec![2_i64, 4_i64, 6_i64];
        let insert_index = 1_usize;
        let inserted_value = 3_i64;
        let remove_index = 3_usize;
        let get_index = 2_usize;
        let sum_left_index = 0_usize;
        let sum_right_index = 3_usize;
        let expected_first_values = initial_values.clone();
        let expected_second_values = initial_values.clone();
        let mut expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        insert_with_clamp(&mut expected_values, insert_index, inserted_value);
        let expected_remove_value = remove_if_exists(&mut expected_values, remove_index);
        let expected_get_value = get_if_exists(&expected_values, get_index);
        let expected_sum = range_sum_if_valid(&expected_values, sum_left_index, sum_right_index);
        let expected_final_values = expected_values.clone();

        // Act
        let actual_first_values = treap.to_vec();
        let actual_second_values = treap.to_vec();
        treap.insert(insert_index, inserted_value);
        let actual_remove_value = treap.remove(remove_index);
        let actual_get_value = treap.get(get_index);
        let actual_sum = treap.range_sum(sum_left_index, sum_right_index);
        let actual_final_values = treap.to_vec();

        // Assert
        assert_eq!(actual_first_values, expected_first_values);
        assert_eq!(actual_second_values, expected_second_values);
        assert_eq!(actual_remove_value, expected_remove_value);
        assert_eq!(actual_get_value, expected_get_value);
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_final_values, expected_final_values);
    }

    #[test]
    fn mixed_operations_sequence_matches_vec_reference_at_each_step() {
        // Arrange
        let initial_values = vec![10_i64, 20_i64, 30_i64, 40_i64];
        let mut expected_values = initial_values.clone();
        let mut treap = create_treap_with_values(&initial_values);

        let first_insert_index = 2_usize;
        let first_inserted_value = 25_i64;
        let first_remove_index = 4_usize;
        let first_add_left_index = 1_usize;
        let first_add_right_index = 4_usize;
        let first_added_value = 3_i64;
        let first_reverse_left_index = 0_usize;
        let first_reverse_right_index = 3_usize;
        let first_get_index = 2_usize;
        let first_sum_left_index = 1_usize;
        let first_sum_right_index = 4_usize;
        let second_insert_index = 10_usize;
        let second_inserted_value = 50_i64;
        let second_remove_index = 0_usize;
        let noop_add_left_index = 2_usize;
        let noop_add_right_index = 2_usize;
        let noop_add_value = 100_i64;
        let noop_reverse_left_index = 1_usize;
        let noop_reverse_right_index = 1_usize;

        insert_with_clamp(
            &mut expected_values,
            first_insert_index,
            first_inserted_value,
        );
        let expected_values_after_first_insert = expected_values.clone();
        let expected_remove_value = remove_if_exists(&mut expected_values, first_remove_index);
        let expected_values_after_first_remove = expected_values.clone();
        let expected_add_result = range_add_if_valid(
            &mut expected_values,
            first_add_left_index,
            first_add_right_index,
            first_added_value,
        );
        let expected_values_after_first_add = expected_values.clone();
        let expected_reverse_result = reverse_if_valid(
            &mut expected_values,
            first_reverse_left_index,
            first_reverse_right_index,
        );
        let expected_values_after_first_reverse = expected_values.clone();
        let expected_get_value = get_if_exists(&expected_values, first_get_index);
        let expected_sum_value = range_sum_if_valid(
            &expected_values,
            first_sum_left_index,
            first_sum_right_index,
        );
        insert_with_clamp(
            &mut expected_values,
            second_insert_index,
            second_inserted_value,
        );
        let expected_values_after_second_insert = expected_values.clone();
        let expected_second_remove_value =
            remove_if_exists(&mut expected_values, second_remove_index);
        let expected_values_after_second_remove = expected_values.clone();
        let expected_noop_add_result = range_add_if_valid(
            &mut expected_values,
            noop_add_left_index,
            noop_add_right_index,
            noop_add_value,
        );
        let expected_noop_reverse_result = reverse_if_valid(
            &mut expected_values,
            noop_reverse_left_index,
            noop_reverse_right_index,
        );
        let expected_final_values = expected_values.clone();

        // Act
        treap.insert(first_insert_index, first_inserted_value);
        let actual_values_after_first_insert = treap.to_vec();
        let actual_remove_value = treap.remove(first_remove_index);
        let actual_values_after_first_remove = treap.to_vec();
        let actual_add_result = treap.range_add(
            first_add_left_index,
            first_add_right_index,
            first_added_value,
        );
        let actual_values_after_first_add = treap.to_vec();
        let actual_reverse_result =
            treap.reverse(first_reverse_left_index, first_reverse_right_index);
        let actual_values_after_first_reverse = treap.to_vec();
        let actual_get_value = treap.get(first_get_index);
        let actual_sum_value = treap.range_sum(first_sum_left_index, first_sum_right_index);
        treap.insert(second_insert_index, second_inserted_value);
        let actual_values_after_second_insert = treap.to_vec();
        let actual_second_remove_value = treap.remove(second_remove_index);
        let actual_values_after_second_remove = treap.to_vec();
        let actual_noop_add_result =
            treap.range_add(noop_add_left_index, noop_add_right_index, noop_add_value);
        let actual_noop_reverse_result =
            treap.reverse(noop_reverse_left_index, noop_reverse_right_index);
        let actual_final_values = treap.to_vec();

        // Assert
        assert_eq!(
            actual_values_after_first_insert,
            expected_values_after_first_insert
        );
        assert_eq!(actual_remove_value, expected_remove_value);
        assert_eq!(
            actual_values_after_first_remove,
            expected_values_after_first_remove
        );
        assert_eq!(actual_add_result, expected_add_result);
        assert_eq!(
            actual_values_after_first_add,
            expected_values_after_first_add
        );
        assert_eq!(actual_reverse_result, expected_reverse_result);
        assert_eq!(
            actual_values_after_first_reverse,
            expected_values_after_first_reverse
        );
        assert_eq!(actual_get_value, expected_get_value);
        assert_eq!(actual_sum_value, expected_sum_value);
        assert_eq!(
            actual_values_after_second_insert,
            expected_values_after_second_insert
        );
        assert_eq!(actual_second_remove_value, expected_second_remove_value);
        assert_eq!(
            actual_values_after_second_remove,
            expected_values_after_second_remove
        );
        assert_eq!(actual_noop_add_result, expected_noop_add_result);
        assert_eq!(actual_noop_reverse_result, expected_noop_reverse_result);
        assert_eq!(actual_final_values, expected_final_values);
    }

    #[test]
    fn generic_empty_and_boundary_ranges_use_identity_and_preserve_state() {
        // Arrange
        let seed = TEST_SEED;
        let inserted_value = "x".to_string();
        let toggle_case_mapping = true;
        let empty_left_index = 0_usize;
        let empty_right_index = 0_usize;
        let reversed_left_index = 1_usize;
        let reversed_right_index = 0_usize;
        let out_of_bounds_left_index = 0_usize;
        let out_of_bounds_right_index = 1_usize;
        let max_bound_left_index = 0_usize;
        let max_bound_right_index = usize::MAX;
        let insert_index = usize::MAX;
        let tail_empty_left_index = 1_usize;
        let tail_empty_right_index = 1_usize;
        let full_query_left_index = 0_usize;
        let full_query_right_index = 1_usize;
        let single_out_of_bounds_left_index = 0_usize;
        let single_out_of_bounds_right_index = 2_usize;
        let expected_success_result = true;
        let expected_failure_result = false;
        let expected_identity = Some(String::new());
        let expected_tail_identity = Some(String::new());
        let expected_invalid_query: Option<String> = None;
        let expected_full_query = Some(inserted_value.clone());
        let expected_values = vec![inserted_value.clone()];
        let mut treap = GenericLazyReversibleTreap::<StringConcatToggleMonoid>::new(seed);

        // Act
        let actual_empty_query = treap.query(empty_left_index, empty_right_index);
        let actual_empty_apply_result =
            treap.apply(empty_left_index, empty_right_index, toggle_case_mapping);
        let actual_empty_reverse_result = treap.reverse(empty_left_index, empty_right_index);
        let actual_reversed_bounds_query = treap.query(reversed_left_index, reversed_right_index);
        let actual_out_of_bounds_query =
            treap.query(out_of_bounds_left_index, out_of_bounds_right_index);
        let actual_max_bound_query = treap.query(max_bound_left_index, max_bound_right_index);
        let actual_out_of_bounds_apply_result = treap.apply(
            out_of_bounds_left_index,
            out_of_bounds_right_index,
            toggle_case_mapping,
        );
        let actual_max_bound_apply_result = treap.apply(
            max_bound_left_index,
            max_bound_right_index,
            toggle_case_mapping,
        );
        let actual_out_of_bounds_reverse_result =
            treap.reverse(out_of_bounds_left_index, out_of_bounds_right_index);
        treap.insert(insert_index, inserted_value);
        let actual_tail_empty_query = treap.query(tail_empty_left_index, tail_empty_right_index);
        let actual_tail_empty_apply_result = treap.apply(
            tail_empty_left_index,
            tail_empty_right_index,
            toggle_case_mapping,
        );
        let actual_tail_empty_reverse_result =
            treap.reverse(tail_empty_left_index, tail_empty_right_index);
        let actual_full_query = treap.query(full_query_left_index, full_query_right_index);
        let actual_single_out_of_bounds_query = treap.query(
            single_out_of_bounds_left_index,
            single_out_of_bounds_right_index,
        );
        let actual_single_reversed_bounds_query =
            treap.query(reversed_left_index, reversed_right_index);
        let actual_values = treap.to_vec();

        // Assert
        assert_eq!(actual_empty_query, expected_identity);
        assert_eq!(actual_empty_apply_result, expected_success_result);
        assert_eq!(actual_empty_reverse_result, expected_success_result);
        assert_eq!(actual_reversed_bounds_query, expected_invalid_query);
        assert_eq!(actual_out_of_bounds_query, expected_invalid_query);
        assert_eq!(actual_max_bound_query, expected_invalid_query);
        assert_eq!(actual_out_of_bounds_apply_result, expected_failure_result);
        assert_eq!(actual_max_bound_apply_result, expected_failure_result);
        assert_eq!(actual_out_of_bounds_reverse_result, expected_failure_result);
        assert_eq!(actual_tail_empty_query, expected_tail_identity);
        assert_eq!(actual_tail_empty_apply_result, expected_success_result);
        assert_eq!(actual_tail_empty_reverse_result, expected_success_result);
        assert_eq!(actual_full_query, expected_full_query);
        assert_eq!(actual_single_out_of_bounds_query, expected_invalid_query);
        assert_eq!(actual_single_reversed_bounds_query, expected_invalid_query);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn randomized_mixed_operations_match_vec_oracle_across_priority_seeds() {
        // Arrange
        let treap_seeds = [0_u64, 1, 2, TEST_SEED, 0xD1B5_4A32_D192_ED03, u64::MAX];
        let expected_result: Result<(), String> = Ok(());

        // Act
        let actual_results: Vec<_> = treap_seeds
            .iter()
            .map(|treap_seed| {
                (
                    *treap_seed,
                    run_randomized_vec_oracle(*treap_seed, RANDOMIZED_OPERATION_COUNT),
                )
            })
            .collect();

        // Assert
        for (treap_seed, actual_result) in actual_results {
            assert_eq!(actual_result, expected_result, "treap seed {treap_seed}");
        }
    }
}

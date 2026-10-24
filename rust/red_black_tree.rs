#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Red,
    Black,
}

#[cfg(test)]
mod test_operation_counters {
    use std::cell::Cell;

    thread_local! {
        static FIND_NODE_CALL_COUNT: Cell<usize> = const { Cell::new(0) };
    }

    pub fn reset_find_node_call_count() {
        FIND_NODE_CALL_COUNT.set(0);
    }

    pub fn increment_find_node_call_count() {
        FIND_NODE_CALL_COUNT.with(|count| count.set(count.get() + 1));
    }

    pub fn load_find_node_call_count() -> usize {
        FIND_NODE_CALL_COUNT.get()
    }
}

#[derive(Debug)]
struct Node<T> {
    key: T,
    count: usize,
    size: usize,
    color: Color,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    fn new(key: T, color: Color) -> Box<Self> {
        Box::new(Self {
            key,
            count: 1,
            size: 1,
            color,
            left: None,
            right: None,
        })
    }
}

fn size_of<T>(node: &Option<Box<Node<T>>>) -> usize {
    node.as_ref().map_or(0, |n| n.size)
}

fn update_size<T>(node: &mut Box<Node<T>>) {
    node.size = node.count + size_of(&node.left) + size_of(&node.right);
}

fn rotate_left<T>(mut root: Box<Node<T>>) -> Box<Node<T>> {
    let mut child = root.right.take().expect("rotate_left requires right child");
    let root_color = root.color;
    root.right = child.left.take();
    root.color = Color::Red;
    update_size(&mut root);
    child.left = Some(root);
    child.color = root_color;
    update_size(&mut child);
    child
}

fn rotate_right<T>(mut root: Box<Node<T>>) -> Box<Node<T>> {
    let mut child = root.left.take().expect("rotate_right requires left child");
    let root_color = root.color;
    root.left = child.right.take();
    root.color = Color::Red;
    update_size(&mut root);
    child.right = Some(root);
    child.color = root_color;
    update_size(&mut child);
    child
}

fn is_red<T>(node: &Option<Box<Node<T>>>) -> bool {
    matches!(node, Some(n) if n.color == Color::Red)
}

fn flip_colors<T>(root: &mut Box<Node<T>>) {
    root.color = match root.color {
        Color::Red => Color::Black,
        Color::Black => Color::Red,
    };
    if let Some(left) = root.left.as_mut() {
        left.color = match left.color {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        };
    }
    if let Some(right) = root.right.as_mut() {
        right.color = match right.color {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        };
    }
}

fn fix_up<T>(mut root: Box<Node<T>>) -> Box<Node<T>> {
    if is_red(&root.right) && !is_red(&root.left) {
        root = rotate_left(root);
    }
    if is_red(&root.left) && is_red(&root.left.as_ref().unwrap().left) {
        root = rotate_right(root);
    }
    if is_red(&root.left) && is_red(&root.right) {
        flip_colors(&mut root);
    }
    update_size(&mut root);
    root
}

fn move_red_left<T>(mut root: Box<Node<T>>) -> Box<Node<T>> {
    flip_colors(&mut root);
    let should_rotate = root
        .right
        .as_ref()
        .map(|right| is_red(&right.left))
        .unwrap_or(false);
    if should_rotate {
        if let Some(right) = root.right.take() {
            root.right = Some(rotate_right(right));
        }
        root = rotate_left(root);
        flip_colors(&mut root);
    }
    root
}

fn move_red_right<T>(mut root: Box<Node<T>>) -> Box<Node<T>> {
    flip_colors(&mut root);
    let should_rotate = root
        .left
        .as_ref()
        .map(|left| is_red(&left.left))
        .unwrap_or(false);
    if should_rotate {
        root = rotate_right(root);
        flip_colors(&mut root);
    }
    root
}

fn take_min_node<T>(mut root: Box<Node<T>>) -> (Option<Box<Node<T>>>, Box<Node<T>>) {
    if root.left.is_none() {
        let right = root.right.take();
        return (right, root);
    }
    if !is_red(&root.left) && !is_red(&root.left.as_ref().unwrap().left) {
        root = move_red_left(root);
    }
    let left = root
        .left
        .take()
        .expect("take_min_node requires left child when descending");
    let (next_left, minimum_node) = take_min_node(left);
    root.left = next_left;
    (Some(fix_up(root)), minimum_node)
}

fn delete_min_one<T: Clone>(mut root: Box<Node<T>>) -> (Option<Box<Node<T>>>, T) {
    if root.left.is_none() {
        if root.count > 1 {
            let deleted_key = root.key.clone();
            root.count -= 1;
            update_size(&mut root);
            return (Some(root), deleted_key);
        }
        let deleted_key = root.key;
        return (root.right.take(), deleted_key);
    }

    if !is_red(&root.left) && !is_red(&root.left.as_ref().unwrap().left) {
        root = move_red_left(root);
    }
    let left = root
        .left
        .take()
        .expect("delete_min_one requires left child when descending");
    let (next_left, deleted_key) = delete_min_one(left);
    root.left = next_left;
    (Some(fix_up(root)), deleted_key)
}

fn delete_max_one<T: Clone>(mut root: Box<Node<T>>) -> (Option<Box<Node<T>>>, T) {
    if is_red(&root.left) {
        root = rotate_right(root);
    }
    if root.right.is_none() {
        if root.count > 1 {
            let deleted_key = root.key.clone();
            root.count -= 1;
            update_size(&mut root);
            return (Some(root), deleted_key);
        }
        let deleted_key = root.key;
        return (root.left.take(), deleted_key);
    }

    if !is_red(&root.right) && !is_red(&root.right.as_ref().unwrap().left) {
        root = move_red_right(root);
    }
    let right = root
        .right
        .take()
        .expect("delete_max_one requires right child when descending");
    let (next_right, deleted_key) = delete_max_one(right);
    root.right = next_right;
    (Some(fix_up(root)), deleted_key)
}

fn find_node<'a, T: Ord>(node: &'a Option<Box<Node<T>>>, key: &T) -> Option<&'a Node<T>> {
    #[cfg(test)]
    test_operation_counters::increment_find_node_call_count();

    let mut current = node.as_deref();
    while let Some(current_node) = current {
        if key < &current_node.key {
            current = current_node.left.as_deref();
        } else if &current_node.key < key {
            current = current_node.right.as_deref();
        } else {
            return Some(current_node);
        }
    }
    None
}

fn insert_node<T: Ord>(root: Option<Box<Node<T>>>, key: T) -> Box<Node<T>> {
    match root {
        None => Node::new(key, Color::Red),
        Some(mut node) => {
            if key < node.key {
                node.left = Some(insert_node(node.left.take(), key));
            } else if key > node.key {
                node.right = Some(insert_node(node.right.take(), key));
            } else {
                node.count += 1;
            }
            fix_up(node)
        }
    }
}

struct DeleteNodeResult<T> {
    updated_root: Option<Box<Node<T>>>,
    deleted: bool,
}

fn delete_node<T: Ord>(root: Option<Box<Node<T>>>, key: &T) -> DeleteNodeResult<T> {
    let Some(mut node) = root else {
        return DeleteNodeResult {
            updated_root: None,
            deleted: false,
        };
    };

    let deleted;
    if key < &node.key {
        if node.left.is_some() {
            if !is_red(&node.left) && !is_red(&node.left.as_ref().unwrap().left) {
                node = move_red_left(node);
            }
            let delete_left_result = delete_node(node.left.take(), key);
            node.left = delete_left_result.updated_root;
            deleted = delete_left_result.deleted;
        } else {
            deleted = false;
        }
    } else {
        if is_red(&node.left) {
            node = rotate_right(node);
        }
        if key == &node.key && node.right.is_none() {
            if node.count > 1 {
                node.count -= 1;
                return DeleteNodeResult {
                    updated_root: Some(fix_up(node)),
                    deleted: true,
                };
            }
            return DeleteNodeResult {
                updated_root: node.left.take(),
                deleted: true,
            };
        }
        if node.right.is_some() {
            if !is_red(&node.right) && !is_red(&node.right.as_ref().unwrap().left) {
                node = move_red_right(node);
            }
            if key == &node.key {
                if node.count > 1 {
                    node.count -= 1;
                    return DeleteNodeResult {
                        updated_root: Some(fix_up(node)),
                        deleted: true,
                    };
                }
                let right = node
                    .right
                    .take()
                    .expect("delete_node requires right child when replacing node");
                let (next_right, minimum_node) = take_min_node(right);
                node.key = minimum_node.key;
                node.count = minimum_node.count;
                node.right = next_right;
                deleted = true;
            } else {
                let delete_right_result = delete_node(node.right.take(), key);
                node.right = delete_right_result.updated_root;
                deleted = delete_right_result.deleted;
            }
        } else {
            deleted = false;
        }
    }
    DeleteNodeResult {
        updated_root: Some(fix_up(node)),
        deleted,
    }
}

fn count_less_than_or_equal<T: Ord>(node: &Option<Box<Node<T>>>, key: &T) -> usize {
    let mut result = 0usize;
    let mut current = node.as_deref();

    while let Some(current_node) = current {
        if key < &current_node.key {
            current = current_node.left.as_deref();
        } else {
            result += size_of(&current_node.left) + current_node.count;
            if key == &current_node.key {
                return result;
            }
            current = current_node.right.as_deref();
        }
    }

    result
}

fn kth_min<T>(node: &Option<Box<Node<T>>>, mut index: usize) -> Option<&T> {
    let mut current = node.as_ref()?;
    loop {
        let left_size = size_of(&current.left);
        if index < left_size {
            current = current.left.as_ref()?;
        } else if index < left_size + current.count {
            return Some(&current.key);
        } else {
            index -= left_size + current.count;
            current = current.right.as_ref()?;
        }
    }
}

fn first_key<T>(node: &Option<Box<Node<T>>>) -> Option<&T> {
    let mut current = node.as_deref()?;
    while let Some(left) = current.left.as_deref() {
        current = left;
    }
    Some(&current.key)
}

fn last_key<T>(node: &Option<Box<Node<T>>>) -> Option<&T> {
    let mut current = node.as_deref()?;
    while let Some(right) = current.right.as_deref() {
        current = right;
    }
    Some(&current.key)
}

#[derive(Debug)]
pub struct RedBlackTree<T: Ord> {
    root: Option<Box<Node<T>>>,
}

impl<T: Ord> Default for RedBlackTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> RedBlackTree<T> {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn len(&self) -> usize {
        size_of(&self.root)
    }

    pub fn insert(&mut self, key: T) {
        let root = self.root.take();
        let mut root = insert_node(root, key);
        root.color = Color::Black;
        self.root = Some(root);
    }

    pub fn remove(&mut self, key: &T) -> bool {
        let Some(mut root) = self.root.take() else {
            return false;
        };
        if !is_red(&root.left) && !is_red(&root.right) {
            root.color = Color::Red;
        }
        let delete_result = delete_node(Some(root), key);
        if let Some(mut root) = delete_result.updated_root {
            root.color = Color::Black;
            self.root = Some(root);
        } else {
            self.root = None;
        }
        delete_result.deleted
    }

    pub fn count(&self, key: &T) -> usize {
        find_node(&self.root, key).map_or(0, |node| node.count)
    }

    pub fn count_less_than_or_equal(&self, key: &T) -> usize {
        count_less_than_or_equal(&self.root, key)
    }

    pub fn contains(&self, key: &T) -> bool {
        find_node(&self.root, key).is_some()
    }

    pub fn first(&self) -> Option<&T> {
        first_key(&self.root)
    }

    pub fn last(&self) -> Option<&T> {
        last_key(&self.root)
    }

    pub fn pop_first(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let mut root = self.root.take()?;
        if !is_red(&root.left) && !is_red(&root.right) {
            root.color = Color::Red;
        }
        let (updated_root, deleted_key) = delete_min_one(root);
        self.root = updated_root.map(|mut root| {
            root.color = Color::Black;
            root
        });
        Some(deleted_key)
    }

    pub fn pop_last(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let mut root = self.root.take()?;
        if !is_red(&root.left) && !is_red(&root.right) {
            root.color = Color::Red;
        }
        let (updated_root, deleted_key) = delete_max_one(root);
        self.root = updated_root.map(|mut root| {
            root.color = Color::Black;
            root
        });
        Some(deleted_key)
    }

    pub fn kth_min(&self, index: usize) -> Option<&T> {
        if self.len() <= index {
            None
        } else {
            kth_min(&self.root, index)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_operation_counters::{load_find_node_call_count, reset_find_node_call_count};
    use super::{Color, Node, RedBlackTree};
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
    struct NonCloneKey(i32);

    fn collect_kth_values(tree: &RedBlackTree<i64>) -> Vec<i64> {
        (0..tree.len())
            .map(|index| *tree.kth_min(index).expect("index in range"))
            .collect()
    }

    fn is_non_decreasing(values: &[i64]) -> bool {
        values.windows(2).all(|pair| pair[0] <= pair[1])
    }

    fn btree_total_count(reference_counts: &BTreeMap<i64, usize>) -> usize {
        reference_counts.values().sum()
    }

    fn btree_count_less_than_or_equal(reference_counts: &BTreeMap<i64, usize>, key: i64) -> usize {
        reference_counts.range(..=key).map(|(_, count)| count).sum()
    }

    fn btree_kth_min(reference_counts: &BTreeMap<i64, usize>, index: usize) -> Option<i64> {
        let mut traversed_count = 0usize;
        for (&key, &count) in reference_counts {
            if index < traversed_count + count {
                return Some(key);
            }
            traversed_count += count;
        }
        None
    }

    fn btree_remove(reference_counts: &mut BTreeMap<i64, usize>, key: i64) -> bool {
        let Some(count) = reference_counts.get_mut(&key) else {
            return false;
        };
        *count -= 1;
        if *count == 0 {
            reference_counts.remove(&key);
        }
        true
    }

    fn btree_pop_first(reference_counts: &mut BTreeMap<i64, usize>) -> Option<i64> {
        let key = *reference_counts.first_key_value()?.0;
        btree_remove(reference_counts, key);
        Some(key)
    }

    fn btree_pop_last(reference_counts: &mut BTreeMap<i64, usize>) -> Option<i64> {
        let key = *reference_counts.last_key_value()?.0;
        btree_remove(reference_counts, key);
        Some(key)
    }

    fn node_invariant_diagnostics(
        node: &Option<Box<Node<i64>>>,
        lower: Option<i64>,
        upper: Option<i64>,
    ) -> Result<(usize, usize), String> {
        let Some(node) = node else {
            return Ok((1, 0));
        };
        if node.count == 0
            || lower.is_some_and(|key| node.key <= key)
            || upper.is_some_and(|key| key <= node.key)
        {
            return Err(format!("key {} has invalid count or ordering", node.key));
        }
        let left_black = node
            .left
            .as_ref()
            .is_none_or(|left| left.color == Color::Black);
        let right_black = node
            .right
            .as_ref()
            .is_none_or(|right| right.color == Color::Black);
        if !right_black || (node.color == Color::Red && !left_black) {
            return Err(format!("key {} has invalid red links", node.key));
        }
        let (left_height, left_size) =
            node_invariant_diagnostics(&node.left, lower, Some(node.key))?;
        let (right_height, right_size) =
            node_invariant_diagnostics(&node.right, Some(node.key), upper)?;
        if left_height != right_height {
            return Err(format!("key {} has unequal black heights", node.key));
        }
        let size = node.count + left_size + right_size;
        if node.size != size {
            return Err(format!(
                "key {} size {} differs from {}",
                node.key, node.size, size
            ));
        }
        Ok((left_height + usize::from(node.color == Color::Black), size))
    }

    fn invariant_diagnostics(tree: &RedBlackTree<i64>) -> Result<(), String> {
        if tree
            .root
            .as_ref()
            .is_some_and(|root| root.color != Color::Black)
        {
            return Err("root is red".to_string());
        }
        let (_, size) = node_invariant_diagnostics(&tree.root, None, None)?;
        if tree.len() != size {
            return Err(format!("tree len {} differs from {}", tree.len(), size));
        }
        Ok(())
    }

    fn assert_red_black_tree_invariants(tree: &RedBlackTree<i64>) {
        assert_eq!(invariant_diagnostics(tree), Ok(()));
    }

    #[derive(Clone, Copy)]
    enum Operation { Insert(i64), Remove(i64), PopFirst, PopLast }

    #[derive(Debug, PartialEq)]
    enum OperationResult { Inserted, Removed(bool), Popped(Option<i64>) }

    #[derive(Debug, PartialEq)]
    struct StateSnapshot {
        len: usize,
        is_empty: bool,
        first: Option<i64>,
        last: Option<i64>,
        queries: Vec<(bool, usize, usize)>,
        ranks: Vec<Option<i64>>,
        invariants: Result<(), String>,
    }

    fn reference_sequence(
        initial: &[i64],
        operations: &[Operation],
        queries: &[i64],
    ) -> Vec<(OperationResult, StateSnapshot)> {
        let mut counts = BTreeMap::<i64, usize>::new();
        for &key in initial {
            *counts.entry(key).or_default() += 1;
        }
        operations
            .iter()
            .map(|&operation| {
                let result = match operation {
                    Operation::Insert(key) => {
                        *counts.entry(key).or_default() += 1;
                        OperationResult::Inserted
                    }
                    Operation::Remove(key) => {
                        OperationResult::Removed(btree_remove(&mut counts, key))
                    }
                    Operation::PopFirst => OperationResult::Popped(btree_pop_first(&mut counts)),
                    Operation::PopLast => OperationResult::Popped(btree_pop_last(&mut counts)),
                };
                let len = btree_total_count(&counts);
                let snapshot = StateSnapshot {
                    len,
                    is_empty: counts.is_empty(),
                    first: counts.first_key_value().map(|(&key, _)| key),
                    last: counts.last_key_value().map(|(&key, _)| key),
                    queries: queries
                        .iter()
                        .map(|key| {
                            (
                                counts.contains_key(key),
                                counts.get(key).copied().unwrap_or(0),
                                btree_count_less_than_or_equal(&counts, *key),
                            )
                        })
                        .collect(),
                    ranks: (0..=len).map(|rank| btree_kth_min(&counts, rank)).collect(),
                    invariants: Ok(()),
                };
                (result, snapshot)
            })
            .collect()
    }

    fn actual_sequence(
        tree: &mut RedBlackTree<i64>,
        operations: &[Operation],
        queries: &[i64],
    ) -> Vec<(OperationResult, StateSnapshot)> {
        operations
            .iter()
            .map(|&operation| {
                let result = match operation {
                    Operation::Insert(key) => {
                        tree.insert(key);
                        OperationResult::Inserted
                    }
                    Operation::Remove(key) => OperationResult::Removed(tree.remove(&key)),
                    Operation::PopFirst => OperationResult::Popped(tree.pop_first()),
                    Operation::PopLast => OperationResult::Popped(tree.pop_last()),
                };
                let len = tree.len();
                let snapshot = StateSnapshot {
                    len,
                    is_empty: tree.is_empty(),
                    first: tree.first().copied(),
                    last: tree.last().copied(),
                    queries: queries
                        .iter()
                        .map(|key| {
                            (
                                tree.contains(key),
                                tree.count(key),
                                tree.count_less_than_or_equal(key),
                            )
                        })
                        .collect(),
                    ranks: (0..=len).map(|rank| tree.kth_min(rank).copied()).collect(),
                    invariants: invariant_diagnostics(tree),
                };
                (result, snapshot)
            })
            .collect()
    }

    fn next_random(random_state: &mut u64) -> u64 {
        *random_state = random_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *random_state
    }

    fn build_ascending_values(value_count: usize) -> Vec<i64> {
        (0..value_count).map(|index| index as i64).collect()
    }

    fn build_descending_values(value_count: usize) -> Vec<i64> {
        (0..value_count).rev().map(|index| index as i64).collect()
    }

    fn build_duplicate_heavy_values(value_count: usize) -> Vec<i64> {
        (0..value_count)
            .map(|index| (index % 64) as i64 - 32)
            .collect()
    }

    fn build_mixed_operations(operation_count: usize) -> Vec<(bool, i64)> {
        let mut random_state = 20260314u64;
        let mut operations = Vec::with_capacity(operation_count);
        for _ in 0..operation_count {
            let operation_selector = (next_random(&mut random_state) >> 32) % 2;
            let key = (next_random(&mut random_state) % 257) as i64 - 128;
            operations.push((operation_selector == 0, key));
        }
        operations
    }

    fn measure_insert_benchmark_duration(
        inserted_values: &[i64],
        repetition_count: usize,
    ) -> Duration {
        let start = Instant::now();
        for _ in 0..repetition_count {
            let mut tree = RedBlackTree::<i64>::new();
            for &value in inserted_values {
                tree.insert(black_box(value));
            }
            black_box(tree.len());
        }
        start.elapsed()
    }

    fn measure_mixed_operations_benchmark_duration(
        operations: &[(bool, i64)],
        repetition_count: usize,
    ) -> Duration {
        let start = Instant::now();
        for _ in 0..repetition_count {
            let mut tree = RedBlackTree::<i64>::new();
            for &(is_insert, key) in operations {
                if is_insert {
                    tree.insert(black_box(key));
                } else {
                    black_box(tree.remove(&black_box(key)));
                }
            }
            black_box(tree.len());
        }
        start.elapsed()
    }

    fn build_tree(inserted_values: &[i64]) -> RedBlackTree<i64> {
        let mut tree = RedBlackTree::<i64>::new();
        for &value in inserted_values {
            tree.insert(value);
        }
        tree
    }

    fn measure_first_benchmark_duration(
        tree: &RedBlackTree<i64>,
        operation_count: usize,
    ) -> Duration {
        let start = Instant::now();
        for _ in 0..operation_count {
            black_box(black_box(tree).first());
        }
        start.elapsed()
    }

    fn measure_last_benchmark_duration(
        tree: &RedBlackTree<i64>,
        operation_count: usize,
    ) -> Duration {
        let start = Instant::now();
        for _ in 0..operation_count {
            black_box(black_box(tree).last());
        }
        start.elapsed()
    }

    fn measure_pop_first_benchmark_duration(
        inserted_values: &[i64],
        repetition_count: usize,
    ) -> Duration {
        let mut trees = (0..repetition_count)
            .map(|_| build_tree(inserted_values))
            .collect::<Vec<_>>();
        let start = Instant::now();
        for tree in &mut trees {
            while !tree.is_empty() {
                black_box(tree.pop_first());
            }
        }
        start.elapsed()
    }

    fn measure_pop_last_benchmark_duration(
        inserted_values: &[i64],
        repetition_count: usize,
    ) -> Duration {
        let mut trees = (0..repetition_count)
            .map(|_| build_tree(inserted_values))
            .collect::<Vec<_>>();
        let start = Instant::now();
        for tree in &mut trees {
            while !tree.is_empty() {
                black_box(tree.pop_last());
            }
        }
        start.elapsed()
    }

    #[test]
    fn default_tree_matches_new_tree_initial_state() {
        // Arrange
        let tree = RedBlackTree::<i64>::new();
        let default_tree = RedBlackTree::<i64>::default();

        let expected_tree_len_right = 0;
        let expected_default_tree_len_right = 0;

        // Act
        let tree_is_empty = tree.is_empty();
        let tree_len = tree.len();
        let default_tree_is_empty = default_tree.is_empty();
        let default_tree_len = default_tree.len();

        // Assert
        assert!(tree_is_empty);
        assert_eq!(tree_len, expected_tree_len_right);
        assert!(default_tree_is_empty);
        assert_eq!(default_tree_len, expected_default_tree_len_right);
    }

    #[test]
    fn is_empty_becomes_false_after_single_insert() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let inserted_key = 42i64;

        // Act
        tree.insert(inserted_key);
        let is_empty_after_insert = tree.is_empty();

        // Assert
        assert!(!is_empty_after_insert);
    }

    #[test]
    fn is_empty_returns_true_after_erasing_last_element() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let inserted_key = 7i64;
        tree.insert(inserted_key);

        // Act
        let erase_result = tree.remove(&inserted_key);
        let is_empty_after_erase = tree.is_empty();

        // Assert
        assert!(erase_result);
        assert!(is_empty_after_erase);
    }

    #[test]
    fn ord_key_without_clone_supports_tree_operations() {
        // Arrange
        let values = [NonCloneKey(2), NonCloneKey(1), NonCloneKey(3)];
        let query_key = NonCloneKey(3);
        let removed_key = NonCloneKey(2);
        let rank = 1;
        let expected_first = Some(&NonCloneKey(1));
        let expected_last = Some(&NonCloneKey(3));
        let expected_count = 1;
        let expected_lower_count = 2;
        let mut tree = RedBlackTree::new();
        for value in values {
            tree.insert(value);
        }

        // Act
        let actual_contains = tree.contains(&query_key);
        let actual_count = tree.count(&removed_key);
        let actual_lower_count = tree.count_less_than_or_equal(&removed_key);
        let actual_removed = tree.remove(&removed_key);
        let actual_first = tree.first();
        let actual_last = tree.last();
        let actual_rank = tree.kth_min(rank);

        // Assert
        assert!(actual_contains);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_lower_count, expected_lower_count);
        assert!(actual_removed);
        assert_eq!(actual_first, expected_first);
        assert_eq!(actual_last, expected_last);
        assert_eq!(actual_rank, expected_last);
    }

    #[test]
    fn string_keys_support_duplicate_pop_operations() {
        // Arrange
        let values = [
            String::from("beta"),
            String::from("alpha"),
            String::from("alpha"),
        ];
        let query_key = String::from("alpha");
        let expected_popped = Some(query_key.clone());
        let expected_count = 1;
        let expected_first = Some(query_key.as_str());
        let mut tree = RedBlackTree::new();
        for value in values {
            tree.insert(value);
        }

        // Act
        let actual_popped = tree.pop_first();
        let actual_count = tree.count(&query_key);
        let actual_first = tree.first().map(String::as_str);

        // Assert
        assert_eq!(actual_popped, expected_popped);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_first, expected_first);
    }

    #[test]
    fn insert_boundary_values_x_minus_1_x_x_plus_1_sets_each_count_to_one() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let center_value = 10i64;
        let lower_value = center_value - 1;
        let upper_value = center_value + 1;

        let expected_lower_count_right = 1;
        let expected_center_count_right = 1;
        let expected_upper_count_right = 1;

        // Act
        tree.insert(lower_value);
        tree.insert(center_value);
        tree.insert(upper_value);
        let lower_count = tree.count(&lower_value);
        let center_count = tree.count(&center_value);
        let upper_count = tree.count(&upper_value);

        // Assert
        assert_eq!(lower_count, expected_lower_count_right);
        assert_eq!(center_count, expected_center_count_right);
        assert_eq!(upper_count, expected_upper_count_right);
    }

    #[test]
    fn insert_same_key_three_times_increases_count_and_len() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let duplicated_key = 3i64;

        let expected_duplicated_count_right = 3;
        let expected_tree_len_right = 3;

        // Act
        tree.insert(duplicated_key);
        tree.insert(duplicated_key);
        tree.insert(duplicated_key);
        let duplicated_count = tree.count(&duplicated_key);
        let tree_len = tree.len();

        // Assert
        assert_eq!(duplicated_count, expected_duplicated_count_right);
        assert_eq!(tree_len, expected_tree_len_right);
    }

    #[test]
    fn insert_i64_min_and_max_allows_count_and_contains_queries() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let min_key = i64::MIN;
        let max_key = i64::MAX;

        let expected_min_count_right = 1;
        let expected_max_count_right = 1;

        // Act
        tree.insert(min_key);
        tree.insert(max_key);
        let min_contains = tree.contains(&min_key);
        let max_contains = tree.contains(&max_key);
        let min_count = tree.count(&min_key);
        let max_count = tree.count(&max_key);

        // Assert
        assert!(min_contains);
        assert!(max_contains);
        assert_eq!(min_count, expected_min_count_right);
        assert_eq!(max_count, expected_max_count_right);
    }

    #[test]
    fn ascending_insertions_preserve_red_black_tree_invariants() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let inserted_values = build_ascending_values(128);

        // Act
        for value in inserted_values {
            tree.insert(value);
        }

        // Assert
        assert_red_black_tree_invariants(&tree);
    }

    #[test]
    fn descending_insertions_preserve_invariants_after_every_insert() {
        // Arrange
        let value_count = 128;
        let initial_values = Vec::<i64>::new();
        let operations = build_descending_values(value_count)
            .into_iter()
            .map(Operation::Insert)
            .collect::<Vec<_>>();
        let query_keys = build_ascending_values(value_count);
        let expected_steps = reference_sequence(&initial_values, &operations, &query_keys);
        let mut tree = build_tree(&initial_values);

        // Act
        let actual_steps = actual_sequence(&mut tree, &operations, &query_keys);

        // Assert
        assert_eq!(actual_steps, expected_steps);
    }

    #[test]
    fn ascending_removals_match_btreemap_and_preserve_invariants() {
        // Arrange
        let value_count = 128;
        let initial_values = build_ascending_values(value_count);
        let operations = build_ascending_values(value_count)
            .into_iter()
            .map(Operation::Remove)
            .collect::<Vec<_>>();
        let query_keys = build_ascending_values(value_count);
        let expected_steps = reference_sequence(&initial_values, &operations, &query_keys);
        let mut tree = build_tree(&initial_values);

        // Act
        let actual_steps = actual_sequence(&mut tree, &operations, &query_keys);

        // Assert
        assert_eq!(actual_steps, expected_steps);
    }

    #[test]
    fn descending_removals_match_btreemap_and_preserve_invariants() {
        // Arrange
        let value_count = 128;
        let initial_values = build_ascending_values(value_count);
        let operations = build_descending_values(value_count)
            .into_iter()
            .map(Operation::Remove)
            .collect::<Vec<_>>();
        let query_keys = build_ascending_values(value_count);
        let expected_steps = reference_sequence(&initial_values, &operations, &query_keys);
        let mut tree = build_tree(&initial_values);

        // Act
        let actual_steps = actual_sequence(&mut tree, &operations, &query_keys);

        // Assert
        assert_eq!(actual_steps, expected_steps);
    }

    #[test]
    fn remove_returns_false_for_empty_tree() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let missing_key = 100i64;

        // Act
        let erase_result = tree.remove(&missing_key);

        // Assert
        assert!(!erase_result);
    }

    #[test]
    fn remove_missing_key_returns_false_and_keeps_len() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let inserted_key = 1i64;
        let missing_key = 2i64;
        tree.insert(inserted_key);

        let expected_len_after_erase_right = 1;

        // Act
        let erase_result = tree.remove(&missing_key);
        let len_after_erase = tree.len();

        // Assert
        assert!(!erase_result);
        assert_eq!(len_after_erase, expected_len_after_erase_right);
    }

    #[test]
    fn remove_reduces_count_by_one_for_duplicated_key() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let duplicated_key = 2i64;
        tree.insert(duplicated_key);
        tree.insert(duplicated_key);

        let expected_count_after_erase_right = 1;

        // Act
        let erase_result = tree.remove(&duplicated_key);
        let count_after_erase = tree.count(&duplicated_key);

        // Assert
        assert!(erase_result);
        assert_eq!(count_after_erase, expected_count_after_erase_right);
    }

    #[test]
    fn remove_removes_key_when_last_occurrence_is_deleted() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let unique_key = 8i64;
        tree.insert(unique_key);

        // Act
        let erase_result = tree.remove(&unique_key);
        let contains_after_erase = tree.contains(&unique_key);

        // Assert
        assert!(erase_result);
        assert!(!contains_after_erase);
    }

    #[test]
    fn remove_removing_minimum_updates_first_kth_value() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [1i64, 2, 3];
        for value in values {
            tree.insert(value);
        }
        let minimum_key = 1i64;

        let expected_first_value_after_erase_right = Some(&2);

        let kth_min_rank = 0;

        // Act
        let erase_result = tree.remove(&minimum_key);
        let first_value_after_erase = tree.kth_min(kth_min_rank);

        // Assert
        assert!(erase_result);
        assert_eq!(
            first_value_after_erase,
            expected_first_value_after_erase_right
        );
    }

    #[test]
    fn remove_removing_maximum_updates_last_kth_value() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [1i64, 2, 3];
        for value in values {
            tree.insert(value);
        }
        let maximum_key = 3i64;

        let expected_len_after_erase_right = 2;
        let expected_last_value_after_erase_right = Some(&2);

        // Act
        let erase_result = tree.remove(&maximum_key);
        let len_after_erase = tree.len();
        let last_value_after_erase = if len_after_erase == 0 {
            None
        } else {
            tree.kth_min(len_after_erase - 1)
        };

        // Assert
        assert!(erase_result);
        assert_eq!(len_after_erase, expected_len_after_erase_right);
        assert_eq!(
            last_value_after_erase,
            expected_last_value_after_erase_right
        );
    }

    #[test]
    fn remove_for_node_with_two_children_keeps_total_order() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [10i64, 5, 15, 12, 18];
        for value in values {
            tree.insert(value);
        }
        let erased_key = 10i64;

        let expected_kth_values_after_erase_right = vec![5, 12, 15, 18];

        // Act
        let erase_result = tree.remove(&erased_key);
        let kth_values_after_erase = collect_kth_values(&tree);

        // Assert
        assert!(erase_result);
        assert_eq!(
            kth_values_after_erase,
            expected_kth_values_after_erase_right
        );
    }

    #[test]
    fn remove_replacing_node_with_successor_preserves_duplicate_count() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [10i64, 5, 15, 12, 12, 12, 18];
        for value in values {
            tree.insert(value);
        }
        let erased_key = 10i64;

        let expected_successor_count_right = 3;
        let expected_kth_values_after_erase_right = vec![5, 12, 12, 12, 15, 18];

        let count_key = &12;

        // Act
        let erase_result = tree.remove(&erased_key);
        let successor_count = tree.count(count_key);
        let kth_values_after_erase = collect_kth_values(&tree);

        // Assert
        assert!(erase_result);
        assert_eq!(successor_count, expected_successor_count_right);
        assert_eq!(
            kth_values_after_erase,
            expected_kth_values_after_erase_right
        );
    }

    #[test]
    fn remove_repeatedly_on_duplicated_key_eventually_makes_tree_empty() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let duplicated_key = 4i64;
        tree.insert(duplicated_key);
        tree.insert(duplicated_key);

        let expected_len_after_erases_right = 0;
        let expected_kth_after_erases_right = None;

        let kth_min_rank = 0;

        // Act
        let first_erase_result = tree.remove(&duplicated_key);
        let second_erase_result = tree.remove(&duplicated_key);
        let len_after_erases = tree.len();
        let kth_after_erases = tree.kth_min(kth_min_rank);

        // Assert
        assert!(first_erase_result);
        assert!(second_erase_result);
        assert_eq!(len_after_erases, expected_len_after_erases_right);
        assert_eq!(kth_after_erases, expected_kth_after_erases_right);
    }

    #[test]
    fn remove_does_not_call_lookup_helper_before_deleting_existing_key() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [8i64, 4, 12, 10, 14];
        for value in values {
            tree.insert(value);
        }
        let erased_key = 8i64;
        reset_find_node_call_count();

        let expected_find_node_call_count_right = 0;

        // Act
        let erase_result = tree.remove(&erased_key);
        let find_node_call_count = load_find_node_call_count();

        // Assert
        assert!(erase_result);
        assert_eq!(find_node_call_count, expected_find_node_call_count_right);
    }

    #[test]
    fn remove_does_not_call_lookup_helper_before_deleting_missing_key() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [8i64, 4, 12, 10, 14];
        for value in values {
            tree.insert(value);
        }
        let missing_key = 9i64;
        reset_find_node_call_count();

        let expected_find_node_call_count_right = 0;

        // Act
        let erase_result = tree.remove(&missing_key);
        let find_node_call_count = load_find_node_call_count();

        // Assert
        assert!(!erase_result);
        assert_eq!(find_node_call_count, expected_find_node_call_count_right);
    }

    #[test]
    fn remove_cases_preserve_subtree_sizes() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let inserted_values = [2i64, 1, 2, 3];
        for value in inserted_values {
            tree.insert(value);
        }
        let unique_key = 1i64;
        let duplicated_key = 2i64;
        let missing_key = 99i64;

        let expected_len_after_erases_right = 2;

        let expected_diagnostics = vec![Ok(()); 3];

        // Act
        let mut actual_diagnostics = Vec::new();
        let unique_erase_result = tree.remove(&unique_key);
        actual_diagnostics.push(invariant_diagnostics(&tree));
        let duplicate_erase_result = tree.remove(&duplicated_key);
        actual_diagnostics.push(invariant_diagnostics(&tree));
        let missing_erase_result = tree.remove(&missing_key);
        actual_diagnostics.push(invariant_diagnostics(&tree));
        let len_after_erases = tree.len();

        // Assert
        assert_eq!(actual_diagnostics, expected_diagnostics);
        assert!(unique_erase_result);
        assert!(duplicate_erase_result);
        assert!(!missing_erase_result);
        assert_eq!(len_after_erases, expected_len_after_erases_right);
    }

    #[test]
    fn remove_two_child_node_rebalances_while_extracting_deep_successor() {
        // Arrange
        let mut tree = build_tree(&[1i64, 3, 6, 0, 2, 4, 5]);

        let expected_tree_len_right = 6;
        let expected_remaining_values_right = vec![0, 1, 2, 4, 5, 6];

        let remove_key = &3;

        // Act
        let remove_result = tree.remove(remove_key);
        let remaining_values = collect_kth_values(&tree);

        let actual_tree_len = tree.len();

        // Assert
        assert!(remove_result);
        assert_eq!(actual_tree_len, expected_tree_len_right);
        assert_eq!(remaining_values, expected_remaining_values_right);
        assert_red_black_tree_invariants(&tree);
    }

    #[test]
    fn remove_decrements_duplicate_internal_node_without_replacing_it() {
        // Arrange
        let mut tree = build_tree(&[2i64, 1, 2, 3]);

        let expected_tree_len_right = 3;
        let expected_tree_count_2_right = 1;
        let expected_remaining_values_right = vec![1, 2, 3];
        let expected_root_key_right = 2;
        let expected_root_count_right = 1;

        let remove_key = &2;
        let count_key = &2;

        // Act
        let remove_result = tree.remove(remove_key);
        let remaining_values = collect_kth_values(&tree);

        let actual_tree_len = tree.len();
        let actual_tree_count_2 = tree.count(count_key);

        let root = tree.root.as_ref().expect("tree remains non-empty");
        let actual_root_key = root.key;
        let actual_root_count = root.count;

        // Assert
        assert!(remove_result);
        assert_eq!(actual_tree_len, expected_tree_len_right);
        assert_eq!(actual_tree_count_2, expected_tree_count_2_right);
        assert_eq!(remaining_values, expected_remaining_values_right);
        assert_eq!(actual_root_key, expected_root_key_right);
        assert_eq!(actual_root_count, expected_root_count_right);
        assert_red_black_tree_invariants(&tree);
    }

    #[test]
    fn remove_missing_key_below_minimum_preserves_tree() {
        // Arrange
        let values = [1i64, 2, 3];
        let missing_key = 0;
        let expected_values = values.to_vec();
        let expected_len = values.len();
        let expected_counts = [1usize; 3];
        let mut tree = build_tree(&values);

        // Act
        let actual_removed = tree.remove(&missing_key);
        let actual_len = tree.len();
        let actual_values = collect_kth_values(&tree);
        let actual_counts = values.map(|key| tree.count(&key));
        let actual_invariants = invariant_diagnostics(&tree);

        // Assert
        assert!(!actual_removed);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_counts, expected_counts);
        assert!(actual_invariants.is_ok());
    }

    #[test]
    fn count_returns_zero_for_empty_tree() {
        // Arrange
        let tree = RedBlackTree::<i64>::new();
        let query_key = 5i64;

        let expected_count_result_right = 0;

        // Act
        let count_result = tree.count(&query_key);

        // Assert
        assert_eq!(count_result, expected_count_result_right);
    }

    #[test]
    fn count_uses_single_shared_lookup_helper_call() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [4i64, 2, 6, 5];
        for value in values {
            tree.insert(value);
        }
        let query_key = 5i64;
        reset_find_node_call_count();

        let expected_count_result_right = 1;
        let expected_find_node_call_count_right = 1;

        // Act
        let count_result = tree.count(&query_key);
        let find_node_call_count = load_find_node_call_count();

        // Assert
        assert_eq!(count_result, expected_count_result_right);
        assert_eq!(find_node_call_count, expected_find_node_call_count_right);
    }

    #[test]
    fn count_less_than_or_equal_returns_zero_for_empty_tree() {
        // Arrange
        let tree = RedBlackTree::<i64>::new();
        let query_key = 5i64;

        let expected_count_result_right = 0;

        // Act
        let count_result = tree.count_less_than_or_equal(&query_key);

        // Assert
        assert_eq!(count_result, expected_count_result_right);
    }

    #[test]
    fn count_less_than_or_equal_handles_missing_duplicate_and_extreme_keys() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [i64::MIN, -2, 0, 0, 3, i64::MAX];
        for value in values {
            tree.insert(value);
        }

        let expected_counts_right = [1, 2, 4, 4, 6];

        let query_keys = [i64::MIN, -1, 0, 2, i64::MAX];

        // Act
        let counts = query_keys.map(|key| tree.count_less_than_or_equal(&key));

        // Assert
        assert_eq!(counts, expected_counts_right);
    }

    #[test]
    fn count_less_than_or_equal_updates_after_erase_and_pop_operations() {
        // Arrange
        let initial_values = [1i64, 2, 2, 3];
        let query_key = 2;
        let expected_counts = [3, 2, 1, 1];
        let expected_pops = (Some(1), Some(3));
        let mut tree = build_tree(&initial_values);

        // Act
        let before = tree.count_less_than_or_equal(&query_key);
        let removed = tree.remove(&query_key);
        let after_remove = tree.count_less_than_or_equal(&query_key);
        let popped_first = tree.pop_first();
        let after_first = tree.count_less_than_or_equal(&query_key);
        let popped_last = tree.pop_last();
        let after_last = tree.count_less_than_or_equal(&query_key);
        let actual_counts = [before, after_remove, after_first, after_last];
        let actual_pops = (popped_first, popped_last);

        // Assert
        assert!(removed);
        assert_eq!(actual_counts, expected_counts);
        assert_eq!(actual_pops, expected_pops);
    }

    #[test]
    fn count_less_than_or_equal_returns_zero_below_minimum_key() {
        // Arrange
        let tree = build_tree(&[10i64, 20, 20, 30]);

        let expected_count_right = 0;

        let count_less_than_or_equal_key = &9;

        // Act
        let count = tree.count_less_than_or_equal(count_less_than_or_equal_key);

        // Assert
        assert_eq!(count, expected_count_right);
    }

    #[test]
    fn count_less_than_or_equal_returns_len_above_maximum_key() {
        // Arrange
        let tree = build_tree(&[10i64, 20, 20, 30]);

        let expected_count_right = 4;

        let count_less_than_or_equal_key = &31;

        // Act
        let count = tree.count_less_than_or_equal(count_less_than_or_equal_key);

        let actual_count_right = tree.len();

        // Assert
        assert_eq!(count, actual_count_right);
        assert_eq!(count, expected_count_right);
    }

    #[test]
    fn contains_returns_false_for_empty_tree() {
        // Arrange
        let tree = RedBlackTree::<i64>::new();
        let query_key = 5i64;

        // Act
        let contains_result = tree.contains(&query_key);

        // Assert
        assert!(!contains_result);
    }

    #[test]
    fn contains_uses_single_shared_lookup_helper_call() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [4i64, 2, 6, 5];
        for value in values {
            tree.insert(value);
        }
        let query_key = 5i64;
        reset_find_node_call_count();

        let expected_find_node_call_count_right = 1;

        // Act
        let contains_result = tree.contains(&query_key);
        let find_node_call_count = load_find_node_call_count();

        // Assert
        assert!(contains_result);
        assert_eq!(find_node_call_count, expected_find_node_call_count_right);
    }

    #[test]
    fn first_returns_none_for_empty_tree() {
        // Arrange
        let tree = RedBlackTree::<i64>::new();

        let expected_first_value_right = None;

        // Act
        let first_value = tree.first();

        // Assert
        assert_eq!(first_value, expected_first_value_right);
    }

    #[test]
    fn first_returns_smallest_value_for_multiple_elements() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [5i64, 1, 3];
        for value in values {
            tree.insert(value);
        }

        let expected_first_value_right = Some(&1);

        // Act
        let first_value = tree.first();

        // Assert
        assert_eq!(first_value, expected_first_value_right);
    }

    #[test]
    fn first_returns_only_element_for_singleton_tree() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let value = 7i64;
        let expected_value = Some(value);

        // Act
        tree.insert(value);
        let actual_value = tree.first();

        // Assert
        assert_eq!(actual_value, expected_value.as_ref());
    }

    #[test]
    fn first_and_last_return_extremes_when_extreme_keys_are_duplicated() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let inserted_values = [3i64, 1, 5, 1, 5];
        let expected_first = Some(1i64);
        let expected_last = Some(5i64);
        for value in inserted_values {
            tree.insert(value);
        }

        // Act
        let actual_first = tree.first();
        let actual_last = tree.last();

        // Assert
        assert_eq!(actual_first, expected_first.as_ref());
        assert_eq!(actual_last, expected_last.as_ref());
    }

    #[test]
    fn last_returns_none_for_empty_tree() {
        // Arrange
        let tree = RedBlackTree::<i64>::new();

        let expected_last_value_right = None;

        // Act
        let last_value = tree.last();

        // Assert
        assert_eq!(last_value, expected_last_value_right);
    }

    #[test]
    fn last_returns_largest_value_for_multiple_elements() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [5i64, 1, 3];
        for value in values {
            tree.insert(value);
        }

        let expected_last_value_right = Some(&5);

        // Act
        let last_value = tree.last();

        // Assert
        assert_eq!(last_value, expected_last_value_right);
    }

    #[test]
    fn last_returns_only_element_for_singleton_tree() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let value = 7i64;
        let expected_value = Some(value);

        // Act
        tree.insert(value);
        let actual_value = tree.last();

        // Assert
        assert_eq!(actual_value, expected_value.as_ref());
    }

    #[test]
    fn pop_first_returns_none_for_empty_tree() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();

        let expected_popped_value_right = None;
        let expected_len_after_pop_right = 0;

        // Act
        let popped_value = tree.pop_first();
        let len_after_pop = tree.len();

        // Assert
        assert_eq!(popped_value, expected_popped_value_right);
        assert_eq!(len_after_pop, expected_len_after_pop_right);
    }

    #[test]
    fn pop_first_removes_one_occurrence_of_minimum_key() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let minimum_key = 1i64;
        tree.insert(minimum_key);
        tree.insert(minimum_key);
        tree.insert(3);

        let expected_count_after_pop_right = 1;
        let expected_len_after_pop_right = 2;

        // Act
        let popped_value = tree.pop_first();
        let count_after_pop = tree.count(&minimum_key);
        let first_after_pop = tree.first();
        let len_after_pop = tree.len();

        let actual_popped_value_right = Some(minimum_key);
        let actual_first_after_pop_right = Some(&minimum_key);

        // Assert
        assert_eq!(popped_value, actual_popped_value_right);
        assert_eq!(count_after_pop, expected_count_after_pop_right);
        assert_eq!(first_after_pop, actual_first_after_pop_right);
        assert_eq!(len_after_pop, expected_len_after_pop_right);
    }

    #[test]
    fn pop_first_on_singleton_tree_returns_element_and_makes_tree_empty() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let value = 7i64;
        let expected_value = Some(value);
        let expected_is_empty = true;

        // Act
        tree.insert(value);
        let actual_value = tree.pop_first();
        let actual_is_empty = tree.is_empty();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn pop_first_updates_first_to_next_key_when_minimum_count_is_one() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let minimum_value = 1i64;
        let next_value = 2i64;
        let expected_popped_value = Some(minimum_value);
        let expected_first_value = Some(next_value);

        // Act
        tree.insert(minimum_value);
        tree.insert(next_value);
        let actual_popped_value = tree.pop_first();
        let actual_first_value = tree.first();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_first_value, expected_first_value.as_ref());
    }

    #[test]
    fn pop_first_repeatedly_returns_all_values_in_ascending_order() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let inserted_values = [3i64, 1, 2, 1];
        let expected_values = vec![1i64, 1, 2, 3];
        for value in inserted_values {
            tree.insert(value);
        }

        let expected_diagnostics = vec![Ok(()); inserted_values.len()];

        // Act
        let mut actual_diagnostics = Vec::new();
        let mut actual_values = Vec::new();
        while let Some(value) = tree.pop_first() {
            actual_values.push(value);
            actual_diagnostics.push(invariant_diagnostics(&tree));
        }
        let actual_is_empty = tree.is_empty();

        // Assert
        assert_eq!(actual_diagnostics, expected_diagnostics);
        assert_eq!(actual_values, expected_values);
        assert!(actual_is_empty);
    }

    #[test]
    fn pop_last_returns_none_for_empty_tree() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();

        let expected_popped_value_right = None;
        let expected_len_after_pop_right = 0;

        // Act
        let popped_value = tree.pop_last();
        let len_after_pop = tree.len();

        // Assert
        assert_eq!(popped_value, expected_popped_value_right);
        assert_eq!(len_after_pop, expected_len_after_pop_right);
    }

    #[test]
    fn pop_last_removes_one_occurrence_of_maximum_key() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let maximum_key = 3i64;
        tree.insert(1);
        tree.insert(maximum_key);
        tree.insert(maximum_key);

        let expected_count_after_pop_right = 1;
        let expected_len_after_pop_right = 2;

        // Act
        let popped_value = tree.pop_last();
        let count_after_pop = tree.count(&maximum_key);
        let last_after_pop = tree.last();
        let len_after_pop = tree.len();

        let actual_popped_value_right = Some(maximum_key);
        let actual_last_after_pop_right = Some(&maximum_key);

        // Assert
        assert_eq!(popped_value, actual_popped_value_right);
        assert_eq!(count_after_pop, expected_count_after_pop_right);
        assert_eq!(last_after_pop, actual_last_after_pop_right);
        assert_eq!(len_after_pop, expected_len_after_pop_right);
    }

    #[test]
    fn pop_last_on_singleton_tree_returns_element_and_makes_tree_empty() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let value = 7i64;
        let expected_value = Some(value);
        let expected_is_empty = true;

        // Act
        tree.insert(value);
        let actual_value = tree.pop_last();
        let actual_is_empty = tree.is_empty();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn pop_last_updates_last_to_previous_key_when_maximum_count_is_one() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let previous_value = 1i64;
        let maximum_value = 2i64;
        let expected_popped_value = Some(maximum_value);
        let expected_last_value = Some(previous_value);

        // Act
        tree.insert(previous_value);
        tree.insert(maximum_value);
        let actual_popped_value = tree.pop_last();
        let actual_last_value = tree.last();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_last_value, expected_last_value.as_ref());
    }

    #[test]
    fn pop_last_repeatedly_returns_all_values_in_descending_order() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let inserted_values = [3i64, 1, 3, 2];
        let expected_values = vec![3i64, 3, 2, 1];
        for value in inserted_values {
            tree.insert(value);
        }

        let expected_diagnostics = vec![Ok(()); inserted_values.len()];

        // Act
        let mut actual_diagnostics = Vec::new();
        let mut actual_values = Vec::new();
        while let Some(value) = tree.pop_last() {
            actual_values.push(value);
            actual_diagnostics.push(invariant_diagnostics(&tree));
        }
        let actual_is_empty = tree.is_empty();

        // Assert
        assert_eq!(actual_diagnostics, expected_diagnostics);
        assert_eq!(actual_values, expected_values);
        assert!(actual_is_empty);
    }

    #[test]
    fn kth_min_is_sorted_after_ascending_insertions() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let inserted_values = [1i64, 2, 3, 4, 5];

        let expected_kth_values_right = vec![1, 2, 3, 4, 5];

        // Act
        for value in inserted_values {
            tree.insert(value);
        }
        let kth_values = collect_kth_values(&tree);

        // Assert
        assert_eq!(kth_values, expected_kth_values_right);
    }

    #[test]
    fn kth_min_is_sorted_after_descending_insertions() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let inserted_values = [5i64, 4, 3, 2, 1];

        let expected_kth_values_right = vec![1, 2, 3, 4, 5];

        // Act
        for value in inserted_values {
            tree.insert(value);
        }
        let kth_values = collect_kth_values(&tree);

        // Assert
        assert_eq!(kth_values, expected_kth_values_right);
    }

    #[test]
    fn kth_min_returns_none_for_empty_tree() {
        // Arrange
        let tree = RedBlackTree::<i64>::new();
        let index = 0usize;

        let expected_kth_value_right = None;

        // Act
        let kth_value = tree.kth_min(index);

        // Assert
        assert_eq!(kth_value, expected_kth_value_right);
    }

    #[test]
    fn kth_min_returns_only_element_for_singleton_tree() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let inserted_key = 9i64;
        tree.insert(inserted_key);
        let index = 0usize;

        // Act
        let kth_value = tree.kth_min(index);

        let actual_kth_value_right = Some(&inserted_key);

        // Assert
        assert_eq!(kth_value, actual_kth_value_right);
    }

    #[test]
    fn kth_min_returns_expected_values_for_boundary_indices() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [7i64, 3, 9, 5];
        for value in values {
            tree.insert(value);
        }
        let first_index = 0usize;
        let last_index = tree.len() - 1;
        let out_of_range_index = tree.len();

        let expected_first_value_right = Some(&3);
        let expected_last_value_right = Some(&9);
        let expected_out_of_range_value_right = None;

        // Act
        let first_value = tree.kth_min(first_index);
        let last_value = tree.kth_min(last_index);
        let out_of_range_value = tree.kth_min(out_of_range_index);

        // Assert
        assert_eq!(first_value, expected_first_value_right);
        assert_eq!(last_value, expected_last_value_right);
        assert_eq!(out_of_range_value, expected_out_of_range_value_right);
    }

    #[test]
    fn kth_min_preserves_multiset_order_for_duplicated_values() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [2i64, 1, 3, 2];

        let expected_kth_values_right = vec![1, 2, 2, 3];

        // Act
        for value in values {
            tree.insert(value);
        }
        let kth_values = collect_kth_values(&tree);

        // Assert
        assert_eq!(kth_values, expected_kth_values_right);
    }

    #[test]
    fn kth_min_updates_after_remove_operation() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let values = [2i64, 1, 3, 2];
        for value in values {
            tree.insert(value);
        }
        let erased_key = 2i64;

        let expected_kth_values_after_erase_right = vec![1, 2, 3];

        // Act
        let erase_result = tree.remove(&erased_key);
        let kth_values_after_erase = collect_kth_values(&tree);

        // Assert
        assert!(erase_result);
        assert_eq!(
            kth_values_after_erase,
            expected_kth_values_after_erase_right
        );
    }

    #[test]
    fn kth_min_with_usize_max_returns_none_without_panicking() {
        // Arrange
        let tree = build_tree(&[10i64, 20, 20, 30]);

        let expected_value_right = None;

        let kth_min_rank = usize::MAX;

        // Act
        let value = tree.kth_min(kth_min_rank);

        // Assert
        assert_eq!(value, expected_value_right);
    }

    #[test]
    fn random_operations_match_btreemap_reference_results() {
        // Arrange
        let mut random_state = 20260221u64;
        let operation_count = 200usize;
        let operation_kinds = 2usize;
        let key_count = 31u64;
        let key_offset = 15i64;
        let initial_values = [];
        let queries = (-key_offset..=key_offset).collect::<Vec<_>>();
        let mut operation_counts = vec![0usize; operation_kinds];
        let operations = (0..operation_count)
            .map(|_| {
                let selector =
                    ((next_random(&mut random_state) >> 32) % operation_kinds as u64) as usize;
                let key = (next_random(&mut random_state) % key_count) as i64 - key_offset;
                operation_counts[selector] += 1;
                match selector {
                    0 => Operation::Insert(key),
                    1 => Operation::Remove(key),
                    2 => Operation::PopFirst,
                    _ => Operation::PopLast,
                }
            })
            .collect::<Vec<_>>();
        let all_operations_are_exercised = operation_counts.iter().all(|&count| count > 0);
        let expected_steps = reference_sequence(&initial_values, &operations, &queries);
        let mut tree = RedBlackTree::new();

        // Act
        let actual_steps = actual_sequence(&mut tree, &operations, &queries);

        // Assert
        assert!(all_operations_are_exercised);
        assert_eq!(actual_steps, expected_steps);
    }

    #[test]
    fn random_operations_with_extreme_pops_match_btreemap_reference_results() {
        // Arrange
        let mut random_state = 20260821u64;
        let operation_count = 2000usize;
        let operation_kinds = 4usize;
        let key_count = 101u64;
        let key_offset = 50i64;
        let initial_values = [];
        let queries = (-key_offset..=key_offset).collect::<Vec<_>>();
        let mut operation_counts = vec![0usize; operation_kinds];
        let operations = (0..operation_count)
            .map(|_| {
                let selector =
                    ((next_random(&mut random_state) >> 32) % operation_kinds as u64) as usize;
                let key = (next_random(&mut random_state) % key_count) as i64 - key_offset;
                operation_counts[selector] += 1;
                match selector {
                    0 => Operation::Insert(key),
                    1 => Operation::Remove(key),
                    2 => Operation::PopFirst,
                    _ => Operation::PopLast,
                }
            })
            .collect::<Vec<_>>();
        let all_operations_are_exercised = operation_counts.iter().all(|&count| count > 0);
        let expected_steps = reference_sequence(&initial_values, &operations, &queries);
        let mut tree = RedBlackTree::new();

        // Act
        let actual_steps = actual_sequence(&mut tree, &operations, &queries);

        // Assert
        assert!(all_operations_are_exercised);
        assert_eq!(actual_steps, expected_steps);
    }

    #[test]
    fn fixed_seed_mixed_insert_remove_and_pop_operations_match_btreemap() {
        // Arrange
        let mut random_state = 20260930u64;
        let operation_count = 512usize;
        let operation_kinds = 4usize;
        let key_count = 33u64;
        let key_offset = 16i64;
        let initial_values = [];
        let queries = (-key_offset..=key_offset).collect::<Vec<_>>();
        let mut operation_counts = vec![0usize; operation_kinds];
        let operations = (0..operation_count)
            .map(|_| {
                let selector =
                    ((next_random(&mut random_state) >> 32) % operation_kinds as u64) as usize;
                let key = (next_random(&mut random_state) % key_count) as i64 - key_offset;
                operation_counts[selector] += 1;
                match selector {
                    0 => Operation::Insert(key),
                    1 => Operation::Remove(key),
                    2 => Operation::PopFirst,
                    _ => Operation::PopLast,
                }
            })
            .collect::<Vec<_>>();
        let all_operations_are_exercised = operation_counts.iter().all(|&count| count > 0);
        let expected_steps = reference_sequence(&initial_values, &operations, &queries);
        let mut tree = RedBlackTree::new();

        // Act
        let actual_steps = actual_sequence(&mut tree, &operations, &queries);

        // Assert
        assert!(all_operations_are_exercised);
        assert_eq!(actual_steps, expected_steps);
    }

    #[test]
    fn tree_can_be_reused_after_removing_all_elements() {
        // Arrange
        let initial_values = [4i64, 1, 7, 3, 9];
        let reinserted_values = [8i64, 2, 5, 5];
        let operations = initial_values
            .into_iter()
            .map(Operation::Remove)
            .chain(reinserted_values.into_iter().map(Operation::Insert))
            .collect::<Vec<_>>();
        let query_keys = [1, 2, 3, 4, 5, 7, 8, 9];
        let expected_steps = reference_sequence(&initial_values, &operations, &query_keys);
        let mut tree = build_tree(&initial_values);

        // Act
        let actual_steps = actual_sequence(&mut tree, &operations, &query_keys);

        // Assert
        assert_eq!(actual_steps, expected_steps);
    }

    #[test]
    fn mixed_operations_keep_kth_min_sequence_non_decreasing() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let operations = [
            (true, 3i64),
            (true, 1),
            (true, 2),
            (true, 2),
            (false, 1),
            (true, 5),
            (false, 3),
            (true, 4),
        ];

        let expected_remove_results = vec![true, true];

        // Act
        let mut actual_remove_results = Vec::new();
        for (is_insert, key) in operations {
            if is_insert {
                tree.insert(key);
            } else {
                actual_remove_results.push(tree.remove(&key));
            }
        }
        let kth_values = collect_kth_values(&tree);
        let non_decreasing = is_non_decreasing(&kth_values);

        // Assert
        assert_eq!(actual_remove_results, expected_remove_results);
        assert!(non_decreasing);
    }

    #[test]
    fn mixed_operations_keep_len_equal_to_sum_of_counts() {
        // Arrange
        let mut tree = RedBlackTree::<i64>::new();
        let operations = [
            (true, 4i64),
            (true, 2),
            (true, 4),
            (false, 4),
            (true, 6),
            (false, 2),
            (true, 5),
        ];

        let expected_remove_results = vec![true, true];

        let first_rank = 0;

        // Act
        let mut actual_remove_results = Vec::new();
        for (is_insert, key) in operations {
            if is_insert {
                tree.insert(key);
            } else {
                actual_remove_results.push(tree.remove(&key));
            }
        }
        let tree_len = tree.len();
        let sum_of_counts = tree
            .kth_min(first_rank)
            .map(|_| {
                let mut total = 0usize;
                let mut index = 0usize;
                while let Some(key) = tree.kth_min(index) {
                    total += tree.count(key);
                    index += tree.count(key);
                }
                total
            })
            .unwrap_or(0);

        // Assert
        assert_eq!(actual_remove_results, expected_remove_results);
        assert_eq!(tree_len, sum_of_counts);
    }

    #[test]
    #[ignore = "performance benchmark"]
    fn benchmark_ascending_insert_pattern_reports_elapsed_duration() {
        // Arrange
        let value_count = 20_000usize;
        let repetition_count = 5usize;
        let inserted_values = build_ascending_values(value_count);

        let duration_lower_bound = Duration::ZERO;

        // Act
        let elapsed_duration =
            measure_insert_benchmark_duration(&inserted_values, repetition_count);
        println!(
            "ascending_insert value_count={} repetition_count={} elapsed={:?}",
            value_count, repetition_count, elapsed_duration
        );

        // Assert
        assert!(duration_lower_bound < elapsed_duration);
    }

    #[test]
    #[ignore = "performance benchmark"]
    fn benchmark_descending_insert_pattern_reports_elapsed_duration() {
        // Arrange
        let value_count = 20_000usize;
        let repetition_count = 5usize;
        let inserted_values = build_descending_values(value_count);

        let duration_lower_bound = Duration::ZERO;

        // Act
        let elapsed_duration =
            measure_insert_benchmark_duration(&inserted_values, repetition_count);
        println!(
            "descending_insert value_count={} repetition_count={} elapsed={:?}",
            value_count, repetition_count, elapsed_duration
        );

        // Assert
        assert!(duration_lower_bound < elapsed_duration);
    }

    #[test]
    #[ignore = "performance benchmark"]
    fn benchmark_duplicate_heavy_insert_pattern_reports_elapsed_duration() {
        // Arrange
        let value_count = 20_000usize;
        let repetition_count = 5usize;
        let inserted_values = build_duplicate_heavy_values(value_count);

        let duration_lower_bound = Duration::ZERO;

        // Act
        let elapsed_duration =
            measure_insert_benchmark_duration(&inserted_values, repetition_count);
        println!(
            "duplicate_heavy_insert value_count={} repetition_count={} elapsed={:?}",
            value_count, repetition_count, elapsed_duration
        );

        // Assert
        assert!(duration_lower_bound < elapsed_duration);
    }

    #[test]
    #[ignore = "performance benchmark"]
    fn benchmark_mixed_insert_and_erase_pattern_reports_elapsed_duration() {
        // Arrange
        let operation_count = 40_000usize;
        let repetition_count = 5usize;
        let operations = build_mixed_operations(operation_count);

        let duration_lower_bound = Duration::ZERO;

        // Act
        let elapsed_duration =
            measure_mixed_operations_benchmark_duration(&operations, repetition_count);
        println!(
            "mixed_insert_erase operation_count={} repetition_count={} elapsed={:?}",
            operation_count, repetition_count, elapsed_duration
        );

        // Assert
        assert!(duration_lower_bound < elapsed_duration);
    }

    #[test]
    #[ignore = "performance benchmark"]
    fn benchmark_first_reports_elapsed_duration() {
        // Arrange
        let value_count = 20_000usize;
        let operation_count = 5_000_000usize;
        let inserted_values = build_ascending_values(value_count);
        let tree = build_tree(&inserted_values);

        let duration_lower_bound = Duration::ZERO;

        // Act
        let elapsed_duration = measure_first_benchmark_duration(&tree, operation_count);
        println!(
            "first value_count={} operation_count={} elapsed={:?}",
            value_count, operation_count, elapsed_duration
        );

        // Assert
        assert!(duration_lower_bound < elapsed_duration);
    }

    #[test]
    #[ignore = "performance benchmark"]
    fn benchmark_last_reports_elapsed_duration() {
        // Arrange
        let value_count = 20_000usize;
        let operation_count = 5_000_000usize;
        let inserted_values = build_ascending_values(value_count);
        let tree = build_tree(&inserted_values);

        let duration_lower_bound = Duration::ZERO;

        // Act
        let elapsed_duration = measure_last_benchmark_duration(&tree, operation_count);
        println!(
            "last value_count={} operation_count={} elapsed={:?}",
            value_count, operation_count, elapsed_duration
        );

        // Assert
        assert!(duration_lower_bound < elapsed_duration);
    }

    #[test]
    #[ignore = "performance benchmark"]
    fn benchmark_pop_first_reports_elapsed_duration() {
        // Arrange
        let value_count = 20_000usize;
        let repetition_count = 10usize;
        let inserted_values = build_ascending_values(value_count);

        let duration_lower_bound = Duration::ZERO;

        // Act
        let elapsed_duration =
            measure_pop_first_benchmark_duration(&inserted_values, repetition_count);
        println!(
            "pop_first value_count={} repetition_count={} elapsed={:?}",
            value_count, repetition_count, elapsed_duration
        );

        // Assert
        assert!(duration_lower_bound < elapsed_duration);
    }

    #[test]
    #[ignore = "performance benchmark"]
    fn benchmark_pop_last_reports_elapsed_duration() {
        // Arrange
        let value_count = 20_000usize;
        let repetition_count = 10usize;
        let inserted_values = build_ascending_values(value_count);

        let duration_lower_bound = Duration::ZERO;

        // Act
        let elapsed_duration =
            measure_pop_last_benchmark_duration(&inserted_values, repetition_count);
        println!(
            "pop_last value_count={} repetition_count={} elapsed={:?}",
            value_count, repetition_count, elapsed_duration
        );

        // Assert
        assert!(duration_lower_bound < elapsed_duration);
    }
}

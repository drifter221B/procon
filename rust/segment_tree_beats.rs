use std::cmp::{max, min};
use std::ops::{Add, Mul, Sub};

pub trait Zero: Copy {
    fn zero() -> Self;
}

macro_rules! impl_zero {
    ($($t:ty),*) => {
        $(
            impl Zero for $t {
                fn zero() -> Self {
                    0 as $t
                }
            }
        )*
    };
}

impl_zero!(i32, i64, i128, isize, usize);

pub trait Bounds: Copy + Ord {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

macro_rules! impl_bounds {
    ($($t:ty),*) => {
        $(
            impl Bounds for $t {
                fn min_value() -> Self {
                    <$t>::MIN
                }

                fn max_value() -> Self {
                    <$t>::MAX
                }
            }
        )*
    };
}

impl_bounds!(i32, i64, i128, isize);

pub trait FromUsize: Copy {
    fn from_usize(value: usize) -> Self;
}

macro_rules! impl_from_usize {
    ($($t:ty),*) => {
        $(
            impl FromUsize for $t {
                fn from_usize(value: usize) -> Self {
                    value as $t
                }
            }
        )*
    };
}

impl_from_usize!(i32, i64, i128, isize, usize);

#[derive(Clone, Copy, Debug)]
struct Node<S, T> {
    sum: T,
    max1: S,
    max2: S,
    max_count: T,
    min1: S,
    min2: S,
    min_count: T,
}

impl<S, T> Node<S, T>
where
    S: Bounds + Zero + Copy + Ord,
    T: Zero + Copy,
{
    fn empty() -> Self {
        Self {
            sum: T::zero(),
            max1: S::min_value(),
            max2: S::min_value(),
            max_count: T::zero(),
            min1: S::max_value(),
            min2: S::max_value(),
            min_count: T::zero(),
        }
    }
}

pub enum QueryKind<'a, S, T> {
    Sum(&'a dyn Fn(&T) -> bool),
    Min(&'a dyn Fn(&S) -> bool),
    Max(&'a dyn Fn(&S) -> bool),
}

pub struct SegmentTreeBeats<S, T>
where
    S: Bounds + Zero + Copy + Ord + Add<Output = S> + Sub<Output = S> + PartialEq,
    T: Zero
        + Copy
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + From<S>
        + FromUsize
        + PartialEq,
{
    size: usize,
    leaf_count: usize,
    nodes: Vec<Node<S, T>>,
    lazy: Vec<S>,
}

impl<S, T> SegmentTreeBeats<S, T>
where
    S: Bounds + Zero + Copy + Ord + Add<Output = S> + Sub<Output = S> + PartialEq,
    T: Zero
        + Copy
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + From<S>
        + FromUsize
        + PartialEq,
{
    pub fn new(size: usize) -> Self {
        let leaf_count = if size == 0 { 1 } else { size.next_power_of_two() };
        let nodes = vec![Node::empty(); leaf_count << 1];
        let lazy = vec![S::zero(); leaf_count << 1];

        Self {
            size,
            leaf_count,
            nodes,
            lazy,
        }
    }

    pub fn from_slice(values: &[S]) -> Self {
        let mut tree = Self::new(values.len());
        tree.build(values);

        tree
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    pub fn range_add(&mut self, left: usize, right: usize, value: S) {
        assert!(left <= right && right <= self.size);
        self.range_add_internal(left, right, value, 1, 0, self.leaf_count);
    }

    pub fn range_chmin(&mut self, left: usize, right: usize, value: S) {
        assert!(left <= right && right <= self.size);
        self.range_chmin_internal(left, right, value, 1, 0, self.leaf_count);
    }

    pub fn range_chmax(&mut self, left: usize, right: usize, value: S) {
        assert!(left <= right && right <= self.size);
        self.range_chmax_internal(left, right, value, 1, 0, self.leaf_count);
    }

    pub fn query_sum(&mut self, left: usize, right: usize) -> T {
        assert!(left <= right && right <= self.size);
        self.query_sum_internal(left, right, 1, 0, self.leaf_count)
    }

    pub fn query_min(&mut self, left: usize, right: usize) -> S {
        assert!(left <= right && right <= self.size);
        self.query_min_internal(left, right, 1, 0, self.leaf_count)
    }

    pub fn query_max(&mut self, left: usize, right: usize) -> S {
        assert!(left <= right && right <= self.size);
        self.query_max_internal(left, right, 1, 0, self.leaf_count)
    }

    pub fn max_right(&mut self, left: usize, metric: QueryKind<'_, S, T>) -> Option<usize> {
        match metric {
            QueryKind::Sum(pred) => self.max_right_sum(left, pred),
            QueryKind::Min(pred) => self.max_right_min(left, pred),
            QueryKind::Max(pred) => self.max_right_max(left, pred),
        }
    }

    pub fn min_left(&mut self, right: usize, metric: QueryKind<'_, S, T>) -> Option<usize> {
        match metric {
            QueryKind::Sum(pred) => self.min_left_sum(right, pred),
            QueryKind::Min(pred) => self.min_left_min(right, pred),
            QueryKind::Max(pred) => self.min_left_max(right, pred),
        }
    }

    pub fn max_right_sum<P>(&mut self, left: usize, predicate: P) -> Option<usize>
    where
        P: Fn(&T) -> bool,
    {
        if self.size < left {
            return None;
        }
        if left == self.size {
            return Some(self.size);
        }
        if !predicate(&T::zero()) {
            return Some(left);
        }
        let mut acc = T::zero();
        let pos = self.max_right_sum_internal(left, &predicate, &mut acc, 1, 0, self.leaf_count);
        Some(pos.min(self.size))
    }

    pub fn max_right_min<P>(&mut self, left: usize, predicate: P) -> Option<usize>
    where
        P: Fn(&S) -> bool,
    {
        if self.size < left {
            return None;
        }
        if left == self.size {
            return Some(self.size);
        }
        if !predicate(&S::max_value()) {
            return Some(left);
        }
        let mut acc = S::max_value();
        let pos = self.max_right_min_internal(left, &predicate, &mut acc, 1, 0, self.leaf_count);
        Some(pos.min(self.size))
    }

    pub fn max_right_max<P>(&mut self, left: usize, predicate: P) -> Option<usize>
    where
        P: Fn(&S) -> bool,
    {
        if self.size < left {
            return None;
        }
        if left == self.size {
            return Some(self.size);
        }
        if !predicate(&S::min_value()) {
            return Some(left);
        }
        let mut acc = S::min_value();
        let pos = self.max_right_max_internal(left, &predicate, &mut acc, 1, 0, self.leaf_count);
        Some(pos.min(self.size))
    }

    pub fn min_left_sum<P>(&mut self, right: usize, predicate: P) -> Option<usize>
    where
        P: Fn(&T) -> bool,
    {
        if self.size < right {
            return None;
        }
        if right == 0 {
            return Some(0);
        }
        if !predicate(&T::zero()) {
            return Some(right);
        }
        let mut acc = T::zero();
        let pos = self.min_left_sum_internal(right, &predicate, &mut acc, 1, 0, self.leaf_count);
        Some(pos.min(self.size))
    }

    pub fn min_left_min<P>(&mut self, right: usize, predicate: P) -> Option<usize>
    where
        P: Fn(&S) -> bool,
    {
        if self.size < right {
            return None;
        }
        if right == 0 {
            return Some(0);
        }
        if !predicate(&S::max_value()) {
            return Some(right);
        }
        let mut acc = S::max_value();
        let pos = self.min_left_min_internal(right, &predicate, &mut acc, 1, 0, self.leaf_count);
        Some(pos.min(self.size))
    }

    pub fn min_left_max<P>(&mut self, right: usize, predicate: P) -> Option<usize>
    where
        P: Fn(&S) -> bool,
    {
        if self.size < right {
            return None;
        }
        if right == 0 {
            return Some(0);
        }
        if !predicate(&S::min_value()) {
            return Some(right);
        }
        let mut acc = S::min_value();
        let pos = self.min_left_max_internal(right, &predicate, &mut acc, 1, 0, self.leaf_count);
        Some(pos.min(self.size))
    }

    fn build(&mut self, values: &[S]) {
        for (idx, value) in values.iter().enumerate() {
            self.nodes[self.leaf_count + idx] = Node {
                sum: T::from(*value),
                max1: *value,
                max2: S::min_value(),
                max_count: T::from_usize(1),
                min1: *value,
                min2: S::max_value(),
                min_count: T::from_usize(1),
            };
        }

        for idx in (1..self.leaf_count).rev() {
            self.nodes[idx] = Self::merge_nodes(self.nodes[idx << 1], self.nodes[idx << 1 | 1]);
        }
    }

    fn merge_nodes(left: Node<S, T>, right: Node<S, T>) -> Node<S, T> {
        let mut res = Node::empty();
        res.sum = left.sum + right.sum;

        if right.max1 < left.max1 {
            res.max1 = left.max1;
            res.max_count = left.max_count;
            res.max2 = max(left.max2, right.max1);
        } else if left.max1 < right.max1 {
            res.max1 = right.max1;
            res.max_count = right.max_count;
            res.max2 = max(left.max1, right.max2);
        } else {
            res.max1 = left.max1;
            res.max_count = left.max_count + right.max_count;
            res.max2 = max(left.max2, right.max2);
        }

        if left.min1 < right.min1 {
            res.min1 = left.min1;
            res.min_count = left.min_count;
            res.min2 = min(left.min2, right.min1);
        } else if right.min1 < left.min1 {
            res.min1 = right.min1;
            res.min_count = right.min_count;
            res.min2 = min(left.min1, right.min2);
        } else {
            res.min1 = left.min1;
            res.min_count = left.min_count + right.min_count;
            res.min2 = min(left.min2, right.min2);
        }

        res
    }

    fn apply_add(&mut self, index: usize, value: S, length: usize) {
        let node = &mut self.nodes[index];
        let length_as_t = T::from_usize(length);
        node.sum = node.sum + T::from(value) * length_as_t;
        node.max1 = node.max1 + value;

        if node.max2 != S::min_value() {
            node.max2 = node.max2 + value;
        }

        node.min1 = node.min1 + value;
        if node.min2 != S::max_value() {
            node.min2 = node.min2 + value;
        }

        self.lazy[index] = self.lazy[index] + value;
    }

    fn apply_chmin(&mut self, index: usize, value: S) {
        let node = &mut self.nodes[index];

        if node.max1 <= value {
            return;
        }

        let diff = T::from(node.max1 - value);
        node.sum = node.sum - diff * node.max_count;
        node.max1 = value;

        if value < node.min1 {
            node.min1 = value;
        }
    }

    fn apply_chmax(&mut self, index: usize, value: S) {
        let node = &mut self.nodes[index];

        if value <= node.min1 {
            return;
        }

        let diff = T::from(value - node.min1);
        node.sum = node.sum + diff * node.min_count;
        node.min1 = value;

        if node.max1 < value {
            node.max1 = value;
        }
    }

    fn push(&mut self, index: usize, length: usize) {
        if self.leaf_count <= index {
            return;
        }

        let left_child = index << 1;
        let right_child = left_child | 1;
        let half = length >> 1;

        if self.lazy[index] != S::zero() {
            let pending = self.lazy[index];
            self.apply_add(left_child, pending, half);
            self.apply_add(right_child, pending, half);
            self.lazy[index] = S::zero();
        }

        self.apply_chmin(left_child, self.nodes[index].max1);
        self.apply_chmin(right_child, self.nodes[index].max1);
        self.apply_chmax(left_child, self.nodes[index].min1);
        self.apply_chmax(right_child, self.nodes[index].min1);
    }

    fn range_add_internal(&mut self, left: usize, right: usize, value: S, index: usize, seg_left: usize, seg_right: usize) {
        if right <= seg_left || seg_right <= left {
            return;
        }

        if left <= seg_left && seg_right <= right {
            self.apply_add(index, value, seg_right - seg_left);
            return;
        }

        self.push(index, seg_right - seg_left);
        let mid = (seg_left + seg_right) >> 1;
        self.range_add_internal(left, right, value, index << 1, seg_left, mid);
        self.range_add_internal(left, right, value, index << 1 | 1, mid, seg_right);
        self.nodes[index] = Self::merge_nodes(self.nodes[index << 1], self.nodes[index << 1 | 1]);
    }

    fn range_chmin_internal(&mut self, left: usize, right: usize, value: S, index: usize, seg_left: usize, seg_right: usize) {
        if right <= seg_left || seg_right <= left || self.nodes[index].max1 <= value {
            return;
        }

        if left <= seg_left && seg_right <= right && self.nodes[index].max2 < value {
            self.apply_chmin(index, value);
            return;
        }

        self.push(index, seg_right - seg_left);
        let mid = (seg_left + seg_right) >> 1;
        self.range_chmin_internal(left, right, value, index << 1, seg_left, mid);
        self.range_chmin_internal(left, right, value, index << 1 | 1, mid, seg_right);
        self.nodes[index] = Self::merge_nodes(self.nodes[index << 1], self.nodes[index << 1 | 1]);
    }

    fn range_chmax_internal(&mut self, left: usize, right: usize, value: S, index: usize, seg_left: usize, seg_right: usize) {
        if right <= seg_left || seg_right <= left || value <= self.nodes[index].min1 {
            return;
        }

        if left <= seg_left && seg_right <= right && value < self.nodes[index].min2 {
            self.apply_chmax(index, value);
            return;
        }

        self.push(index, seg_right - seg_left);
        let mid = (seg_left + seg_right) >> 1;
        self.range_chmax_internal(left, right, value, index << 1, seg_left, mid);
        self.range_chmax_internal(left, right, value, index << 1 | 1, mid, seg_right);
        self.nodes[index] = Self::merge_nodes(self.nodes[index << 1], self.nodes[index << 1 | 1]);
    }

    fn query_sum_internal(&mut self, left: usize, right: usize, index: usize, seg_left: usize, seg_right: usize) -> T {
        if right <= seg_left || seg_right <= left {
            return T::zero();
        }

        if left <= seg_left && seg_right <= right {
            return self.nodes[index].sum;
        }

        self.push(index, seg_right - seg_left);
        let mid = (seg_left + seg_right) >> 1;
        self.query_sum_internal(left, right, index << 1, seg_left, mid)
            + self.query_sum_internal(left, right, index << 1 | 1, mid, seg_right)
    }

    fn query_min_internal(&mut self, left: usize, right: usize, index: usize, seg_left: usize, seg_right: usize) -> S {
        if right <= seg_left || seg_right <= left {
            return S::max_value();
        }

        if left <= seg_left && seg_right <= right {
            return self.nodes[index].min1;
        }

        self.push(index, seg_right - seg_left);
        let mid = (seg_left + seg_right) >> 1;
        min(
            self.query_min_internal(left, right, index << 1, seg_left, mid),
            self.query_min_internal(left, right, index << 1 | 1, mid, seg_right),
        )
    }

    fn query_max_internal(&mut self, left: usize, right: usize, index: usize, seg_left: usize, seg_right: usize) -> S {
        if right <= seg_left || seg_right <= left {
            return S::min_value();
        }

        if left <= seg_left && seg_right <= right {
            return self.nodes[index].max1;
        }

        self.push(index, seg_right - seg_left);
        let mid = (seg_left + seg_right) >> 1;
        max(
            self.query_max_internal(left, right, index << 1, seg_left, mid),
            self.query_max_internal(left, right, index << 1 | 1, mid, seg_right),
        )
    }

    fn max_right_sum_internal<P>(&mut self, left: usize, predicate: &P, acc: &mut T, index: usize, seg_left: usize, seg_right: usize) -> usize
    where
        P: Fn(&T) -> bool,
    {
        if seg_right <= left {
            return left;
        }
        if left <= seg_left {
            let next = *acc + self.nodes[index].sum;
            if predicate(&next) {
                *acc = next;
                return seg_right;
            }
        }
        if seg_right - seg_left == 1 {
            return seg_left;
        }
        self.push(index, seg_right - seg_left);
        let mid = (seg_left + seg_right) >> 1;
        let res_left = self.max_right_sum_internal(left, predicate, acc, index << 1, seg_left, mid);
        if res_left < mid {
            return res_left;
        }
        self.max_right_sum_internal(left, predicate, acc, index << 1 | 1, mid, seg_right)
    }

    fn max_right_min_internal<P>(&mut self, left: usize, predicate: &P, acc: &mut S, index: usize, seg_left: usize, seg_right: usize) -> usize
    where
        P: Fn(&S) -> bool,
    {
        if seg_right <= left {
            return left;
        }
        if left <= seg_left {
            let next = min(*acc, self.nodes[index].min1);
            if predicate(&next) {
                *acc = next;
                return seg_right;
            }
        }
        if seg_right - seg_left == 1 {
            return seg_left;
        }
        self.push(index, seg_right - seg_left);
        let mid = (seg_left + seg_right) >> 1;
        let res_left = self.max_right_min_internal(left, predicate, acc, index << 1, seg_left, mid);
        if res_left < mid {
            return res_left;
        }
        self.max_right_min_internal(left, predicate, acc, index << 1 | 1, mid, seg_right)
    }

    fn max_right_max_internal<P>(&mut self, left: usize, predicate: &P, acc: &mut S, index: usize, seg_left: usize, seg_right: usize) -> usize
    where
        P: Fn(&S) -> bool,
    {
        if seg_right <= left {
            return left;
        }
        if left <= seg_left {
            let next = max(*acc, self.nodes[index].max1);
            if predicate(&next) {
                *acc = next;
                return seg_right;
            }
        }
        if seg_right - seg_left == 1 {
            return seg_left;
        }
        self.push(index, seg_right - seg_left);
        let mid = (seg_left + seg_right) >> 1;
        let res_left = self.max_right_max_internal(left, predicate, acc, index << 1, seg_left, mid);
        if res_left < mid {
            return res_left;
        }
        self.max_right_max_internal(left, predicate, acc, index << 1 | 1, mid, seg_right)
    }

    fn min_left_sum_internal<P>(&mut self, right: usize, predicate: &P, acc: &mut T, index: usize, seg_left: usize, seg_right: usize) -> usize
    where
        P: Fn(&T) -> bool,
    {
        if right <= seg_left {
            return right;
        }
        if seg_right <= right {
            let next = self.nodes[index].sum + *acc;
            if predicate(&next) {
                *acc = next;
                return seg_left;
            }
        }
        if seg_right - seg_left == 1 {
            return seg_right;
        }
        self.push(index, seg_right - seg_left);
        let mid = (seg_left + seg_right) >> 1;
        let res_right = self.min_left_sum_internal(right, predicate, acc, index << 1 | 1, mid, seg_right);
        if res_right > mid {
            return res_right;
        }
        self.min_left_sum_internal(right, predicate, acc, index << 1, seg_left, mid)
    }

    fn min_left_min_internal<P>(&mut self, right: usize, predicate: &P, acc: &mut S, index: usize, seg_left: usize, seg_right: usize) -> usize
    where
        P: Fn(&S) -> bool,
    {
        if right <= seg_left {
            return right;
        }
        if seg_right <= right {
            let next = min(self.nodes[index].min1, *acc);
            if predicate(&next) {
                *acc = next;
                return seg_left;
            }
        }
        if seg_right - seg_left == 1 {
            return seg_right;
        }
        self.push(index, seg_right - seg_left);
        let mid = (seg_left + seg_right) >> 1;
        let res_right = self.min_left_min_internal(right, predicate, acc, index << 1 | 1, mid, seg_right);
        if res_right > mid {
            return res_right;
        }
        self.min_left_min_internal(right, predicate, acc, index << 1, seg_left, mid)
    }

    fn min_left_max_internal<P>(&mut self, right: usize, predicate: &P, acc: &mut S, index: usize, seg_left: usize, seg_right: usize) -> usize
    where
        P: Fn(&S) -> bool,
    {
        if right <= seg_left {
            return right;
        }
        if seg_right <= right {
            let next = max(self.nodes[index].max1, *acc);
            if predicate(&next) {
                *acc = next;
                return seg_left;
            }
        }
        if seg_right - seg_left == 1 {
            return seg_right;
        }
        self.push(index, seg_right - seg_left);
        let mid = (seg_left + seg_right) >> 1;
        let res_right = self.min_left_max_internal(right, predicate, acc, index << 1 | 1, mid, seg_right);
        if res_right > mid {
            return res_right;
        }
        self.min_left_max_internal(right, predicate, acc, index << 1, seg_left, mid)
    }
}


#[cfg(test)]
mod tests {
    use super::{QueryKind, SegmentTreeBeats};
    use std::cmp::{max, min};

    type Value = i64;

    type Tree = SegmentTreeBeats<Value, Value>;

    fn build_tree(values: &[Value]) -> Tree {
        SegmentTreeBeats::from_slice(values)
    }

    fn naive_sum(values: &[Value], left: usize, right: usize) -> Value {
        values[left..right].iter().copied().sum::<Value>()
    }

    fn naive_min(values: &[Value], left: usize, right: usize) -> Value {
        if left == right {
            Value::MAX
        } else {
            values[left..right]
                .iter()
                .copied()
                .min()
                .unwrap_or(Value::MAX)
        }
    }

    fn naive_max(values: &[Value], left: usize, right: usize) -> Value {
        if left == right {
            Value::MIN
        } else {
            values[left..right]
                .iter()
                .copied()
                .max()
                .unwrap_or(Value::MIN)
        }
    }

    fn apply_naive_add(values: &mut [Value], left: usize, right: usize, delta: Value) {
        for index in left..right {
            values[index] += delta;
        }
    }

    fn apply_naive_chmin(values: &mut [Value], left: usize, right: usize, threshold: Value) {
        for index in left..right {
            if threshold < values[index] {
                values[index] = threshold;
            }
        }
    }

    fn apply_naive_chmax(values: &mut [Value], left: usize, right: usize, threshold: Value) {
        for index in left..right {
            if values[index] < threshold {
                values[index] = threshold;
            }
        }
    }

    fn apply_search_consistency_updates(tree: &mut Tree, naive_values: &mut [Value]) {
        let add_left = 1usize;
        let add_right = 5usize;
        let add_value = 3i64;
        let chmin_left = 0usize;
        let chmin_right = 4usize;
        let chmin_value = 6i64;
        let chmax_left = 2usize;
        let chmax_right = 6usize;
        let chmax_value = 0i64;

        tree.range_add(add_left, add_right, add_value);
        apply_naive_add(naive_values, add_left, add_right, add_value);
        tree.range_chmin(chmin_left, chmin_right, chmin_value);
        apply_naive_chmin(naive_values, chmin_left, chmin_right, chmin_value);
        tree.range_chmax(chmax_left, chmax_right, chmax_value);
        apply_naive_chmax(naive_values, chmax_left, chmax_right, chmax_value);
    }

    fn naive_max_right_sum<P>(values: &[Value], left: usize, predicate: P) -> Option<usize>
    where
        P: Fn(&Value) -> bool,
    {
        if values.len() < left {
            return None;
        }
        if left == values.len() {
            return Some(values.len());
        }
        if !predicate(&0) {
            return Some(left);
        }
        let mut acc = 0;
        let mut position = left;
        while position < values.len() {
            let next = acc + values[position];
            if !predicate(&next) {
                return Some(position);
            }
            acc = next;
            position += 1;
        }
        Some(values.len())
    }

    fn naive_min_left_sum<P>(values: &[Value], right: usize, predicate: P) -> Option<usize>
    where
        P: Fn(&Value) -> bool,
    {
        if values.len() < right {
            return None;
        }
        if right == 0 {
            return Some(0);
        }
        if !predicate(&0) {
            return Some(right);
        }
        let mut acc = 0;
        let mut position = right;
        while 0 < position {
            let next = values[position - 1] + acc;
            if !predicate(&next) {
                return Some(position);
            }
            acc = next;
            position -= 1;
        }
        Some(0)
    }

    fn naive_max_right_min<P>(values: &[Value], left: usize, predicate: P) -> Option<usize>
    where
        P: Fn(&Value) -> bool,
    {
        if values.len() < left {
            return None;
        }
        if left == values.len() {
            return Some(values.len());
        }
        if !predicate(&Value::MAX) {
            return Some(left);
        }
        let mut acc = Value::MAX;
        let mut position = left;
        while position < values.len() {
            let next = min(acc, values[position]);
            if !predicate(&next) {
                return Some(position);
            }
            acc = next;
            position += 1;
        }
        Some(values.len())
    }

    fn naive_min_left_min<P>(values: &[Value], right: usize, predicate: P) -> Option<usize>
    where
        P: Fn(&Value) -> bool,
    {
        if values.len() < right {
            return None;
        }
        if right == 0 {
            return Some(0);
        }
        if !predicate(&Value::MAX) {
            return Some(right);
        }
        let mut acc = Value::MAX;
        let mut position = right;
        while 0 < position {
            let next = min(values[position - 1], acc);
            if !predicate(&next) {
                return Some(position);
            }
            acc = next;
            position -= 1;
        }
        Some(0)
    }

    fn naive_max_right_max<P>(values: &[Value], left: usize, predicate: P) -> Option<usize>
    where
        P: Fn(&Value) -> bool,
    {
        if values.len() < left {
            return None;
        }
        if left == values.len() {
            return Some(values.len());
        }
        if !predicate(&Value::MIN) {
            return Some(left);
        }
        let mut acc = Value::MIN;
        let mut position = left;
        while position < values.len() {
            let next = max(acc, values[position]);
            if !predicate(&next) {
                return Some(position);
            }
            acc = next;
            position += 1;
        }
        Some(values.len())
    }

    fn naive_min_left_max<P>(values: &[Value], right: usize, predicate: P) -> Option<usize>
    where
        P: Fn(&Value) -> bool,
    {
        if values.len() < right {
            return None;
        }
        if right == 0 {
            return Some(0);
        }
        if !predicate(&Value::MIN) {
            return Some(right);
        }
        let mut acc = Value::MIN;
        let mut position = right;
        while 0 < position {
            let next = max(values[position - 1], acc);
            if !predicate(&next) {
                return Some(position);
            }
            acc = next;
            position -= 1;
        }
        Some(0)
    }

    fn next_random_u64(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *state
    }

    #[test]
    fn new_size_zero_len_is_zero_and_boundary_searches_do_not_panic() {
        // Arrange
        let mut tree = Tree::new(0);
        let left = 0usize;
        let right = 0usize;
        let sum_predicate = |value: &Value| *value == 0;
        let min_predicate = |value: &Value| *value == Value::MAX;
        let max_predicate = |value: &Value| *value == Value::MIN;
        let expected_len = 0usize;
        let expected_max_right_sum = Some(0usize);
        let expected_max_right_min = Some(0usize);
        let expected_max_right_max = Some(0usize);
        let expected_min_left_sum = Some(0usize);
        let expected_min_left_min = Some(0usize);
        let expected_min_left_max = Some(0usize);

        // Act
        let actual_len = tree.len();
        let actual_max_right_sum = tree.max_right_sum(left, sum_predicate);
        let actual_max_right_min = tree.max_right_min(left, min_predicate);
        let actual_max_right_max = tree.max_right_max(left, max_predicate);
        let actual_min_left_sum = tree.min_left_sum(right, sum_predicate);
        let actual_min_left_min = tree.min_left_min(right, min_predicate);
        let actual_min_left_max = tree.min_left_max(right, max_predicate);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_max_right_sum, expected_max_right_sum);
        assert_eq!(actual_max_right_min, expected_max_right_min);
        assert_eq!(actual_max_right_max, expected_max_right_max);
        assert_eq!(actual_min_left_sum, expected_min_left_sum);
        assert_eq!(actual_min_left_min, expected_min_left_min);
        assert_eq!(actual_min_left_max, expected_min_left_max);
    }

    #[test]
    fn range_add_with_zero_is_no_op() {
        // Arrange
        let initial_values = vec![-3i64, 0i64, 5i64];
        let mut tree = build_tree(&initial_values);
        let left = 0usize;
        let right = initial_values.len();
        let add_value = 0i64;
        let expected_sum = naive_sum(&initial_values, left, right);
        let expected_min = naive_min(&initial_values, left, right);
        let expected_max = naive_max(&initial_values, left, right);

        // Act
        tree.range_add(left, right, add_value);
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn query_sum_empty_range_returns_zero() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = 1usize;
        let right = 1usize;
        let expected_sum = 0i64;

        // Act
        let actual_sum = tree.query_sum(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn min_left_sum_returns_zero_when_right_is_zero() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let right = 0usize;
        let predicate = |sum: &Value| *sum <= 10i64;
        let expected = Some(0usize);

        // Act
        let actual = tree.min_left_sum(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_sum_returns_zero_when_predicate_is_always_true() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let right = values.len();
        let predicate = |_sum: &Value| true;
        let expected = Some(0usize);

        // Act
        let actual = tree.min_left_sum(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn new_size_zero_range_add_keeps_empty_query_contract() {
        // Arrange
        let mut tree = Tree::new(0);
        let update_left = 0usize;
        let update_right = 0usize;
        let update_value = 5i64;
        let query_left = 0usize;
        let query_right = 0usize;
        let expected_sum = 0i64;
        let expected_min = Value::MAX;
        let expected_max = Value::MIN;

        // Act
        tree.range_add(update_left, update_right, update_value);
        let actual_sum = tree.query_sum(query_left, query_right);
        let actual_min = tree.query_min(query_left, query_right);
        let actual_max = tree.query_max(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn new_size_zero_range_chmin_keeps_empty_query_contract() {
        // Arrange
        let mut tree = Tree::new(0);
        let update_left = 0usize;
        let update_right = 0usize;
        let update_value = -3i64;
        let query_left = 0usize;
        let query_right = 0usize;
        let expected_sum = 0i64;
        let expected_min = Value::MAX;
        let expected_max = Value::MIN;

        // Act
        tree.range_chmin(update_left, update_right, update_value);
        let actual_sum = tree.query_sum(query_left, query_right);
        let actual_min = tree.query_min(query_left, query_right);
        let actual_max = tree.query_max(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn new_size_zero_range_chmax_keeps_empty_query_contract() {
        // Arrange
        let mut tree = Tree::new(0);
        let update_left = 0usize;
        let update_right = 0usize;
        let update_value = 9i64;
        let query_left = 0usize;
        let query_right = 0usize;
        let expected_sum = 0i64;
        let expected_min = Value::MAX;
        let expected_max = Value::MIN;

        // Act
        tree.range_chmax(update_left, update_right, update_value);
        let actual_sum = tree.query_sum(query_left, query_right);
        let actual_min = tree.query_min(query_left, query_right);
        let actual_max = tree.query_max(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn max_right_min_with_left_greater_than_zero_returns_first_failing_position() {
        // Arrange
        let values = vec![9i64, 7i64, 6i64, 4i64, 8i64];
        let mut tree = build_tree(&values);
        let left = 1usize;
        let predicate = |minimum: &Value| *minimum >= 5i64;
        let expected = naive_max_right_min(&values, left, predicate);

        // Act
        let actual = tree.max_right_min(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_max_with_left_greater_than_zero_returns_first_failing_position() {
        // Arrange
        let values = vec![1i64, 4i64, 3i64, 6i64, 2i64];
        let mut tree = build_tree(&values);
        let left = 1usize;
        let predicate = |maximum: &Value| *maximum <= 4i64;
        let expected = naive_max_right_max(&values, left, predicate);

        // Act
        let actual = tree.max_right_max(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_min_returns_zero_when_right_is_zero() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let right = 0usize;
        let predicate = |minimum: &Value| *minimum >= 0i64;
        let expected = Some(0usize);

        // Act
        let actual = tree.min_left_min(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_min_returns_zero_when_predicate_is_always_true() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let right = values.len();
        let predicate = |_minimum: &Value| true;
        let expected = Some(0usize);

        // Act
        let actual = tree.min_left_min(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_max_returns_zero_when_right_is_zero() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let right = 0usize;
        let predicate = |maximum: &Value| *maximum <= 10i64;
        let expected = Some(0usize);

        // Act
        let actual = tree.min_left_max(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_max_returns_zero_when_predicate_is_always_true() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let right = values.len();
        let predicate = |_maximum: &Value| true;
        let expected = Some(0usize);

        // Act
        let actual = tree.min_left_max(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn query_max_empty_range_returns_min_value() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = 0usize;
        let right = 0usize;
        let expected_max = Value::MIN;

        // Act
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn query_min_empty_range_returns_max_value() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = 2usize;
        let right = 2usize;
        let expected_min = Value::MAX;

        // Act
        let actual_min = tree.query_min(left, right);

        // Assert
        assert_eq!(actual_min, expected_min);
    }

    #[test]
    fn new_size_one_query_results_match_empty_node_contract() {
        // Arrange
        let mut tree = Tree::new(1);
        let left = 0usize;
        let right = 1usize;
        let expected_sum = 0i64;
        let expected_min = Value::MAX;
        let expected_max = Value::MIN;

        // Act
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn from_slice_empty_behaves_as_empty_tree() {
        // Arrange
        let values: Vec<Value> = vec![];
        let mut tree = build_tree(&values);
        let query_left = 0usize;
        let query_right = 0usize;
        let expected_sum = 0i64;
        let expected_min = Value::MAX;
        let expected_max = Value::MIN;

        // Act
        let actual_sum = tree.query_sum(query_left, query_right);
        let actual_min = tree.query_min(query_left, query_right);
        let actual_max = tree.query_max(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn range_add_empty_range_is_no_op() {
        // Arrange
        let initial_values = vec![2i64, 4i64, 6i64];
        let mut tree = build_tree(&initial_values);
        let left = 1usize;
        let right = 1usize;
        let add_value = 100i64;
        let expected_sum = naive_sum(&initial_values, 0, initial_values.len());
        let expected_min = naive_min(&initial_values, 0, initial_values.len());
        let expected_max = naive_max(&initial_values, 0, initial_values.len());

        let query_sum_left = 0;
        let query_min_left = 0;
        let query_max_left = 0;

        // Act
        tree.range_add(left, right, add_value);
        let actual_sum = tree.query_sum(query_sum_left, initial_values.len());
        let actual_min = tree.query_min(query_min_left, initial_values.len());
        let actual_max = tree.query_max(query_max_left, initial_values.len());

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn range_chmin_empty_range_is_no_op() {
        // Arrange
        let initial_values = vec![4i64, 6i64, 8i64];
        let mut tree = build_tree(&initial_values);
        let left = 2usize;
        let right = 2usize;
        let threshold = 0i64;
        let expected_sum = naive_sum(&initial_values, 0, initial_values.len());
        let expected_min = naive_min(&initial_values, 0, initial_values.len());
        let expected_max = naive_max(&initial_values, 0, initial_values.len());

        let query_sum_left = 0;
        let query_min_left = 0;
        let query_max_left = 0;

        // Act
        tree.range_chmin(left, right, threshold);
        let actual_sum = tree.query_sum(query_sum_left, initial_values.len());
        let actual_min = tree.query_min(query_min_left, initial_values.len());
        let actual_max = tree.query_max(query_max_left, initial_values.len());

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn range_chmax_empty_range_is_no_op() {
        // Arrange
        let initial_values = vec![4i64, 6i64, 8i64];
        let mut tree = build_tree(&initial_values);
        let left = 2usize;
        let right = 2usize;
        let threshold = 100i64;
        let expected_sum = naive_sum(&initial_values, 0, initial_values.len());
        let expected_min = naive_min(&initial_values, 0, initial_values.len());
        let expected_max = naive_max(&initial_values, 0, initial_values.len());

        let query_sum_left = 0;
        let query_min_left = 0;
        let query_max_left = 0;

        // Act
        tree.range_chmax(left, right, threshold);
        let actual_sum = tree.query_sum(query_sum_left, initial_values.len());
        let actual_min = tree.query_min(query_min_left, initial_values.len());
        let actual_max = tree.query_max(query_max_left, initial_values.len());

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn new_with_non_power_of_two_size_preserves_len() {
        // Arrange
        let size = 3usize;
        let tree = Tree::new(size);
        let expected_len = 3usize;

        // Act
        let actual_len = tree.len();

        // Assert
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn from_slice_single_element_query_values_equal_element() {
        // Arrange
        let single_value = 42i64;
        let values = vec![single_value];
        let mut tree = build_tree(&values);
        let left = 0usize;
        let right = 1usize;
        let expected_sum = single_value;
        let expected_min = single_value;
        let expected_max = single_value;

        // Act
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn from_slice_non_power_of_two_values_full_range_matches_naive_aggregation() {
        // Arrange
        let values = vec![3i64, -1i64, 4i64];
        let mut tree = build_tree(&values);
        let left = 0usize;
        let right = values.len();

        let expected_sum = naive_sum(&values, left, right);

        let expected_min = naive_min(&values, left, right);

        let expected_max = naive_max(&values, left, right);

        // Act
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn from_slice_with_duplicates_reflects_duplicate_counts_in_queries() {
        // Arrange
        let values = vec![5i64, 5i64, 5i64, 5i64];
        let mut tree = build_tree(&values);
        let left = 0usize;
        let right = values.len();
        let expected_sum = 20i64;
        let expected_min = 5i64;
        let expected_max = 5i64;

        // Act
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn from_slice_with_i64_bounds_handles_extreme_values() {
        // Arrange
        let lower_bound = Value::MIN;
        let upper_bound = Value::MAX;
        let values = vec![lower_bound, upper_bound];
        let mut tree = build_tree(&values);
        let left = 0usize;
        let right = values.len();
        let expected_sum = -1i64;
        let expected_min = lower_bound;
        let expected_max = upper_bound;

        // Act
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn max_right_sum_returns_none_when_left_is_greater_than_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = values.len() + 1usize;
        let predicate = |sum: &Value| *sum <= 10i64;
        let expected = None;

        // Act
        let actual = tree.max_right_sum(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_sum_returns_len_when_left_equals_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = values.len();
        let predicate = |sum: &Value| *sum <= 10i64;
        let expected = Some(values.len());

        // Act
        let actual = tree.max_right_sum(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_sum_returns_len_when_predicate_is_always_true() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = 0usize;
        let predicate = |_sum: &Value| true;
        let expected = Some(values.len());

        // Act
        let actual = tree.max_right_sum(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_sum_returns_none_when_right_is_greater_than_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let right = values.len() + 1usize;
        let predicate = |sum: &Value| *sum <= 10i64;
        let expected = None;

        // Act
        let actual = tree.min_left_sum(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_min_returns_none_when_left_is_greater_than_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = values.len() + 1usize;
        let predicate = |minimum: &Value| *minimum >= 0i64;
        let expected = None;

        // Act
        let actual = tree.max_right_min(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_min_returns_len_when_left_equals_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = values.len();
        let predicate = |minimum: &Value| *minimum >= 0i64;
        let expected = Some(values.len());

        // Act
        let actual = tree.max_right_min(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_min_returns_len_when_predicate_is_always_true() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = 0usize;
        let predicate = |_minimum: &Value| true;
        let expected = Some(values.len());

        // Act
        let actual = tree.max_right_min(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_max_returns_none_when_left_is_greater_than_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = values.len() + 1usize;
        let predicate = |maximum: &Value| *maximum <= 10i64;
        let expected = None;

        // Act
        let actual = tree.max_right_max(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_max_returns_len_when_left_equals_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = values.len();
        let predicate = |maximum: &Value| *maximum <= 10i64;
        let expected = Some(values.len());

        // Act
        let actual = tree.max_right_max(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_max_returns_len_when_predicate_is_always_true() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = 0usize;
        let predicate = |_maximum: &Value| true;
        let expected = Some(values.len());

        // Act
        let actual = tree.max_right_max(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_min_returns_none_when_right_is_greater_than_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let right = values.len() + 1usize;
        let predicate = |minimum: &Value| *minimum >= 0i64;
        let expected = None;

        // Act
        let actual = tree.min_left_min(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_min_with_right_less_than_len_returns_first_failing_position() {
        // Arrange
        let values = vec![8i64, 6i64, 7i64, 9i64, 5i64];
        let mut tree = build_tree(&values);
        let right = 4usize;
        let predicate = |minimum: &Value| *minimum >= 7i64;
        let expected = naive_min_left_min(&values, right, predicate);

        // Act
        let actual = tree.min_left_min(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_max_returns_none_when_right_is_greater_than_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let right = values.len() + 1usize;
        let predicate = |maximum: &Value| *maximum <= 10i64;
        let expected = None;

        // Act
        let actual = tree.min_left_max(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_max_with_right_less_than_len_returns_first_failing_position() {
        // Arrange
        let values = vec![1i64, 4i64, 3i64, 2i64, 10i64];
        let mut tree = build_tree(&values);
        let right = 4usize;
        let predicate = |maximum: &Value| *maximum <= 3i64;
        let expected = naive_min_left_max(&values, right, predicate);

        // Act
        let actual = tree.min_left_max(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn range_add_panics_when_right_greater_than_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);

        let left = 0;
        let right = 4;
        let update_value = 1;

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tree.range_add(left, right, update_value)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn range_chmin_panics_when_right_greater_than_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);

        let left = 0;
        let right = 4;
        let update_value = 1;

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tree.range_chmin(left, right, update_value)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn range_chmax_panics_when_right_greater_than_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);

        let left = 0;
        let right = 4;
        let update_value = 1;

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tree.range_chmax(left, right, update_value)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn query_sum_panics_when_right_greater_than_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);

        let left = 0;
        let right = 4;

        // Act
        let actual_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tree.query_sum(left, right)));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn query_min_panics_when_right_greater_than_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);

        let left = 0;
        let right = 4;

        // Act
        let actual_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tree.query_min(left, right)));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn query_max_panics_when_right_greater_than_len() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);

        let left = 0;
        let right = 4;

        // Act
        let actual_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tree.query_max(left, right)));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn range_add_full_range_updates_sum_min_and_max_uniformly() {
        // Arrange
        let initial_values = vec![1i64, 2i64, 3i64, 4i64];
        let mut tree = build_tree(&initial_values);
        let mut naive_values = initial_values.clone();
        let left = 0usize;
        let right = naive_values.len();
        let add_value = 3i64;
        apply_naive_add(&mut naive_values, left, right, add_value);

        let query_left = left;
        let query_right = right;
        let expected_queries = (
            naive_sum(&naive_values, query_left, query_right),
            naive_min(&naive_values, query_left, query_right),
            naive_max(&naive_values, query_left, query_right),
        );

        // Act
        tree.range_add(left, right, add_value);

        let actual_queries = (
            tree.query_sum(query_left, query_right),
            tree.query_min(query_left, query_right),
            tree.query_max(query_left, query_right),
        );

        // Assert
        assert_eq!(actual_queries, expected_queries);
    }

    #[test]
    fn range_add_partial_range_does_not_modify_outside_elements() {
        // Arrange
        let initial_values = vec![10i64, 20i64, 30i64, 40i64];
        let mut tree = build_tree(&initial_values);
        let update_left = 1usize;
        let update_right = 3usize;
        let add_value = 5i64;
        let left_outside_left = 0usize;
        let left_outside_right = 1usize;
        let right_outside_left = 3usize;
        let right_outside_right = 4usize;
        let expected_left_outside_sum = 10i64;
        let expected_right_outside_sum = 40i64;

        // Act
        tree.range_add(update_left, update_right, add_value);
        let actual_left_outside_sum = tree.query_sum(left_outside_left, left_outside_right);
        let actual_right_outside_sum = tree.query_sum(right_outside_left, right_outside_right);

        // Assert
        assert_eq!(actual_left_outside_sum, expected_left_outside_sum);
        assert_eq!(actual_right_outside_sum, expected_right_outside_sum);
    }

    #[test]
    fn range_add_point_update_changes_single_element_only() {
        // Arrange
        let initial_values = vec![7i64, 8i64, 9i64];
        let mut tree = build_tree(&initial_values);
        let point_index = 1usize;
        let left = point_index;
        let right = point_index + 1usize;
        let add_value = 4i64;
        let expected_point_value = 12i64;
        let expected_other_left = 7i64;
        let expected_other_right = 9i64;

        let query_sum_left = 0;
        let query_sum_right = 1;
        let query_sum_left_2 = 2;
        let query_sum_right_2 = 3;

        // Act
        tree.range_add(left, right, add_value);
        let actual_point_value = tree.query_sum(left, right);
        let actual_other_left = tree.query_sum(query_sum_left, query_sum_right);
        let actual_other_right = tree.query_sum(query_sum_left_2, query_sum_right_2);

        // Assert
        assert_eq!(actual_point_value, expected_point_value);
        assert_eq!(actual_other_left, expected_other_left);
        assert_eq!(actual_other_right, expected_other_right);
    }

    #[test]
    fn range_add_negative_value_decreases_sum_min_and_max() {
        // Arrange
        let initial_values = vec![9i64, 7i64, 11i64];
        let mut tree = build_tree(&initial_values);
        let mut naive_values = initial_values.clone();
        let left = 0usize;
        let right = 3usize;
        let add_value = -4i64;
        apply_naive_add(&mut naive_values, left, right, add_value);

        let query_left = left;
        let query_right = right;
        let expected_queries = (
            naive_sum(&naive_values, query_left, query_right),
            naive_min(&naive_values, query_left, query_right),
            naive_max(&naive_values, query_left, query_right),
        );

        // Act
        tree.range_add(left, right, add_value);

        let actual_queries = (
            tree.query_sum(query_left, query_right),
            tree.query_min(query_left, query_right),
            tree.query_max(query_left, query_right),
        );

        // Assert
        assert_eq!(actual_queries, expected_queries);
    }

    #[test]
    fn range_add_multiple_updates_compose_additively() {
        // Arrange
        let initial_values = vec![1i64, 2i64, 3i64, 4i64];
        let mut tree = build_tree(&initial_values);
        let mut naive_values = initial_values.clone();
        let left = 1usize;
        let right = 4usize;
        let first_add = 5i64;
        let second_add = -2i64;
        apply_naive_add(&mut naive_values, left, right, first_add);
        apply_naive_add(&mut naive_values, left, right, second_add);

        let query_left = 0;
        let query_right = naive_values.len();
        let expected_queries = (
            naive_sum(&naive_values, query_left, query_right),
            naive_min(&naive_values, query_left, query_right),
            naive_max(&naive_values, query_left, query_right),
        );

        // Act
        tree.range_add(left, right, first_add);
        tree.range_add(left, right, second_add);

        let actual_queries = (
            tree.query_sum(query_left, query_right),
            tree.query_min(query_left, query_right),
            tree.query_max(query_left, query_right),
        );

        // Assert
        assert_eq!(actual_queries, expected_queries);
    }

    #[test]
    fn range_add_then_range_chmin_matches_naive_simulation() {
        // Arrange
        let initial_values = vec![2i64, 7i64, 1i64, 9i64];
        let mut tree = build_tree(&initial_values);
        let mut naive_values = initial_values.clone();
        let add_left = 0usize;
        let add_right = 3usize;
        let add_value = 4i64;
        let chmin_left = 1usize;
        let chmin_right = 4usize;
        let chmin_value = 8i64;
        apply_naive_add(&mut naive_values, add_left, add_right, add_value);
        apply_naive_chmin(&mut naive_values, chmin_left, chmin_right, chmin_value);

        let query_left = 0;
        let query_right = naive_values.len();
        let expected_queries = (
            naive_sum(&naive_values, query_left, query_right),
            naive_min(&naive_values, query_left, query_right),
            naive_max(&naive_values, query_left, query_right),
        );

        // Act
        tree.range_add(add_left, add_right, add_value);
        tree.range_chmin(chmin_left, chmin_right, chmin_value);

        let actual_queries = (
            tree.query_sum(query_left, query_right),
            tree.query_min(query_left, query_right),
            tree.query_max(query_left, query_right),
        );

        // Assert
        assert_eq!(actual_queries, expected_queries);
    }

    #[test]
    fn range_add_then_range_chmax_matches_naive_simulation() {
        // Arrange
        let initial_values = vec![2i64, 7i64, 1i64, 9i64];
        let mut tree = build_tree(&initial_values);
        let mut naive_values = initial_values.clone();
        let add_left = 0usize;
        let add_right = 4usize;
        let add_value = -3i64;
        let chmax_left = 1usize;
        let chmax_right = 3usize;
        let chmax_value = 0i64;
        apply_naive_add(&mut naive_values, add_left, add_right, add_value);
        apply_naive_chmax(&mut naive_values, chmax_left, chmax_right, chmax_value);

        let query_left = 0;
        let query_right = naive_values.len();
        let expected_queries = (
            naive_sum(&naive_values, query_left, query_right),
            naive_min(&naive_values, query_left, query_right),
            naive_max(&naive_values, query_left, query_right),
        );

        // Act
        tree.range_add(add_left, add_right, add_value);
        tree.range_chmax(chmax_left, chmax_right, chmax_value);

        let actual_queries = (
            tree.query_sum(query_left, query_right),
            tree.query_min(query_left, query_right),
            tree.query_max(query_left, query_right),
        );

        // Assert
        assert_eq!(actual_queries, expected_queries);
    }

    #[test]
    fn range_add_panics_when_left_greater_than_right() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);

        let left = 2;
        let right = 1;
        let update_value = 1;

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tree.range_add(left, right, update_value)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn range_chmin_full_range_clamps_only_values_greater_than_threshold() {
        // Arrange
        let initial_values = vec![1i64, 6i64, 3i64, 8i64];
        let mut tree = build_tree(&initial_values);
        let mut naive_values = initial_values.clone();
        let left = 0usize;
        let right = naive_values.len();
        let threshold = 5i64;
        apply_naive_chmin(&mut naive_values, left, right, threshold);

        let query_left = left;
        let query_right = right;
        let expected_queries = (
            naive_sum(&naive_values, query_left, query_right),
            naive_min(&naive_values, query_left, query_right),
            naive_max(&naive_values, query_left, query_right),
        );

        // Act
        tree.range_chmin(left, right, threshold);

        let actual_queries = (
            tree.query_sum(query_left, query_right),
            tree.query_min(query_left, query_right),
            tree.query_max(query_left, query_right),
        );

        // Assert
        assert_eq!(actual_queries, expected_queries);
    }

    #[test]
    fn range_chmin_threshold_greater_than_current_max_is_no_op() {
        // Arrange
        let initial_values = vec![2i64, 4i64, 6i64];
        let mut tree = build_tree(&initial_values);
        let left = 0usize;
        let right = initial_values.len();
        let threshold = 10i64;
        let expected_sum = naive_sum(&initial_values, left, right);
        let expected_min = naive_min(&initial_values, left, right);
        let expected_max = naive_max(&initial_values, left, right);

        // Act
        tree.range_chmin(left, right, threshold);
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn range_chmin_threshold_equal_to_current_max_is_no_op() {
        // Arrange
        let initial_values = vec![2i64, 4i64, 6i64];
        let mut tree = build_tree(&initial_values);
        let left = 0usize;
        let right = initial_values.len();
        let threshold = 6i64;
        let expected_sum = naive_sum(&initial_values, left, right);
        let expected_min = naive_min(&initial_values, left, right);
        let expected_max = naive_max(&initial_values, left, right);

        // Act
        tree.range_chmin(left, right, threshold);
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn range_chmin_threshold_less_than_current_min_sets_all_to_threshold() {
        // Arrange
        let initial_values = vec![7i64, 9i64, 11i64];
        let mut tree = build_tree(&initial_values);
        let left = 0usize;
        let right = initial_values.len();
        let threshold = 3i64;
        let expected_sum = 9i64;
        let expected_min = threshold;
        let expected_max = threshold;

        // Act
        tree.range_chmin(left, right, threshold);
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn range_chmin_partial_range_does_not_modify_outside_elements() {
        // Arrange
        let initial_values = vec![1i64, 10i64, 7i64, 8i64];
        let mut tree = build_tree(&initial_values);
        let left = 1usize;
        let right = 3usize;
        let threshold = 6i64;
        let expected_left_outside = 1i64;
        let expected_right_outside = 8i64;

        let query_sum_left = 0;
        let query_sum_right = 1;
        let query_sum_left_2 = 3;
        let query_sum_right_2 = 4;

        // Act
        tree.range_chmin(left, right, threshold);
        let actual_left_outside = tree.query_sum(query_sum_left, query_sum_right);
        let actual_right_outside = tree.query_sum(query_sum_left_2, query_sum_right_2);

        // Assert
        assert_eq!(actual_left_outside, expected_left_outside);
        assert_eq!(actual_right_outside, expected_right_outside);
    }

    #[test]
    fn range_chmin_point_range_updates_single_element() {
        // Arrange
        let initial_values = vec![3i64, 9i64, 5i64];
        let mut tree = build_tree(&initial_values);
        let point_left = 1usize;
        let point_right = 2usize;
        let threshold = 4i64;
        let expected_point = 4i64;
        let expected_left = 3i64;
        let expected_right = 5i64;

        let query_sum_left = 0;
        let query_sum_right = 1;
        let query_sum_left_2 = 2;
        let query_sum_right_2 = 3;

        // Act
        tree.range_chmin(point_left, point_right, threshold);
        let actual_point = tree.query_sum(point_left, point_right);
        let actual_left = tree.query_sum(query_sum_left, query_sum_right);
        let actual_right = tree.query_sum(query_sum_left_2, query_sum_right_2);

        // Assert
        assert_eq!(actual_point, expected_point);
        assert_eq!(actual_left, expected_left);
        assert_eq!(actual_right, expected_right);
    }

    #[test]
    fn range_chmin_same_threshold_twice_is_idempotent() {
        // Arrange
        let initial_values = vec![9i64, 3i64, 7i64, 6i64];
        let mut tree = build_tree(&initial_values);
        let left = 0usize;
        let right = initial_values.len();
        let threshold = 5i64;

        // Act
        tree.range_chmin(left, right, threshold);
        let first_sum = tree.query_sum(left, right);
        let first_min = tree.query_min(left, right);
        let first_max = tree.query_max(left, right);
        tree.range_chmin(left, right, threshold);
        let second_sum = tree.query_sum(left, right);
        let second_min = tree.query_min(left, right);
        let second_max = tree.query_max(left, right);

        // Assert
        assert_eq!(second_sum, first_sum);
        assert_eq!(second_min, first_min);
        assert_eq!(second_max, first_max);
    }

    #[test]
    fn range_chmin_then_range_chmax_matches_naive_simulation() {
        // Arrange
        let initial_values = vec![10i64, 3i64, 8i64, 1i64];
        let mut tree = build_tree(&initial_values);
        let mut naive_values = initial_values.clone();
        let chmin_left = 0usize;
        let chmin_right = 3usize;
        let chmin_value = 6i64;
        let chmax_left = 1usize;
        let chmax_right = 4usize;
        let chmax_value = 4i64;
        apply_naive_chmin(&mut naive_values, chmin_left, chmin_right, chmin_value);
        apply_naive_chmax(&mut naive_values, chmax_left, chmax_right, chmax_value);

        let query_left = 0;
        let query_right = naive_values.len();
        let expected_queries = (
            naive_sum(&naive_values, query_left, query_right),
            naive_min(&naive_values, query_left, query_right),
            naive_max(&naive_values, query_left, query_right),
        );

        // Act
        tree.range_chmin(chmin_left, chmin_right, chmin_value);
        tree.range_chmax(chmax_left, chmax_right, chmax_value);

        let actual_queries = (
            tree.query_sum(query_left, query_right),
            tree.query_min(query_left, query_right),
            tree.query_max(query_left, query_right),
        );

        // Assert
        assert_eq!(actual_queries, expected_queries);
    }

    #[test]
    fn range_chmax_then_range_chmin_matches_naive_simulation() {
        // Arrange
        let initial_values = vec![10i64, 3i64, 8i64, 1i64];
        let mut tree = build_tree(&initial_values);
        let mut naive_values = initial_values.clone();
        let chmax_left = 0usize;
        let chmax_right = 4usize;
        let chmax_value = 5i64;
        let chmin_left = 1usize;
        let chmin_right = 3usize;
        let chmin_value = 6i64;
        apply_naive_chmax(&mut naive_values, chmax_left, chmax_right, chmax_value);
        apply_naive_chmin(&mut naive_values, chmin_left, chmin_right, chmin_value);

        let query_left = 0;
        let query_right = naive_values.len();
        let expected_queries = (
            naive_sum(&naive_values, query_left, query_right),
            naive_min(&naive_values, query_left, query_right),
            naive_max(&naive_values, query_left, query_right),
        );

        // Act
        tree.range_chmax(chmax_left, chmax_right, chmax_value);
        tree.range_chmin(chmin_left, chmin_right, chmin_value);

        let actual_queries = (
            tree.query_sum(query_left, query_right),
            tree.query_min(query_left, query_right),
            tree.query_max(query_left, query_right),
        );

        // Assert
        assert_eq!(actual_queries, expected_queries);
    }

    #[test]
    fn range_chmin_panics_when_left_greater_than_right() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);

        let left = 2;
        let right = 1;
        let update_value = 1;

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tree.range_chmin(left, right, update_value)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn range_chmax_full_range_clamps_only_values_less_than_threshold() {
        // Arrange
        let initial_values = vec![1i64, 6i64, 3i64, 8i64];
        let mut tree = build_tree(&initial_values);
        let mut naive_values = initial_values.clone();
        let left = 0usize;
        let right = naive_values.len();
        let threshold = 5i64;
        apply_naive_chmax(&mut naive_values, left, right, threshold);

        let query_left = left;
        let query_right = right;
        let expected_queries = (
            naive_sum(&naive_values, query_left, query_right),
            naive_min(&naive_values, query_left, query_right),
            naive_max(&naive_values, query_left, query_right),
        );

        // Act
        tree.range_chmax(left, right, threshold);

        let actual_queries = (
            tree.query_sum(query_left, query_right),
            tree.query_min(query_left, query_right),
            tree.query_max(query_left, query_right),
        );

        // Assert
        assert_eq!(actual_queries, expected_queries);
    }

    #[test]
    fn range_chmax_threshold_less_than_current_min_is_no_op() {
        // Arrange
        let initial_values = vec![2i64, 4i64, 6i64];
        let mut tree = build_tree(&initial_values);
        let left = 0usize;
        let right = initial_values.len();
        let threshold = -3i64;
        let expected_sum = naive_sum(&initial_values, left, right);
        let expected_min = naive_min(&initial_values, left, right);
        let expected_max = naive_max(&initial_values, left, right);

        // Act
        tree.range_chmax(left, right, threshold);
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn range_chmax_threshold_equal_to_current_min_is_no_op() {
        // Arrange
        let initial_values = vec![2i64, 4i64, 6i64];
        let mut tree = build_tree(&initial_values);
        let left = 0usize;
        let right = initial_values.len();
        let threshold = 2i64;
        let expected_sum = naive_sum(&initial_values, left, right);
        let expected_min = naive_min(&initial_values, left, right);
        let expected_max = naive_max(&initial_values, left, right);

        // Act
        tree.range_chmax(left, right, threshold);
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn range_chmax_threshold_greater_than_current_max_sets_all_to_threshold() {
        // Arrange
        let initial_values = vec![1i64, 4i64, 6i64];
        let mut tree = build_tree(&initial_values);
        let left = 0usize;
        let right = initial_values.len();
        let threshold = 10i64;
        let expected_sum = 30i64;
        let expected_min = threshold;
        let expected_max = threshold;

        // Act
        tree.range_chmax(left, right, threshold);
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn range_chmax_partial_range_does_not_modify_outside_elements() {
        // Arrange
        let initial_values = vec![1i64, 2i64, 3i64, 4i64];
        let mut tree = build_tree(&initial_values);
        let left = 1usize;
        let right = 3usize;
        let threshold = 10i64;
        let expected_left_outside = 1i64;
        let expected_right_outside = 4i64;

        let query_sum_left = 0;
        let query_sum_right = 1;
        let query_sum_left_2 = 3;
        let query_sum_right_2 = 4;

        // Act
        tree.range_chmax(left, right, threshold);
        let actual_left_outside = tree.query_sum(query_sum_left, query_sum_right);
        let actual_right_outside = tree.query_sum(query_sum_left_2, query_sum_right_2);

        // Assert
        assert_eq!(actual_left_outside, expected_left_outside);
        assert_eq!(actual_right_outside, expected_right_outside);
    }

    #[test]
    fn range_chmax_point_range_updates_single_element() {
        // Arrange
        let initial_values = vec![3i64, 1i64, 5i64];
        let mut tree = build_tree(&initial_values);
        let point_left = 1usize;
        let point_right = 2usize;
        let threshold = 4i64;
        let expected_point = 4i64;
        let expected_left = 3i64;
        let expected_right = 5i64;

        let query_sum_left = 0;
        let query_sum_right = 1;
        let query_sum_left_2 = 2;
        let query_sum_right_2 = 3;

        // Act
        tree.range_chmax(point_left, point_right, threshold);
        let actual_point = tree.query_sum(point_left, point_right);
        let actual_left = tree.query_sum(query_sum_left, query_sum_right);
        let actual_right = tree.query_sum(query_sum_left_2, query_sum_right_2);

        // Assert
        assert_eq!(actual_point, expected_point);
        assert_eq!(actual_left, expected_left);
        assert_eq!(actual_right, expected_right);
    }

    #[test]
    fn range_chmax_same_threshold_twice_is_idempotent() {
        // Arrange
        let initial_values = vec![1i64, 5i64, 2i64, 8i64];
        let mut tree = build_tree(&initial_values);
        let left = 0usize;
        let right = initial_values.len();
        let threshold = 4i64;

        // Act
        tree.range_chmax(left, right, threshold);
        let first_sum = tree.query_sum(left, right);
        let first_min = tree.query_min(left, right);
        let first_max = tree.query_max(left, right);
        tree.range_chmax(left, right, threshold);
        let second_sum = tree.query_sum(left, right);
        let second_min = tree.query_min(left, right);
        let second_max = tree.query_max(left, right);

        // Assert
        assert_eq!(second_sum, first_sum);
        assert_eq!(second_min, first_min);
        assert_eq!(second_max, first_max);
    }

    #[test]
    fn range_chmax_panics_when_left_greater_than_right() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);

        let left = 2;
        let right = 1;
        let update_value = 1;

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tree.range_chmax(left, right, update_value)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn after_updates_query_sum_matches_element_wise_total() {
        // Arrange
        let mut naive_values = vec![4i64, -2i64, 9i64, 1i64, 7i64];
        let mut tree = build_tree(&naive_values);
        apply_naive_add(&mut naive_values, 0, 4, 3);
        apply_naive_chmin(&mut naive_values, 2, 5, 6);
        apply_naive_chmax(&mut naive_values, 1, 3, 0);
        let left = 0usize;
        let right = naive_values.len();
        tree.range_add(0, 4, 3);
        tree.range_chmin(2, 5, 6);
        tree.range_chmax(1, 3, 0);
        let expected_sum = naive_values.iter().copied().sum::<Value>();

        // Act
        let actual_sum = tree.query_sum(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn query_sum_panics_when_left_greater_than_right() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);

        let left = 2;
        let right = 1;

        // Act
        let actual_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tree.query_sum(left, right)));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn after_updates_query_min_is_not_greater_than_query_max() {
        // Arrange
        let values = vec![4i64, -2i64, 9i64, 1i64, 7i64];
        let mut tree = build_tree(&values);
        let left = 1usize;
        let right = 5usize;
        tree.range_add(0, 4, 3);
        tree.range_chmin(2, 5, 6);
        tree.range_chmax(1, 3, 0);

        // Act
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert!(actual_min <= actual_max);
    }

    #[test]
    fn query_min_panics_when_left_greater_than_right() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);

        let left = 2;
        let right = 1;

        // Act
        let actual_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tree.query_min(left, right)));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn query_max_panics_when_left_greater_than_right() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);

        let left = 2;
        let right = 1;

        // Act
        let actual_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tree.query_max(left, right)));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn max_right_sum_returns_left_when_predicate_is_false_on_identity() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = 1usize;
        let predicate = |sum: &Value| *sum < 0i64;
        let expected = Some(left);

        // Act
        let actual = tree.max_right_sum(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_sum_returns_first_failing_position() {
        // Arrange
        let values = vec![2i64, 4i64, 3i64, 1i64];
        let mut tree = build_tree(&values);
        let left = 0usize;
        let predicate = |sum: &Value| *sum <= 6i64;
        let expected = naive_max_right_sum(&values, left, predicate);

        // Act
        let actual = tree.max_right_sum(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_min_returns_correct_boundary_for_monotone_predicate() {
        // Arrange
        let values = vec![7i64, 5i64, 6i64, 3i64];
        let mut tree = build_tree(&values);
        let left = 0usize;
        let predicate = |minimum: &Value| *minimum >= 5i64;
        let expected = naive_max_right_min(&values, left, predicate);

        // Act
        let actual = tree.max_right_min(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_max_returns_correct_boundary_for_monotone_predicate() {
        // Arrange
        let values = vec![1i64, 3i64, 2i64, 5i64];
        let mut tree = build_tree(&values);
        let left = 0usize;
        let predicate = |maximum: &Value| *maximum <= 3i64;
        let expected = naive_max_right_max(&values, left, predicate);

        // Act
        let actual = tree.max_right_max(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_query_kind_variants_match_specialized_methods() {
        // Arrange
        let values = vec![2i64, 4i64, 3i64, 1i64];
        let mut tree_with_query_kind = build_tree(&values);
        let mut tree_with_specialized = build_tree(&values);
        let left = 0usize;
        let sum_predicate = |sum: &Value| *sum <= 6i64;
        let min_predicate = |minimum: &Value| *minimum >= 2i64;
        let max_predicate = |maximum: &Value| *maximum <= 4i64;

        // Act
        let actual_sum = tree_with_query_kind.max_right(left, QueryKind::Sum(&sum_predicate));
        let actual_min = tree_with_query_kind.max_right(left, QueryKind::Min(&min_predicate));
        let actual_max = tree_with_query_kind.max_right(left, QueryKind::Max(&max_predicate));
        let actual_specialized_sum = tree_with_specialized.max_right_sum(left, sum_predicate);
        let actual_specialized_min = tree_with_specialized.max_right_min(left, min_predicate);
        let actual_specialized_max = tree_with_specialized.max_right_max(left, max_predicate);

        // Assert
        assert_eq!(actual_sum, actual_specialized_sum);
        assert_eq!(actual_min, actual_specialized_min);
        assert_eq!(actual_max, actual_specialized_max);
    }

    #[test]
    fn max_right_min_returns_left_when_predicate_is_false_on_identity() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = 1usize;
        let predicate = |minimum: &Value| *minimum < Value::MAX;
        let expected = Some(left);

        // Act
        let actual = tree.max_right_min(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_right_max_returns_left_when_predicate_is_false_on_identity() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let left = 1usize;
        let predicate = |maximum: &Value| *maximum > Value::MIN;
        let expected = Some(left);

        // Act
        let actual = tree.max_right_max(left, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn mixed_updates_max_right_queries_match_naive_model() {
        // Arrange
        let initial_values = vec![5i64, -2i64, 7i64, 1i64, 4i64, -1i64];
        let mut tree = build_tree(&initial_values);
        let mut naive_values = initial_values.clone();
        apply_search_consistency_updates(&mut tree, &mut naive_values);
        let left = 0usize;
        let sum_predicate = |sum: &Value| *sum <= 12i64;
        let min_predicate = |minimum: &Value| *minimum >= 1i64;
        let max_predicate = |maximum: &Value| *maximum <= 6i64;
        let expected_sum = naive_max_right_sum(&naive_values, left, sum_predicate);
        let expected_min = naive_max_right_min(&naive_values, left, min_predicate);
        let expected_max = naive_max_right_max(&naive_values, left, max_predicate);

        // Act
        let actual_sum = tree.max_right_sum(left, sum_predicate);
        let actual_min = tree.max_right_min(left, min_predicate);
        let actual_max = tree.max_right_max(left, max_predicate);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn min_left_sum_returns_right_when_predicate_is_false_on_identity() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let right = 2usize;
        let predicate = |sum: &Value| *sum < 0i64;
        let expected = Some(right);

        // Act
        let actual = tree.min_left_sum(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_sum_returns_first_failing_position_from_right() {
        // Arrange
        let values = vec![2i64, 4i64, 3i64, 1i64];
        let mut tree = build_tree(&values);
        let right = values.len();
        let predicate = |sum: &Value| *sum <= 6i64;
        let expected = naive_min_left_sum(&values, right, predicate);

        // Act
        let actual = tree.min_left_sum(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_min_returns_correct_boundary_for_monotone_predicate() {
        // Arrange
        let values = vec![7i64, 5i64, 6i64, 3i64];
        let mut tree = build_tree(&values);
        let right = values.len();
        let predicate = |minimum: &Value| *minimum >= 5i64;
        let expected = naive_min_left_min(&values, right, predicate);

        // Act
        let actual = tree.min_left_min(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_max_returns_correct_boundary_for_monotone_predicate() {
        // Arrange
        let values = vec![1i64, 3i64, 2i64, 5i64];
        let mut tree = build_tree(&values);
        let right = values.len();
        let predicate = |maximum: &Value| *maximum <= 3i64;
        let expected = naive_min_left_max(&values, right, predicate);

        // Act
        let actual = tree.min_left_max(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_query_kind_variants_match_specialized_methods() {
        // Arrange
        let values = vec![2i64, 4i64, 3i64, 1i64];
        let mut tree_with_query_kind = build_tree(&values);
        let mut tree_with_specialized = build_tree(&values);
        let right = values.len();
        let sum_predicate = |sum: &Value| *sum <= 6i64;
        let min_predicate = |minimum: &Value| *minimum >= 2i64;
        let max_predicate = |maximum: &Value| *maximum <= 4i64;

        // Act
        let actual_sum = tree_with_query_kind.min_left(right, QueryKind::Sum(&sum_predicate));
        let actual_min = tree_with_query_kind.min_left(right, QueryKind::Min(&min_predicate));
        let actual_max = tree_with_query_kind.min_left(right, QueryKind::Max(&max_predicate));
        let actual_specialized_sum = tree_with_specialized.min_left_sum(right, sum_predicate);
        let actual_specialized_min = tree_with_specialized.min_left_min(right, min_predicate);
        let actual_specialized_max = tree_with_specialized.min_left_max(right, max_predicate);

        // Assert
        assert_eq!(actual_sum, actual_specialized_sum);
        assert_eq!(actual_min, actual_specialized_min);
        assert_eq!(actual_max, actual_specialized_max);
    }

    #[test]
    fn min_left_min_returns_right_when_predicate_is_false_on_identity() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let right = 2usize;
        let predicate = |minimum: &Value| *minimum < Value::MAX;
        let expected = Some(right);

        // Act
        let actual = tree.min_left_min(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_left_max_returns_right_when_predicate_is_false_on_identity() {
        // Arrange
        let values = vec![1i64, 2i64, 3i64];
        let mut tree = build_tree(&values);
        let right = 2usize;
        let predicate = |maximum: &Value| *maximum < Value::MIN;
        let expected = Some(right);

        // Act
        let actual = tree.min_left_max(right, predicate);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn mixed_updates_min_left_queries_match_naive_model() {
        // Arrange
        let initial_values = vec![5i64, -2i64, 7i64, 1i64, 4i64, -1i64];
        let mut tree = build_tree(&initial_values);
        let mut naive_values = initial_values.clone();
        apply_search_consistency_updates(&mut tree, &mut naive_values);
        let right = 4usize;
        let sum_predicate = |sum: &Value| *sum <= 10i64;
        let min_predicate = |minimum: &Value| *minimum >= 2i64;
        let max_predicate = |maximum: &Value| *maximum <= 6i64;
        let expected_sum = naive_min_left_sum(&naive_values, right, sum_predicate);
        let expected_min = naive_min_left_min(&naive_values, right, min_predicate);
        let expected_max = naive_min_left_max(&naive_values, right, max_predicate);

        // Act
        let actual_sum = tree.min_left_sum(right, sum_predicate);
        let actual_min = tree.min_left_min(right, min_predicate);
        let actual_max = tree.min_left_max(right, max_predicate);

        // Assert
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn adjacent_boundary_updates_affect_only_expected_indices() {
        // Arrange
        let x_minus_1 = 2i64;
        let x = 5i64;
        let x_plus_1 = 9i64;
        let initial_values = vec![x_minus_1, x, x_plus_1];
        let mut tree = build_tree(&initial_values);
        let mut naive_values = initial_values.clone();
        let first_left = 0usize;
        let first_right = 2usize;
        let second_left = 1usize;
        let second_right = 3usize;
        let add_value = 4i64;
        let chmin_value = 7i64;
        apply_naive_add(&mut naive_values, first_left, first_right, add_value);
        apply_naive_chmin(&mut naive_values, second_left, second_right, chmin_value);

        let query_sum_left = 0;
        let query_sum_right = 1;
        let query_sum_left_2 = 1;
        let query_sum_right_2 = 2;
        let query_sum_left_3 = 2;
        let query_sum_right_3 = 3;

        // Act
        tree.range_add(first_left, first_right, add_value);
        tree.range_chmin(second_left, second_right, chmin_value);
        let actual_first = tree.query_sum(query_sum_left, query_sum_right);
        let actual_second = tree.query_sum(query_sum_left_2, query_sum_right_2);
        let actual_third = tree.query_sum(query_sum_left_3, query_sum_right_3);

        // Assert
        assert_eq!(actual_first, naive_values[0]);
        assert_eq!(actual_second, naive_values[1]);
        assert_eq!(actual_third, naive_values[2]);
    }

    #[test]
    fn random_update_and_query_sequence_matches_naive_model() {
        // Arrange
        let size = 10usize;
        let operation_count = 300usize;
        let mut seed = 123456789u64;
        let initial_values = (0..size)
            .map(|index| index as i64 - 5i64)
            .collect::<Vec<_>>();
        let mut tree = build_tree(&initial_values);
        let mut naive_values = initial_values.clone();
        let mut operations = Vec::new();
        let mut expected_queries = Vec::new();
        for _ in 0..operation_count {
            let op = (next_random_u64(&mut seed) % 6) as usize;
            let left = (next_random_u64(&mut seed) as usize) % size;
            let right = left + (next_random_u64(&mut seed) as usize) % (size - left + 1usize);
            let update_value = (next_random_u64(&mut seed) % 21) as i64 - 10i64;
            operations.push((op, left, right, update_value));
            match op {
                0 => apply_naive_add(&mut naive_values, left, right, update_value),
                1 => apply_naive_chmin(&mut naive_values, left, right, update_value),
                2 => apply_naive_chmax(&mut naive_values, left, right, update_value),
                _ => expected_queries.push((
                    naive_sum(&naive_values, left, right),
                    naive_min(&naive_values, left, right),
                    naive_max(&naive_values, left, right),
                )),
            }
        }
        let full_left = 0;
        let full_right = size;
        expected_queries.push((
            naive_sum(&naive_values, full_left, full_right),
            naive_min(&naive_values, full_left, full_right),
            naive_max(&naive_values, full_left, full_right),
        ));

        // Act
        let mut actual_queries = Vec::new();
        for (op, left, right, update_value) in operations {
            match op {
                0 => tree.range_add(left, right, update_value),
                1 => tree.range_chmin(left, right, update_value),
                2 => tree.range_chmax(left, right, update_value),
                _ => actual_queries.push((
                    tree.query_sum(left, right),
                    tree.query_min(left, right),
                    tree.query_max(left, right),
                )),
            }
        }
        actual_queries.push((
            tree.query_sum(full_left, full_right),
            tree.query_min(full_left, full_right),
            tree.query_max(full_left, full_right),
        ));

        // Assert
        assert_eq!(actual_queries, expected_queries);
    }

    #[test]
    fn point_query_returns_single_element_value() {
        // Arrange
        let values = vec![8i64, 5i64, -3i64];
        let mut tree = build_tree(&values);
        let index = 1usize;
        let left = index;
        let right = index + 1usize;
        let expected_value = 5i64;

        // Act
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_value);
        assert_eq!(actual_min, expected_value);
        assert_eq!(actual_max, expected_value);
    }

    #[test]
    fn query_left_edge_range_is_correct() {
        // Arrange
        let values = vec![8i64, 5i64, -3i64];
        let mut tree = build_tree(&values);
        let left = 0usize;
        let right = 1usize;

        let expected_sum_right = 8i64;
        let expected_min_right = 8i64;
        let expected_max_right = 8i64;

        // Act
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum_right);
        assert_eq!(actual_min, expected_min_right);
        assert_eq!(actual_max, expected_max_right);
    }

    #[test]
    fn query_right_edge_range_is_correct() {
        // Arrange
        let values = vec![8i64, 5i64, -3i64];
        let mut tree = build_tree(&values);
        let left = values.len() - 1usize;
        let right = values.len();

        let expected_sum_right = -3i64;
        let expected_min_right = -3i64;
        let expected_max_right = -3i64;

        // Act
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum_right);
        assert_eq!(actual_min, expected_min_right);
        assert_eq!(actual_max, expected_max_right);
    }

    #[test]
    fn query_full_range_is_correct() {
        // Arrange
        let values = vec![8i64, 5i64, -3i64];
        let mut tree = build_tree(&values);
        let left = 0usize;
        let right = values.len();

        let expected_sum_right = 10i64;
        let expected_min_right = -3i64;
        let expected_max_right = 8i64;

        // Act
        let actual_sum = tree.query_sum(left, right);
        let actual_min = tree.query_min(left, right);
        let actual_max = tree.query_max(left, right);

        // Assert
        assert_eq!(actual_sum, expected_sum_right);
        assert_eq!(actual_min, expected_min_right);
        assert_eq!(actual_max, expected_max_right);
    }

    #[test]
    fn after_mixed_updates_query_kind_searches_match_specialized_methods() {
        // Arrange
        let initial_values = vec![5i64, -2i64, 7i64, 1i64, 4i64, -1i64];
        let mut tree_with_query_kind = build_tree(&initial_values);
        let mut tree_with_specialized = build_tree(&initial_values);
        let mut naive_values_for_query_kind = initial_values.clone();
        let mut naive_values_for_specialized = initial_values.clone();
        apply_search_consistency_updates(
            &mut tree_with_query_kind,
            &mut naive_values_for_query_kind,
        );
        apply_search_consistency_updates(
            &mut tree_with_specialized,
            &mut naive_values_for_specialized,
        );
        let left = 0usize;
        let right = 4usize;
        let sum_predicate = |sum: &Value| *sum <= 10i64;
        let min_predicate = |minimum: &Value| *minimum >= 2i64;
        let max_predicate = |maximum: &Value| *maximum <= 6i64;

        // Act
        let actual_max_right_sum =
            tree_with_query_kind.max_right(left, QueryKind::Sum(&sum_predicate));
        let actual_max_right_min =
            tree_with_query_kind.max_right(left, QueryKind::Min(&min_predicate));
        let actual_max_right_max =
            tree_with_query_kind.max_right(left, QueryKind::Max(&max_predicate));
        let actual_min_left_sum =
            tree_with_query_kind.min_left(right, QueryKind::Sum(&sum_predicate));
        let actual_min_left_min =
            tree_with_query_kind.min_left(right, QueryKind::Min(&min_predicate));
        let actual_min_left_max =
            tree_with_query_kind.min_left(right, QueryKind::Max(&max_predicate));
        let actual_specialized_max_right_sum =
            tree_with_specialized.max_right_sum(left, sum_predicate);
        let actual_specialized_max_right_min =
            tree_with_specialized.max_right_min(left, min_predicate);
        let actual_specialized_max_right_max =
            tree_with_specialized.max_right_max(left, max_predicate);
        let actual_specialized_min_left_sum =
            tree_with_specialized.min_left_sum(right, sum_predicate);
        let actual_specialized_min_left_min =
            tree_with_specialized.min_left_min(right, min_predicate);
        let actual_specialized_min_left_max =
            tree_with_specialized.min_left_max(right, max_predicate);

        // Assert
        assert_eq!(actual_max_right_sum, actual_specialized_max_right_sum);
        assert_eq!(actual_max_right_min, actual_specialized_max_right_min);
        assert_eq!(actual_max_right_max, actual_specialized_max_right_max);
        assert_eq!(actual_min_left_sum, actual_specialized_min_left_sum);
        assert_eq!(actual_min_left_min, actual_specialized_min_left_min);
        assert_eq!(actual_min_left_max, actual_specialized_min_left_max);
    }
}

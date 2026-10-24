use std::collections::BTreeSet;
use std::ops::Bound::{Excluded, Included, Unbounded};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RangeSet<T>
where
    T: Ord + Copy + Default,
{
    total: usize,
    ranges: BTreeSet<(T, T)>,
}

impl<T> IntoIterator for RangeSet<T>
where
    T: Ord + Copy + Default + std::fmt::Debug,
{
    type Item = (T, T);
    type IntoIter = std::collections::btree_set::IntoIter<(T, T)>;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a RangeSet<T>
where
    T: Ord + Copy + Default + std::fmt::Debug,
{
    type Item = &'a (T, T);
    type IntoIter = std::collections::btree_set::Iter<'a, (T, T)>;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges.iter()
    }
}

impl<T> Default for RangeSet<T>
where
    T: Ord + Copy + Default,
{
    fn default() -> Self {
        Self {
            total: 0,
            ranges: BTreeSet::new()
        }
    }
}

impl<T> RangeSet<T>
where
    T: Ord + Copy + Default + std::fmt::Debug + TryInto<i128> + TryInto<u128>,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(&self) -> std::collections::btree_set::Iter<'_, (T, T)> {
        self.ranges.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    pub fn does_overlap(&self, left: T, right: T) -> bool {

        assert!(left < right);

        let range = (left, right);

        if let Some(pt) = self.max_lt_segment(right) {
            if self.is_left_crossed(&range, pt)
                || self.is_covered_by(&range, pt)
                || self.does_cover(&range, pt)
            {
                return true;
            }
        }

        if let Some(pt) = self.min_ge_segment(left) {
            if self.is_right_crossed(&range, pt)
                || self.does_cover(&range, pt)
                || self.is_covered_by(&range, pt) {
                    return true;
                }
        }

        false
    }

    pub fn is_covered(&self, left: T, right: T) -> bool {
        let range = (left, right);

        self.is_in(&range)
    }

    // [left, right)
    pub fn insert(&mut self, left: T, right: T) -> usize {

        let mut result = 0;
        let mut new_range = (left, right);
        if self.is_in(&new_range) {
            return result;
        }

        let mut deleted_sum = 0;
        if let Some(&pt) = self.max_lt_segment(left) {

            if self.is_enabled_to_merge_left(&new_range, &pt) {
                deleted_sum += Self::length(&pt);
                self.ranges.remove(&pt);

                new_range.0 = pt.0;
            }

        }

        deleted_sum += self.erase_covering_segments(&new_range);

        if let Some(&pt) = self.min_gt_segment(left) {

            if self.is_enabled_to_merge_right(&new_range, &pt) {
                deleted_sum += Self::length(&pt);
                self.ranges.remove(&pt);

                new_range.1 = pt.1;
            }

        }

        result += Self::length(&new_range);
        assert!(deleted_sum <= result);
        result -= deleted_sum;
        self.total += result;

        self.ranges.insert(new_range);

        result
    }

    pub fn erase(&mut self, left: T, right: T) -> usize {

        assert!(left < right);
        let mut result = 0;

        let new_range = (left, right);
        if let Some(&pt) = self.covering_segment(&new_range) {

            result += Self::length(&pt);
            self.ranges.remove(&pt);

            if pt.0 < left {
                let range = (pt.0, left);
                result -= Self::length(&range);
                self.ranges.insert(range);
            }

            if right < pt.1 {
                let range = (right, pt.1);
                result -= Self::length(&range);
                self.ranges.insert(range);
            }

            self.total -= result;

            return result;
        }

        // [?, x] x < right
        if let Some(&pt) = self.max_lt_segment(left) {

            if self.is_left_crossed(&new_range, &pt) {
                result += Self::length(&pt);
                self.ranges.remove(&pt);

                let range = (pt.0, left);
                result -= Self::length(&range);
                self.ranges.insert(range);
            }

        }

        result += self.erase_covering_segments(&new_range);

        // [?, x] (x < right)
        if let Some(&pt) = self.max_lt_segment(right) {

            if self.is_right_crossed(&new_range, &pt) {
                result += Self::length(&pt);
                self.ranges.remove(&pt);

                let range = (right, pt.1);
                result -= Self::length(&range);
                self.ranges.insert(range);
            }

        }

        self.total -= result;

        result
    }

    pub fn lower_bound(&self, x: T) -> Option<&(T, T)> {
        self.min_ge_segment(x)
    }

    pub fn upper_bound(&self, x: T) -> Option<&(T, T)> {
        self.ranges.range((Included((x, x)), Unbounded)).find(|&key| x < key.0)
    }

    pub fn min_ge_segment(&self, x: T) -> Option<&(T, T)> {
        let range = (x, x);

        self.ranges.range((Included(range), Unbounded)).next()
    }

    pub fn min_gt_segment(&self, x: T) -> Option<&(T, T)> {
        let range = (x, x);
        let mut range_iter = self.ranges.range((Included(range), Unbounded));

        if let Some(segment) = range_iter.next() {
            if segment.0 == x {
                return range_iter.next();
            }

            return Some(segment);
        }

        None
    }

    pub fn max_le_segment(&self, x: T) -> Option<&(T, T)> {
        let range = (x, x);

        if let Some(segment) = self.ranges.range((Included(range), Unbounded)).next() && segment.0 == x {
            return Some(segment);
        }

        self.ranges.range((Unbounded, Included(range))).next_back()
    }

    pub fn max_lt_segment(&self, x: T) -> Option<&(T, T)> {
        let range = (x, x);

        self.ranges.range((Unbounded, Excluded(range))).next_back()
    }

    pub fn contains_segment(&self, x: T) -> Option<&(T, T)> {

        let range = (x, x);

        if let Some(segment) = self.max_le_segment(x) && self.is_inside(&range, segment) {
            return Some(segment);
        }

        if let Some(segment) = self.min_ge_segment(x) && self.is_inside(&range, segment) {
            return Some(segment);
        }

        None
    }

    pub fn mex(&self) -> T {

        let range = (T::default(), T::default());
        if let Some(pt) = self.min_ge_segment(T::default()) && self.is_covered_by(&range, pt) {
            return pt.1;
        }

        // for segments in negative range
        if let Some(pt) = self.max_lt_segment(T::default()) && self.is_covered_by(&range, pt) {
            return pt.1;
        }

        T::default()
    }

    pub fn print(&self) {
        println!("{:?}", self.ranges);
    }

    #[inline]
    fn length(range: &(T, T)) -> usize {
        Self::length_from_i128(range)
            .or_else(|| Self::length_from_u128(range))
            .unwrap_or_else(|| panic!("failed to convert range length into usize"))
    }

    #[inline]
    fn length_from_i128(range: &(T, T)) -> Option<usize> {
        let left_value: i128 = range.0.try_into().ok()?;
        let right_value: i128 = range.1.try_into().ok()?;
        let length_value = right_value.checked_sub(left_value)?;

        usize::try_from(length_value).ok()
    }

    #[inline]
    fn length_from_u128(range: &(T, T)) -> Option<usize> {
        let left_value: u128 = range.0.try_into().ok()?;
        let right_value: u128 = range.1.try_into().ok()?;
        let length_value = right_value.checked_sub(left_value)?;

        usize::try_from(length_value).ok()
    }

    #[inline]
    fn does_cover(&self, self_range: &(T, T), other: &(T, T)) -> bool {
        self_range.0 <= other.0 && other.1 <= self_range.1
    }

    #[inline]
    fn is_covered_by(&self, self_range: &(T, T), other: &(T, T)) -> bool {
        other.0 <= self_range.0 && self_range.1 <= other.1
    }

    #[inline]
    fn is_inside(&self, self_range: &(T, T), other: &(T, T)) -> bool {
        other.0 <= self_range.0 && self_range.1 < other.1
    }

    #[inline]
    fn is_left_crossed(&self, self_range: &(T, T), other: &(T, T)) -> bool {
        other.0 < self_range.0 && self_range.0 < other.1 && other.1 < self_range.1
    }

    #[inline]
    fn is_right_crossed(&self, self_range: &(T, T), other: &(T, T)) -> bool {
        self_range.0 < other.0 && other.0 < self_range.1 && self_range.1 < other.1
    }

    #[inline]
    fn is_enabled_to_merge_left(&self, self_range: &(T, T), other: &(T, T)) -> bool {
        other.0 < self_range.0 && self_range.0 <= other.1 && other.1 < self_range.1
    }

    #[inline]
    fn is_enabled_to_merge_right(&self, self_range: &(T, T), other: &(T, T)) -> bool {
        self_range.0 < other.0 && other.0 <= self_range.1 && self_range.1 < other.1
    }

    fn covering_segment(&self, range: &(T, T)) -> Option<&(T, T)> {

        assert!(range.0 < range.1);

        match self.max_le_segment(range.1) {
            Some(max_key) if self.is_covered_by(range, max_key) => Some(max_key),
            _ => None,
        }

    }

    fn is_in(&self, range: &(T, T)) -> bool {

        assert!(range.0 < range.1);

        match self.max_lt_segment(range.1) {
            Some(max_key) => self.is_covered_by(range, max_key),
            None => false,
        }

    }

    #[inline]
    fn erase_covering_segments(&mut self, range: &(T, T)) -> usize {

        let mut result = 0;
        let left = range.0;

        while let Some(&pt) = self.min_ge_segment(left) && self.does_cover(range, &pt) {
            result += Self::length(&pt);
            self.ranges.remove(&pt);
        }

        result
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn calculate_total_length(range_set: &RangeSet<i32>) -> usize {
        range_set
            .iter()
            .map(|&(left_value, right_value)| (right_value - left_value) as usize)
            .sum()
    }

    #[test]
    fn into_iter_over_disjoint_segments_returns_ascending_segments() {
        // Arrange
        let segments = [(1, 3), (5, 8), (10, 12)];
        let expected = vec![(1, 3), (5, 8), (10, 12)];
        let mut range_set = RangeSet::new();
        for segment in segments {
            range_set.ranges.insert(segment);
        }
        // Act
        let actual = range_set.into_iter().collect::<Vec<_>>();
        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn into_iterator_for_reference_matches_iter_order() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 3;
        let input_value_3 = 5;
        let input_value_4 = 8;
        let input_value_5 = 10;
        let input_value_6 = 12;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);
        range_set.insert(input_value_3, input_value_4);
        range_set.insert(input_value_5, input_value_6);

        // Act
        let iter_values: Vec<(i32, i32)> = range_set.iter().copied().collect();
        let into_iter_values: Vec<(i32, i32)> = (&range_set).into_iter().copied().collect();

        // Assert
        assert_eq!(iter_values, into_iter_values);
    }

    #[test]
    fn new_initial_state_is_empty_and_total_is_zero() {
        // Arrange
        let range_set: RangeSet<i32> = RangeSet::new();
        let expected_check_2 = 0;
        let expected_check_4 = 0;

        let expected_boolean_1 = true;
        let expected_missing_value_2 = None;

        // Act
        let is_empty_value = range_set.is_empty();
        let range_count = range_set.len();
        let first_segment = range_set.ranges.first();
        let total_length = range_set.total;

        // Assert
        assert_eq!(is_empty_value, expected_boolean_1);
        assert_eq!(range_count, expected_check_2);
        assert_eq!(first_segment, expected_missing_value_2);
        assert_eq!(total_length, expected_check_4);
    }

    #[test]
    fn iter_over_disjoint_segments_returns_ascending_segments() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 3;
        let input_value_3 = 5;
        let input_value_4 = 8;
        let input_value_5 = 10;
        let input_value_6 = 12;

        let mut range_set = RangeSet::new();

        range_set.ranges.insert((input_value_1, input_value_2));
        range_set.ranges.insert((input_value_3, input_value_4));
        range_set.ranges.insert((input_value_5, input_value_6));
        let expected_check_1 = Some(&(1, 3));
        let expected_check_2 = Some(&(5, 8));
        let expected_check_3 = Some(&(10, 12));

        let expected_missing_value_1 = None;

        // Act
        let mut iter = range_set.iter();

        let actual_check_1 = iter.next();
        let actual_check_2 = iter.next();
        let actual_check_3 = iter.next();
        let actual_check_4 = iter.next();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(actual_check_3, expected_check_3);
        assert_eq!(actual_check_4, expected_missing_value_1);
    }

    #[test]
    fn does_overlap_non_overlap_range_returns_false() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 0;
        let input_value_4 = 4;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = false;

        // Act
        let actual_check_1 = rs.does_overlap(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn does_overlap_left_crossed_range_returns_true() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 3;
        let input_value_4 = 7;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = rs.does_overlap(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn does_overlap_right_crossed_range_returns_true() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 7;
        let input_value_4 = 12;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = rs.does_overlap(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn does_overlap_middle_crossed_range_returns_true() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 8;
        let input_value_4 = 10;
        let input_value_5 = 3;
        let input_value_6 = 9;

        let mut rs = RangeSet::new();
        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = rs.does_overlap(input_value_5, input_value_6);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn does_overlap_the_exact_range_returns_true() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 1;
        let input_value_4 = 5;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = rs.does_overlap(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn does_overlap_included_range_returns_true() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 3;
        let input_value_4 = 4;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = rs.does_overlap(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn does_overlap_overlap_range_returns_true() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = -1;
        let input_value_4 = 7;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = rs.does_overlap(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn does_overlap_left_prefix_range_returns_true() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 1;
        let input_value_4 = 3;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = rs.does_overlap(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn does_overlap_right_prefix_range_returns_true() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 3;
        let input_value_4 = 5;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = rs.does_overlap(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn does_overlap_returns_false_for_empty_set() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let range_set: RangeSet<i32> = RangeSet::new();

        let expected_boolean_1 = false;

        // Act
        let overlap_result = range_set.does_overlap(input_value_1, input_value_2);

        // Assert
        assert_eq!(overlap_result, expected_boolean_1);
    }

    #[test]
    fn does_overlap_returns_false_for_adjacent_half_open_ranges() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 5;
        let input_value_4 = 8;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);

        let expected_boolean_1 = false;

        // Act
        let overlap_result = range_set.does_overlap(input_value_3, input_value_4);

        // Assert
        assert_eq!(overlap_result, expected_boolean_1);
    }

    #[test]
    fn does_overlap_panics_when_left_equals_right() {
        // Arrange
        let range_set: RangeSet<i32> = RangeSet::new();
        let left_value = 3;
        let right_value = 3;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _overlap_result = range_set.does_overlap(left_value, right_value);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn does_overlap_panics_when_right_is_less_than_left() {
        // Arrange
        let range_set: RangeSet<i32> = RangeSet::new();
        let left_value = 4;
        let right_value = 3;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _overlap_result = range_set.does_overlap(left_value, right_value);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn is_covered_left_crossed_range_returns_false() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 3;
        let input_value_4 = 7;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = false;

        // Act
        let actual_check_1 = rs.is_covered(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn is_covered_right_crossed_range_returns_false() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 7;
        let input_value_4 = 12;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = false;

        // Act
        let actual_check_1 = rs.is_covered(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn is_covered_middle_crossed_range_returns_false() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 8;
        let input_value_4 = 10;
        let input_value_5 = 3;
        let input_value_6 = 9;

        let mut rs = RangeSet::new();
        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);

        let expected_boolean_1 = false;

        // Act
        let actual_check_1 = rs.is_covered(input_value_5, input_value_6);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn is_covered_exact_range_returns_true() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 1;
        let input_value_4 = 5;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = rs.is_covered(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn is_covered_included_range_returns_true() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 3;
        let input_value_4 = 4;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = rs.is_covered(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn is_covered_overlap_range_returns_false() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = -1;
        let input_value_4 = 7;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = false;

        // Act
        let actual_check_1 = rs.is_covered(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn is_covered_left_prefix_range_returns_true() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 1;
        let input_value_4 = 3;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = rs.is_covered(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn is_covered_right_prefix_range_returns_true() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 3;
        let input_value_4 = 5;

        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = rs.is_covered(input_value_3, input_value_4);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn is_covered_returns_false_for_non_overlap_range() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 0;
        let input_value_4 = 4;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);

        let expected_boolean_1 = false;

        // Act
        let covered_result = range_set.is_covered(input_value_3, input_value_4);

        // Assert
        assert_eq!(covered_result, expected_boolean_1);
    }

    #[test]
    fn is_covered_returns_false_for_empty_set() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let range_set: RangeSet<i32> = RangeSet::new();

        let expected_boolean_1 = false;

        // Act
        let covered_result = range_set.is_covered(input_value_1, input_value_2);

        // Assert
        assert_eq!(covered_result, expected_boolean_1);
    }

    #[test]
    fn is_covered_panics_when_left_equals_right() {
        // Arrange
        let range_set: RangeSet<i32> = RangeSet::new();
        let left_value = 2;
        let right_value = 2;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _covered_result = range_set.is_covered(left_value, right_value);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn is_covered_panics_when_right_is_less_than_left() {
        // Arrange
        let range_set: RangeSet<i32> = RangeSet::new();
        let left_value = 3;
        let right_value = 2;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _covered_result = range_set.is_covered(left_value, right_value);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn insert_into_empty_set_stores_range_and_returns_added_length() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let mut rs = RangeSet::new();

        let expected = 4;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(1, 5));

        // Act
        let actual = rs.insert(input_value_1, input_value_2);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_non_overlap_range_returns_added_length_and_expected_merged_segments() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 0;
        let input_value_4 = 4;

        let mut rs = RangeSet::new();

        let expected = 9;
        let expected_check_1 = 2;
        let expected_check_2 = Some(&(0, 4));
        let expected_check_3 = Some(&(5, 10));

        // Act
        let mut actual = 0;
        actual += rs.insert(input_value_1, input_value_2);
        actual += rs.insert(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();
        let actual_check_3 = rs.ranges.last();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(actual_check_3, expected_check_3);
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_left_crossed_range_returns_added_length_and_expected_merged_segments() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 3;
        let input_value_4 = 7;

        let mut rs = RangeSet::new();

        let expected = 7;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(3, 10));

        // Act
        let mut actual = 0;
        actual += rs.insert(input_value_1, input_value_2);
        actual += rs.insert(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.last();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_left_point_range_returns_added_length_and_expected_merged_segments() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 10;
        let input_value_4 = 14;

        let mut rs = RangeSet::new();

        let expected = 9;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(5, 14));

        // Act
        let mut actual = 0;
        actual += rs.insert(input_value_1, input_value_2);
        actual += rs.insert(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.last();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_right_crossed_range_returns_added_length_and_expected_merged_segments() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 7;
        let input_value_4 = 12;

        let mut rs = RangeSet::new();

        let expected = 7;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(5, 12));

        // Act
        let mut actual = 0;
        actual += rs.insert(input_value_1, input_value_2);
        actual += rs.insert(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.last();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_right_point_range_returns_added_length_and_expected_merged_segments() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 0;
        let input_value_4 = 5;

        let mut rs = RangeSet::new();

        let expected = 10;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(0, 10));

        // Act
        let mut actual = 0;
        actual += rs.insert(input_value_1, input_value_2);
        actual += rs.insert(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.last();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_existing_range_returns_added_length_and_expected_merged_segments() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 5;
        let input_value_4 = 10;

        let mut rs = RangeSet::new();

        let expected = 5;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(5, 10));

        // Act
        let mut actual = 0;
        actual += rs.insert(input_value_1, input_value_2);
        actual += rs.insert(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_included_range_returns_added_length_and_expected_merged_segments() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 3;
        let input_value_4 = 4;

        let mut rs = RangeSet::new();

        let expected = 4;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(1, 5));

        // Act
        let mut actual = 0;
        actual += rs.insert(input_value_1, input_value_2);
        actual += rs.insert(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_overlap_range_returns_added_length_and_expected_merged_segments() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = -1;
        let input_value_4 = 7;

        let mut rs = RangeSet::new();

        let expected = 8;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(-1, 7));

        // Act
        let mut actual = 0;
        actual += rs.insert(input_value_1, input_value_2);
        actual += rs.insert(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_middle_crossed_range_returns_added_length_and_expected_merged_segments() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 8;
        let input_value_4 = 10;
        let input_value_5 = 3;
        let input_value_6 = 9;

        let mut rs = RangeSet::new();

        let expected = 9;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(1, 10));

        // Act
        let mut actual = 0;
        actual += rs.insert(input_value_1, input_value_2);
        actual += rs.insert(input_value_3, input_value_4);
        actual += rs.insert(input_value_5, input_value_6);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert_updates_empty_state_to_false() {
        // Arrange
        let mut range_set: RangeSet<i32> = RangeSet::new();
        let left_value = 1;
        let right_value = 3;
        let expected_check_1 = 2;

        let expected_boolean_1 = false;

        // Act
        let inserted_length = range_set.insert(left_value, right_value);
        let is_empty_value = range_set.is_empty();

        // Assert
        assert_eq!(inserted_length, expected_check_1);
        assert_eq!(is_empty_value, expected_boolean_1);
    }

    #[test]
    fn insert_panics_when_left_equals_right() {
        // Arrange
        let mut range_set: RangeSet<i32> = RangeSet::new();
        let left_value = 5;
        let right_value = 5;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _inserted_length = range_set.insert(left_value, right_value);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn insert_panics_when_right_is_less_than_left() {
        // Arrange
        let mut range_set: RangeSet<i32> = RangeSet::new();
        let left_value = 6;
        let right_value = 5;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _inserted_length = range_set.insert(left_value, right_value);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn insert_merges_two_adjacent_segments_with_bridge_range() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 3;
        let input_value_3 = 5;
        let input_value_4 = 7;
        let input_value_5 = 3;
        let input_value_6 = 5;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);
        range_set.insert(input_value_3, input_value_4);
        let expected_check_1 = 2;
        let expected_check_2 = 1;
        let expected_check_3 = Some(&(1, 7));

        // Act
        let inserted_length = range_set.insert(input_value_5, input_value_6);
        let range_count = range_set.len();
        let merged_segment = range_set.ranges.first();

        // Assert
        assert_eq!(inserted_length, expected_check_1);
        assert_eq!(range_count, expected_check_2);
        assert_eq!(merged_segment, expected_check_3);
    }

    #[test]
    fn insert_merges_three_or_more_segments_with_single_range() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 2;
        let input_value_3 = 4;
        let input_value_4 = 5;
        let input_value_5 = 7;
        let input_value_6 = 8;
        let input_value_7 = 2;
        let input_value_8 = 7;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);
        range_set.insert(input_value_3, input_value_4);
        range_set.insert(input_value_5, input_value_6);
        let expected_check_1 = 4;
        let expected_check_2 = 1;
        let expected_check_3 = Some(&(1, 8));

        // Act
        let inserted_length = range_set.insert(input_value_7, input_value_8);
        let range_count = range_set.len();
        let merged_segment = range_set.ranges.first();

        // Assert
        assert_eq!(inserted_length, expected_check_1);
        assert_eq!(range_count, expected_check_2);
        assert_eq!(merged_segment, expected_check_3);
    }

    #[test]
    fn insert_keeps_total_equal_to_sum_of_segment_lengths() {
        // Arrange
        let operations = [(1, 3), (5, 9), (3, 5), (12, 14)];
        let expected_segments = vec![(1, 9), (12, 14)];
        let expected_total = 10;
        let mut range_set = RangeSet::<i32>::new();

        // Act
        for (left, right) in operations {
            range_set.insert(left, right);
        }
        let actual_segments = range_set.iter().copied().collect::<Vec<_>>();
        let actual_total = range_set.total;
        let actual_segment_length_sum = calculate_total_length(&range_set);

        // Assert
        assert_eq!(actual_segments, expected_segments);
        assert_eq!(actual_total, expected_total);
        assert_eq!(actual_segment_length_sum, expected_total);
    }

    #[test]
    fn insert_with_i32_extreme_range_updates_total_without_overflow() {
        // Arrange
        let mut range_set: RangeSet<i32> = RangeSet::new();
        let left_value = i32::MIN;
        let right_value = i32::MAX;
        let expected_inserted_length = u32::MAX as usize;
        let expected_total_length = u32::MAX as usize;

        // Act
        let inserted_length = range_set.insert(left_value, right_value);
        let actual_total_length = range_set.total;

        // Assert
        assert_eq!(inserted_length, expected_inserted_length);
        assert_eq!(actual_total_length, expected_total_length);
    }

    #[test]
    fn erase_non_overlap_range_returns_removed_length_and_expected_remaining_segments() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 0;
        let input_value_4 = 4;

        let mut rs = RangeSet::new();

        let expected = 0;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(5, 10));

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_overlap_range_returns_removed_length_and_expected_remaining_segments() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 0;
        let input_value_4 = 12;

        let mut rs = RangeSet::new();

        let expected = 5;
        let expected_check_1 = 0;

        let expected_missing_value_1 = None;

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_missing_value_1);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_exact_range_returns_removed_length_and_expected_remaining_segments() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 1;
        let input_value_4 = 5;

        let mut rs = RangeSet::new();

        let expected = 4;
        let expected_check_1 = 0;

        let expected_missing_value_1 = None;

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_missing_value_1);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_included_range_returns_removed_length_and_expected_remaining_segments() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = -1;
        let input_value_4 = 6;

        let mut rs = RangeSet::new();

        let expected = 4;
        let expected_check_1 = 0;

        let expected_missing_value_1 = None;

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_missing_value_1);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_left_crossed_range_returns_removed_length_and_expected_remaining_segments() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 3;
        let input_value_4 = 6;

        let mut rs = RangeSet::new();

        let expected = 1;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(6, 10));

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_right_crossed_range_returns_removed_length_and_expected_remaining_segments() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 8;
        let input_value_4 = 10;

        let mut rs = RangeSet::new();

        let expected = 2;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(5, 8));

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_middle_crossed_range_returns_removed_length_and_expected_remaining_segments() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 6;
        let input_value_4 = 8;

        let mut rs = RangeSet::new();

        let expected = 2;
        let expected_check_1 = 2;
        let expected_check_2 = Some(&(5, 6));
        let expected_check_3 = Some(&(8, 10));

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();
        let actual_check_3 = rs.ranges.last();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(actual_check_3, expected_check_3);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_exact_left_prefix_range_returns_removed_length_and_expected_remaining_segments() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 5;
        let input_value_4 = 8;

        let mut rs = RangeSet::new();
        let expected = 3;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(8, 10));

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_exact_right_prefix_range_returns_removed_length_and_expected_remaining_segments() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 7;
        let input_value_4 = 10;

        let mut rs = RangeSet::new();
        let expected = 3;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(5, 7));

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase(input_value_3, input_value_4);

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_all_ranges_returns_to_empty_state() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 1;
        let input_value_4 = 5;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);
        let expected_check_1 = 4;
        let expected_check_3 = 0;

        let expected_boolean_1 = true;

        // Act
        let erased_length = range_set.erase(input_value_3, input_value_4);
        let is_empty_value = range_set.is_empty();
        let range_count = range_set.len();

        // Assert
        assert_eq!(erased_length, expected_check_1);
        assert_eq!(is_empty_value, expected_boolean_1);
        assert_eq!(range_count, expected_check_3);
    }

    #[test]
    fn erase_on_empty_set_returns_zero_and_keeps_state() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        let expected_check_1 = 0;
        let expected_check_2 = 0;

        let expected_boolean_1 = true;

        // Act
        let erased_length = range_set.erase(input_value_1, input_value_2);
        let range_count = range_set.len();
        let is_empty_value = range_set.is_empty();

        // Assert
        assert_eq!(erased_length, expected_check_1);
        assert_eq!(range_count, expected_check_2);
        assert_eq!(is_empty_value, expected_boolean_1);
    }

    #[test]
    fn erase_returns_zero_for_adjacent_half_open_range() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 5;
        let input_value_4 = 8;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);
        let expected_check_1 = 0;
        let expected_check_2 = Some(&(1, 5));

        // Act
        let erased_length = range_set.erase(input_value_3, input_value_4);
        let remained_segment = range_set.ranges.first();

        // Assert
        assert_eq!(erased_length, expected_check_1);
        assert_eq!(remained_segment, expected_check_2);
    }

    #[test]
    fn erase_panics_when_left_equals_right() {
        // Arrange
        let mut range_set: RangeSet<i32> = RangeSet::new();
        let left_value = 4;
        let right_value = 4;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _erased_length = range_set.erase(left_value, right_value);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn erase_panics_when_right_is_less_than_left() {
        // Arrange
        let mut range_set: RangeSet<i32> = RangeSet::new();
        let left_value = 5;
        let right_value = 4;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _erased_length = range_set.erase(left_value, right_value);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn erase_partial_range_across_multiple_segments_keeps_only_ends() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 3;
        let input_value_3 = 5;
        let input_value_4 = 7;
        let input_value_5 = 9;
        let input_value_6 = 11;
        let input_value_7 = 2;
        let input_value_8 = 10;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);
        range_set.insert(input_value_3, input_value_4);
        range_set.insert(input_value_5, input_value_6);
        let expected_check_1 = 4;
        let expected_check_2 = vec![(1, 2), (10, 11)];

        // Act
        let erased_length = range_set.erase(input_value_7, input_value_8);
        let remained_segments: Vec<(i32, i32)> = range_set.iter().copied().collect();

        // Assert
        assert_eq!(erased_length, expected_check_1);
        assert_eq!(remained_segments, expected_check_2);
    }

    #[test]
    fn erase_keeps_total_equal_to_sum_of_segment_lengths() {
        // Arrange
        let initial_segments = [(1, 5), (8, 12), (15, 18)];
        let erase_range = (3, 16);
        let expected_segments = vec![(1, 3), (16, 18)];
        let expected_total = 4;
        let mut range_set = RangeSet::<i32>::new();
        for (left, right) in initial_segments {
            range_set.insert(left, right);
        }
        let expected_check_4 = 7;

        // Act
        let erased_length = range_set.erase(erase_range.0, erase_range.1);
        let actual_segments = range_set.iter().copied().collect::<Vec<_>>();
        let actual_total = range_set.total;
        let actual_segment_length_sum = calculate_total_length(&range_set);

        // Assert
        assert_eq!(actual_segments, expected_segments);
        assert_eq!(actual_total, expected_total);
        assert_eq!(actual_segment_length_sum, expected_total);
        assert_eq!(erased_length, expected_check_4);
    }

    #[test]
    fn erase_with_i32_extreme_covering_segment_updates_total_without_overflow() {
        // Arrange
        let mut range_set: RangeSet<i32> = RangeSet::new();
        let insert_left_value = i32::MIN;
        let insert_right_value = i32::MAX;
        let erase_left_value = -1;
        let erase_right_value = 1;
        let expected_erased_length = 2;
        let expected_total_length = (u32::MAX - 2) as usize;
        range_set.insert(insert_left_value, insert_right_value);

        // Act
        let erased_length = range_set.erase(erase_left_value, erase_right_value);
        let actual_total_length = range_set.total;

        // Assert
        assert_eq!(erased_length, expected_erased_length);
        assert_eq!(actual_total_length, expected_total_length);
    }

    #[test]
    fn lower_bound_returns_none() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 1;
        let input_value_6 = 6;
        let input_value_7 = 20;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(1, 5));
        let expected_check_2 = Some(&(10, 20));

        let expected_missing_value_1 = None;

        // Act
        let actual_check_1 = rs.lower_bound(input_value_5);
        let actual_check_2 = rs.lower_bound(input_value_6);
        let actual_check_3 = rs.lower_bound(input_value_7);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(actual_check_3, expected_missing_value_1);
    }

    #[test]
    fn lower_bound_exact_left_point_returns_expected_neighbor_segment() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 1;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(1, 5));

        // Act
        let actual_check_1 = rs.lower_bound(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn lower_bound_less_left_point_returns_expected_neighbor_segment() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 0;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(1, 5));

        // Act
        let actual_check_1 = rs.lower_bound(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn lower_bound_larger_left_point_returns_expected_neighbor_segment() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 2;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(10, 20));

        // Act
        let actual_check_1 = rs.lower_bound(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn lower_bound_not_included_point_returns_none() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 21;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);

        let expected_missing_value_1 = None;

        // Act
        let actual_check_1 = rs.lower_bound(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_missing_value_1);
    }

    #[test]
    fn lower_bound_returns_none_for_empty_set() {
        // Arrange
        let input_value_1 = 1;
        let range_set: RangeSet<i32> = RangeSet::new();

        let expected_missing_value_1 = None;

        // Act
        let lower_bound_segment = range_set.lower_bound(input_value_1);

        // Assert
        assert_eq!(lower_bound_segment, expected_missing_value_1);
    }

    #[test]
    fn upper_bound_returns_none() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 1;
        let input_value_6 = 5;
        let input_value_7 = 20;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(10, 20));
        let expected_check_2 = Some(&(10, 20));

        let expected_missing_value_1 = None;

        // Act
        let actual_check_1 = rs.upper_bound(input_value_5);
        let actual_check_2 = rs.upper_bound(input_value_6);
        let actual_check_3 = rs.upper_bound(input_value_7);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(actual_check_3, expected_missing_value_1);
    }

    #[test]
    fn upper_bound_exact_left_point_returns_expected_neighbor_segment() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 1;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(10, 20));

        // Act
        let actual_check_1 = rs.upper_bound(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn upper_bound_less_left_point_returns_expected_neighbor_segment() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 0;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(1, 5));

        // Act
        let actual_check_1 = rs.upper_bound(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn upper_bound_larger_left_point_returns_expected_neighbor_segment() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 2;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(10, 20));

        // Act
        let actual_check_1 = rs.upper_bound(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn upper_bound_not_included_point_returns_none() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 21;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);

        let expected_missing_value_1 = None;

        // Act
        let actual_check_1 = rs.upper_bound(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_missing_value_1);
    }

    #[test]
    fn upper_bound_returns_none_for_empty_set() {
        // Arrange
        let input_value_1 = 1;
        let range_set: RangeSet<i32> = RangeSet::new();

        let expected_missing_value_1 = None;

        // Act
        let upper_bound_segment = range_set.upper_bound(input_value_1);

        // Assert
        assert_eq!(upper_bound_segment, expected_missing_value_1);
    }

    #[test]
    fn min_ge_segment_returns_segment_with_same_left_endpoint() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 15;
        let input_value_5 = 1;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);
        range_set.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(1, 5));

        // Act
        let min_ge_segment = range_set.min_ge_segment(input_value_5);

        // Assert
        assert_eq!(min_ge_segment, expected_check_1);
    }

    #[test]
    fn min_gt_segment_skips_segment_with_same_left_endpoint() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 15;
        let input_value_5 = 1;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);
        range_set.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(10, 15));

        // Act
        let min_gt_segment = range_set.min_gt_segment(input_value_5);

        // Assert
        assert_eq!(min_gt_segment, expected_check_1);
    }

    #[test]
    fn max_le_segment_returns_segment_with_same_left_endpoint() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 15;
        let input_value_5 = 1;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);
        range_set.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(1, 5));

        // Act
        let max_le_segment = range_set.max_le_segment(input_value_5);

        // Assert
        assert_eq!(max_le_segment, expected_check_1);
    }

    #[test]
    fn max_lt_segment_skips_segment_with_same_left_endpoint() {
        // Arrange
        let input_value_1 = -3;
        let input_value_2 = -1;
        let input_value_3 = 1;
        let input_value_4 = 5;
        let input_value_5 = 10;
        let input_value_6 = 15;
        let input_value_7 = 1;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);
        range_set.insert(input_value_3, input_value_4);
        range_set.insert(input_value_5, input_value_6);
        let expected_check_1 = Some(&(-3, -1));

        // Act
        let max_lt_segment = range_set.max_lt_segment(input_value_7);

        // Assert
        assert_eq!(max_lt_segment, expected_check_1);
    }

    #[test]
    fn contains_segment_exact_left_point_returns_containing_segment() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 1;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(1, 5));

        // Act
        let actual_check_1 = rs.contains_segment(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn contains_segment_less_left_point_returns_none() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 0;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);

        let expected_missing_value_1 = None;

        // Act
        let actual_check_1 = rs.contains_segment(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_missing_value_1);
    }

    #[test]
    fn contains_segment_larger_left_point_returns_containing_segment() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 2;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(1, 5));

        // Act
        let actual_check_1 = rs.contains_segment(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn contains_segment_exact_right_point_returns_none() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 5;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);

        let expected_missing_value_1 = None;

        // Act
        let actual_check_1 = rs.contains_segment(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_missing_value_1);
    }

    #[test]
    fn contains_segment_less_right_point_returns_containing_segment() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 4;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = Some(&(1, 5));

        // Act
        let actual_check_1 = rs.contains_segment(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn contains_segment_larger_right_point_returns_none() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 6;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);

        let expected_missing_value_1 = None;

        // Act
        let actual_check_1 = rs.contains_segment(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_missing_value_1);
    }

    #[test]
    fn contains_segment_not_included_point_returns_none() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 8;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);

        let expected_missing_value_1 = None;

        // Act
        let actual_check_1 = rs.contains_segment(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_missing_value_1);
    }

    #[test]
    fn contains_segment_returns_none_for_empty_set() {
        // Arrange
        let input_value_1 = 1;
        let range_set: RangeSet<i32> = RangeSet::new();

        let expected_missing_value_1 = None;

        // Act
        let contains_segment = range_set.contains_segment(input_value_1);

        // Assert
        assert_eq!(contains_segment, expected_missing_value_1);
    }

    #[test]
    fn mex_zero_returns_first_uncovered_nonnegative_value() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 15;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = 0;

        // Act
        let actual_check_1 = rs.mex();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn mex_exact_point_returns_first_uncovered_nonnegative_value() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 5;
        let input_value_3 = 6;
        let input_value_4 = 10;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = 5;

        // Act
        let actual_check_1 = rs.mex();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn mex_first_right_point_returns_first_uncovered_nonnegative_value() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 15;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = 5;

        // Act
        let actual_check_1 = rs.mex();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn mex_containing_negative_segment_returns_first_uncovered_nonnegative_value() {
        // Arrange
        let input_value_1 = -3;
        let input_value_2 = 5;
        let input_value_3 = 10;
        let input_value_4 = 15;
        let mut rs = RangeSet::new();

        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        let expected_check_1 = 5;

        // Act
        let actual_check_1 = rs.mex();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn mex_returns_default_for_empty_set() {
        // Arrange
        let range_set: RangeSet<i32> = RangeSet::new();
        let expected_check_1 = 0;

        // Act
        let mex_value = range_set.mex();

        // Assert
        assert_eq!(mex_value, expected_check_1);
    }

    #[test]
    fn mex_returns_zero_when_only_negative_ranges_exist() {
        // Arrange
        let input_value_1 = -5;
        let input_value_2 = -1;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);
        let expected_check_1 = 0;

        // Act
        let mex_value = range_set.mex();

        // Assert
        assert_eq!(mex_value, expected_check_1);
    }

    #[test]
    fn mex_returns_next_value_for_merged_leading_segments() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 5;
        let input_value_3 = 5;
        let input_value_4 = 10;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);
        range_set.insert(input_value_3, input_value_4);
        let expected_check_1 = 10;

        // Act
        let mex_value = range_set.mex();

        // Assert
        assert_eq!(mex_value, expected_check_1);
    }

    #[test]
    fn erase_covering_segments_returns_removed_length_and_expected_remaining_segments() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 2;
        let input_value_3 = 3;
        let input_value_4 = 5;
        let input_value_5 = -1;
        let input_value_6 = 9;
        let mut rs = RangeSet::new();

        let expected = 4;
        let expected_check_1 = rs.len();

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        rs.insert(input_value_3, input_value_4);
        actual += rs.erase_covering_segments(&(input_value_5, input_value_6));

        let actual_check_1 = 0;

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_covering_segments_non_overlap_range_returns_removed_length_and_expected_remaining_segments()
     {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 0;
        let input_value_4 = 4;

        let mut rs = RangeSet::new();

        let expected = 0;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(5, 10));

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase_covering_segments(&(input_value_3, input_value_4));

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_covering_segments_overlap_range_returns_removed_length_and_expected_remaining_segments()
     {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 0;
        let input_value_4 = 12;

        let mut rs = RangeSet::new();

        let expected = 5;
        let expected_check_1 = 0;

        let expected_missing_value_1 = None;

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase_covering_segments(&(input_value_3, input_value_4));

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_missing_value_1);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_covering_segments_exact_range_returns_removed_length_and_expected_remaining_segments()
    {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = 1;
        let input_value_4 = 5;

        let mut rs = RangeSet::new();

        let expected = 4;
        let expected_check_1 = 0;

        let expected_missing_value_1 = None;

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase_covering_segments(&(input_value_3, input_value_4));
        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_missing_value_1);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_covering_segments_included_range_returns_removed_length_and_expected_remaining_segments()
     {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 5;
        let input_value_3 = -1;
        let input_value_4 = 6;

        let mut rs = RangeSet::new();

        let expected = 4;
        let expected_check_1 = 0;

        let expected_missing_value_1 = None;

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase_covering_segments(&(input_value_3, input_value_4));

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_missing_value_1);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_covering_segments_left_crossed_range_returns_removed_length_and_expected_remaining_segments()
     {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 3;
        let input_value_4 = 6;

        let mut rs = RangeSet::new();
        let expected = 0;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(5, 10));

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase_covering_segments(&(input_value_3, input_value_4));

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_covering_segments_right_crossed_range_returns_removed_length_and_expected_remaining_segments()
     {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 8;
        let input_value_4 = 10;

        let mut rs = RangeSet::new();

        let expected = 0;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(5, 10));

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase_covering_segments(&(input_value_3, input_value_4));

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_covering_segments_middle_crossed_range_returns_removed_length_and_expected_remaining_segments()
     {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 6;
        let input_value_4 = 8;

        let mut rs = RangeSet::new();

        let expected = 0;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(5, 10));

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase_covering_segments(&(input_value_3, input_value_4));

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_covering_segments_exact_left_prefix_range_returns_removed_length_and_expected_remaining_segments()
     {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 5;
        let input_value_4 = 8;

        let mut rs = RangeSet::new();

        let expected = 0;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(5, 10));

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase_covering_segments(&(input_value_3, input_value_4));

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_covering_segments_exact_right_prefix_range_returns_removed_length_and_expected_remaining_segments()
     {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 10;
        let input_value_3 = 7;
        let input_value_4 = 10;

        let mut rs = RangeSet::new();

        let expected = 0;
        let expected_check_1 = 1;
        let expected_check_2 = Some(&(5, 10));

        // Act
        let mut actual = 0;
        rs.insert(input_value_1, input_value_2);
        actual += rs.erase_covering_segments(&(input_value_3, input_value_4));

        let actual_check_1 = rs.ranges.len();
        let actual_check_2 = rs.ranges.first();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_covering_segments_removes_only_fully_covered_segments() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 4;
        let input_value_3 = 5;
        let input_value_4 = 8;
        let input_value_5 = 10;
        let input_value_6 = 12;
        let mut range_set: RangeSet<i32> = RangeSet::new();
        range_set.insert(input_value_1, input_value_2);
        range_set.insert(input_value_3, input_value_4);
        range_set.insert(input_value_5, input_value_6);
        let target_range = (2, 11);
        let expected_check_1 = 3;
        let expected_check_2 = vec![(1, 4), (10, 12)];

        // Act
        let erased_length = range_set.erase_covering_segments(&target_range);
        let remained_segments: Vec<(i32, i32)> = range_set.iter().copied().collect();

        // Assert
        assert_eq!(erased_length, expected_check_1);
        assert_eq!(remained_segments, expected_check_2);
    }

    #[test]
    fn lower_bound_inside_segment_and_gap_returns_next_segment() {
        // Arrange
        let segments = [(1, 5), (10, 20)];
        let initial_queries = [2, 7];
        let expected = [Some((10, 20)), Some((10, 20))];
        let mut rs = RangeSet::new();
        for (left, right) in segments {
            rs.insert(left, right);
        }

        // Act
        let actual = initial_queries.map(|query| rs.lower_bound(query).copied());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn upper_bound_inside_segment_and_at_last_end_returns_next_or_none() {
        // Arrange
        let segments = [(1, 5), (10, 20)];
        let queries = [2, 20];
        let expected = [Some((10, 20)), None];
        let mut rs = RangeSet::new();
        for (left, right) in segments {
            rs.insert(left, right);
        }

        // Act
        let actual = queries.map(|query| rs.upper_bound(query).copied());

        // Assert
        assert_eq!(actual, expected);
    }
    #[test]
    fn contains_segment_inside_each_segment_and_after_end_returns_segment_or_none() {
        // Arrange
        let segments = [(1, 5), (10, 20)];
        let queries = [3, 15, 25];
        let expected = [Some((1, 5)), Some((10, 20)), None];
        let mut rs = RangeSet::new();
        for (left, right) in segments {
            rs.insert(left, right);
        }

        // Act
        let actual = queries.map(|query| rs.contains_segment(query).copied());

        // Assert
        assert_eq!(actual, expected);
    }
    #[test]
    fn insert_covering_zero_moves_mex_to_end_of_merged_leading_segment() {
        // Arrange
        let segments = [(1, 5), (10, 20)];
        let added_segment = (0, 1);
        let expected = (0, 5);
        let mut rs = RangeSet::new();
        for (left, right) in segments {
            rs.insert(left, right);
        }

        // Act
        let before = rs.mex();
        rs.insert(added_segment.0, added_segment.1);
        let after = rs.mex();

        let actual_check_1 = (before, after);

        // Assert
        assert_eq!(actual_check_1, expected);
    }
    #[test]
    fn insert_and_erase_near_u128_max_preserve_short_interval_lengths() {
        // Arrange
        let left = u128::MAX - 2;
        let middle = u128::MAX - 1;
        let right = u128::MAX;
        let expected = (2, 2, 1, 1, vec![(middle, right)]);
        let mut ranges = RangeSet::<u128>::new();

        // Act
        let inserted = ranges.insert(left, right);
        let total_after_insert = ranges.total;
        let erased = ranges.erase(left, middle);
        let total_after_erase = ranges.total;
        let segments = ranges.iter().copied().collect::<Vec<_>>();
        let actual = (
            inserted,
            total_after_insert,
            erased,
            total_after_erase,
            segments,
        );

        // Assert
        assert_eq!(actual, expected);
    }
}

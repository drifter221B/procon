use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitSet {
    bit_len: usize,
    words: Vec<u64>,
}

impl BitSet {
    pub fn new(bit_len: usize) -> Self {
        let word_len = (bit_len + 63) / 64;
        Self {
            bit_len,
            words: vec![0; word_len],
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.bit_len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bit_len == 0
    }

    pub fn get(&self, index: usize) -> bool {
        self.ensure_in_bounds(index);
        let (word_index, mask) = Self::mask(index);
        (self.words[word_index] & mask) != 0
    }

    pub fn set(&mut self, index: usize) {
        self.ensure_in_bounds(index);
        let (word_index, mask) = Self::mask(index);
        self.words[word_index] |= mask;
    }

    pub fn reset(&mut self, index: usize) {
        self.ensure_in_bounds(index);
        let (word_index, mask) = Self::mask(index);
        self.words[word_index] &= !mask;
    }

    pub fn flip(&mut self, index: usize) {
        self.ensure_in_bounds(index);
        let (word_index, mask) = Self::mask(index);
        self.words[word_index] ^= mask;
    }

    pub fn set_all(&mut self) {
        self.words.iter_mut().for_each(|word| *word = !0);
        self.mask_tail_bits();
    }

    pub fn clear_all(&mut self) {
        self.words.iter_mut().for_each(|word| *word = 0);
    }

    /// Updates this bit set as `self |= self << shift` without allocating.
    ///
    /// Bits shifted beyond `self.len()` are discarded. A zero shift, a shift at
    /// least `self.len()`, and an empty bit set leave the value unchanged.
    /// Runs in `O(ceil(self.len() / 64))` time and uses `O(1)` extra space.
    pub fn or_shift_left(&mut self, shift: usize) {
        if self.is_empty() || shift == 0 || shift >= self.bit_len {
            return;
        }

        let word_shift = shift / 64;
        let bit_shift = shift % 64;

        for destination_index in (word_shift..self.words.len()).rev() {
            let source_index = destination_index - word_shift;
            let mut shifted_word = self.words[source_index] << bit_shift;

            if bit_shift != 0 && source_index != 0 {
                shifted_word |= self.words[source_index - 1] >> (64 - bit_shift);
            }

            self.words[destination_index] |= shifted_word;
        }

        self.mask_tail_bits();
    }

    pub fn count_ones(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn any(&self) -> bool {
        self.words.iter().any(|&w| w != 0)
    }

    pub fn none(&self) -> bool {
        !self.any()
    }

    pub fn all(&self) -> bool {
        if self.bit_len == 0 {
            return true;
        }
        let full_words = self.bit_len / 64;
        let tail_bits = self.bit_len % 64;

        if self.words[..full_words]
            .iter()
            .any(|&word| word != u64::MAX)
        {
            return false;
        }

        if tail_bits == 0 {
            return true;
        }

        let tail_mask = (1u64 << tail_bits) - 1;

        self.words[full_words] == tail_mask
    }

    pub fn bit_or(&self, other: &Self) -> Self {
        self.apply_binary_operation(other, |left_word, right_word| left_word | right_word)
    }

    pub fn bit_and(&self, other: &Self) -> Self {
        self.apply_binary_operation(other, |left_word, right_word| left_word & right_word)
    }

    pub fn bit_xor(&self, other: &Self) -> Self {
        self.apply_binary_operation(other, |left_word, right_word| left_word ^ right_word)
    }

    fn apply_binary_operation<F>(&self, other: &Self, mut operation: F) -> Self
    where
        F: FnMut(u64, u64) -> u64,
    {
        self.ensure_same_bit_len(other);
        let result_words = self
            .words
            .iter()
            .copied()
            .zip(other.words.iter().copied())
            .map(|(left_word, right_word)| operation(left_word, right_word))
            .collect();

        Self {
            bit_len: self.bit_len,
            words: result_words,
        }
    }

    fn apply_binary_assign_operation<F>(&mut self, other: &Self, mut operation: F)
    where
        F: FnMut(u64, u64) -> u64,
    {
        self.ensure_same_bit_len(other);
        self.words
            .iter_mut()
            .zip(other.words.iter().copied())
            .for_each(|(left_word, right_word)| *left_word = operation(*left_word, right_word));
    }

    fn ensure_same_bit_len(&self, other: &Self) {
        assert_eq!(
            self.bit_len, other.bit_len,
            "bit length mismatch: left={}, right={}",
            self.bit_len, other.bit_len
        );
    }

    fn ensure_in_bounds(&self, index: usize) {
        assert!(
            index < self.bit_len,
            "index {} out of bounds {}",
            index,
            self.bit_len
        );
    }

    #[inline]
    fn mask(index: usize) -> (usize, u64) {
        let word_index = index / 64;
        let bit_offset = index % 64;

        (word_index, 1u64 << bit_offset)
    }

    fn mask_tail_bits(&mut self) {
        let tail_bits = self.bit_len % 64;
        if tail_bits == 0 {
            return;
        }
        let last = self.words.len() - 1;
        let tail_mask = (1u64 << tail_bits) - 1;

        self.words[last] &= tail_mask;
    }
}

impl BitOr<&BitSet> for &BitSet {
    type Output = BitSet;

    fn bitor(self, rhs: &BitSet) -> Self::Output {
        self.bit_or(rhs)
    }
}

impl BitOr for BitSet {
    type Output = BitSet;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.bit_or(&rhs)
    }
}

impl BitOrAssign<&BitSet> for BitSet {
    fn bitor_assign(&mut self, rhs: &BitSet) {
        self.apply_binary_assign_operation(rhs, |left_word, right_word| left_word | right_word);
    }
}

impl BitOrAssign for BitSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.apply_binary_assign_operation(&rhs, |left_word, right_word| left_word | right_word);
    }
}

impl BitAnd<&BitSet> for &BitSet {
    type Output = BitSet;

    fn bitand(self, rhs: &BitSet) -> Self::Output {
        self.bit_and(rhs)
    }
}

impl BitAnd for BitSet {
    type Output = BitSet;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.bit_and(&rhs)
    }
}

impl BitAndAssign<&BitSet> for BitSet {
    fn bitand_assign(&mut self, rhs: &BitSet) {
        self.apply_binary_assign_operation(rhs, |left_word, right_word| left_word & right_word);
    }
}

impl BitAndAssign for BitSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.apply_binary_assign_operation(&rhs, |left_word, right_word| left_word & right_word);
    }
}

impl BitXor<&BitSet> for &BitSet {
    type Output = BitSet;

    fn bitxor(self, rhs: &BitSet) -> Self::Output {
        self.bit_xor(rhs)
    }
}

impl BitXor for BitSet {
    type Output = BitSet;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.bit_xor(&rhs)
    }
}

impl BitXorAssign<&BitSet> for BitSet {
    fn bitxor_assign(&mut self, rhs: &BitSet) {
        self.apply_binary_assign_operation(rhs, |left_word, right_word| left_word ^ right_word);
    }
}

impl BitXorAssign for BitSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.apply_binary_assign_operation(&rhs, |left_word, right_word| left_word ^ right_word);
    }
}

impl Not for BitSet {
    type Output = BitSet;

    fn not(mut self) -> Self::Output {
        self.words.iter_mut().for_each(|word| *word = !*word);
        self.mask_tail_bits();
        self
    }
}

impl Not for &BitSet {
    type Output = BitSet;

    fn not(self) -> Self::Output {
        let mut result = self.clone();
        result.words.iter_mut().for_each(|word| *word = !*word);
        result.mask_tail_bits();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::BitSet;

    fn create_bitset_with_set_indices(bit_len: usize, set_indices: &[usize]) -> BitSet {
        let mut bitset = BitSet::new(bit_len);
        set_indices
            .iter()
            .copied()
            .for_each(|set_index| bitset.set(set_index));
        bitset
    }

    fn bitwise_or_shift_left_reference(
        bit_len: usize,
        source_set_indices: &[usize],
        shift: usize,
    ) -> BitSet {
        let mut expected = create_bitset_with_set_indices(bit_len, source_set_indices);
        for &source_index in source_set_indices {
            if let Some(destination_index) = source_index.checked_add(shift) {
                if destination_index < bit_len {
                    expected.set(destination_index);
                }
            }
        }
        expected
    }

    #[test]
    fn new_with_zero_length_returns_empty_state() {
        // Arrange
        let bit_len = 0;
        let expected_len = 0;
        let expected_is_empty = true;
        let expected_count_ones = 0;
        let expected_any = false;
        let expected_none = true;
        let expected_all = true;
        let bitset = BitSet::new(bit_len);

        // Act
        let actual_len = bitset.len();
        let actual_is_empty = bitset.is_empty();
        let actual_count_ones = bitset.count_ones();
        let actual_any = bitset.any();
        let actual_none = bitset.none();
        let actual_all = bitset.all();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_count_ones, expected_count_ones);
        assert_eq!(actual_any, expected_any);
        assert_eq!(actual_none, expected_none);
        assert_eq!(actual_all, expected_all);
    }

    #[test]
    fn new_with_one_length_reports_non_empty() {
        // Arrange
        let bit_len = 1;
        let expected_len = 1;
        let expected_is_empty = false;
        let bitset = BitSet::new(bit_len);

        // Act
        let actual_len = bitset.len();
        let actual_is_empty = bitset.is_empty();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn bit_operations_with_zero_length_return_empty_without_panicking() {
        // Arrange
        let bit_len = 0;
        let expected_result = BitSet::new(bit_len);
        let left = BitSet::new(bit_len);
        let right = BitSet::new(bit_len);

        // Act
        let actual_or_result = left.bit_or(&right);
        let actual_and_result = left.bit_and(&right);
        let actual_xor_result = left.bit_xor(&right);

        // Assert
        assert_eq!(actual_or_result, expected_result);
        assert_eq!(actual_and_result, expected_result);
        assert_eq!(actual_xor_result, expected_result);
    }

    #[test]
    fn get_panics_when_index_equals_len() {
        // Arrange
        let bit_len = 5;
        let out_of_bounds_index = bit_len;
        let bitset = BitSet::new(bit_len);

        // Act
        let actual_result = std::panic::catch_unwind(|| bitset.get(out_of_bounds_index));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn set_panics_when_index_equals_len() {
        // Arrange
        let bit_len = 5;
        let out_of_bounds_index = bit_len;
        let mut bitset = BitSet::new(bit_len);

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            bitset.set(out_of_bounds_index)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn reset_panics_when_index_equals_len() {
        // Arrange
        let bit_len = 5;
        let out_of_bounds_index = bit_len;
        let mut bitset = BitSet::new(bit_len);

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            bitset.reset(out_of_bounds_index)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn flip_panics_when_index_equals_len() {
        // Arrange
        let bit_len = 5;
        let out_of_bounds_index = bit_len;
        let mut bitset = BitSet::new(bit_len);

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            bitset.flip(out_of_bounds_index)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn set_and_get_work_for_minimum_index_zero() {
        // Arrange
        let bit_len = 10;
        let target_index = 0;
        let expected_value = true;
        let mut bitset = BitSet::new(bit_len);

        // Act
        bitset.set(target_index);
        let actual_value = bitset.get(target_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn set_and_get_work_for_maximum_valid_index() {
        // Arrange
        let bit_len = 10;
        let target_index = bit_len - 1;
        let expected_value = true;
        let mut bitset = BitSet::new(bit_len);

        // Act
        bitset.set(target_index);
        let actual_value = bitset.get(target_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn set_all_for_length_sixty_three_sets_all_bits() {
        // Arrange
        let bit_len = 63;
        let expected_count_ones = 63;
        let expected_all = true;
        let mut bitset = BitSet::new(bit_len);

        // Act
        bitset.set_all();
        let actual_count_ones = bitset.count_ones();
        let actual_all = bitset.all();

        // Assert
        assert_eq!(actual_count_ones, expected_count_ones);
        assert_eq!(actual_all, expected_all);
    }

    #[test]
    fn set_all_for_length_sixty_four_sets_all_bits() {
        // Arrange
        let bit_len = 64;
        let expected_count_ones = 64;
        let expected_all = true;
        let mut bitset = BitSet::new(bit_len);

        // Act
        bitset.set_all();
        let actual_count_ones = bitset.count_ones();
        let actual_all = bitset.all();

        // Assert
        assert_eq!(actual_count_ones, expected_count_ones);
        assert_eq!(actual_all, expected_all);
    }

    #[test]
    fn set_all_for_length_sixty_five_sets_all_bits() {
        // Arrange
        let bit_len = 65;
        let expected_count_ones = 65;
        let expected_all = true;
        let mut bitset = BitSet::new(bit_len);

        // Act
        bitset.set_all();
        let actual_count_ones = bitset.count_ones();
        let actual_all = bitset.all();

        // Assert
        assert_eq!(actual_count_ones, expected_count_ones);
        assert_eq!(actual_all, expected_all);
    }

    #[test]
    fn reset_clears_previously_set_bit() {
        // Arrange
        let bit_len = 12;
        let target_index = 4;
        let expected_value = false;
        let mut bitset = BitSet::new(bit_len);
        bitset.set(target_index);

        // Act
        bitset.reset(target_index);
        let actual_value = bitset.get(target_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn clear_all_clears_bits_after_set_all() {
        // Arrange
        let bit_len = 70;
        let expected_none = true;
        let mut bitset = BitSet::new(bit_len);
        bitset.set_all();

        // Act
        bitset.clear_all();
        let actual_none = bitset.none();

        // Assert
        assert_eq!(actual_none, expected_none);
    }

    #[test]
    fn any_and_none_transition_consistently_after_set_and_reset() {
        // Arrange
        let bit_len = 16;
        let target_index = 9;
        let expected_any_before_set = false;
        let expected_none_before_set = true;
        let expected_any_after_set = true;
        let expected_none_after_set = false;
        let expected_any_after_reset = false;
        let expected_none_after_reset = true;
        let mut bitset = BitSet::new(bit_len);

        // Act
        let actual_any_before_set = bitset.any();
        let actual_none_before_set = bitset.none();
        bitset.set(target_index);
        let actual_any_after_set = bitset.any();
        let actual_none_after_set = bitset.none();
        bitset.reset(target_index);
        let actual_any_after_reset = bitset.any();
        let actual_none_after_reset = bitset.none();

        // Assert
        assert_eq!(actual_any_before_set, expected_any_before_set);
        assert_eq!(actual_none_before_set, expected_none_before_set);
        assert_eq!(actual_any_after_set, expected_any_after_set);
        assert_eq!(actual_none_after_set, expected_none_after_set);
        assert_eq!(actual_any_after_reset, expected_any_after_reset);
        assert_eq!(actual_none_after_reset, expected_none_after_reset);
    }

    #[test]
    fn all_returns_false_when_one_bit_in_full_word_is_reset() {
        // Arrange
        let bit_len = 128;
        let target_index = 10;
        let expected_all = false;
        let mut bitset = BitSet::new(bit_len);
        bitset.set_all();

        // Act
        bitset.reset(target_index);
        let actual_all = bitset.all();

        // Assert
        assert_eq!(actual_all, expected_all);
    }

    #[test]
    fn all_returns_false_when_one_valid_tail_bit_is_reset() {
        // Arrange
        let bit_len = 65;
        let target_index = 64;
        let expected_all = false;
        let mut bitset = BitSet::new(bit_len);
        bitset.set_all();

        // Act
        bitset.reset(target_index);
        let actual_all = bitset.all();

        // Assert
        assert_eq!(actual_all, expected_all);
    }

    #[test]
    fn flip_once_turns_false_into_true() {
        // Arrange
        let bit_len = 8;
        let target_index = 3;
        let expected_value = true;
        let mut bitset = BitSet::new(bit_len);

        // Act
        bitset.flip(target_index);
        let actual_value = bitset.get(target_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn flip_twice_turns_true_back_to_false() {
        // Arrange
        let bit_len = 8;
        let target_index = 3;
        let expected_value = false;
        let mut bitset = BitSet::new(bit_len);

        // Act
        bitset.flip(target_index);
        bitset.flip(target_index);
        let actual_value = bitset.get(target_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn count_ones_counts_bits_across_multiple_words() {
        // Arrange
        let bit_len = 128;
        let set_indices = [0, 63, 64, 127];
        let expected_count_ones = 4;
        let bitset = create_bitset_with_set_indices(bit_len, &set_indices);

        // Act
        let actual_count_ones = bitset.count_ones();

        // Assert
        assert_eq!(actual_count_ones, expected_count_ones);
    }

    #[test]
    fn bit_or_returns_union_without_mutating_inputs() {
        // Arrange
        let bit_len = 10;
        let left_set_indices = [1, 3];
        let right_set_indices = [3, 4];
        let expected_result = create_bitset_with_set_indices(bit_len, &[1, 3, 4]);
        let left = create_bitset_with_set_indices(bit_len, &left_set_indices);
        let right = create_bitset_with_set_indices(bit_len, &right_set_indices);
        let expected_left_after = left.clone();
        let expected_right_after = right.clone();

        // Act
        let actual_result = left.bit_or(&right);
        let actual_left_after = left;
        let actual_right_after = right;

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_left_after, expected_left_after);
        assert_eq!(actual_right_after, expected_right_after);
    }

    #[test]
    fn bit_or_panics_when_lengths_differ() {
        // Arrange
        let left = BitSet::new(10);
        let right = BitSet::new(11);

        // Act
        let actual_result = std::panic::catch_unwind(|| left.bit_or(&right));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn bitor_operator_for_references_matches_bit_or_method() {
        // Arrange
        let bit_len = 12;
        let left = create_bitset_with_set_indices(bit_len, &[0, 1, 8]);
        let right = create_bitset_with_set_indices(bit_len, &[1, 2, 7]);
        let expected_result = left.bit_or(&right);

        // Act
        let actual_result = &left | &right;

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn bitor_operator_for_owned_values_matches_bit_or_method() {
        // Arrange
        let bit_len = 12;
        let left = create_bitset_with_set_indices(bit_len, &[0, 1, 8]);
        let right = create_bitset_with_set_indices(bit_len, &[1, 2, 7]);
        let expected_result = left.bit_or(&right);

        // Act
        let actual_result = left.clone() | right.clone();

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn bit_and_returns_intersection_without_mutating_inputs() {
        // Arrange
        let bit_len = 10;
        let left_set_indices = [1, 3];
        let right_set_indices = [3, 4];
        let expected_result = create_bitset_with_set_indices(bit_len, &[3]);
        let left = create_bitset_with_set_indices(bit_len, &left_set_indices);
        let right = create_bitset_with_set_indices(bit_len, &right_set_indices);
        let expected_left_after = left.clone();
        let expected_right_after = right.clone();

        // Act
        let actual_result = left.bit_and(&right);
        let actual_left_after = left;
        let actual_right_after = right;

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_left_after, expected_left_after);
        assert_eq!(actual_right_after, expected_right_after);
    }

    #[test]
    fn bit_and_panics_when_lengths_differ() {
        // Arrange
        let left = BitSet::new(10);
        let right = BitSet::new(11);

        // Act
        let actual_result = std::panic::catch_unwind(|| left.bit_and(&right));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn bitand_operator_for_references_matches_bit_and_method() {
        // Arrange
        let bit_len = 12;
        let left = create_bitset_with_set_indices(bit_len, &[0, 1, 8]);
        let right = create_bitset_with_set_indices(bit_len, &[1, 2, 7]);
        let expected_result = left.bit_and(&right);

        // Act
        let actual_result = &left & &right;

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn bitand_operator_for_owned_values_matches_bit_and_method() {
        // Arrange
        let bit_len = 12;
        let left = create_bitset_with_set_indices(bit_len, &[0, 1, 8]);
        let right = create_bitset_with_set_indices(bit_len, &[1, 2, 7]);
        let expected_result = left.bit_and(&right);

        // Act
        let actual_result = left.clone() & right.clone();

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn bit_xor_returns_symmetric_difference_without_mutating_inputs() {
        // Arrange
        let bit_len = 10;
        let left_set_indices = [1, 3];
        let right_set_indices = [3, 4];
        let expected_result = create_bitset_with_set_indices(bit_len, &[1, 4]);
        let left = create_bitset_with_set_indices(bit_len, &left_set_indices);
        let right = create_bitset_with_set_indices(bit_len, &right_set_indices);
        let expected_left_after = left.clone();
        let expected_right_after = right.clone();

        // Act
        let actual_result = left.bit_xor(&right);
        let actual_left_after = left;
        let actual_right_after = right;

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_left_after, expected_left_after);
        assert_eq!(actual_right_after, expected_right_after);
    }

    #[test]
    fn bit_xor_panics_when_lengths_differ() {
        // Arrange
        let left = BitSet::new(10);
        let right = BitSet::new(11);

        // Act
        let actual_result = std::panic::catch_unwind(|| left.bit_xor(&right));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn bitxor_operator_for_references_matches_bit_xor_method() {
        // Arrange
        let bit_len = 12;
        let left = create_bitset_with_set_indices(bit_len, &[0, 1, 8]);
        let right = create_bitset_with_set_indices(bit_len, &[1, 2, 7]);
        let expected_result = left.bit_xor(&right);

        // Act
        let actual_result = &left ^ &right;

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn bitxor_operator_for_owned_values_matches_bit_xor_method() {
        // Arrange
        let bit_len = 12;
        let left = create_bitset_with_set_indices(bit_len, &[0, 1, 8]);
        let right = create_bitset_with_set_indices(bit_len, &[1, 2, 7]);
        let expected_result = left.bit_xor(&right);

        // Act
        let actual_result = left.clone() ^ right.clone();

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn bitor_assign_ref_updates_only_left_side() {
        // Arrange
        let bit_len = 10;
        let left = create_bitset_with_set_indices(bit_len, &[0, 2]);
        let right = create_bitset_with_set_indices(bit_len, &[2, 5]);
        let expected_left_after = create_bitset_with_set_indices(bit_len, &[0, 2, 5]);
        let expected_right_after = right.clone();
        let mut actual_left = left;

        // Act
        actual_left |= &right;
        let actual_right_after = right;

        // Assert
        assert_eq!(actual_left, expected_left_after);
        assert_eq!(actual_right_after, expected_right_after);
    }

    #[test]
    fn bitor_assign_owned_panics_when_lengths_differ() {
        // Arrange
        let mut left = BitSet::new(10);
        let right = BitSet::new(11);

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| left |= right));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn bitand_assign_ref_updates_only_left_side() {
        // Arrange
        let bit_len = 10;
        let left = create_bitset_with_set_indices(bit_len, &[0, 2]);
        let right = create_bitset_with_set_indices(bit_len, &[2, 5]);
        let expected_left_after = create_bitset_with_set_indices(bit_len, &[2]);
        let expected_right_after = right.clone();
        let mut actual_left = left;

        // Act
        actual_left &= &right;
        let actual_right_after = right;

        // Assert
        assert_eq!(actual_left, expected_left_after);
        assert_eq!(actual_right_after, expected_right_after);
    }

    #[test]
    fn bitand_assign_owned_panics_when_lengths_differ() {
        // Arrange
        let mut left = BitSet::new(10);
        let right = BitSet::new(11);

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| left &= right));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn bitxor_assign_ref_updates_only_left_side() {
        // Arrange
        let bit_len = 10;
        let left = create_bitset_with_set_indices(bit_len, &[0, 2]);
        let right = create_bitset_with_set_indices(bit_len, &[2, 5]);
        let expected_left_after = create_bitset_with_set_indices(bit_len, &[0, 5]);
        let expected_right_after = right.clone();
        let mut actual_left = left;

        // Act
        actual_left ^= &right;
        let actual_right_after = right;

        // Assert
        assert_eq!(actual_left, expected_left_after);
        assert_eq!(actual_right_after, expected_right_after);
    }

    #[test]
    fn bitxor_assign_owned_panics_when_lengths_differ() {
        // Arrange
        let mut left = BitSet::new(10);
        let right = BitSet::new(11);

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| left ^= right));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn not_for_owned_value_flips_only_valid_bits() {
        // Arrange
        let bit_len = 65;
        let source_set_indices = [0, 64];
        let indices = (0, 63, 64);
        let expected_count_ones = 63;
        let expected_bit_at_zero = false;
        let expected_bit_at_sixty_three = true;
        let expected_bit_at_sixty_four = false;
        let source = create_bitset_with_set_indices(bit_len, &source_set_indices);

        // Act
        let actual_result = !source;
        let actual_count_ones = actual_result.count_ones();
        let actual_bit_at_zero = actual_result.get(indices.0);
        let actual_bit_at_sixty_three = actual_result.get(indices.1);
        let actual_bit_at_sixty_four = actual_result.get(indices.2);

        // Assert
        assert_eq!(actual_count_ones, expected_count_ones);
        assert_eq!(actual_bit_at_zero, expected_bit_at_zero);
        assert_eq!(actual_bit_at_sixty_three, expected_bit_at_sixty_three);
        assert_eq!(actual_bit_at_sixty_four, expected_bit_at_sixty_four);
    }

    #[test]
    fn not_for_reference_returns_flipped_copy_without_mutating_source() {
        // Arrange
        let bit_len = 3;
        let source_set_indices = [1];
        let expected_source_after = create_bitset_with_set_indices(bit_len, &[1]);
        let expected_result = create_bitset_with_set_indices(bit_len, &[0, 2]);
        let source = create_bitset_with_set_indices(bit_len, &source_set_indices);

        // Act
        let actual_result = !&source;
        let actual_source_after = source;

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_source_after, expected_source_after);
    }

    #[test]
    fn double_not_returns_original_bitset() {
        // Arrange
        let bit_len = 70;
        let source_set_indices = [0, 63, 69];
        let expected_result = create_bitset_with_set_indices(bit_len, &source_set_indices);
        let source = create_bitset_with_set_indices(bit_len, &source_set_indices);

        // Act
        let actual_result = !!source;

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn or_shift_left_handles_bit_and_word_boundaries_and_discards_tail_bits() {
        // Arrange
        let bit_len = 130;
        let source_set_indices = [0, 2, 62, 64, 127];
        let shifts = [1, 63, 64, 65];
        let expected_results: Vec<_> = shifts
            .iter()
            .map(|&shift| bitwise_or_shift_left_reference(bit_len, &source_set_indices, shift))
            .collect();

        // Act
        let actual_results: Vec<_> = shifts
            .iter()
            .map(|&shift| {
                let mut bitset = create_bitset_with_set_indices(bit_len, &source_set_indices);
                bitset.or_shift_left(shift);
                bitset
            })
            .collect();

        // Assert
        for (actual_result, expected_result) in actual_results.iter().zip(expected_results.iter()) {
            assert_eq!(actual_result, expected_result);
        }
    }

    #[test]
    fn or_shift_left_does_not_reuse_bits_added_in_same_call() {
        // Arrange
        let bit_len = 130;
        let shift = 1;
        let queried_index = 65;
        let source_set_indices = [0, 63];
        let expected_result = bitwise_or_shift_left_reference(bit_len, &source_set_indices, shift);
        let expected_bit_at_sixty_five = false;
        let mut bitset = create_bitset_with_set_indices(bit_len, &source_set_indices);
        let expected_words_len = bitset.words.len();
        let expected_words_capacity = bitset.words.capacity();
        let expected_words_ptr = bitset.words.as_ptr();

        // Act
        bitset.or_shift_left(shift);
        let actual_result = bitset;
        let actual_words_len = actual_result.words.len();
        let actual_words_capacity = actual_result.words.capacity();
        let actual_words_ptr = actual_result.words.as_ptr();
        let actual_bit_at_sixty_five = actual_result.get(queried_index);

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_bit_at_sixty_five, expected_bit_at_sixty_five);
        assert_eq!(actual_words_len, expected_words_len);
        assert_eq!(actual_words_capacity, expected_words_capacity);
        assert_eq!(actual_words_ptr, expected_words_ptr);
    }

    #[test]
    fn or_shift_left_matches_naive_subset_sum_updates_for_multiple_weights() {
        // Arrange
        let bit_len = 140;
        let weights = [1, 63, 64, 65, 7];
        let mut expected_reachable = vec![false; bit_len];
        expected_reachable[0] = true;
        for &weight in &weights {
            let reachable_before = expected_reachable.clone();
            for index in 0..bit_len.saturating_sub(weight) {
                if reachable_before[index] {
                    expected_reachable[index + weight] = true;
                }
            }
        }
        let mut bitset = create_bitset_with_set_indices(bit_len, &[0]);

        // Act
        for &weight in &weights {
            bitset.or_shift_left(weight);
        }
        let actual_reachable: Vec<_> = (0..bit_len).map(|index| bitset.get(index)).collect();

        // Assert
        assert_eq!(actual_reachable, expected_reachable);
    }

    #[test]
    fn or_shift_left_is_no_op_for_empty_zero_and_large_shifts_without_reallocation() {
        // Arrange
        let empty_bit_len = 0;
        let bit_len = 65;
        let source_set_indices = [0, 64];
        let zero_shift = 0;
        let length_shift = bit_len;
        let large_shift = usize::MAX;
        let expected_empty = BitSet::new(empty_bit_len);
        let mut empty = BitSet::new(empty_bit_len);
        let mut bitset = create_bitset_with_set_indices(bit_len, &source_set_indices);
        let expected_bitset = bitset.clone();
        let expected_words_len = bitset.words.len();
        let expected_words_capacity = bitset.words.capacity();
        let expected_words_ptr = bitset.words.as_ptr();

        // Act
        empty.or_shift_left(large_shift);
        bitset.or_shift_left(zero_shift);
        bitset.or_shift_left(length_shift);
        bitset.or_shift_left(large_shift);
        let actual_empty = empty;
        let actual_words_len = bitset.words.len();
        let actual_words_capacity = bitset.words.capacity();
        let actual_words_ptr = bitset.words.as_ptr();

        // Assert
        assert_eq!(actual_empty, expected_empty);
        assert_eq!(bitset, expected_bitset);
        assert_eq!(actual_words_len, expected_words_len);
        assert_eq!(actual_words_capacity, expected_words_capacity);
        assert_eq!(actual_words_ptr, expected_words_ptr);
    }
}

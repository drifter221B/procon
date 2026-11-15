use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const DEFAULT_SEED: u64 = 0x9e37_79b9_7f4a_7c15;

/// 多重集合のハッシュを、各値のハッシュの和で管理する。
///
/// `H = Σ count(x) * h(seed, x) mod 2^64` として計算するため、挿入順序に依存しない。
/// 同じ seed、キーの型、ハッシュ実装で作成した多重集合同士を比較できる。
/// また、多重集合の和のハッシュは、各ハッシュの `wrapping_add` に等しい。
/// キーのハッシュ計算と比較を除き、追加・削除は平均 O(1)、空間は O(異なる値の数)。
///
/// 非暗号用途向けであり、異なる多重集合のハッシュが衝突することがある。
/// ハッシュが 0 でも空とは限らないため、空判定には [`Self::is_empty`] を使う。
/// `DefaultHasher` の仕様は固定されていないため、Rust のバージョンや環境をまたぐ
/// 永続化・比較にハッシュ値を使わないこと。
#[derive(Clone, Debug)]
pub struct AdditiveZobristHash<T> {
    seed: u64,
    hash_value: u64,
    element_count: usize,
    element_counts: HashMap<T, usize>,
}

impl<T: Eq + Hash> Default for AdditiveZobristHash<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash> AdditiveZobristHash<T> {
    /// 既定の seed で空の多重集合を作る。
    pub fn new() -> Self {
        Self::with_seed(DEFAULT_SEED)
    }

    /// 指定した seed で空の多重集合を作る。
    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed,
            hash_value: 0,
            element_count: 0,
            element_counts: HashMap::new(),
        }
    }

    /// スライスの各値を複製し、既定の seed で多重集合を作る。
    pub fn from_slice(values: &[T]) -> Self
    where
        T: Clone,
    {
        let mut zobrist_hash = Self::new();

        for value in values {
            zobrist_hash.insert(value.clone());
        }

        zobrist_hash
    }

    /// 重複を含む要素数を返す。
    #[inline]
    pub fn len(&self) -> usize {
        self.element_count
    }

    /// 異なる値の数を返す。
    #[inline]
    pub fn distinct_len(&self) -> usize {
        self.element_counts.len()
    }

    /// 要素が存在しないかを返す。
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.element_count == 0
    }

    /// 多重集合のハッシュを返す。空なら 0 だが、0 でも空とは限らない。
    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash_value
    }

    /// 指定した値が存在するかを返す。
    pub fn contains(&self, value: &T) -> bool {
        self.element_counts.contains_key(value)
    }

    /// 指定した値の個数を返す。未登録なら 0 を返す。
    pub fn count(&self, value: &T) -> usize {
        self.element_counts.get(value).copied().unwrap_or(0)
    }

    /// 指定した値を 1 個追加する。
    ///
    /// # Panics
    ///
    /// 総要素数または値の個数が `usize::MAX` を超える場合、状態を変更せず panic する。
    pub fn insert(&mut self, value: T) {
        self.insert_multiple(value, 1);
    }

    /// 指定した値を `insert_count` 個追加する。0 個なら何もしない。
    ///
    /// # Panics
    ///
    /// 総要素数または値の個数が `usize::MAX` を超える場合、状態を変更せず panic する。
    pub fn insert_multiple(&mut self, value: T, insert_count: usize) {
        if insert_count == 0 {
            return;
        }

        let current_count = self.count(&value);
        let Some(next_count) = current_count.checked_add(insert_count) else {
            panic!("element count exceeds usize::MAX");
        };
        let Some(next_element_count) = self.element_count.checked_add(insert_count) else {
            panic!("total element count exceeds usize::MAX");
        };
        let added_hash = Self::element_hash(self.seed, &value).wrapping_mul(insert_count as u64);

        self.hash_value = self.hash_value.wrapping_add(added_hash);
        self.element_count = next_element_count;
        self.element_counts.insert(value, next_count);
    }

    /// 指定した値を 1 個削除し、削除できたかを返す。
    pub fn remove(&mut self, value: &T) -> bool {
        self.remove_multiple(value, 1) == 1
    }

    /// 指定した値を最大 `remove_count` 個削除し、実際に削除した個数を返す。
    ///
    /// 0 個の指定や未登録の値に対しては、状態を変更せず 0 を返す。
    pub fn remove_multiple(&mut self, value: &T, remove_count: usize) -> usize {
        if remove_count == 0 {
            return 0;
        }

        let Some(current_count) = self.element_counts.get(value).copied() else {
            return 0;
        };

        let removed_count = current_count.min(remove_count);
        let next_count = current_count - removed_count;
        let removed_hash = Self::element_hash(self.seed, value).wrapping_mul(removed_count as u64);

        self.hash_value = self.hash_value.wrapping_sub(removed_hash);
        self.element_count -= removed_count;

        if next_count == 0 {
            self.element_counts.remove(value);
        } else if let Some(count) = self.element_counts.get_mut(value) {
            *count = next_count;
        }

        removed_count
    }

    /// 指定した値をすべて削除し、削除した個数を返す。
    pub fn remove_all(&mut self, value: &T) -> usize {
        let current_count = self.count(value);

        self.remove_multiple(value, current_count)
    }

    /// seed を保ったまま、ハッシュと全要素を空の状態に戻す。
    pub fn clear(&mut self) {
        self.hash_value = 0;
        self.element_count = 0;
        self.element_counts.clear();
    }

    fn element_hash(seed: u64, value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        value.hash(&mut hasher);

        Self::splitmix64(hasher.finish())
    }

    fn splitmix64(value: u64) -> u64 {
        let mut x = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);

        x ^ (x >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::AdditiveZobristHash;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    const TEST_SEED: u64 = 1_234_567_891;
    const HASH_MODULUS: u128 = 1u128 << 64;

    #[derive(Debug, PartialEq, Eq)]
    struct State {
        hash: u64,
        len: usize,
        distinct_len: usize,
        is_empty: bool,
        counts: Vec<(usize, bool)>,
    }

    fn build_seeded_hash(seed: u64, values: &[i32]) -> AdditiveZobristHash<i32> {
        let mut hash = AdditiveZobristHash::with_seed(seed);
        for &value in values {
            hash.insert(value);
        }
        hash
    }

    fn state(hash: &AdditiveZobristHash<i32>, values: &[i32]) -> State {
        State {
            hash: hash.hash(),
            len: hash.len(),
            distinct_len: hash.distinct_len(),
            is_empty: hash.is_empty(),
            counts: values
                .iter()
                .map(|value| (hash.count(value), hash.contains(value)))
                .collect(),
        }
    }

    fn model_state(seed: u64, values: &[i32], probes: &[i32]) -> State {
        let mut distinct_values = values.to_vec();
        distinct_values.sort_unstable();
        distinct_values.dedup();
        let total_hash: u128 = values
            .iter()
            .map(|&value| u128::from(build_seeded_hash(seed, &[value]).hash()))
            .sum();
        State {
            hash: (total_hash % HASH_MODULUS) as u64,
            len: values.len(),
            distinct_len: distinct_values.len(),
            is_empty: values.is_empty(),
            counts: probes
                .iter()
                .map(|probe| {
                    let count = values.iter().filter(|value| *value == probe).count();
                    (count, count != 0)
                })
                .collect(),
        }
    }

    #[test]
    fn empty_constructors_return_zero_hash_and_counts() {
        // Arrange
        let values: [i32; 0] = [];
        let probes = [7];
        let expected_state = State {
            hash: 0,
            len: 0,
            distinct_len: 0,
            is_empty: true,
            counts: vec![(0, false)],
        };

        // Act
        let hashes = [
            AdditiveZobristHash::new(),
            AdditiveZobristHash::default(),
            AdditiveZobristHash::from_slice(&values),
        ];
        let actual_states = hashes.map(|hash| state(&hash, &probes));

        // Assert
        for actual_state in actual_states {
            assert_eq!(actual_state, expected_state);
        }
    }

    #[test]
    fn default_and_from_slice_use_the_same_seed_as_new() {
        // Arrange
        let values = [7];
        let probes = [7, 9];
        let mut expected_hash = AdditiveZobristHash::new();
        expected_hash.insert(values[0]);
        let expected_state = state(&expected_hash, &probes);
        let mut default_hash = AdditiveZobristHash::default();

        // Act
        default_hash.insert(values[0]);
        let slice_hash = AdditiveZobristHash::from_slice(&values);
        let actual_states = [state(&default_hash, &probes), state(&slice_hash, &probes)];

        // Assert
        for actual_state in actual_states {
            assert_eq!(actual_state, expected_state);
        }
    }

    #[test]
    fn from_slice_counts_duplicates_and_is_order_independent() {
        // Arrange
        let first_values = [1, 2, 1, 3, 2, 1];
        let second_values = [3, 1, 2, 1, 1, 2];
        let probes = [1, 2, 3, 4];
        let expected_counts = vec![(3, true), (2, true), (1, true), (0, false)];
        let expected_len = first_values.len();
        let expected_distinct_len = 3;

        // Act
        let first_hash = AdditiveZobristHash::from_slice(&first_values);
        let second_hash = AdditiveZobristHash::from_slice(&second_values);
        let actual_first_state = state(&first_hash, &probes);
        let actual_second_state = state(&second_hash, &probes);

        // Assert
        assert_eq!(actual_first_state, actual_second_state);
        assert_eq!(actual_first_state.counts, expected_counts);
        assert_eq!(actual_first_state.len, expected_len);
        assert_eq!(actual_first_state.distinct_len, expected_distinct_len);
    }

    #[test]
    fn seeded_hash_is_reproducible_and_uses_the_seed() {
        // Arrange
        let values = [4, 4, 9];
        let first_seed = TEST_SEED;
        let second_seed = TEST_SEED + 1;
        let expected_hash = build_seeded_hash(first_seed, &values).hash();

        // Act
        let actual_same_seed_hash = build_seeded_hash(first_seed, &values).hash();
        let actual_other_seed_hash = build_seeded_hash(second_seed, &values).hash();

        // Assert
        assert_eq!(actual_same_seed_hash, expected_hash);
        assert_ne!(actual_other_seed_hash, expected_hash);
    }

    #[test]
    fn multiset_union_hash_is_the_sum_even_with_shared_duplicate_keys() {
        // Arrange
        let left_values = [1, 1, 2, 3];
        let right_values = [1, 2, 2, 4];
        let probes = [1, 2, 3, 4, 5];
        let union_values: Vec<_> = left_values.into_iter().chain(right_values).collect();
        let expected_state = model_state(TEST_SEED, &union_values, &probes);
        let left_hash = build_seeded_hash(TEST_SEED, &left_values).hash();
        let right_hash = build_seeded_hash(TEST_SEED, &right_values).hash();
        let expected_sum = left_hash.wrapping_add(right_hash);

        // Act
        let union_hash = build_seeded_hash(TEST_SEED, &union_values);
        let actual_state = state(&union_hash, &probes);

        // Assert
        assert_eq!(actual_state.hash, expected_sum);
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn insert_multiple_for_new_and_existing_keys_matches_repeated_insertions() {
        // Arrange
        let target_value = 10;
        let other_value = 20;
        let initial_values = [vec![other_value], vec![other_value, target_value]];
        let probes = [target_value, other_value];
        let insert_count = 3;

        // Act
        let actual_and_expected_states = initial_values.map(|values| {
            let mut bulk_hash = build_seeded_hash(TEST_SEED, &values);
            let mut repeated_hash = build_seeded_hash(TEST_SEED, &values);
            bulk_hash.insert_multiple(target_value, insert_count);
            for _ in 0..insert_count {
                repeated_hash.insert(target_value);
            }
            (state(&bulk_hash, &probes), state(&repeated_hash, &probes))
        });

        // Assert
        for (actual_state, expected_state) in actual_and_expected_states {
            assert_eq!(actual_state, expected_state);
        }
    }

    #[test]
    fn inserting_then_removing_the_same_multiplicity_restores_the_state() {
        // Arrange
        let target_value = 10;
        let initial_values = [target_value, 20];
        let probes = [target_value, 20];
        let change_count = 7;
        let mut hash = build_seeded_hash(TEST_SEED, &initial_values);
        let expected_state = state(&hash, &probes);
        let expected_removed = change_count;

        // Act
        hash.insert_multiple(target_value, change_count);
        let actual_removed = hash.remove_multiple(&target_value, change_count);
        let actual_state = state(&hash, &probes);

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn zero_count_and_missing_key_operations_leave_state_unchanged() {
        // Arrange
        let target_value = 10;
        let missing_value = 20;
        let zero_count = 0;
        let remove_count = 3;
        let probes = [target_value, missing_value];
        let mut hash = build_seeded_hash(TEST_SEED, &[target_value, target_value]);
        let expected_state = state(&hash, &probes);
        let expected_removed = false;
        let expected_removed_counts = [zero_count; 4];

        // Act
        hash.insert_multiple(target_value, zero_count);
        hash.insert_multiple(missing_value, zero_count);
        let actual_removed = hash.remove(&missing_value);
        let actual_removed_counts = [
            hash.remove_multiple(&target_value, zero_count),
            hash.remove_multiple(&missing_value, zero_count),
            hash.remove_multiple(&missing_value, remove_count),
            hash.remove_all(&missing_value),
        ];
        let actual_state = state(&hash, &probes);

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_removed_counts, expected_removed_counts);
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn remove_multiple_at_count_boundaries_preserves_other_keys() {
        // Arrange
        let target_value = 10;
        let other_value = 20;
        let initial_values = [target_value, target_value, target_value, other_value];
        let probes = [target_value, other_value];
        let count = 3;
        let remove_counts = [count - 1, count, count + 1];
        let expected_states = [
            (
                count - 1,
                model_state(TEST_SEED, &[target_value, other_value], &probes),
            ),
            (count, model_state(TEST_SEED, &[other_value], &probes)),
            (count, model_state(TEST_SEED, &[other_value], &probes)),
        ];

        // Act
        let actual_states = remove_counts.map(|remove_count| {
            let mut hash = build_seeded_hash(TEST_SEED, &initial_values);
            let removed = hash.remove_multiple(&target_value, remove_count);
            (removed, state(&hash, &probes))
        });

        // Assert
        assert_eq!(actual_states, expected_states);
    }

    #[test]
    fn remove_last_element_clears_its_key_and_hash() {
        // Arrange
        let target_value = 10;
        let probes = [target_value];
        let mut hash = build_seeded_hash(TEST_SEED, &[target_value]);
        let expected_removed = true;
        let expected_state = model_state(TEST_SEED, &[], &probes);

        // Act
        let actual_removed = hash.remove(&target_value);
        let actual_state = state(&hash, &probes);

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn remove_all_removes_only_the_selected_key() {
        // Arrange
        let target_value = 10;
        let other_value = 20;
        let probes = [target_value, other_value];
        let mut hash = build_seeded_hash(TEST_SEED, &[target_value, other_value, target_value]);
        let expected_removed = 2;
        let expected_state = model_state(TEST_SEED, &[other_value], &probes);

        // Act
        let actual_removed = hash.remove_all(&target_value);
        let actual_state = state(&hash, &probes);

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn clear_resets_all_counts_and_preserves_seed_when_reused() {
        // Arrange
        let values = [10, 20, 10];
        let probes = [10, 20];
        let mut hash = build_seeded_hash(TEST_SEED, &values);
        let expected_empty_state = model_state(TEST_SEED, &[], &probes);
        let expected_reused_state = model_state(TEST_SEED, &values, &probes);

        // Act
        hash.clear();
        let actual_empty_state = state(&hash, &probes);
        for value in values {
            hash.insert(value);
        }
        let actual_reused_state = state(&hash, &probes);

        // Assert
        assert_eq!(actual_empty_state, expected_empty_state);
        assert_eq!(actual_reused_state, expected_reused_state);
    }

    #[test]
    fn hash_addition_multiplication_and_subtraction_wrap_modulo_two_to_64() {
        // Arrange
        let candidates = -128..=128;
        let half_range = u64::MAX / 2;
        let target_value = candidates
            .into_iter()
            .find(|&value| build_seeded_hash(TEST_SEED, &[value]).hash() > half_range)
            .expect("at least one candidate must exercise hash arithmetic overflow");
        let singleton_hash = build_seeded_hash(TEST_SEED, &[target_value]).hash();
        let doubled_hash = u128::from(singleton_hash) * 2;
        let insert_count = 3;
        let multiplied_hash = u128::from(singleton_hash) * insert_count as u128;
        let expected_doubled_hash = (doubled_hash % HASH_MODULUS) as u64;
        let expected_multiplied_hash = (multiplied_hash % HASH_MODULUS) as u64;
        let expected_removed = true;
        let mut hash = build_seeded_hash(TEST_SEED, &[target_value]);
        let mut bulk_hash = AdditiveZobristHash::with_seed(TEST_SEED);

        // Act
        hash.insert(target_value);
        let actual_doubled_hash = hash.hash();
        let actual_removed = hash.remove(&target_value);
        let actual_restored_hash = hash.hash();
        bulk_hash.insert_multiple(target_value, insert_count);
        let actual_multiplied_hash = bulk_hash.hash();

        // Assert
        assert!(doubled_hash >= HASH_MODULUS);
        assert!(multiplied_hash >= HASH_MODULUS);
        assert!(expected_doubled_hash < singleton_hash);
        assert_eq!(actual_doubled_hash, expected_doubled_hash);
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_restored_hash, singleton_hash);
        assert_eq!(actual_multiplied_hash, expected_multiplied_hash);
    }

    #[test]
    fn insert_at_key_count_limit_panics_without_changing_state() {
        // Arrange
        let target_value = 10;
        let probes = [target_value, 20];
        let maximum_count = usize::MAX;
        let below_maximum_count = maximum_count - 1;
        let singleton_hash = build_seeded_hash(TEST_SEED, &[target_value]).hash();
        let expected_hash =
            (u128::from(singleton_hash) * maximum_count as u128 % HASH_MODULUS) as u64;
        let expected_state = State {
            hash: expected_hash,
            len: maximum_count,
            distinct_len: 1,
            is_empty: false,
            counts: vec![(maximum_count, true), (0, false)],
        };
        let mut hash = AdditiveZobristHash::with_seed(TEST_SEED);
        hash.insert_multiple(target_value, below_maximum_count);

        // Act
        hash.insert(target_value);
        let actual_maximum_state = state(&hash, &probes);
        let actual_result = catch_unwind(AssertUnwindSafe(|| hash.insert(target_value)));
        let actual_after_panic_state = state(&hash, &probes);

        // Assert
        assert_eq!(actual_maximum_state, expected_state);
        assert!(actual_result.is_err());
        assert_eq!(actual_after_panic_state, expected_state);
    }

    #[test]
    fn insert_at_total_count_limit_preserves_existing_and_missing_keys_on_panic() {
        // Arrange
        let first_value = 10;
        let second_value = 20;
        let missing_value = 30;
        let maximum_count = usize::MAX;
        let first_count = maximum_count - 1;
        let second_count = 1;
        let probes = [first_value, second_value, missing_value];
        let first_hash = build_seeded_hash(TEST_SEED, &[first_value]).hash();
        let second_hash = build_seeded_hash(TEST_SEED, &[second_value]).hash();
        let expected_hash = ((u128::from(first_hash) * first_count as u128
            + u128::from(second_hash))
            % HASH_MODULUS) as u64;
        let expected_state = State {
            hash: expected_hash,
            len: maximum_count,
            distinct_len: 2,
            is_empty: false,
            counts: vec![(first_count, true), (second_count, true), (0, false)],
        };
        let mut hash = AdditiveZobristHash::with_seed(TEST_SEED);
        hash.insert_multiple(first_value, first_count);
        hash.insert_multiple(second_value, second_count);

        // Act
        let actual_existing_result = catch_unwind(AssertUnwindSafe(|| hash.insert(second_value)));
        let actual_after_existing_state = state(&hash, &probes);
        let actual_missing_result = catch_unwind(AssertUnwindSafe(|| hash.insert(missing_value)));
        let actual_after_missing_state = state(&hash, &probes);

        // Assert
        assert!(actual_existing_result.is_err());
        assert!(actual_missing_result.is_err());
        assert_eq!(actual_after_existing_state, expected_state);
        assert_eq!(actual_after_missing_state, expected_state);
    }

    #[test]
    fn insert_and_remove_support_keys_without_clone() {
        // Arrange
        #[derive(Debug, PartialEq, Eq, Hash)]
        struct Key(i32);
        let target_id = 10;
        let lookup_key = Key(target_id);
        let insert_count = 3;
        let expected_count = 2;
        let expected_removed = true;
        let expected_removed_all = expected_count;
        let expected_empty_state = (0, 0, 0, true);
        let mut hash = AdditiveZobristHash::with_seed(TEST_SEED);

        // Act
        hash.insert_multiple(Key(target_id), insert_count);
        let actual_removed = hash.remove(&lookup_key);
        let actual_count = hash.count(&lookup_key);
        let actual_removed_all = hash.remove_all(&lookup_key);
        let actual_empty_state = (
            hash.hash(),
            hash.len(),
            hash.distinct_len(),
            hash.is_empty(),
        );

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_removed_all, expected_removed_all);
        assert_eq!(actual_empty_state, expected_empty_state);
    }

    #[test]
    fn mixed_insertions_and_removals_match_a_vector_multiset_model_after_each_step() {
        // Arrange
        let operations = [
            (10, 3),
            (20, 2),
            (10, -1),
            (30, -4),
            (20, 0),
            (20, -5),
            (10, 2),
            (30, 1),
            (10, -4),
            (30, -1),
            (20, 2),
        ];
        let probes = [10, 20, 30];
        let mut hash = AdditiveZobristHash::with_seed(TEST_SEED);
        let mut model = Vec::new();
        let mut actual_and_expected_states = Vec::new();

        // Act
        for (value, delta) in operations {
            let count = i32::unsigned_abs(delta) as usize;
            let expected_removed = model
                .iter()
                .filter(|&&entry| entry == value)
                .count()
                .min(count);
            let actual_and_expected_removed = if delta < 0 {
                let removed = hash.remove_multiple(&value, count);
                for _ in 0..expected_removed {
                    let index = model.iter().position(|&entry| entry == value).unwrap();
                    model.swap_remove(index);
                }
                (removed, expected_removed)
            } else {
                hash.insert_multiple(value, count);
                model.extend(std::iter::repeat_n(value, count));
                (0, 0)
            };
            actual_and_expected_states.push((
                actual_and_expected_removed,
                state(&hash, &probes),
                model_state(TEST_SEED, &model, &probes),
            ));
        }

        // Assert
        for ((actual_removed, expected_removed), actual_state, expected_state) in
            actual_and_expected_states
        {
            assert_eq!(actual_removed, expected_removed);
            assert_eq!(actual_state, expected_state);
        }
    }
}

/// 64bit 整数の下位 64bit について、立っているビット数を数える。
pub fn pop_count(n: u64) -> u64 {

    let mut count = (n & 0x5555_5555_5555_5555) + ((n >> 1) & 0x5555_5555_5555_5555);
    count = (count & 0x3333_3333_3333_3333) + ((count >> 2) & 0x3333_3333_3333_3333);
    count = (count & 0x0f0f_0f0f_0f0f_0f0f) + ((count >> 4) & 0x0f0f_0f0f_0f0f_0f0f);
    count = (count & 0x00ff_00ff_00ff_00ff) + ((count >> 8) & 0x00ff_00ff_00ff_00ff);
    count = (count & 0x0000_ffff_0000_ffff) + ((count >> 16) & 0x0000_ffff_0000_ffff);
    count = (count & 0x0000_0000_ffff_ffff) + ((count >> 32) & 0x0000_0000_ffff_ffff);

    count
}

#[cfg(test)]
mod tests {
    use super::pop_count;

    #[test]
    fn pop_count_for_zero_and_small_numbers_matches_expected_counts() {
        // Arrange
        let values = [0, 1, 2, 3];
        let expected_counts = [0, 1, 1, 2];
        // Act
        let actual_counts = values.map(pop_count);
        // Assert
        assert_eq!(actual_counts, expected_counts);
    }

    #[test]
    fn pop_count_with_highest_bit_and_lowest_bit_counts_each_bit_once() {
        // Arrange
        let highest_bit = 1u64 << 63;
        let values = [highest_bit, highest_bit | 1];
        let expected_counts = [1, 2];
        // Act
        let actual_counts = values.map(pop_count);
        // Assert
        assert_eq!(actual_counts, expected_counts);
    }

    #[test]
    fn pop_count_for_sparse_pattern_returns_four() {
        // Arrange
        let value = 0b101101u64;
        let expected_count = 4;
        // Act
        let actual_count = pop_count(value);
        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn pop_count_at_u64_max_and_previous_value_returns_sixty_four_and_sixty_three() {
        // Arrange
        let values = [u64::MAX - 1, u64::MAX];
        let expected_counts = [63, 64];
        // Act
        let actual_counts = values.map(pop_count);
        // Assert
        assert_eq!(actual_counts, expected_counts);
    }

    #[test]
    fn power_of_two_boundaries_have_expected_popcounts() {
        // Arrange
        let center = 1u64 << 5;
        let values = [center - 1, center, center + 1];
        let expected_counts = [5, 1, 2];
        // Act
        let actual_counts = values.map(pop_count);
        // Assert
        assert_eq!(actual_counts, expected_counts);
    }

    #[test]
    fn alternating_bit_pattern_is_counted_correctly() {
        // Arrange
        let value = 0b1010_1010_1010_1010u64;
        let expected_count = 8;
        // Act
        let actual_count = pop_count(value);
        // Assert
        assert_eq!(actual_count, expected_count);
    }
}

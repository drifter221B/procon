fn bit_reverse_permute<T>(values: &mut [T]) {
    let value_count = values.len();
    assert!(
        value_count.is_power_of_two(),
        "input length must be power of two"
    );

    let mut reversed_index = 0usize;
    for current_index in 1..value_count {
        let mut bit = value_count >> 1;
        while reversed_index & bit != 0 {
            reversed_index ^= bit;
            bit >>= 1;
        }
        reversed_index ^= bit;

        if current_index < reversed_index {
            values.swap(current_index, reversed_index);
        }
    }
}

pub fn convolution_fft_i64(lhs_values: &[i64], rhs_values: &[i64]) -> Vec<i64> {
    if lhs_values.is_empty() || rhs_values.is_empty() {
        return vec![];
    }

    let residue_values0 = convolution_i64_with_config(lhs_values, rhs_values, I64_NTT_CONFIGS[0]);
    let residue_values1 = convolution_i64_with_config(lhs_values, rhs_values, I64_NTT_CONFIGS[1]);
    let residue_values2 = convolution_i64_with_config(lhs_values, rhs_values, I64_NTT_CONFIGS[2]);
    let crt_context = CrtContext::new(I64_NTT_CONFIGS);

    let result_length = lhs_values.len() + rhs_values.len() - 1;
    let mut result_values = Vec::with_capacity(result_length);
    for index in 0..result_length {
        let residues = [
            residue_values0[index],
            residue_values1[index],
            residue_values2[index],
        ];
        result_values.push(crt_context.reconstruct_signed_i64(residues));
    }

    result_values
}

const MODULUS: u64 = 998_244_353;
const PRIMITIVE_ROOT: u64 = 3;
const MAX_TWO_POWER_LENGTH: usize = 1usize << 23;

#[derive(Clone, Copy)]
struct NttConfig {
    modulus: u64,
    primitive_root: u64,
    max_two_power_length: usize,
}

const MOD998244353_CONFIG: NttConfig = NttConfig {
    modulus: MODULUS,
    primitive_root: PRIMITIVE_ROOT,
    max_two_power_length: MAX_TWO_POWER_LENGTH,
};

const I64_NTT_CONFIGS: [NttConfig; 3] = [
    NttConfig {
        modulus: 167_772_161,
        primitive_root: 3,
        max_two_power_length: 1usize << 25,
    },
    NttConfig {
        modulus: 469_762_049,
        primitive_root: 3,
        max_two_power_length: 1usize << 26,
    },
    NttConfig {
        modulus: 1_224_736_769,
        primitive_root: 3,
        max_two_power_length: 1usize << 24,
    },
];

struct CrtContext {
    first_modulus: i128,
    second_modulus: i128,
    third_modulus: i128,
    first_second_modulus: i128,
    total_modulus: i128,
    inverse_first_modulus_in_second: i128,
    inverse_first_second_modulus_in_third: i128,
}

impl CrtContext {
    fn new(configs: [NttConfig; 3]) -> Self {
        let first_modulus = configs[0].modulus as i128;
        let second_modulus = configs[1].modulus as i128;
        let third_modulus = configs[2].modulus as i128;
        let first_second_modulus = first_modulus * second_modulus;
        let total_modulus = first_second_modulus * third_modulus;
        let inverse_first_modulus_in_second =
            mod_inverse_with_modulus(configs[0].modulus % configs[1].modulus, configs[1].modulus)
                as i128;
        let inverse_first_second_modulus_in_third = mod_inverse_with_modulus(
            ((configs[0].modulus as u128 * configs[1].modulus as u128) % configs[2].modulus as u128)
                as u64,
            configs[2].modulus,
        ) as i128;

        Self {
            first_modulus,
            second_modulus,
            third_modulus,
            first_second_modulus,
            total_modulus,
            inverse_first_modulus_in_second,
            inverse_first_second_modulus_in_third,
        }
    }

    fn reconstruct_signed_i64(&self, residues: [u64; 3]) -> i64 {
        let first_residue = residues[0] as i128;
        let second_residue = residues[1] as i128;
        let third_residue = residues[2] as i128;

        let second_component = ((second_residue - first_residue).rem_euclid(self.second_modulus)
            * self.inverse_first_modulus_in_second)
            .rem_euclid(self.second_modulus);
        let first_two_value = first_residue + self.first_modulus * second_component;
        let third_component = ((third_residue - (first_two_value % self.third_modulus))
            .rem_euclid(self.third_modulus)
            * self.inverse_first_second_modulus_in_third)
            .rem_euclid(self.third_modulus);
        let combined_value = first_two_value + self.first_second_modulus * third_component;
        let signed_value = if self.total_modulus / 2 < combined_value {
            combined_value - self.total_modulus
        } else {
            combined_value
        };

        clamp_i128_to_i64(signed_value)
    }
}

fn mod_pow_with_modulus(mut base_value: u64, mut exponent: u64, modulus: u64) -> u64 {
    let mut result_value = 1u64;

    while 0 < exponent {
        if exponent & 1 == 1 {
            result_value = result_value * base_value % modulus;
        }
        base_value = base_value * base_value % modulus;
        exponent >>= 1;
    }

    result_value
}

fn mod_inverse_with_modulus(value: u64, modulus: u64) -> u64 {
    mod_pow_with_modulus(value, modulus - 2, modulus)
}

fn ntt_in_place_with_config(values: &mut [u64], invert: bool, config: NttConfig) {
    let value_count = values.len();
    assert!(
        value_count.is_power_of_two(),
        "input length must be power of two"
    );
    assert!(
        value_count <= config.max_two_power_length,
        "input length exceeds NTT limit for modulus"
    );

    bit_reverse_permute(values);

    let mut segment_length = 2usize;
    while segment_length <= value_count {
        let mut primitive_root = mod_pow_with_modulus(
            config.primitive_root,
            (config.modulus - 1) / segment_length as u64,
            config.modulus,
        );
        if invert {
            primitive_root = mod_inverse_with_modulus(primitive_root, config.modulus);
        }

        for segment_start in (0..value_count).step_by(segment_length) {
            let mut current_root = 1u64;
            for offset in 0..segment_length / 2 {
                let left_value = values[segment_start + offset];
                let right_index = segment_start + offset + segment_length / 2;
                let right_value = values[right_index] * current_root % config.modulus;

                values[segment_start + offset] = (left_value + right_value) % config.modulus;
                values[right_index] = (left_value + config.modulus - right_value) % config.modulus;

                current_root = current_root * primitive_root % config.modulus;
            }
        }

        segment_length <<= 1;
    }

    if invert {
        let inverse_count = mod_inverse_with_modulus(value_count as u64, config.modulus);
        for value in values.iter_mut() {
            *value = *value * inverse_count % config.modulus;
        }
    }
}

pub fn convolution_ntt_mod998244353(lhs_values: &[u64], rhs_values: &[u64]) -> Vec<u64> {
    convolution_u64_with_config(lhs_values, rhs_values, MOD998244353_CONFIG)
}

fn convolution_u64_with_config(
    lhs_values: &[u64],
    rhs_values: &[u64],
    config: NttConfig,
) -> Vec<u64> {
    if lhs_values.is_empty() || rhs_values.is_empty() {
        return vec![];
    }

    let result_length = lhs_values.len() + rhs_values.len() - 1;
    let mut ntt_length = 1usize;
    while ntt_length < result_length {
        ntt_length <<= 1;
    }

    assert!(
        ntt_length <= config.max_two_power_length,
        "convolution size exceeds NTT limit for modulus"
    );

    let mut lhs_mod_values = vec![0u64; ntt_length];
    let mut rhs_mod_values = vec![0u64; ntt_length];

    for (index, &value) in lhs_values.iter().enumerate() {
        lhs_mod_values[index] = value % config.modulus;
    }
    for (index, &value) in rhs_values.iter().enumerate() {
        rhs_mod_values[index] = value % config.modulus;
    }

    ntt_in_place_with_config(&mut lhs_mod_values, false, config);
    ntt_in_place_with_config(&mut rhs_mod_values, false, config);

    for index in 0..ntt_length {
        lhs_mod_values[index] = lhs_mod_values[index] * rhs_mod_values[index] % config.modulus;
    }

    ntt_in_place_with_config(&mut lhs_mod_values, true, config);
    lhs_mod_values.truncate(result_length);
    lhs_mod_values
}

fn convolution_i64_with_config(
    lhs_values: &[i64],
    rhs_values: &[i64],
    config: NttConfig,
) -> Vec<u64> {
    if lhs_values.is_empty() || rhs_values.is_empty() {
        return vec![];
    }

    let result_length = lhs_values.len() + rhs_values.len() - 1;
    let mut ntt_length = 1usize;
    while ntt_length < result_length {
        ntt_length <<= 1;
    }

    assert!(
        ntt_length <= config.max_two_power_length,
        "convolution size exceeds NTT limit for modulus"
    );

    let mut lhs_mod_values = vec![0u64; ntt_length];
    let mut rhs_mod_values = vec![0u64; ntt_length];

    for (index, &value) in lhs_values.iter().enumerate() {
        lhs_mod_values[index] = normalize_i64_value(value, config.modulus);
    }
    for (index, &value) in rhs_values.iter().enumerate() {
        rhs_mod_values[index] = normalize_i64_value(value, config.modulus);
    }

    ntt_in_place_with_config(&mut lhs_mod_values, false, config);
    ntt_in_place_with_config(&mut rhs_mod_values, false, config);

    for index in 0..ntt_length {
        lhs_mod_values[index] = lhs_mod_values[index] * rhs_mod_values[index] % config.modulus;
    }

    ntt_in_place_with_config(&mut lhs_mod_values, true, config);
    lhs_mod_values.truncate(result_length);
    lhs_mod_values
}

fn normalize_i64_value(value: i64, modulus: u64) -> u64 {
    (value as i128).rem_euclid(modulus as i128) as u64
}

fn clamp_i128_to_i64(value: i128) -> i64 {
    if value < i64::MIN as i128 {
        i64::MIN
    } else if (i64::MAX as i128) < value {
        i64::MAX
    } else {
        value as i64
    }
}

#[cfg(test)]
mod tests {
    use super::{convolution_fft_i64, convolution_ntt_mod998244353, MODULUS};

    fn naive_convolution_i64(lhs_values: &[i64], rhs_values: &[i64]) -> Vec<i64> {
        if lhs_values.is_empty() || rhs_values.is_empty() {
            return vec![];
        }

        let result_length = lhs_values.len() + rhs_values.len() - 1;
        let mut result_values = vec![0i64; result_length];
        for (lhs_index, &lhs_value) in lhs_values.iter().enumerate() {
            for (rhs_index, &rhs_value) in rhs_values.iter().enumerate() {
                result_values[lhs_index + rhs_index] += lhs_value * rhs_value;
            }
        }

        result_values
    }

    fn naive_convolution_mod(lhs_values: &[u64], rhs_values: &[u64]) -> Vec<u64> {
        if lhs_values.is_empty() || rhs_values.is_empty() {
            return vec![];
        }

        let result_length = lhs_values.len() + rhs_values.len() - 1;
        let mut result_values = vec![0u64; result_length];
        for (lhs_index, &lhs_value) in lhs_values.iter().enumerate() {
            for (rhs_index, &rhs_value) in rhs_values.iter().enumerate() {
                let index = lhs_index + rhs_index;
                let add_value = (lhs_value % MODULUS) * (rhs_value % MODULUS) % MODULUS;
                result_values[index] = (result_values[index] + add_value) % MODULUS;
            }
        }

        result_values
    }

    #[test]
    fn convolution_fft_i64_returns_empty_when_lhs_is_empty() {
        // Arrange
        let lhs_values: Vec<i64> = vec![];
        let rhs_values = vec![1i64, 2, 3];
        let expected_values: Vec<i64> = vec![];

        // Act
        let actual_values = convolution_fft_i64(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_fft_i64_returns_empty_when_rhs_is_empty() {
        // Arrange
        let lhs_values = vec![1i64, 2, 3];
        let rhs_values: Vec<i64> = vec![];
        let expected_values: Vec<i64> = vec![];

        // Act
        let actual_values = convolution_fft_i64(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_fft_i64_matches_naive_for_single_element_input() {
        // Arrange
        let lhs_values = vec![7i64];
        let rhs_values = vec![-3i64];
        let expected_values = naive_convolution_i64(&lhs_values, &rhs_values);

        // Act
        let actual_values = convolution_fft_i64(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_fft_i64_matches_naive_for_length_x_minus_1_x_x_plus_1_cases() {
        // Arrange
        let center_length = 2usize;
        let lhs_x_minus_1 = vec![1i64; center_length - 1];
        let lhs_x = vec![1i64; center_length];
        let lhs_x_plus_1 = vec![1i64; center_length + 1];
        let rhs_values = vec![2i64, -1i64];
        let expected_x_minus_1 = naive_convolution_i64(&lhs_x_minus_1, &rhs_values);
        let expected_x = naive_convolution_i64(&lhs_x, &rhs_values);
        let expected_x_plus_1 = naive_convolution_i64(&lhs_x_plus_1, &rhs_values);

        // Act
        let actual_x_minus_1 = convolution_fft_i64(&lhs_x_minus_1, &rhs_values);
        let actual_x = convolution_fft_i64(&lhs_x, &rhs_values);
        let actual_x_plus_1 = convolution_fft_i64(&lhs_x_plus_1, &rhs_values);

        // Assert
        assert_eq!(actual_x_minus_1, expected_x_minus_1);
        assert_eq!(actual_x, expected_x);
        assert_eq!(actual_x_plus_1, expected_x_plus_1);
    }

    #[test]
    fn convolution_fft_i64_matches_naive_for_mixed_sign_multi_element_case() {
        // Arrange
        let lhs_values = vec![1i64, -3, 5, -7];
        let rhs_values = vec![2i64, 0, -4];
        let expected_values = naive_convolution_i64(&lhs_values, &rhs_values);

        // Act
        let actual_values = convolution_fft_i64(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_fft_i64_matches_exact_value_for_large_single_element_input() {
        // Arrange
        let lhs_values = vec![1_000_000_007i64];
        let rhs_values = vec![1_000_000_009i64];
        let expected_values = naive_convolution_i64(&lhs_values, &rhs_values);

        // Act
        let actual_values = convolution_fft_i64(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_fft_i64_clamps_results_outside_i64_range_and_keeps_boundary_values() {
        // Arrange
        let inputs = [
            (vec![i64::MAX - 1], vec![1]),
            (vec![i64::MAX], vec![1]),
            (vec![i64::MAX, 1], vec![1, 1]),
            (vec![i64::MIN + 1], vec![1]),
            (vec![i64::MIN], vec![1]),
            (vec![i64::MIN, -1], vec![1, 1]),
        ];
        let expected_values: Vec<Vec<i64>> = inputs
            .iter()
            .map(|(lhs, rhs)| {
                let mut sums = vec![0_i128; lhs.len() + rhs.len() - 1];
                for (i, &left) in lhs.iter().enumerate() {
                    for (j, &right) in rhs.iter().enumerate() {
                        sums[i + j] += i128::from(left) * i128::from(right);
                    }
                }
                sums.into_iter()
                    .map(|sum| sum.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64)
                    .collect()
            })
            .collect();

        // Act
        let actual_values: Vec<_> = inputs
            .iter()
            .map(|(lhs, rhs)| convolution_fft_i64(lhs, rhs))
            .collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_fft_i64_matches_naive_for_large_mixed_sign_input() {
        // Arrange
        let lhs_values = vec![1_000_000_007i64, -1_000_000_003];
        let rhs_values = vec![1_000_000_009i64, 2];
        let expected_values = naive_convolution_i64(&lhs_values, &rhs_values);

        // Act
        let actual_values = convolution_fft_i64(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_ntt_mod998244353_returns_empty_when_lhs_is_empty() {
        // Arrange
        let lhs_values: Vec<u64> = vec![];
        let rhs_values = vec![1u64, 2, 3];
        let expected_values: Vec<u64> = vec![];

        // Act
        let actual_values = convolution_ntt_mod998244353(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_ntt_mod998244353_returns_empty_when_rhs_is_empty() {
        // Arrange
        let lhs_values = vec![1u64, 2, 3];
        let rhs_values: Vec<u64> = vec![];
        let expected_values: Vec<u64> = vec![];

        // Act
        let actual_values = convolution_ntt_mod998244353(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_ntt_mod998244353_matches_naive_for_single_element_input() {
        // Arrange
        let lhs_values = vec![7u64];
        let rhs_values = vec![MODULUS - 3];
        let expected_values = naive_convolution_mod(&lhs_values, &rhs_values);

        // Act
        let actual_values = convolution_ntt_mod998244353(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_ntt_mod998244353_matches_naive_for_length_x_minus_1_x_x_plus_1_cases() {
        // Arrange
        let center_length = 2usize;
        let lhs_x_minus_1 = vec![1u64; center_length - 1];
        let lhs_x = vec![1u64; center_length];
        let lhs_x_plus_1 = vec![1u64; center_length + 1];
        let rhs_values = vec![2u64, MODULUS - 1];
        let expected_x_minus_1 = naive_convolution_mod(&lhs_x_minus_1, &rhs_values);
        let expected_x = naive_convolution_mod(&lhs_x, &rhs_values);
        let expected_x_plus_1 = naive_convolution_mod(&lhs_x_plus_1, &rhs_values);

        // Act
        let actual_x_minus_1 = convolution_ntt_mod998244353(&lhs_x_minus_1, &rhs_values);
        let actual_x = convolution_ntt_mod998244353(&lhs_x, &rhs_values);
        let actual_x_plus_1 = convolution_ntt_mod998244353(&lhs_x_plus_1, &rhs_values);

        // Assert
        assert_eq!(actual_x_minus_1, expected_x_minus_1);
        assert_eq!(actual_x, expected_x);
        assert_eq!(actual_x_plus_1, expected_x_plus_1);
    }

    #[test]
    fn convolution_ntt_mod998244353_reduces_input_values_modulo_before_convolution() {
        // Arrange
        let lhs_values = vec![MODULUS + 1, MODULUS * 2 + 3];
        let rhs_values = vec![MODULUS + 4, MODULUS * 3 + 5];
        let expected_values = naive_convolution_mod(&lhs_values, &rhs_values);

        // Act
        let actual_values = convolution_ntt_mod998244353(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_fft_i64_with_left_unit_returns_rhs_values() {
        // Arrange
        let lhs_values = vec![1i64];
        let rhs_values = vec![3i64, -2, 5];
        let expected_values = rhs_values.clone();

        // Act
        let actual_values = convolution_fft_i64(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_fft_i64_matches_naive_for_sparse_zero_heavy_input() {
        // Arrange
        let lhs_values = vec![0i64, 4, 0, -2];
        let rhs_values = vec![0i64, 0, 3];
        let expected_values = naive_convolution_i64(&lhs_values, &rhs_values);

        // Act
        let actual_values = convolution_fft_i64(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_fft_i64_truncates_results_around_power_of_two_boundary() {
        // Arrange
        let lhs_exact_boundary = vec![1i64, 2, 3];
        let rhs_exact_boundary = vec![4i64, 5];
        let lhs_over_boundary = vec![1i64, 2, 3, 4];
        let rhs_over_boundary = vec![5i64, 6];
        let expected_exact_length = lhs_exact_boundary.len() + rhs_exact_boundary.len() - 1;
        let expected_over_length = lhs_over_boundary.len() + rhs_over_boundary.len() - 1;

        // Act
        let actual_exact_values = convolution_fft_i64(&lhs_exact_boundary, &rhs_exact_boundary);
        let actual_over_values = convolution_fft_i64(&lhs_over_boundary, &rhs_over_boundary);

        // Assert
        assert_eq!(actual_exact_values.len(), expected_exact_length);
        assert_eq!(actual_over_values.len(), expected_over_length);
    }

    #[test]
    fn convolution_fft_i64_restores_negative_values_after_crt() {
        // Arrange
        let lhs_values = vec![2i64, -3];
        let rhs_values = vec![4i64, 5];
        let expected_values = naive_convolution_i64(&lhs_values, &rhs_values);

        // Act
        let actual_values = convolution_fft_i64(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_ntt_mod998244353_with_left_unit_returns_normalized_rhs_values() {
        // Arrange
        let lhs_values = vec![1u64];
        let rhs_values = vec![MODULUS + 1, 2, MODULUS * 2 + 3];
        let expected_values = vec![1u64, 2, 3];

        // Act
        let actual_values = convolution_ntt_mod998244353(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_ntt_mod998244353_matches_naive_for_sparse_zero_heavy_input() {
        // Arrange
        let lhs_values = vec![0u64, 4, 0, MODULUS - 2];
        let rhs_values = vec![0u64, 0, 3];
        let expected_values = naive_convolution_mod(&lhs_values, &rhs_values);

        // Act
        let actual_values = convolution_ntt_mod998244353(&lhs_values, &rhs_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn convolution_ntt_mod998244353_truncates_results_around_power_of_two_boundary() {
        // Arrange
        let lhs_exact_boundary = vec![1u64, 2, 3];
        let rhs_exact_boundary = vec![4u64, 5];
        let lhs_over_boundary = vec![1u64, 2, 3, 4];
        let rhs_over_boundary = vec![5u64, 6];
        let expected_exact_length = lhs_exact_boundary.len() + rhs_exact_boundary.len() - 1;
        let expected_over_length = lhs_over_boundary.len() + rhs_over_boundary.len() - 1;

        // Act
        let actual_exact_values =
            convolution_ntt_mod998244353(&lhs_exact_boundary, &rhs_exact_boundary);
        let actual_over_values =
            convolution_ntt_mod998244353(&lhs_over_boundary, &rhs_over_boundary);

        // Assert
        assert_eq!(actual_exact_values.len(), expected_exact_length);
        assert_eq!(actual_over_values.len(), expected_over_length);
    }
}

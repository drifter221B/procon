/// 文字列で表現された符号付き整数同士の加減乗を実装する。
#[derive(Debug, Clone)]
pub struct StringMath {
    lhs: Vec<u8>,
    rhs: Vec<u8>,
    lhs_neg: bool,
    rhs_neg: bool,
}

impl StringMath {
    pub fn new(lhs: &str, rhs: &str) -> Self {
        let (lhs_neg, lhs_digits) = split_sign(lhs);
        let (rhs_neg, rhs_digits) = split_sign(rhs);
        Self {
            lhs: lhs_digits,
            rhs: rhs_digits,
            lhs_neg,
            rhs_neg,
        }
    }

    pub fn add(&self) -> String {
        if self.lhs_neg == self.rhs_neg {
            let mut digits = add_abs(&self.lhs, &self.rhs);
            let mut result = to_string(&digits, self.lhs_neg);
            trim_leading_zeros(&mut result);
            result
        } else {
            match cmp_abs(&self.lhs, &self.rhs) {
                std::cmp::Ordering::Greater => {
                    let mut digits = sub_abs(&self.lhs, &self.rhs);
                    let mut result = to_string(&digits, self.lhs_neg);
                    trim_leading_zeros(&mut result);
                    result
                }
                std::cmp::Ordering::Less => {
                    let mut digits = sub_abs(&self.rhs, &self.lhs);
                    let mut result = to_string(&digits, self.rhs_neg);
                    trim_leading_zeros(&mut result);
                    result
                }
                std::cmp::Ordering::Equal => "0".to_string(),
            }
        }
    }

    pub fn subtract(&self) -> String {
        // lhs - rhs
        let rhs_negated = !self.rhs_neg;
        if self.lhs_neg == rhs_negated {
            let mut digits = add_abs(&self.lhs, &self.rhs);
            let mut result = to_string(&digits, self.lhs_neg);
            trim_leading_zeros(&mut result);
            result
        } else {
            match cmp_abs(&self.lhs, &self.rhs) {
                std::cmp::Ordering::Greater => {
                    let mut digits = sub_abs(&self.lhs, &self.rhs);
                    let mut result = to_string(&digits, self.lhs_neg);
                    trim_leading_zeros(&mut result);
                    result
                }
                std::cmp::Ordering::Less => {
                    let mut digits = sub_abs(&self.rhs, &self.lhs);
                    let mut result = to_string(&digits, !self.lhs_neg);
                    trim_leading_zeros(&mut result);
                    result
                }
                std::cmp::Ordering::Equal => "0".to_string(),
            }
        }
    }

    pub fn multiply(&self) -> String {
        if self.lhs == [0] || self.rhs == [0] {
            return "0".to_string();
        }
        let mut digits = mul_abs(&self.lhs, &self.rhs);
        let negative = self.lhs_neg ^ self.rhs_neg;
        let mut result = to_string(&digits, negative);
        trim_leading_zeros(&mut result);
        result
    }
}

fn split_sign(s: &str) -> (bool, Vec<u8>) {
    let bytes = s.as_bytes();
    let (neg, digits) = if bytes.first() == Some(&b'-') {
        (true, &bytes[1..])
    } else {
        (false, bytes)
    };
    let mut vec = digits.iter().map(|b| b - b'0').collect::<Vec<_>>();
    if vec.is_empty() {
        vec.push(0);
    }
    trim_leading_zeros_bytes(&mut vec);
    (neg, vec)
}

fn to_string(digits: &[u8], negative: bool) -> String {
    let mut s = String::with_capacity(digits.len() + 1);
    if negative {
        s.push('-');
    }
    for &d in digits {
        s.push(char::from(b'0' + d));
    }
    s
}

fn trim_leading_zeros(s: &mut String) {
    if s == "0" || s == "-0" {
        *s = "0".to_string();
        return;
    }
    if s.starts_with('-') {
        while s.as_bytes().get(1) == Some(&b'0') && s.len() > 2 {
            s.remove(1);
        }
    } else {
        while s.as_bytes().first() == Some(&b'0') && s.len() > 1 {
            s.remove(0);
        }
    }
}

fn trim_leading_zeros_bytes(digits: &mut Vec<u8>) {
    while digits.len() > 1 && digits[0] == 0 {
        digits.remove(0);
    }
}

fn cmp_abs(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    if a.len() != b.len() {
        return a.len().cmp(&b.len());
    }
    a.cmp(b)
}

fn add_abs(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut carry = 0;
    let mut res = Vec::new();
    let mut ia = a.iter().rev();
    let mut ib = b.iter().rev();
    loop {
        match (ia.next(), ib.next()) {
            (Some(&da), Some(&db)) => {
                let sum = da as u16 + db as u16 + carry;
                res.push((sum % 10) as u8);
                carry = sum / 10;
            }
            (Some(&da), None) => {
                let sum = da as u16 + carry;
                res.push((sum % 10) as u8);
                carry = sum / 10;
            }
            (None, Some(&db)) => {
                let sum = db as u16 + carry;
                res.push((sum % 10) as u8);
                carry = sum / 10;
            }
            (None, None) => break,
        }
    }
    if carry > 0 {
        res.push(carry as u8);
    }
    res.reverse();
    res
}

fn sub_abs(a: &[u8], b: &[u8]) -> Vec<u8> {
    // assume a >= b
    let mut res = Vec::new();
    let mut borrow = 0i8;
    let mut ia = a.iter().rev();
    let mut ib = b.iter().rev();
    loop {
        match (ia.next(), ib.next()) {
            (Some(&da), Some(&db)) => {
                let mut diff = da as i8 - db as i8 - borrow;
                if diff < 0 {
                    diff += 10;
                    borrow = 1;
                } else {
                    borrow = 0;
                }
                res.push(diff as u8);
            }
            (Some(&da), None) => {
                let mut diff = da as i8 - borrow;
                if diff < 0 {
                    diff += 10;
                    borrow = 1;
                } else {
                    borrow = 0;
                }
                res.push(diff as u8);
            }
            (None, None) => break,
            (None, Some(_)) => unreachable!("a must be >= b"),
        }
    }
    while res.len() > 1 && res.last() == Some(&0) {
        res.pop();
    }
    res.reverse();
    res
}

fn mul_abs(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut res = vec![0u8; a.len() + b.len()];
    for (i, &da) in a.iter().rev().enumerate() {
        let mut carry = 0u16;
        let ai = a.len() - 1 - i;
        for (j, &db) in b.iter().rev().enumerate() {
            let bj = b.len() - 1 - j;
            let idx = res.len() - 1 - (i + j);
            let prod = da as u16 * db as u16 + res[idx] as u16 + carry;
            res[idx] = (prod % 10) as u8;
            carry = prod / 10;
        }
        let idx = res.len() - 1 - (i + b.len());
        res[idx] = (res[idx] as u16 + carry) as u8;
    }
    trim_leading_zeros_bytes(&mut res);
    res
}

#[cfg(test)]
mod tests {
    use super::StringMath;

    #[test]
    fn new_with_both_empty_strings_add_returns_zero() {
        // Arrange
        let lhs_value = "";
        let rhs_value = "";
        let expected_sum = "0".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_sum = string_math.add();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn new_with_only_minus_sign_inputs_add_returns_zero_without_negative_zero() {
        // Arrange
        let lhs_value = "-";
        let rhs_value = "-";
        let expected_sum = "0".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_sum = string_math.add();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn new_with_positive_leading_zero_inputs_add_returns_trimmed_sum() {
        // Arrange
        let lhs_value = "00012";
        let rhs_value = "0003";
        let expected_sum = "15".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_sum = string_math.add();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn new_with_negative_leading_zero_inputs_add_returns_trimmed_negative_sum() {
        // Arrange
        let lhs_value = "-00012";
        let rhs_value = "-0003";
        let expected_sum = "-15".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_sum = string_math.add();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn new_with_left_empty_string_add_returns_right_operand_value() {
        // Arrange
        let lhs_value = "";
        let rhs_value = "123";
        let expected_sum = "123".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_sum = string_math.add();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn new_with_plus_prefixed_input_panics_in_current_behavior() {
        // Arrange
        let lhs_value = "+1";
        let rhs_value = "2";

        // Act
        let add_result = std::panic::catch_unwind(|| {
            let string_math = StringMath::new(lhs_value, rhs_value);
            string_math.add()
        });

        // Assert
        assert!(add_result.is_err());
    }

    #[test]
    fn add_with_alphabetic_operand_a1_and_two_returns_numeric_string_493() {
        // Arrange
        let lhs_value = "a1";
        let rhs_value = "2";
        let expected_sum = "493".to_string();

        // Act
        let add_result = std::panic::catch_unwind(|| {
            let string_math = StringMath::new(lhs_value, rhs_value);
            string_math.add()
        });

        // Assert
        assert!(add_result.is_ok());
        let actual_sum = add_result.unwrap();
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_with_positive_operands_and_no_carry_returns_sum() {
        // Arrange
        let lhs_value = "12";
        let rhs_value = "34";
        let expected_sum = "46".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_sum = string_math.add();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_with_positive_operands_and_single_digit_carry_returns_sum() {
        // Arrange
        let lhs_value = "19";
        let rhs_value = "2";
        let expected_sum = "21".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_sum = string_math.add();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_with_positive_operands_and_cascade_carry_returns_sum() {
        // Arrange
        let lhs_value = "999";
        let rhs_value = "1";
        let expected_sum = "1000".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_sum = string_math.add();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_with_both_negative_operands_returns_negative_sum() {
        // Arrange
        let lhs_value = "-12";
        let rhs_value = "-34";
        let expected_sum = "-46".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_sum = string_math.add();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_with_different_signs_and_larger_left_absolute_returns_positive_difference() {
        // Arrange
        let lhs_value = "123";
        let rhs_value = "-23";
        let expected_sum = "100".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_sum = string_math.add();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_with_different_signs_and_larger_right_absolute_returns_negative_difference() {
        // Arrange
        let lhs_value = "23";
        let rhs_value = "-123";
        let expected_sum = "-100".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_sum = string_math.add();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_with_different_signs_and_equal_absolute_returns_zero() {
        // Arrange
        let lhs_value = "500";
        let rhs_value = "-500";
        let expected_sum = "0".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_sum = string_math.add();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_boundary_values_x_minus_1_x_x_plus_1_with_x_equals_1000_return_expected_results() {
        // Arrange
        let first_lhs_value = "999";
        let first_rhs_value = "1";
        let second_lhs_value = "1000";
        let second_rhs_value = "1";
        let third_lhs_value = "1001";
        let third_rhs_value = "1";
        let expected_first_sum = "1000".to_string();
        let expected_second_sum = "1001".to_string();
        let expected_third_sum = "1002".to_string();

        // Act
        let first_actual_sum = StringMath::new(first_lhs_value, first_rhs_value).add();
        let second_actual_sum = StringMath::new(second_lhs_value, second_rhs_value).add();
        let third_actual_sum = StringMath::new(third_lhs_value, third_rhs_value).add();

        // Assert
        assert_eq!(first_actual_sum, expected_first_sum);
        assert_eq!(second_actual_sum, expected_second_sum);
        assert_eq!(third_actual_sum, expected_third_sum);
    }

    #[test]
    fn add_with_two_100_digit_operands_returns_expected_large_sum() {
        // Arrange
        let lhs_value = "9".repeat(100);
        let rhs_value = "1".repeat(100);
        let expected_sum = format!("{}0", "1".repeat(100));

        // Act
        let string_math = StringMath::new(lhs_value.as_str(), rhs_value.as_str());
        let actual_sum = string_math.add();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn subtract_with_positive_operands_and_no_borrow_returns_difference() {
        // Arrange
        let lhs_value = "34";
        let rhs_value = "12";
        let expected_difference = "22".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_difference = string_math.subtract();

        // Assert
        assert_eq!(actual_difference, expected_difference);
    }

    #[test]
    fn subtract_with_positive_operands_and_cascade_borrow_returns_difference() {
        // Arrange
        let lhs_value = "1000";
        let rhs_value = "1";
        let expected_difference = "999".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_difference = string_math.subtract();

        // Assert
        assert_eq!(actual_difference, expected_difference);
    }

    #[test]
    fn subtract_with_positive_operands_and_larger_right_returns_negative_difference() {
        // Arrange
        let lhs_value = "12";
        let rhs_value = "34";
        let expected_difference = "-22".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_difference = string_math.subtract();

        // Assert
        assert_eq!(actual_difference, expected_difference);
    }

    #[test]
    fn subtract_with_equal_positive_operands_returns_zero() {
        // Arrange
        let lhs_value = "777";
        let rhs_value = "777";
        let expected_difference = "0".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_difference = string_math.subtract();

        // Assert
        assert_eq!(actual_difference, expected_difference);
    }

    #[test]
    fn subtract_with_negative_left_and_positive_right_returns_negative_sum_of_absolute_values() {
        // Arrange
        let lhs_value = "-12";
        let rhs_value = "34";
        let expected_difference = "-46".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_difference = string_math.subtract();

        // Assert
        assert_eq!(actual_difference, expected_difference);
    }

    #[test]
    fn subtract_with_positive_left_and_negative_right_returns_positive_sum_of_absolute_values() {
        // Arrange
        let lhs_value = "12";
        let rhs_value = "-34";
        let expected_difference = "46".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_difference = string_math.subtract();

        // Assert
        assert_eq!(actual_difference, expected_difference);
    }

    #[test]
    fn subtract_with_both_negative_and_larger_left_absolute_returns_negative_difference() {
        // Arrange
        let lhs_value = "-34";
        let rhs_value = "-12";
        let expected_difference = "-22".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_difference = string_math.subtract();

        // Assert
        assert_eq!(actual_difference, expected_difference);
    }

    #[test]
    fn subtract_with_both_negative_and_larger_right_absolute_returns_positive_difference() {
        // Arrange
        let lhs_value = "-12";
        let rhs_value = "-34";
        let expected_difference = "22".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_difference = string_math.subtract();

        // Assert
        assert_eq!(actual_difference, expected_difference);
    }

    #[test]
    fn subtract_boundary_values_x_minus_1_x_x_plus_1_with_x_equals_1000_return_expected_results() {
        // Arrange
        let rhs_value = "1000";
        let first_lhs_value = "999";
        let second_lhs_value = "1000";
        let third_lhs_value = "1001";
        let expected_first_difference = "-1".to_string();
        let expected_second_difference = "0".to_string();
        let expected_third_difference = "1".to_string();

        // Act
        let first_actual_difference = StringMath::new(first_lhs_value, rhs_value).subtract();
        let second_actual_difference = StringMath::new(second_lhs_value, rhs_value).subtract();
        let third_actual_difference = StringMath::new(third_lhs_value, rhs_value).subtract();

        // Assert
        assert_eq!(first_actual_difference, expected_first_difference);
        assert_eq!(second_actual_difference, expected_second_difference);
        assert_eq!(third_actual_difference, expected_third_difference);
    }

    #[test]
    fn subtract_with_negative_zero_and_zero_returns_zero_without_negative_sign() {
        // Arrange
        let lhs_value = "-0";
        let rhs_value = "0";
        let expected_difference = "0".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_difference = string_math.subtract();

        // Assert
        assert_eq!(actual_difference, expected_difference);
    }

    #[test]
    fn subtract_with_two_100_digit_operands_returns_expected_large_difference() {
        // Arrange
        let lhs_value = "9".repeat(100);
        let rhs_value = "1".repeat(100);
        let expected_difference = "8".repeat(100);

        // Act
        let string_math = StringMath::new(lhs_value.as_str(), rhs_value.as_str());
        let actual_difference = string_math.subtract();

        // Assert
        assert_eq!(actual_difference, expected_difference);
    }

    #[test]
    fn multiply_with_zero_left_operand_returns_zero() {
        // Arrange
        let lhs_value = "0";
        let rhs_value = "123";
        let expected_product = "0".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_product = string_math.multiply();

        // Assert
        assert_eq!(actual_product, expected_product);
    }

    #[test]
    fn multiply_with_zero_and_negative_operand_returns_zero_without_negative_sign() {
        // Arrange
        let lhs_value = "0";
        let rhs_value = "-123";
        let expected_product = "0".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_product = string_math.multiply();

        // Assert
        assert_eq!(actual_product, expected_product);
    }

    #[test]
    fn multiply_with_one_as_left_operand_returns_right_operand() {
        // Arrange
        let lhs_value = "1";
        let rhs_value = "-234";
        let expected_product = "-234".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_product = string_math.multiply();

        // Assert
        assert_eq!(actual_product, expected_product);
    }

    #[test]
    fn multiply_with_positive_operands_and_no_carry_returns_product() {
        // Arrange
        let lhs_value = "12";
        let rhs_value = "3";
        let expected_product = "36".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_product = string_math.multiply();

        // Assert
        assert_eq!(actual_product, expected_product);
    }

    #[test]
    fn multiply_with_positive_operands_and_cascade_carry_returns_product() {
        // Arrange
        let lhs_value = "99";
        let rhs_value = "99";
        let expected_product = "9801".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_product = string_math.multiply();

        // Assert
        assert_eq!(actual_product, expected_product);
    }

    #[test]
    fn multiply_with_negative_and_positive_operands_returns_negative_product() {
        // Arrange
        let lhs_value = "-12";
        let rhs_value = "3";
        let expected_product = "-36".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_product = string_math.multiply();

        // Assert
        assert_eq!(actual_product, expected_product);
    }

    #[test]
    fn multiply_with_both_negative_operands_returns_positive_product() {
        // Arrange
        let lhs_value = "-12";
        let rhs_value = "-3";
        let expected_product = "36".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_product = string_math.multiply();

        // Assert
        assert_eq!(actual_product, expected_product);
    }

    #[test]
    fn multiply_with_leading_zero_inputs_returns_trimmed_product() {
        // Arrange
        let lhs_value = "00012";
        let rhs_value = "00003";
        let expected_product = "36".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_product = string_math.multiply();

        // Assert
        assert_eq!(actual_product, expected_product);
    }

    #[test]
    fn multiply_boundary_values_x_minus_1_x_x_plus_1_with_x_equals_10_return_expected_results() {
        // Arrange
        let rhs_value = "10";
        let first_lhs_value = "9";
        let second_lhs_value = "10";
        let third_lhs_value = "11";
        let expected_first_product = "90".to_string();
        let expected_second_product = "100".to_string();
        let expected_third_product = "110".to_string();

        // Act
        let first_actual_product = StringMath::new(first_lhs_value, rhs_value).multiply();
        let second_actual_product = StringMath::new(second_lhs_value, rhs_value).multiply();
        let third_actual_product = StringMath::new(third_lhs_value, rhs_value).multiply();

        // Assert
        assert_eq!(first_actual_product, expected_first_product);
        assert_eq!(second_actual_product, expected_second_product);
        assert_eq!(third_actual_product, expected_third_product);
    }

    #[test]
    fn multiply_with_top_digit_carry_case_returns_expected_product() {
        // Arrange
        let lhs_value = "99999";
        let rhs_value = "99999";
        let expected_product = "9999800001".to_string();

        // Act
        let string_math = StringMath::new(lhs_value, rhs_value);
        let actual_product = string_math.multiply();

        // Assert
        assert_eq!(actual_product, expected_product);
    }

    #[test]
    fn multiply_with_two_100_digit_operands_returns_expected_large_product() {
        // Arrange
        let lhs_value = format!("1{}", "0".repeat(99));
        let rhs_value = format!("1{}", "0".repeat(99));
        let expected_product = format!("1{}", "0".repeat(198));

        // Act
        let string_math = StringMath::new(lhs_value.as_str(), rhs_value.as_str());
        let actual_product = string_math.multiply();

        // Assert
        assert_eq!(actual_product, expected_product);
    }

    #[test]
    fn multiply_is_commutative_for_positive_and_negative_operands() {
        // Arrange
        let lhs_value = "12345";
        let rhs_value = "-678";

        // Act
        let product_lhs_rhs = StringMath::new(lhs_value, rhs_value).multiply();
        let product_rhs_lhs = StringMath::new(rhs_value, lhs_value).multiply();

        // Assert
        assert_eq!(product_lhs_rhs, product_rhs_lhs);
    }

    #[test]
    fn identity_add_then_subtract_returns_original_left_operand() {
        // Arrange
        let lhs_value = "-12345";
        let rhs_value = "6789";
        let expected_value = lhs_value.to_string();

        // Act
        let sum_value = StringMath::new(lhs_value, rhs_value).add();
        let restored_value = StringMath::new(sum_value.as_str(), rhs_value).subtract();

        // Assert
        assert_eq!(restored_value, expected_value);
    }

    #[test]
    fn identity_subtract_then_add_returns_original_left_operand() {
        // Arrange
        let lhs_value = "12345";
        let rhs_value = "-6789";
        let expected_value = lhs_value.to_string();

        // Act
        let difference_value = StringMath::new(lhs_value, rhs_value).subtract();
        let restored_value = StringMath::new(difference_value.as_str(), rhs_value).add();

        // Assert
        assert_eq!(restored_value, expected_value);
    }
}

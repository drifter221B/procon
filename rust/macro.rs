#[allow(dead_code)]
fn read_input() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap_or_else(|_| panic!("Failed to read line"));

    input
}

#[allow(dead_code)]
fn parse_input<T: std::str::FromStr>() -> T {
    parse_input_from_line(&read_input())
}

fn parse_input_from_line<T: std::str::FromStr>(input: &str) -> T {
    input.trim().parse().ok().expect("Failed to parse")
}

#[allow(dead_code)]
fn parse_inputs<T: std::str::FromStr>() -> Vec<T> {
    parse_inputs_from_line(&read_input())
}

fn parse_inputs_from_line<T: std::str::FromStr>(input: &str) -> Vec<T> {
    input.split_whitespace()
        .map(|token| token.parse().ok().expect("Failed to parse"))
        .collect()
}

#[macro_export]
macro_rules! __read_item_from_iter {

    ($it:ident, chars) => {{
        $it.next().expect("not enough tokens").chars().collect::<Vec<char>>()
    }};

    ($it:ident, bytes) => {{
        $it.next().expect("not enough tokens").as_bytes().to_vec()
    }};

    ($it:ident, byte) => {{
        let c = $it.next().expect("not enough tokens").parse::<char>().unwrap();

        u8::try_from(c).unwrap()
    }};

    ($it:ident, usize1) => {{
        let x: usize = $it.next().expect("not enough tokens").parse().unwrap();

        x - 1
    }};

    ($it:ident, $t:ty) => {{
        $it.next().expect("not enough tokens").parse::<$t>().unwrap()
    }};

}

#[macro_export]
macro_rules! read_value {

    ($sc:expr, [bytes; $len:expr]) => {{
        let len = $len;

        (0..len).map(|_| parse_input::<String>().into_bytes())
            .collect::<Vec<Vec<u8>>>()
    }};

    ($sc:expr, [chars; $len:expr]) => {{
        let len = $len;

        (0..len).map(|_| parse_input::<String>().chars().collect::<Vec<char>>())
            .collect::<Vec<Vec<char>>>()
    }};

    ($sc:expr, [usize1; $len:expr]) => {{
        let len = $len;

        (0..len).map(|_| parse_inputs::<usize>().into_iter().map(|x| x - 1).collect::<Vec<_>>())
            .collect::<Vec<Vec<usize>>>()
    }};

    ($sc:expr, [ ( $($t:tt),+ ) ; $len:expr ]) => {{
        let len = $len;

        (0..len).map(|_| read_value!(_, $( $t ),+ ; tuple_line)).collect::<Vec<_>>()
    }};

    ($sc:expr, [[ $t:ty ]; $len:expr ]) => {{
        let len = $len;

        (0..len).map(|_| parse_inputs::<$t>()).collect::<Vec<_>>()
    }};

    ($sc:expr, [ $t:tt ; $len:expr ]) => {{
        let len = $len;

        (0..len).map(|_| parse_input::<$t>()).collect::<Vec<_>>()
    }};

    (_,$( $t:tt ),* ; tuple_line) => {{
        let mut __it = parse_inputs::<String>().into_iter();

        ( $( __read_item_from_iter!(__it, $t) ),* )
    }};

    ($sc:expr, [chars]) => {{
        parse_inputs::<String>().into_iter()
            .map(|s| s.chars().collect())
            .collect::<Vec<Vec<_>>>()
    }};

    ($sc:expr, [bytes]) => {{
        parse_inputs::<String>().into_iter()
            .map(|s| s.into_bytes())
            .collect::<Vec<Vec<_>>>()
    }};

    ($sc:expr, [ usize1 ]) => {{
        parse_inputs::<usize>().into_iter().map(|x| x - 1).collect::<Vec<_>>()
    }};

    ($sc:expr, [ $t:ty ]) => {{
        parse_inputs::<$t>()
    }};

    ($sc:expr, chars) => {{
        parse_input::<String>().chars().collect::<Vec<char>>()
    }};

    ($sc:expr, byte) => {{
        let character = parse_input::<char>();
        u8::try_from(character).unwrap()
    }};

    ($sc:expr, bytes) => {{
        parse_input::<String>().into_bytes()
    }};

    ($sc:expr, usize1) => {{
        let x: usize = parse_input::<usize>();

        x - 1
    }};

    ($sc:expr, $t:ty) => {{
        parse_input::<$t>()
    }};

}

#[allow(unused_macros)]
#[macro_export]
macro_rules! input_inner {

    (@zip $it:ident, () ; () ) => {};

    (@zip $it:ident, (mut $v:ident, $($rest_vars:tt)*) ; ($t:tt, $($rest_ts:tt)*)) => {
        let mut $v = __read_item_from_iter!($it, $t);
        input_inner!(@zip $it, ($($rest_vars)*); ($($rest_ts)*));
    };

    (@zip $it:ident, ($v:ident, $($rest_vars:tt)*) ; ($t:tt, $($rest_ts:tt)*)) => {
        let $v = __read_item_from_iter!($it, $t);
        input_inner!(@zip $it, ($($rest_vars)*); ($($rest_ts)*));
    };

    (@zip $it:ident, (mut $v:ident) ; ($t:tt)) => {
        let mut $v = __read_item_from_iter!($it, $t);
    };

    (@zip $it:ident, ($v:ident) ; ($t:tt)) => {
        let $v = __read_item_from_iter!($it, $t);
    };

    (@bind_same $it:ident, () : $t:tt) => {};

    (@bind_same $it:ident, (mut $v:ident, $($rest_vars:tt)*) : $t:tt) => {
        let mut $v = __read_item_from_iter!($it, $t);
        input_inner!(@bind_same $it, ($($rest_vars)*): $t);
    };

    (@bind_same $it:ident, ($v:ident, $($rest_vars:tt)*) : $t:tt) => {
        let $v = __read_item_from_iter!($it, $t);
        input_inner!(@bind_same $it, ($($rest_vars)*): $t);
    };

    (@bind_same $it:ident, (mut $v:ident) : $t:tt) => {
        let mut $v = __read_item_from_iter!($it, $t);
    };

    (@bind_same $it:ident, ($v:ident) : $t:tt) => {
        let $v = __read_item_from_iter!($it, $t);
    };

    (@from $d:expr, ) => {};

    (@from $d:expr, ( $($vars:tt)+ ) : ( $($types:tt)+ ), $($rest:tt)*) => {
        let mut __it = parse_inputs::<String>().into_iter();
        input_inner!(@zip __it, ( $($vars)+ ); ( $($types)+ ));
        input_inner!(@from $d, $($rest)*);
    };

    (@from $d:expr, ( $($vars:tt)+ ) : ( $($types:tt)+ )) => {
        let mut __it = parse_inputs::<String>().into_iter();
        input_inner!(@zip __it, ( $($vars)+ ); ( $($types)+ ));
    };

    (@from $d:expr, ( $($vars:tt)+ ) : $t:tt, $($rest:tt)*) => {
        let mut __it = parse_inputs::<String>().into_iter();
        input_inner!(@bind_same __it, ( $($vars)+ ) : $t);
        input_inner!(@from $d, $($rest)*);
    };

    (@from $d:expr, ( $($vars:tt)+ ) : $t:tt) => {
        let mut __it = parse_inputs::<String>().into_iter();
        input_inner!(@bind_same __it, ( $($vars)+ ) : $t);
    };

    (@from $d:expr, mut $var:ident : ( $($t:tt),+ ), $($rest:tt)*) => {
        let mut $var = read_value!(_, $( $t ),+ ; tuple_line);
        input_inner!(@from $d, $($rest)*);
    };

    (@from $d:expr, mut $var:ident : ( $($t:tt),+ )) => {
        let mut $var = read_value!(_, $( $t ),+ ; tuple_line);
    };

    (@from $d:expr, $var:ident : ( $($t:tt),+ ), $($rest:tt)*) => {
        let $var = read_value!(_, $( $t ),+ ; tuple_line);
        input_inner!(@from $d, $($rest)*);
    };

    (@from $d:expr, $var:ident : ( $($t:tt),+ )) => {
        let $var = read_value!(_, $( $t ),+ ; tuple_line);
    };

    (@from $d:expr, mut $var:ident : $t:tt, $($rest:tt)*) => {
        let mut $var = read_value!($d, $t);
        input_inner!(@from $d, $($rest)*);
    };

    (@from $d:expr, mut $var:ident : $t:tt) => { let mut $var = read_value!($d, $t); };

    (@from $d:expr, $var:ident : $t:tt, $($rest:tt)*) => {
        let $var = read_value!($d, $t);
        input_inner!(@from $d, $($rest)*);
    };

    (@from $d:expr, $var:ident : $t:tt) => { let $var = read_value!($d, $t); };

}

/// A macro to get stdin and declare a variable / variables.
///
/// ```
/// input! {
///     n: usize,
///     mut v: [i32; n],
///     (p, mut q): (i32, usize),
///     mut s: [bytes],
///     s: chars,
/// }
///
/// ```
#[allow(unused_macros)]
#[macro_export]
macro_rules! input {
    ($($r:tt)*) => { input_inner!(@from (), $($r)*); };
}

/// A macro to join the string representations of elements from an iterator using a separator.
/// Each element is converted to a [`String`] using [`ToString::to_string`].
///
/// ```
/// let s = join!([1, 2, 3], ", ");
///
/// assert_eq!(s, "1, 2, 3");
/// ```
#[allow(unused_macros)]
#[macro_export]
macro_rules! join {
    ($iter:expr, $separator:expr $(,)?) => {
        ($iter).into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join($separator)
    };
}

/// A macro to create a `BTreeSet` from a list of elements.
///
/// ```
/// let s = tree_set![1, 2, 2];
///
/// assert_eq!(s, [1, 2, 2].into_iter().collect::<std::collections::BTreeSet<_>>());
/// ```
#[allow(unused_macros)]
#[macro_export]
macro_rules! tree_set {

    ($($key:expr),* $(,)?) => {{

        let mut set = std::collections::BTreeSet::new();
        $(
            set.insert($key);
        )*

        set
    }};

}

/// A macro to create a `HashSet` from a list of elements.
///
/// ```
/// let s = hash_set![1, 2, 2];
///
/// assert_eq!(s, [1, 2, 2].into_iter().collect::<std::collections::HashSet<_>>());
/// ```
#[allow(unused_macros)]
#[macro_export]
macro_rules! hash_set {

    ($($key:expr),* $(,)?) => {{
        let mut set = std::collections::HashSet::new();
        $(
            set.insert($key);
        )*

        set
    }};

}

/// A macro to update a mutable variable with a new value if the new value is **less than** the current value,
/// returning bool whether the current value is updated or not
///
/// ```
/// let mut a = 4;
/// let b = 2;
///
/// assert!(chmin!(a, b));
/// assert_eq!(a, b);
/// ```
#[allow(unused_macros)]
#[macro_export]
macro_rules! chmin {

    ($a:expr, $b:expr) => {{
        if $b < $a {
            $a = $b;
            true
        } else {
            false
        }
    }};

}

/// A macro to update a mutable variable with a new value if the new value is **larger than** the current value,
/// returning bool whether the current value is updated or not
///
/// ```
/// let mut a = 2;
/// let b = 4;
///
/// assert!(chmax!(a, b));
/// assert_eq!(a, b);
/// ```
#[allow(unused_macros)]
#[macro_export]
macro_rules! chmax {

    ($a:expr, $b:expr) => {{
        if $a < $b {
            $a = $b;
            true
        } else {
            false
        }
    }};

}

/// A lightweight logging macro that prints to standard output **only in debug builds**.
///
/// `debug!` mirrors the syntax of `println!`, but it executes its body only when
/// the `debug_assertions` configuration flag is enabled (typically in `cargo build`　and `cargo test`).
/// In release builds (`cargo build --release`), the call　is compiled but elided at runtime.
///
/// ```
/// let a = 2;
/// debug!("{}")
///
/// ```
#[allow(unused_macros)]
#[macro_export]
macro_rules! debug {

    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!($($arg)*);
        }
    };

}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::{BTreeSet, HashSet};
    use std::panic::{self, AssertUnwindSafe};
    use super::{parse_input_from_line, parse_inputs_from_line};

    thread_local! {
        static MOCK_INPUT_LINES: RefCell<Vec<String>> = RefCell::new(Vec::new());
    }

    fn set_mock_input_lines(input_lines: &[&str]) {
        MOCK_INPUT_LINES.with(|stored_lines| {
            let mut line_stack = stored_lines.borrow_mut();
            line_stack.clear();
            line_stack.extend(input_lines.iter().rev().map(|line| (*line).to_string()));
        });
    }

    fn take_next_mock_input_line() -> String {
        MOCK_INPUT_LINES.with(|stored_lines| {
            let mut line_stack = stored_lines.borrow_mut();
            line_stack.pop().unwrap_or_else(|| panic!("No mocked input line"))
        })
    }

    fn remaining_mock_input_line_count() -> usize {
        MOCK_INPUT_LINES.with(|stored_lines| {
            let line_stack = stored_lines.borrow();
            line_stack.len()
        })
    }

    fn parse_input<T: std::str::FromStr>() -> T
    where
        T::Err: std::fmt::Debug,
    {
        let input_line = take_next_mock_input_line();
        parse_input_from_line::<T>(&input_line)
    }

    fn parse_inputs<T: std::str::FromStr>() -> Vec<T>
    where
        T::Err: std::fmt::Debug,
    {
        let input_line = take_next_mock_input_line();
        parse_inputs_from_line::<T>(&input_line)
    }

    fn did_panic<F, R>(target_function: F) -> bool
    where
        F: FnOnce() -> R,
    {
        panic::catch_unwind(AssertUnwindSafe(target_function)).is_err()
    }

    #[test]
    fn parse_input_panics_when_input_cannot_be_converted_to_number() {
        // Arrange
        let input_lines = ["xyz"];
        set_mock_input_lines(&input_lines);

        // Act
        let actual_panicked = did_panic(|| {
            let _actual_value: i64 = parse_input::<i64>();
        });

        // Assert
        assert!(actual_panicked);
    }

    #[test]
    fn parse_input_from_line_trims_whitespace_and_newline_before_parsing() {
        // Arrange
        let source_line = "  42 \n";
        let expected_value = 42i64;

        // Act
        let actual_value = parse_input_from_line::<i64>(source_line);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn parse_inputs_from_line_parses_values_split_by_spaces_and_tabs() {
        // Arrange
        let source_line = "1\t2  3";
        let expected_values = vec![1i64, 2i64, 3i64];

        // Act
        let actual_values = parse_inputs_from_line::<i64>(source_line);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn parse_inputs_from_line_returns_empty_vector_for_empty_line() {
        // Arrange
        let source_line = "";
        let expected_values = Vec::<i64>::new();

        // Act
        let actual_values = parse_inputs_from_line::<i64>(source_line);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn read_value_byte_panics_for_empty_input() {
        // Arrange
        let input_lines = [""];
        set_mock_input_lines(&input_lines);

        // Act
        let actual_panicked = did_panic(|| crate::read_value!((), byte));

        // Assert
        assert!(actual_panicked);
    }

    #[test]
    fn read_value_byte_panics_for_multiple_characters() {
        // Arrange
        let input_lines = ["ab"];
        set_mock_input_lines(&input_lines);

        // Act
        let actual_panicked = did_panic(|| crate::read_value!((), byte));

        // Assert
        assert!(actual_panicked);
    }

    #[test]
    fn read_value_byte_panics_for_code_point_above_u8_max() {
        // Arrange
        let input_lines = ["Ā"];
        set_mock_input_lines(&input_lines);

        // Act
        let actual_panicked = did_panic(|| crate::read_value!((), byte));

        // Assert
        assert!(actual_panicked);
    }

    #[test]
    fn read_value_vector_with_length_zero_returns_empty_vector() {
        // Arrange
        let input_lines = [];
        let expected_values = Vec::<i64>::new();
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        let length = 0;

        // Act
        let actual_values: Vec<i64> = crate::read_value!((), [i64; length]);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_vector_with_length_one_returns_single_element() {
        // Arrange
        let input_lines = ["15"];
        let expected_values = vec![15i64];
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        let length = 1;

        // Act
        let actual_values: Vec<i64> = crate::read_value!((), [i64; length]);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_vector_with_length_two_preserves_element_order() {
        // Arrange
        let input_lines = ["3", "9"];
        let expected_values = vec![3i64, 9i64];
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        let length = 2;

        // Act
        let actual_values: Vec<i64> = crate::read_value!((), [i64; length]);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_nested_vector_with_length_zero_returns_empty_outer_vector() {
        // Arrange
        let expected_values = Vec::<Vec<i64>>::new();
        set_mock_input_lines(&[]);

        let expected_remaining_line_count = 0usize;

        let length = 0;

        // Act
        let actual_values: Vec<Vec<i64>> = crate::read_value!((), [[i64]; length]);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_nested_vector_with_length_one_returns_single_row() {
        // Arrange
        let input_lines = ["1 2 3"];
        let expected_values = vec![vec![1i64, 2i64, 3i64]];
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        let length = 1;

        // Act
        let actual_values: Vec<Vec<i64>> = crate::read_value!((), [[i64]; length]);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_nested_vector_with_length_two_preserves_row_order() {
        // Arrange
        let input_lines = ["1 2", "3 4"];
        let expected_values = vec![vec![1i64, 2i64], vec![3i64, 4i64]];
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        let length = 2;

        // Act
        let actual_values: Vec<Vec<i64>> = crate::read_value!((), [[i64]; length]);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_tuple_vector_with_length_zero_returns_empty_vector() {
        // Arrange
        let expected_values = Vec::<(usize, usize)>::new();
        set_mock_input_lines(&[]);

        let expected_remaining_line_count = 0usize;

        let length = 0;

        // Act
        let actual_values: Vec<(usize, usize)> = crate::read_value!((), [(usize, usize); length]);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_tuple_vector_with_length_one_returns_single_tuple() {
        // Arrange
        let input_lines = ["2 5"];
        let expected_values = vec![(2usize, 5usize)];
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        let length = 1;

        // Act
        let actual_values: Vec<(usize, usize)> = crate::read_value!((), [(usize, usize); length]);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_tuple_vector_with_length_two_preserves_row_order() {
        // Arrange
        let input_lines = ["1 4", "7 9"];
        let expected_values = vec![(1usize, 4usize), (7usize, 9usize)];
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        let length = 2;

        // Act
        let actual_values: Vec<(usize, usize)> = crate::read_value!((), [(usize, usize); length]);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_chars_converts_single_token_into_character_vector() {
        // Arrange
        let input_lines = ["rust"];
        let expected_values = vec!['r', 'u', 's', 't'];
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        let actual_values: Vec<char> = crate::read_value!((), chars);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_bytes_converts_single_token_into_byte_vector() {
        // Arrange
        let input_lines = ["AZ"];
        let expected_values = vec![65u8, 90u8];
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        let actual_values: Vec<u8> = crate::read_value!((), bytes);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_usize1_with_input_one_returns_zero() {
        // Arrange
        let input_lines = ["1"];
        let expected_value = 0usize;
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        let actual_value: usize = crate::read_value!((), usize1);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_chars_vector_with_zero_tokens_returns_empty_vector() {
        // Arrange
        let input_lines = [""];
        let expected_values = Vec::<Vec<char>>::new();
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        let actual_values: Vec<Vec<char>> = crate::read_value!((), [chars]);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_chars_vector_with_single_token_returns_single_element_vector() {
        // Arrange
        let input_lines = ["abc"];
        let expected_values = vec![vec!['a', 'b', 'c']];
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        let actual_values: Vec<Vec<char>> = crate::read_value!((), [chars]);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_chars_vector_with_multiple_tokens_returns_each_token_as_char_vector() {
        // Arrange
        let input_lines = ["ab cd"];
        let expected_values = vec![vec!['a', 'b'], vec!['c', 'd']];
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        let actual_values: Vec<Vec<char>> = crate::read_value!((), [chars]);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_usize1_vector_converts_each_token_to_zero_based_index() {
        // Arrange
        let input_lines = ["1 2 3"];
        let expected_values = vec![0usize, 1usize, 2usize];
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        let actual_values: Vec<usize> = crate::read_value!((), [usize1]);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn read_value_fixed_length_vector_panics_when_input_lines_are_insufficient() {
        // Arrange
        let input_lines = ["1", "2"];
        set_mock_input_lines(&input_lines);

        let length = 3;

        // Act
        let actual_panicked = did_panic(|| {
            let _actual_values: Vec<i64> = crate::read_value!((), [i64; length]);
        });

        // Assert
        assert!(actual_panicked);
    }

    #[test]
    fn read_value_scalar_panics_when_token_cannot_be_parsed_as_number() {
        // Arrange
        let input_lines = ["abc"];
        set_mock_input_lines(&input_lines);

        // Act
        let actual_panicked = did_panic(|| {
            let _actual_value: i64 = crate::read_value!((), i64);
        });

        // Assert
        assert!(actual_panicked);
    }

    #[test]
    fn read_value_tuple_vector_panics_when_tuple_columns_are_insufficient() {
        // Arrange
        let input_lines = ["8"];
        set_mock_input_lines(&input_lines);

        let length = 1;

        // Act
        let actual_panicked = did_panic(|| {
            let _actual_values: Vec<(usize, usize)> =
                crate::read_value!((), [(usize, usize); length]);
        });

        // Assert
        assert!(actual_panicked);
    }

    #[test]
    fn read_value_usize1_with_zero_input_follows_current_overflow_behavior() {
        // Arrange
        let input_lines = ["0"];
        set_mock_input_lines(&input_lines);

        // Act
        let actual_result = panic::catch_unwind(AssertUnwindSafe(|| crate::read_value!((), usize1)));

        // Assert
        if cfg!(debug_assertions) {
            assert!(actual_result.is_err());
        } else {
            let actual_value = actual_result.expect("release build should wrap subtraction");
            assert_eq!(actual_value, usize::MAX);
        }
    }

    #[test]
    fn input_macro_binds_single_scalar_value() {
        // Arrange
        let input_lines = ["42"];
        let expected_value = 42usize;
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        crate::input! {
            n: usize,
        }
        let actual_value = n;
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn input_macro_byte_with_surrounding_whitespace_returns_ascii_u8() {
        // Arrange
        let input_lines = [" \ta \n"];
        let expected_value = b'a';
        set_mock_input_lines(&input_lines);

        // Act
        crate::input! { c: byte }
        let actual_value: u8 = c;

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn input_macro_tuple_with_byte_and_number_returns_ascii_u8_and_number() {
        // Arrange
        let input_lines = ["a -42"];
        let expected_values = (b'a', -42i32);
        set_mock_input_lines(&input_lines);

        // Act
        crate::input! { (c, x): (byte, i32), }
        let actual_values: (u8, i32) = (c, x);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn input_macro_byte_at_u8_code_point_limit_returns_u8_max() {
        // Arrange
        let input_lines = ["ÿ"];
        let expected_value = u8::MAX;
        set_mock_input_lines(&input_lines);

        // Act
        crate::input! { c: byte }
        let actual_value: u8 = c;

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn input_macro_tuple_byte_at_u8_code_point_limit_returns_u8_max() {
        // Arrange
        let input_lines = ["ÿ 42"];
        let expected_values = (u8::MAX, 42i32);
        set_mock_input_lines(&input_lines);

        // Act
        crate::input! { (c, x): (byte, i32), }
        let actual_values: (u8, i32) = (c, x);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn input_macro_tuple_byte_panics_for_multiple_characters() {
        // Arrange
        let input_lines = ["ab 42"];
        set_mock_input_lines(&input_lines);

        // Act
        let actual_panicked = did_panic(|| {
            crate::input! { (c, x): (byte, i32), }
            (c, x)
        });

        // Assert
        assert!(actual_panicked);
    }

    #[test]
    fn input_macro_tuple_byte_panics_for_code_point_above_u8_max() {
        // Arrange
        let input_lines = ["Ā 42"];
        set_mock_input_lines(&input_lines);

        // Act
        let actual_panicked = did_panic(|| {
            crate::input! { (c, x): (byte, i32), }
            (c, x)
        });

        // Assert
        assert!(actual_panicked);
    }

    #[test]
    fn input_macro_binds_mutable_single_scalar_value() {
        // Arrange
        let input_lines = ["10"];
        let increment_value = 5usize;
        let expected_updated_value = 15usize;
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        crate::input! {
            mut n: usize,
        }
        n += increment_value;
        let actual_updated_value = n;
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_updated_value, expected_updated_value);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn input_macro_binds_single_line_tuple_value() {
        // Arrange
        let input_lines = ["-3 7"];
        let expected_value = (-3i32, 7usize);
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        crate::input! {
            p: (i32, usize),
        }
        let actual_value = p;
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn input_macro_binds_mutable_tuple_value() {
        // Arrange
        let input_lines = ["5 8"];
        let increment_value = 2usize;
        let expected_updated_value = 10usize;
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        crate::input! {
            mut p: (i32, usize),
        }
        p.1 += increment_value;
        let actual_updated_value = p.1;
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_updated_value, expected_updated_value);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn input_macro_bind_same_binds_multiple_variables_with_same_type() {
        // Arrange
        let input_lines = ["1 2 3"];
        let expected_values = (1usize, 2usize, 3usize);
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        crate::input! {
            (a, b, c): usize,
        }
        let actual_values = (a, b, c);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn input_macro_bind_same_supports_mutable_and_immutable_mixed_binding() {
        // Arrange
        let input_lines = ["4 6"];
        let increment_value = 3usize;
        let expected_values = (7usize, 6usize);
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        crate::input! {
            (mut a, b): usize,
        }
        a += increment_value;
        let actual_values = (a, b);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn input_macro_zip_supports_mutable_and_immutable_mixed_binding() {
        // Arrange
        let input_lines = ["5 -2"];
        let increment_value = 10i64;
        let expected_values = (5usize, 8i64);
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        crate::input! {
            (a, mut b): (usize, i64),
        }
        b += increment_value;
        let actual_values = (a, b);
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn input_macro_with_multiple_declarations_reads_each_line_independently() {
        // Arrange
        let input_lines = ["7", "8 9", "ab cd"];
        let expected_n = 7usize;
        let expected_pair = (8usize, 9usize);
        let expected_letters = vec![vec!['a', 'b'], vec!['c', 'd']];
        set_mock_input_lines(&input_lines);

        let expected_remaining_line_count = 0usize;

        // Act
        crate::input! {
            n: usize,
            pair: (usize, usize),
            letters: [chars],
        }
        let actual_n = n;
        let actual_pair = pair;
        let actual_letters = letters;
        let actual_remaining_line_count = remaining_mock_input_line_count();

        // Assert
        assert_eq!(actual_n, expected_n);
        assert_eq!(actual_pair, expected_pair);
        assert_eq!(actual_letters, expected_letters);
        assert_eq!(actual_remaining_line_count, expected_remaining_line_count);
    }

    #[test]
    fn join_empty_collection_returns_empty_string() {
        // Arrange
        let values: [i32; 0] = [];
        let separator = ", ";
        let expected_value = "";

        // Act
        let actual_value = crate::join!(values, separator);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn join_single_element_omits_separator() {
        // Arrange
        let values = [42];
        let separator = ", ";
        let expected_value = "42";

        // Act
        let actual_value = crate::join!(values, separator);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn join_multiple_numbers_preserves_order_and_duplicates() {
        // Arrange
        let values = [3, -1, 3, 0];
        let separator = ", ";
        let expected_value = "3, -1, 3, 0";

        // Act
        let actual_value = crate::join!(values, separator);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn join_iterator_adapter_uses_iteration_order() {
        // Arrange
        let values = [1, 2, 3];
        let iterator = values.iter().rev();
        let separator = "/";
        let expected_value = "3/2/1";

        // Act
        let actual_value = crate::join!(iterator, separator);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn join_borrowed_collection_returns_joined_elements() {
        // Arrange
        let values = vec![String::from("left"), String::from("right")];
        let separator = " -> ";
        let expected_value = "left -> right";

        // Act
        let actual_value = crate::join!(&values, separator);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn join_empty_separator_concatenates_elements() {
        // Arrange
        let values = ["ab", "cd", "ef"];
        let separator = "";
        let expected_value = "abcdef";

        // Act
        let actual_value = crate::join!(values, separator);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn join_unicode_and_empty_elements_preserves_separators() {
        // Arrange
        let values = ["", "赤", "", "青", ""];
        let separator = "/";
        let expected_value = "/赤//青/";

        // Act
        let actual_value = crate::join!(values, separator);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn join_with_trailing_comma_returns_joined_elements() {
        // Arrange
        let values = [1, 2];
        let separator = "-";
        let expected_value = "1-2";

        // Act
        let actual_value = crate::join!(values, separator,);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn tree_set_macro_with_no_arguments_returns_empty_set() {
        // Arrange
        let expected_set = BTreeSet::<i32>::new();

        // Act
        let actual_set = crate::tree_set![];

        // Assert
        assert_eq!(actual_set, expected_set);
    }

    #[test]
    fn tree_set_macro_with_single_argument_returns_singleton_set() {
        // Arrange
        let first_value = 1i32;
        let expected_set = vec![first_value].into_iter().collect::<BTreeSet<_>>();

        // Act
        let actual_set = crate::tree_set![first_value];

        // Assert
        assert_eq!(actual_set, expected_set);
    }

    #[test]
    fn tree_set_macro_removes_duplicate_elements() {
        // Arrange
        let first_value = 1i32;
        let second_value = 2i32;
        let expected_set = vec![first_value, second_value]
            .into_iter()
            .collect::<BTreeSet<_>>();

        // Act
        let actual_set = crate::tree_set![first_value, second_value, second_value];

        // Assert
        assert_eq!(actual_set, expected_set);
    }

    #[test]
    fn tree_set_macro_accepts_trailing_comma() {
        // Arrange
        let first_value = 1i32;
        let second_value = 2i32;
        let expected_set = vec![first_value, second_value]
            .into_iter()
            .collect::<BTreeSet<_>>();

        // Act
        let actual_set = crate::tree_set![first_value, second_value,];

        // Assert
        assert_eq!(actual_set, expected_set);
    }

    #[test]
    fn hash_set_macro_with_no_arguments_returns_empty_set() {
        // Arrange
        let expected_set = HashSet::<i32>::new();

        // Act
        let actual_set = crate::hash_set![];

        // Assert
        assert_eq!(actual_set, expected_set);
    }

    #[test]
    fn hash_set_macro_with_single_argument_returns_singleton_set() {
        // Arrange
        let first_value = 1i32;
        let expected_set = vec![first_value].into_iter().collect::<HashSet<_>>();

        // Act
        let actual_set = crate::hash_set![first_value];

        // Assert
        assert_eq!(actual_set, expected_set);
    }

    #[test]
    fn hash_set_macro_removes_duplicate_elements() {
        // Arrange
        let first_value = 1i32;
        let second_value = 2i32;
        let expected_set = vec![first_value, second_value]
            .into_iter()
            .collect::<HashSet<_>>();

        // Act
        let actual_set = crate::hash_set![first_value, second_value, second_value];

        // Assert
        assert_eq!(actual_set, expected_set);
    }

    #[test]
    fn hash_set_macro_accepts_trailing_comma() {
        // Arrange
        let first_value = 1i32;
        let second_value = 2i32;
        let expected_set = vec![first_value, second_value]
            .into_iter()
            .collect::<HashSet<_>>();

        // Act
        let actual_set = crate::hash_set![first_value, second_value,];

        // Assert
        assert_eq!(actual_set, expected_set);
    }

    #[test]
    fn chmin_returns_true_and_updates_value_when_rhs_is_smaller() {
        // Arrange
        let mut target_value = 10i64;
        let candidate_value = 7i64;
        let expected_updated_value = 7i64;
        let expected_updated_flag = true;

        // Act
        let actual_updated_flag = crate::chmin!(target_value, candidate_value);
        let actual_updated_value = target_value;

        // Assert
        assert_eq!(actual_updated_flag, expected_updated_flag);
        assert_eq!(actual_updated_value, expected_updated_value);
    }

    #[test]
    fn chmin_returns_false_and_keeps_value_when_values_are_equal() {
        // Arrange
        let mut target_value = 5i64;
        let candidate_value = 5i64;
        let expected_value = 5i64;
        let expected_updated_flag = false;

        // Act
        let actual_updated_flag = crate::chmin!(target_value, candidate_value);
        let actual_value = target_value;

        // Assert
        assert_eq!(actual_updated_flag, expected_updated_flag);
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn chmin_returns_false_and_keeps_value_when_rhs_is_larger() {
        // Arrange
        let mut target_value = 4i64;
        let candidate_value = 9i64;
        let expected_value = 4i64;
        let expected_updated_flag = false;

        // Act
        let actual_updated_flag = crate::chmin!(target_value, candidate_value);
        let actual_value = target_value;

        // Assert
        assert_eq!(actual_updated_flag, expected_updated_flag);
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn chmax_returns_true_and_updates_value_when_rhs_is_larger() {
        // Arrange
        let mut target_value = 3i64;
        let candidate_value = 8i64;
        let expected_updated_value = 8i64;
        let expected_updated_flag = true;

        // Act
        let actual_updated_flag = crate::chmax!(target_value, candidate_value);
        let actual_updated_value = target_value;

        // Assert
        assert_eq!(actual_updated_flag, expected_updated_flag);
        assert_eq!(actual_updated_value, expected_updated_value);
    }

    #[test]
    fn chmax_returns_false_and_keeps_value_when_values_are_equal() {
        // Arrange
        let mut target_value = 6i64;
        let candidate_value = 6i64;
        let expected_value = 6i64;
        let expected_updated_flag = false;

        // Act
        let actual_updated_flag = crate::chmax!(target_value, candidate_value);
        let actual_value = target_value;

        // Assert
        assert_eq!(actual_updated_flag, expected_updated_flag);
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn chmax_returns_false_and_keeps_value_when_rhs_is_smaller() {
        // Arrange
        let mut target_value = 12i64;
        let candidate_value = 4i64;
        let expected_value = 12i64;
        let expected_updated_flag = false;

        // Act
        let actual_updated_flag = crate::chmax!(target_value, candidate_value);
        let actual_value = target_value;

        // Assert
        assert_eq!(actual_updated_flag, expected_updated_flag);
        assert_eq!(actual_value, expected_value);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn debug_macro_in_debug_build_accepts_format_argument_without_panic() {
        // Arrange
        let mut evaluation_count = 0usize;
        let expected_evaluation_count = 1usize;

        // Act
        let actual_result = panic::catch_unwind(AssertUnwindSafe(|| {
            crate::debug!("{}", {
                evaluation_count += 1usize;
                evaluation_count
            });
        }));
        let actual_evaluation_count = evaluation_count;

        // Assert
        assert!(actual_result.is_ok());
        assert_eq!(actual_evaluation_count, expected_evaluation_count);
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn debug_macro_in_release_build_does_not_evaluate_format_argument() {
        // Arrange
        let mut evaluation_count = 0usize;
        let expected_evaluation_count = 0usize;

        // Act
        crate::debug!("{}", {
            evaluation_count += 1usize;
            evaluation_count
        });
        let actual_evaluation_count = evaluation_count;

        // Assert
        assert_eq!(actual_evaluation_count, expected_evaluation_count);
    }
}

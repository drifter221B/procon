fn solve(out: &mut impl std::io::Write) -> std::io::Result<()> {

    input! {
        n: usize,
        (x, y, z): (usize, usize, usize),
        a: [i64],
        s: bytes,
    }

    Ok(())
}

fn main() {
    let is_repeat = false;

    Answer::new(is_repeat).answer();
}

struct Answer(usize);

impl Answer {

    pub fn new(is_repeat: bool) -> Self {
        let cycle = if is_repeat { input!{ t: usize } t } else { 1 };

        Self(cycle)
    }

    pub fn answer(&self) {
        let stdout = std::io::stdout();
        let mut out = std::io::BufWriter::new(stdout.lock());

        (0..self.0).for_each(|_| solve(&mut out).unwrap_or_else(|_| eprintln!("IO Error!")));
    }

}

#[cfg(not(test))]
#[allow(dead_code)]
fn read_input() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap_or_else(|_| panic!("Failed to read line"));

    input
}

#[allow(dead_code)]
fn parse_input<T: std::str::FromStr>() -> T {
    read_input().trim()
        .parse()
        .ok()
        .expect("Failed to parse")
}

#[allow(dead_code)]
fn parse_inputs<T: std::str::FromStr>() -> Vec<T> {
    read_input().split_whitespace()
        .map(|s| s.parse::<T>().ok().expect("Failed to parse"))
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
fn read_input() -> String {
    tests::take_input_line()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    thread_local! {
        static INPUT_LINES: RefCell<VecDeque<String>> = RefCell::new(VecDeque::new());
    }

    fn set_input_lines(lines: &[&str]) {
        INPUT_LINES.with(|stored| {
            *stored.borrow_mut() = lines.iter().map(|line| (*line).to_owned()).collect();
        });
    }

    pub(super) fn take_input_line() -> String {
        INPUT_LINES.with(|stored| stored.borrow_mut().pop_front().expect("no test input line"))
    }

    #[test]
    fn answer_new_without_repetition_sets_one_cycle_without_reading_input() {
        // Arrange
        let is_repeat = false;
        let input_lines = [];
        set_input_lines(&input_lines);
        let expected_cycles = 1;

        // Act
        let actual_answer = Answer::new(is_repeat);

        // Assert
        assert_eq!(actual_answer.0, expected_cycles);
    }

    #[test]
    fn parse_inputs_with_empty_line_returns_empty_vector() {
        // Arrange
        let input_lines = [" \t\n"];
        set_input_lines(&input_lines);
        let expected_values = Vec::<i64>::new();

        // Act
        let actual_values = parse_inputs::<i64>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn parse_inputs_with_single_token_returns_one_value() {
        // Arrange
        let input_lines = [" 42\n"];
        set_input_lines(&input_lines);
        let expected_values = vec![42i64];

        // Act
        let actual_values = parse_inputs::<i64>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn parse_inputs_with_multiple_tokens_preserves_order_and_duplicates() {
        // Arrange
        let input_lines = ["3 -1\t3\n"];
        set_input_lines(&input_lines);
        let expected_values = vec![3i64, -1, 3];

        // Act
        let actual_values = parse_inputs::<i64>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn read_value_usize1_at_lower_boundary_converts_one_and_two_to_zero_and_one() {
        // Arrange
        let input_lines = ["1", "2"];
        set_input_lines(&input_lines);
        let expected_values = [0usize, 1usize];

        // Act
        let actual_values = [read_value!((), usize1), read_value!((), usize1)];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn read_value_chars_preserves_unicode_character_order() {
        // Arrange
        let input_lines = ["aあ🦀"];
        set_input_lines(&input_lines);
        let expected_characters = vec!['a', 'あ', '🦀'];

        // Act
        let actual_characters = read_value!((), chars);

        // Assert
        assert_eq!(actual_characters, expected_characters);
    }
}

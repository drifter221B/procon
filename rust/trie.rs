use std::collections::HashMap;

#[derive(Default)]
pub struct Node {
    pub children: HashMap<u8, Node>,
    pub counter: usize,
    pub end_counter: usize,
}

impl Node {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Default)]
pub struct Trie {
    root: Node,
}

impl Trie {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, word: String) {
        let mut node = &mut self.root;

        for c in word.bytes() {
            node = node.children.entry(c).or_default();
            node.counter += 1;
        }

        node.end_counter += 1;
    }

    pub fn search(&self, word: &str) -> bool {
        let mut node = &self.root;

        for c in word.as_bytes() {
            node = match node.children.get(c) {
                Some(child) => child,
                None => return false,
            }
        }

        0 < node.end_counter
    }

    pub fn remove(&mut self, word: &str) -> bool {
        if self.count_word(word) == 0 {
            return false;
        }

        let mut node = &mut self.root;

        for c in word.as_bytes() {
            match node.children.get_mut(c) {
                None => return false,
                Some(child) => {
                    child.counter -= 1;
                    if child.counter == 0 {
                        node.children.remove(c).unwrap();
                        return true;
                    }

                    node = node.children.get_mut(c).unwrap();
                }
            }
        }

        node.end_counter -= 1;

        true
    }

    pub fn remove_all(&mut self, word: &str) -> usize {
        let delete_count = self.count_word(word);
        if delete_count == 0 {
            return 0;
        }

        let mut node = &mut self.root;

        for c in word.as_bytes() {
            match node.children.get_mut(c) {
                None => return 0,
                Some(child) => {
                    child.counter -= delete_count;
                    if child.counter == 0 {
                        node.children.remove(c).unwrap();
                        return delete_count;
                    }

                    node = node.children.get_mut(c).unwrap();
                }
            }
        }

        node.end_counter -= delete_count;

        delete_count
    }

    pub fn is_prefix(&self, word: &str) -> bool {
        let mut node = &self.root;

        for c in word.as_bytes() {
            if let Some(child) = node.children.get(c) {
                node = child;
            } else {
                return false;
            }
        }

        true
    }

    pub fn is_word(&self, word: &str) -> bool {
        let mut node = &self.root;

        for c in word.as_bytes() {
            if let Some(child) = node.children.get(c) {
                node = child;
            } else {
                return false;
            }
        }

        0 < node.end_counter
    }

    pub fn count_prefix(&self, word: &str) -> usize {
        let mut node = &self.root;

        for c in word.as_bytes() {
            if let Some(child) = node.children.get(c) {
                node = child;
            } else {
                return 0;
            }
        }

        node.counter
    }

    pub fn count_prefix_all(&self, word: &str) -> usize {
        let mut node = &self.root;
        let mut result = 0;

        for c in word.as_bytes() {
            if let Some(child) = node.children.get(c) {
                result += child.counter;
                node = child;
            } else {
                return result;
            }
        }

        result
    }

    pub fn count_word(&self, word: &str) -> usize {
        let mut node = &self.root;

        for c in word.as_bytes() {
            if let Some(child) = node.children.get(c) {
                node = child;
            } else {
                return 0;
            }
        }

        node.end_counter
    }

    pub fn dfs<'a>(&'a self, c: u8, node: Option<&'a Node>, is_root: bool) -> Option<&'a Node> {
        let node = match node {
            Some(node) => node,
            None if !is_root => return None,
            None => &self.root,
        };

        node.children.get(&c)
    }

    pub fn lcp(&self, word: &str) -> usize {
        let mut node = &self.root;
        let mut result = 0;
        for c in word.as_bytes() {
            if let Some(child) = node.children.get(c) {
                result += 1;
                node = child;
            } else {
                return result;
            }
        }

        result
    }

    pub fn substring_indices(&self, word: &str) -> Vec<usize> {
        let mut result = vec![];
        let mut node = &self.root;

        for (i, c) in word.as_bytes().iter().enumerate() {
            if let Some(child) = node.children.get(c) {
                node = child;
            } else {
                break;
            }

            if 0 < node.end_counter {
                result.push(i);
            }
        }

        result
    }

    pub fn smallest_word(&self, prefix: &str) -> Option<String> {
        let mut node = &self.root;

        for c in prefix.as_bytes().iter() {
            node = node.children.get(c)?;
        }

        let mut result = prefix.as_bytes().to_vec();
        while node.end_counter == 0 {
            let min_child = node.children.keys().min()?;
            result.push(*min_child);

            node = node.children.get(min_child)?;
        }

        String::from_utf8(result).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::{Node, Trie};
    use std::collections::HashMap as StdHashMap;

    fn build_trie_with_words(words: &[&str]) -> Trie {
        let mut trie = Trie::new();
        for word in words {
            trie.insert((*word).to_string());
        }
        trie
    }

    fn insert_word_to_reference_counts(
        reference_word_counts: &mut StdHashMap<String, usize>,
        word: &str,
    ) {
        let current_count = reference_word_counts.get(word).copied().unwrap_or(0);
        reference_word_counts.insert(word.to_string(), current_count + 1);
    }

    fn remove_word_from_reference_counts(
        reference_word_counts: &mut StdHashMap<String, usize>,
        word: &str,
    ) -> bool {
        let current_count = reference_word_counts.get(word).copied().unwrap_or(0);
        if current_count == 0 {
            return false;
        }
        if current_count == 1 {
            reference_word_counts.remove(word);
        } else {
            reference_word_counts.insert(word.to_string(), current_count - 1);
        }
        true
    }

    fn remove_all_words_from_reference_counts(
        reference_word_counts: &mut StdHashMap<String, usize>,
        word: &str,
    ) -> usize {
        reference_word_counts.remove(word).unwrap_or(0)
    }

    fn bytes_lcp_length(lhs: &str, rhs: &str) -> usize {
        lhs.as_bytes()
            .iter()
            .zip(rhs.as_bytes().iter())
            .take_while(|(left_byte, right_byte)| left_byte == right_byte)
            .count()
    }

    fn assert_descendant_node_invariants(node: &Node) {
        for child in node.children.values() {
            let expected_counter = child.end_counter
                + child
                    .children
                    .values()
                    .map(|grandchild| grandchild.counter)
                    .sum::<usize>();
            assert_eq!(child.counter, expected_counter);
            assert!(child.counter > 0);
            assert_descendant_node_invariants(child);
        }
    }

    fn expected_is_prefix(reference_word_counts: &StdHashMap<String, usize>, prefix: &str) -> bool {
        prefix.is_empty()
            || reference_word_counts
                .iter()
                .any(|(word, count)| *count > 0 && word.starts_with(prefix))
    }

    fn expected_count_prefix(
        reference_word_counts: &StdHashMap<String, usize>,
        prefix: &str,
    ) -> usize {
        if prefix.is_empty() {
            return 0;
        }

        reference_word_counts
            .iter()
            .filter(|(word, _)| word.starts_with(prefix))
            .map(|(_, count)| *count)
            .sum()
    }

    fn expected_count_prefix_all(
        reference_word_counts: &StdHashMap<String, usize>,
        word: &str,
    ) -> usize {
        let word_bytes = word.as_bytes();
        let mut result = 0;

        for prefix_len in 1..=word_bytes.len() {
            let prefix_count = reference_word_counts
                .iter()
                .filter(|(stored_word, _)| {
                    stored_word
                        .as_bytes()
                        .starts_with(&word_bytes[..prefix_len])
                })
                .map(|(_, count)| *count)
                .sum::<usize>();
            if prefix_count == 0 {
                break;
            }
            result += prefix_count;
        }

        result
    }

    fn expected_lcp(reference_word_counts: &StdHashMap<String, usize>, word: &str) -> usize {
        reference_word_counts
            .keys()
            .map(|stored_word| bytes_lcp_length(stored_word, word))
            .max()
            .unwrap_or(0)
    }

    fn expected_substring_indices(
        reference_word_counts: &StdHashMap<String, usize>,
        word: &str,
    ) -> Vec<usize> {
        let word_bytes = word.as_bytes();

        (0..word_bytes.len())
            .filter(|index| {
                reference_word_counts.iter().any(|(stored_word, count)| {
                    *count > 0 && stored_word.as_bytes() == &word_bytes[..=*index]
                })
            })
            .collect()
    }

    fn expected_smallest_word(
        reference_word_counts: &StdHashMap<String, usize>,
        prefix: &str,
    ) -> Option<String> {
        reference_word_counts
            .iter()
            .filter(|(word, count)| **count > 0 && word.starts_with(prefix))
            .map(|(word, _)| word)
            .min()
            .cloned()
    }

    type QuerySnapshot = (bool, bool, bool, usize, usize, usize, usize, Vec<usize>, Option<String>);

    fn reference_queries(
        reference: &StdHashMap<String, usize>,
        words: &[&str],
    ) -> Vec<QuerySnapshot> {
        words
            .iter()
            .map(|&word| {
                let count = reference.get(word).copied().unwrap_or(0);
                (
                    count > 0,
                    count > 0,
                    expected_is_prefix(reference, word),
                    count,
                    expected_count_prefix(reference, word),
                    expected_count_prefix_all(reference, word),
                    expected_lcp(reference, word),
                    expected_substring_indices(reference, word),
                    expected_smallest_word(reference, word),
                )
            })
            .collect()
    }

    fn actual_queries(trie: &Trie, words: &[&str]) -> Vec<QuerySnapshot> {
        words
            .iter()
            .map(|&word| {
                (
                    trie.search(word),
                    trie.is_word(word),
                    trie.is_prefix(word),
                    trie.count_word(word),
                    trie.count_prefix(word),
                    trie.count_prefix_all(word),
                    trie.lcp(word),
                    trie.substring_indices(word),
                    trie.smallest_word(word),
                )
            })
            .collect()
    }

    #[test]
    fn node_new_returns_empty_node_with_zero_counters() {
        // Arrange
        let expected_state = (true, 0usize, 0usize);

        // Act
        let node = Node::new();
        let actual_state = (node.children.is_empty(), node.counter, node.end_counter);

        // Assert
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn trie_default_has_same_empty_state_as_new() {
        // Arrange
        let new_trie = Trie::new();
        let reference_word_counts = StdHashMap::new();
        let words_to_check = ["", "a", "あ"];
        let expected_state = reference_queries(&reference_word_counts, &words_to_check);

        // Act
        let default_trie = Trie::default();
        let actual_default_state = actual_queries(&default_trie, &words_to_check);
        let actual_new_state = actual_queries(&new_trie, &words_to_check);

        // Assert
        assert_eq!(actual_default_state, expected_state);
        assert_eq!(actual_new_state, expected_state);
    }

    #[test]
    fn insert_single_word_makes_search_true_and_count_one() {
        // Arrange
        let mut trie = Trie::new();
        let insert_word = "a";

        let expected_count_right = 1;

        // Act
        trie.insert(insert_word.to_string());
        let actual_search = trie.search(insert_word);
        let actual_count = trie.count_word(insert_word);

        // Assert
        assert!(actual_search);
        assert_eq!(actual_count, expected_count_right);
    }

    #[test]
    fn insert_same_word_twice_increases_word_count_to_two() {
        // Arrange
        let mut trie = Trie::new();
        let insert_word = "a";

        let expected_count_right = 2;

        // Act
        trie.insert(insert_word.to_string());
        trie.insert(insert_word.to_string());
        let actual_count = trie.count_word(insert_word);

        // Assert
        assert_eq!(actual_count, expected_count_right);
    }

    #[test]
    fn insert_empty_string_makes_empty_search_true() {
        // Arrange
        let mut trie = Trie::new();
        let empty_word = "";

        let expected_count_right = 1;

        // Act
        trie.insert(empty_word.to_string());
        let actual_search = trie.search(empty_word);
        let actual_count = trie.count_word(empty_word);

        // Assert
        assert!(actual_search);
        assert_eq!(actual_count, expected_count_right);
    }

    #[test]
    fn search_returns_false_for_unregistered_word_in_empty_trie() {
        // Arrange
        let trie = Trie::new();
        let search_word = "a";

        // Act
        let actual = trie.search(search_word);

        // Assert
        assert!(!actual);
    }

    #[test]
    fn search_returns_false_for_prefix_only_word() {
        // Arrange
        let trie = build_trie_with_words(&["ab"]);
        let search_word = "a";

        // Act
        let actual = trie.search(search_word);

        // Assert
        assert!(!actual);
    }

    #[test]
    fn search_and_is_word_return_same_boolean_for_each_word() {
        // Arrange
        let trie = build_trie_with_words(&["ab", "abc", "xyz"]);
        let test_words = ["ab", "abc", "a", "xy", "xyz", "zzz"];

        // Act
        let actual_pairs: Vec<(bool, bool)> = test_words
            .iter()
            .map(|word| (trie.search(word), trie.is_word(word)))
            .collect();

        // Assert
        for (search_result, is_word_result) in actual_pairs {
            assert_eq!(search_result, is_word_result);
        }
    }

    #[test]
    fn search_for_empty_word_returns_false_in_empty_trie() {
        // Arrange
        let trie = Trie::new();

        let search_key = "";

        // Act
        let actual = trie.search(search_key);

        // Assert
        assert!(!actual);
    }

    #[test]
    fn remove_returns_false_for_unregistered_word_and_keeps_state() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab"]);
        let remove_word = "ac";
        let remain_word = "ab";

        let expected_remain_count_right = 1;

        // Act
        let actual_removed = trie.remove(remove_word);
        let actual_remain_count = trie.count_word(remain_word);

        // Assert
        assert!(!actual_removed);
        assert_eq!(actual_remain_count, expected_remain_count_right);
    }

    #[test]
    fn remove_unique_word_makes_search_false_after_removal() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab"]);
        let remove_word = "ab";

        // Act
        let actual_removed = trie.remove(remove_word);
        let actual_search = trie.search(remove_word);

        // Assert
        assert!(actual_removed);
        assert!(!actual_search);
    }

    #[test]
    fn remove_once_from_duplicated_word_decrements_count_by_one() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab", "ab"]);
        let remove_word = "ab";

        let expected_count_right = 1;

        // Act
        let actual_removed = trie.remove(remove_word);
        let actual_count = trie.count_word(remove_word);

        // Assert
        assert!(actual_removed);
        assert_eq!(actual_count, expected_count_right);
    }

    #[test]
    fn remove_word_keeps_other_word_with_shared_prefix() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab", "abc"]);
        let remove_word = "ab";
        let remain_word = "abc";

        // Act
        let actual_removed = trie.remove(remove_word);
        let actual_remain_search = trie.search(remain_word);

        // Assert
        assert!(actual_removed);
        assert!(actual_remain_search);
    }

    #[test]
    fn remove_registered_empty_string_decrements_empty_count() {
        // Arrange
        let mut trie = build_trie_with_words(&[""]);
        let empty_word = "";

        let expected_count_right = 0;

        // Act
        let actual_removed = trie.remove(empty_word);
        let actual_count = trie.count_word(empty_word);

        // Assert
        assert!(actual_removed);
        assert_eq!(actual_count, expected_count_right);
    }

    #[test]
    fn remove_unregistered_empty_string_returns_false() {
        // Arrange
        let mut trie = Trie::new();
        let empty_word = "";

        // Act
        let actual_removed = trie.remove(empty_word);

        // Assert
        assert!(!actual_removed);
    }

    #[test]
    fn remove_duplicate_updates_all_shared_path_counters() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab", "ab", "abc", "ax"]);

        let expected_trie_count_word_ab_right = 1;
        let expected_trie_count_prefix_a_right = 3;
        let expected_trie_count_prefix_ab_right = 2;
        let expected_trie_count_prefix_abc_right = 1;

        let remove_key = "ab";
        let count_word_word = "ab";
        let count_prefix_prefix = "a";
        let count_prefix_prefix_2 = "ab";
        let count_prefix_prefix_3 = "abc";

        // Act
        let actual_removed = trie.remove(remove_key);

        let actual_trie_count_word_ab = trie.count_word(count_word_word);
        let actual_trie_count_prefix_a = trie.count_prefix(count_prefix_prefix);
        let actual_trie_count_prefix_ab = trie.count_prefix(count_prefix_prefix_2);
        let actual_trie_count_prefix_abc = trie.count_prefix(count_prefix_prefix_3);

        // Assert
        assert!(actual_removed);
        assert_eq!(actual_trie_count_word_ab, expected_trie_count_word_ab_right);
        assert_eq!(
            actual_trie_count_prefix_a,
            expected_trie_count_prefix_a_right
        );
        assert_eq!(
            actual_trie_count_prefix_ab,
            expected_trie_count_prefix_ab_right
        );
        assert_eq!(
            actual_trie_count_prefix_abc,
            expected_trie_count_prefix_abc_right
        );
    }

    #[test]
    fn remove_terminal_word_keeps_its_path_when_descendant_word_exists() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab", "abc"]);

        let expected_trie_count_word_ab_right = 0;

        let remove_key = "ab";
        let count_word_word = "ab";

        let search_key = "ab";
        let is_word_word = "ab";
        let is_prefix_prefix = "ab";
        let search_key_2 = "abc";

        // Act
        let actual_removed = trie.remove(remove_key);

        let actual_trie_count_word_ab = trie.count_word(count_word_word);

        let actual_condition_1 = trie.search(search_key);
        let actual_condition_2 = trie.is_word(is_word_word);
        let actual_condition_3 = trie.is_prefix(is_prefix_prefix);
        let actual_condition_4 = trie.search(search_key_2);

        // Assert
        assert!(actual_removed);
        assert!(!actual_condition_1);
        assert!(!actual_condition_2);
        assert_eq!(actual_trie_count_word_ab, expected_trie_count_word_ab_right);
        assert!(actual_condition_3);
        assert!(actual_condition_4);
    }

    #[test]
    fn remove_prunes_only_target_branch_and_keeps_terminal_ancestor_and_sibling() {
        // Arrange
        let mut trie = build_trie_with_words(&["a", "ab", "ac"]);

        let expected_trie_count_prefix_a_right = 2;

        let remove_key = "ab";
        let dfs_node = None;
        let dfs_is_root = true;
        let count_prefix_prefix = "a";

        let search_key = "a";
        let search_key_2 = "ac";
        let is_prefix_prefix = "ab";
        let dfs_is_root_2 = false;

        // Act
        let actual_removed = trie.remove(remove_key);
        let a_node = trie.dfs(b'a', dfs_node, dfs_is_root).unwrap();

        let actual_trie_count_prefix_a = trie.count_prefix(count_prefix_prefix);

        let actual_condition_1 = trie.search(search_key);
        let actual_condition_2 = trie.search(search_key_2);
        let actual_condition_3 = trie.is_prefix(is_prefix_prefix);
        let actual_condition_4 = trie.dfs(b'b', Some(a_node), dfs_is_root_2).is_none();

        // Assert
        assert!(actual_removed);
        assert!(actual_condition_1);
        assert!(actual_condition_2);
        assert!(!actual_condition_3);
        assert!(actual_condition_4);
        assert_eq!(
            actual_trie_count_prefix_a,
            expected_trie_count_prefix_a_right
        );
    }

    #[test]
    fn remove_last_word_prunes_entire_path() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab"]);

        let expected_trie_lcp_ab_right = 0;

        let remove_key = "ab";
        let lcp_query = "ab";

        let dfs_node = None;
        let dfs_is_root = true;
        let is_prefix_prefix = "a";

        // Act
        let actual_removed = trie.remove(remove_key);

        let actual_trie_lcp_ab = trie.lcp(lcp_query);

        let actual_condition_1 = trie.dfs(b'a', dfs_node, dfs_is_root).is_none();
        let actual_condition_2 = trie.is_prefix(is_prefix_prefix);

        // Assert
        assert!(actual_removed);
        assert!(actual_condition_1);
        assert!(!actual_condition_2);
        assert_eq!(actual_trie_lcp_ab, expected_trie_lcp_ab_right);
    }

    #[test]
    fn remove_existing_nonterminal_prefix_keeps_all_counts_unchanged() {
        // Arrange
        let mut trie = build_trie_with_words(&["abc"]);
        let counts_before = (
            trie.count_prefix("a"),
            trie.count_prefix("ab"),
            trie.count_prefix("abc"),
            trie.count_word("abc"),
        );

        let remove_key = "ab";
        let count_prefix_prefix = "a";
        let count_prefix_prefix_2 = "ab";
        let count_prefix_prefix_3 = "abc";
        let count_word_word = "abc";

        // Act
        let actual_removed = trie.remove(remove_key);

        let actual_trie_count_prefix_a_trie_count_prefix_ab_trie_count_pre = (
            trie.count_prefix(count_prefix_prefix),
            trie.count_prefix(count_prefix_prefix_2),
            trie.count_prefix(count_prefix_prefix_3),
            trie.count_word(count_word_word),
        );

        // Assert
        assert!(!actual_removed);
        assert_eq!(
            actual_trie_count_prefix_a_trie_count_prefix_ab_trie_count_pre,
            counts_before
        );
    }

    #[test]
    fn remove_empty_word_keeps_nonempty_word_state_unchanged() {
        // Arrange
        let mut trie = build_trie_with_words(&["", "a"]);

        let expected_trie_count_prefix_a_right = 1;

        let remove_key = "";
        let count_prefix_prefix = "a";

        let search_key = "";
        let search_key_2 = "a";

        // Act
        let actual_removed = trie.remove(remove_key);

        let actual_trie_count_prefix_a = trie.count_prefix(count_prefix_prefix);

        let actual_condition_1 = trie.search(search_key);
        let actual_condition_2 = trie.search(search_key_2);

        // Assert
        assert!(actual_removed);
        assert!(!actual_condition_1);
        assert!(actual_condition_2);
        assert_eq!(
            actual_trie_count_prefix_a,
            expected_trie_count_prefix_a_right
        );
    }

    #[test]
    fn repeated_removal_after_last_copy_keeps_trie_empty() {
        // Arrange
        let mut trie = build_trie_with_words(&["abc"]);
        assert!(trie.remove("abc"));

        let expected_remove_all_right = 0;

        let remove_key = "abc";
        let remove_all_word = "abc";

        let search_key = "abc";
        let dfs_node = None;
        let dfs_is_root = true;

        // Act
        let actual_second_remove = trie.remove(remove_key);
        let actual_remove_all = trie.remove_all(remove_all_word);

        let actual_condition_1 = trie.search(search_key);
        let actual_condition_2 = trie.dfs(b'a', dfs_node, dfs_is_root).is_none();

        // Assert
        assert!(!actual_second_remove);
        assert_eq!(actual_remove_all, expected_remove_all_right);
        assert!(!actual_condition_1);
        assert!(actual_condition_2);
    }

    #[test]
    fn remove_all_returns_zero_for_unregistered_word_and_keeps_state() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab"]);
        let remove_word = "ac";
        let remain_word = "ab";

        let expected_deleted_right = 0;
        let expected_remain_count_right = 1;

        // Act
        let actual_deleted = trie.remove_all(remove_word);
        let actual_remain_count = trie.count_word(remain_word);

        // Assert
        assert_eq!(actual_deleted, expected_deleted_right);
        assert_eq!(actual_remain_count, expected_remain_count_right);
    }

    #[test]
    fn remove_all_returns_registered_count_and_clears_word() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab", "ab", "ab"]);
        let remove_word = "ab";

        let expected_deleted_right = 3;
        let expected_count_right = 0;

        // Act
        let actual_deleted = trie.remove_all(remove_word);
        let actual_count = trie.count_word(remove_word);

        // Assert
        assert_eq!(actual_deleted, expected_deleted_right);
        assert_eq!(actual_count, expected_count_right);
    }

    #[test]
    fn remove_all_for_one_shared_prefix_word_keeps_other_word() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab", "abc"]);
        let remove_word = "ab";
        let remain_word = "abc";

        let expected_deleted_right = 1;

        // Act
        let actual_deleted = trie.remove_all(remove_word);
        let actual_remain_search = trie.search(remain_word);

        // Assert
        assert_eq!(actual_deleted, expected_deleted_right);
        assert!(actual_remain_search);
    }

    #[test]
    fn remove_all_for_empty_word_returns_all_count_and_clears_it() {
        // Arrange
        let mut trie = build_trie_with_words(&["", "", "a"]);
        let empty_word = "";

        let expected_deleted_right = 2;
        let expected_count_right = 0;

        // Act
        let actual_deleted = trie.remove_all(empty_word);
        let actual_count = trie.count_word(empty_word);

        // Assert
        assert_eq!(actual_deleted, expected_deleted_right);
        assert_eq!(actual_count, expected_count_right);
    }

    #[test]
    fn remove_all_duplicates_updates_counters_while_keeping_descendant_path() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab", "ab", "ab", "abc", "ax"]);

        let expected_removed_right = 3;
        let expected_trie_count_word_ab_right = 0;
        let expected_trie_count_prefix_a_right = 2;
        let expected_trie_count_prefix_ab_right = 1;
        let expected_trie_count_prefix_abc_right = 1;
        let expected_trie_count_prefix_ax_right = 1;

        let remove_all_word = "ab";
        let count_word_word = "ab";
        let count_prefix_prefix = "a";
        let count_prefix_prefix_2 = "ab";
        let count_prefix_prefix_3 = "abc";
        let count_prefix_prefix_4 = "ax";

        let is_prefix_prefix = "ab";

        // Act
        let actual_removed = trie.remove_all(remove_all_word);

        let actual_trie_count_word_ab = trie.count_word(count_word_word);
        let actual_trie_count_prefix_a = trie.count_prefix(count_prefix_prefix);
        let actual_trie_count_prefix_ab = trie.count_prefix(count_prefix_prefix_2);
        let actual_trie_count_prefix_abc = trie.count_prefix(count_prefix_prefix_3);
        let actual_trie_count_prefix_ax = trie.count_prefix(count_prefix_prefix_4);

        let actual_condition_1 = trie.is_prefix(is_prefix_prefix);

        // Assert
        assert_eq!(actual_removed, expected_removed_right);
        assert_eq!(actual_trie_count_word_ab, expected_trie_count_word_ab_right);
        assert!(actual_condition_1);
        assert_eq!(
            actual_trie_count_prefix_a,
            expected_trie_count_prefix_a_right
        );
        assert_eq!(
            actual_trie_count_prefix_ab,
            expected_trie_count_prefix_ab_right
        );
        assert_eq!(
            actual_trie_count_prefix_abc,
            expected_trie_count_prefix_abc_right
        );
        assert_eq!(
            actual_trie_count_prefix_ax,
            expected_trie_count_prefix_ax_right
        );
    }

    #[test]
    fn remove_all_prunes_only_target_branch_and_keeps_terminal_ancestor_and_sibling() {
        // Arrange
        let mut trie = build_trie_with_words(&["a", "ab", "ab", "ab", "ac"]);

        let expected_removed_right = 3;
        let expected_trie_count_prefix_a_right = 2;

        let remove_all_word = "ab";
        let dfs_node = None;
        let dfs_is_root = true;
        let count_prefix_prefix = "a";

        let search_key = "a";
        let search_key_2 = "ac";
        let is_prefix_prefix = "ab";
        let dfs_is_root_2 = false;

        // Act
        let actual_removed = trie.remove_all(remove_all_word);
        let a_node = trie.dfs(b'a', dfs_node, dfs_is_root).unwrap();

        let actual_trie_count_prefix_a = trie.count_prefix(count_prefix_prefix);

        let actual_condition_1 = trie.search(search_key);
        let actual_condition_2 = trie.search(search_key_2);
        let actual_condition_3 = trie.is_prefix(is_prefix_prefix);
        let actual_condition_4 = trie.dfs(b'b', Some(a_node), dfs_is_root_2).is_none();

        // Assert
        assert_eq!(actual_removed, expected_removed_right);
        assert!(actual_condition_1);
        assert!(actual_condition_2);
        assert!(!actual_condition_3);
        assert!(actual_condition_4);
        assert_eq!(
            actual_trie_count_prefix_a,
            expected_trie_count_prefix_a_right
        );
    }

    #[test]
    fn remove_all_existing_nonterminal_prefix_keeps_all_counts_unchanged() {
        // Arrange
        let mut trie = build_trie_with_words(&["abc"]);
        let counts_before = (
            trie.count_prefix("a"),
            trie.count_prefix("ab"),
            trie.count_prefix("abc"),
            trie.count_word("abc"),
        );

        let expected_removed_right = 0;

        let remove_all_word = "ab";
        let count_prefix_prefix = "a";
        let count_prefix_prefix_2 = "ab";
        let count_prefix_prefix_3 = "abc";
        let count_word_word = "abc";

        // Act
        let actual_removed = trie.remove_all(remove_all_word);

        let actual_trie_count_prefix_a_trie_count_prefix_ab_trie_count_pre = (
            trie.count_prefix(count_prefix_prefix),
            trie.count_prefix(count_prefix_prefix_2),
            trie.count_prefix(count_prefix_prefix_3),
            trie.count_word(count_word_word),
        );

        // Assert
        assert_eq!(actual_removed, expected_removed_right);
        assert_eq!(
            actual_trie_count_prefix_a_trie_count_prefix_ab_trie_count_pre,
            counts_before
        );
    }

    #[test]
    fn remove_all_empty_words_keeps_nonempty_word_state_unchanged() {
        // Arrange
        let mut trie = build_trie_with_words(&["", "", "a"]);

        let expected_removed_right = 2;
        let expected_trie_count_prefix_a_right = 1;

        let remove_all_word = "";
        let count_prefix_prefix = "a";

        let search_key = "";
        let search_key_2 = "a";

        // Act
        let actual_removed = trie.remove_all(remove_all_word);

        let actual_trie_count_prefix_a = trie.count_prefix(count_prefix_prefix);

        let actual_condition_1 = trie.search(search_key);
        let actual_condition_2 = trie.search(search_key_2);

        // Assert
        assert_eq!(actual_removed, expected_removed_right);
        assert!(!actual_condition_1);
        assert!(actual_condition_2);
        assert_eq!(
            actual_trie_count_prefix_a,
            expected_trie_count_prefix_a_right
        );
    }

    #[test]
    fn mixed_mutations_preserve_all_descendant_counter_invariants() {
        // Arrange
        let mut trie =
            build_trie_with_words(&["", "a", "ab", "ab", "abc", "ax", "xyz", "あ", "あい"]);

        let remove_key = "ab";
        let remove_all_word = "ax";
        let remove_key_2 = "";
        let insert_value = "aby".to_string();
        let remove_all_word_2 = "あ";

        let expected_removed_count = 1;

        // Act
        let actual_removed = trie.remove(remove_key);
        let actual_removed_all = trie.remove_all(remove_all_word);
        let actual_removed_empty = trie.remove(remove_key_2);
        trie.insert(insert_value);
        let actual_removed_unicode = trie.remove_all(remove_all_word_2);

        // Assert
        assert!(actual_removed);
        assert!(actual_removed_empty);
        assert_eq!(actual_removed_all, expected_removed_count);
        assert_eq!(actual_removed_unicode, expected_removed_count);
        assert_descendant_node_invariants(&trie.root);
    }

    #[test]
    fn prefix_and_word_checks_for_empty_string_in_empty_trie() {
        // Arrange
        let trie = Trie::new();
        let empty_word = "";

        // Act
        let actual_is_prefix = trie.is_prefix(empty_word);
        let actual_is_word = trie.is_word(empty_word);

        // Assert
        assert!(actual_is_prefix);
        assert!(!actual_is_word);
    }

    #[test]
    fn is_prefix_returns_true_for_exact_registered_word() {
        // Arrange
        let trie = build_trie_with_words(&["abc"]);
        let target_prefix = "abc";

        // Act
        let actual = trie.is_prefix(target_prefix);

        // Assert
        assert!(actual);
    }

    #[test]
    fn is_prefix_returns_true_for_intermediate_prefix() {
        // Arrange
        let trie = build_trie_with_words(&["abc"]);
        let target_prefix = "ab";

        // Act
        let actual = trie.is_prefix(target_prefix);

        // Assert
        assert!(actual);
    }

    #[test]
    fn is_prefix_returns_false_for_missing_prefix() {
        // Arrange
        let trie = build_trie_with_words(&["abc"]);
        let target_prefix = "ax";

        // Act
        let actual = trie.is_prefix(target_prefix);

        // Assert
        assert!(!actual);
    }

    #[test]
    fn shared_prefix_words_produce_stepwise_prefix_counts() {
        // Arrange
        let trie = build_trie_with_words(&["a", "ab", "abc"]);
        let prefix_a = "a";
        let prefix_ab = "ab";
        let prefix_abc = "abc";

        let expected_a_right = 3;
        let expected_ab_right = 2;
        let expected_abc_right = 1;

        // Act
        let actual_a = trie.count_prefix(prefix_a);
        let actual_ab = trie.count_prefix(prefix_ab);
        let actual_abc = trie.count_prefix(prefix_abc);

        // Assert
        assert_eq!(actual_a, expected_a_right);
        assert_eq!(actual_ab, expected_ab_right);
        assert_eq!(actual_abc, expected_abc_right);
    }

    #[test]
    fn disjoint_prefix_words_do_not_interfere_with_each_other() {
        // Arrange
        let trie = build_trie_with_words(&["ab", "xy"]);
        let word_left = "ab";
        let word_right = "xy";

        let expected_left_count_right = 1;
        let expected_right_count_right = 1;
        let expected_left_prefix_right = 1;
        let expected_right_prefix_right = 1;

        let count_prefix_prefix = "a";
        let count_prefix_prefix_2 = "x";

        // Act
        let actual_left_count = trie.count_word(word_left);
        let actual_right_count = trie.count_word(word_right);
        let actual_left_prefix = trie.count_prefix(count_prefix_prefix);
        let actual_right_prefix = trie.count_prefix(count_prefix_prefix_2);

        // Assert
        assert_eq!(actual_left_count, expected_left_count_right);
        assert_eq!(actual_right_count, expected_right_count_right);
        assert_eq!(actual_left_prefix, expected_left_prefix_right);
        assert_eq!(actual_right_prefix, expected_right_prefix_right);
    }

    #[test]
    fn count_prefix_returns_number_of_words_with_target_prefix() {
        // Arrange
        let trie = build_trie_with_words(&["a", "ab", "abc", "ab", "ax"]);
        let target_prefix = "a";

        let expected_value_right = 5;

        // Act
        let actual = trie.count_prefix(target_prefix);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn count_prefix_returns_zero_for_missing_prefix() {
        // Arrange
        let trie = build_trie_with_words(&["ab", "abc"]);
        let target_prefix = "x";

        let expected_value_right = 0;

        // Act
        let actual = trie.count_prefix(target_prefix);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn count_prefix_for_empty_string_returns_zero_in_current_implementation() {
        // Arrange
        let trie = build_trie_with_words(&["a", "ab", ""]);
        let empty_prefix = "";

        let expected_value_right = 0;

        // Act
        let actual = trie.count_prefix(empty_prefix);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn count_methods_return_zero_in_empty_trie() {
        // Arrange
        let trie = Trie::new();
        let target_word = "a";

        let expected_word_count_right = 0;
        let expected_prefix_count_right = 0;
        let expected_prefix_all_count_right = 0;

        // Act
        let actual_word_count = trie.count_word(target_word);
        let actual_prefix_count = trie.count_prefix(target_word);
        let actual_prefix_all_count = trie.count_prefix_all(target_word);

        // Assert
        assert_eq!(actual_word_count, expected_word_count_right);
        assert_eq!(actual_prefix_count, expected_prefix_count_right);
        assert_eq!(actual_prefix_all_count, expected_prefix_all_count_right);
    }

    #[test]
    fn count_prefix_all_returns_cumulative_counts_on_full_match_path() {
        // Arrange
        let trie = build_trie_with_words(&["a", "ab", "abc", "ab"]);
        let target_word = "abc";

        let expected_value_right = 8;

        // Act
        let actual = trie.count_prefix_all(target_word);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn count_prefix_all_returns_matched_prefix_sum_on_partial_mismatch() {
        // Arrange
        let trie = build_trie_with_words(&["a", "ab", "abc", "ab"]);
        let target_word = "abx";

        let expected_value_right = 7;

        // Act
        let actual = trie.count_prefix_all(target_word);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn count_prefix_all_returns_zero_when_first_character_mismatches() {
        // Arrange
        let trie = build_trie_with_words(&["ab", "abc"]);
        let target_word = "x";

        let expected_value_right = 0;

        // Act
        let actual = trie.count_prefix_all(target_word);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn count_prefix_all_for_empty_word_returns_zero() {
        // Arrange
        let trie = build_trie_with_words(&["ab", "abc"]);
        let target_word = "";

        let expected_value_right = 0;

        // Act
        let actual = trie.count_prefix_all(target_word);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn count_prefix_all_counts_each_utf8_byte_node() {
        // Arrange
        let trie = build_trie_with_words(&["あ", "あい"]);

        let expected_value_right = 9;

        let count_prefix_all_prefix = "あい";

        // Act
        let actual = trie.count_prefix_all(count_prefix_all_prefix);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn count_word_counts_only_exact_match_not_prefix_of_longer_word() {
        // Arrange
        let trie = build_trie_with_words(&["ab", "ab", "abc"]);
        let exact_word = "ab";
        let prefix_only_word = "a";

        let expected_exact_right = 2;
        let expected_prefix_only_right = 0;

        // Act
        let actual_exact = trie.count_word(exact_word);
        let actual_prefix_only = trie.count_word(prefix_only_word);

        // Assert
        assert_eq!(actual_exact, expected_exact_right);
        assert_eq!(actual_prefix_only, expected_prefix_only_right);
    }

    #[test]
    fn dfs_with_none_and_non_root_flag_returns_none() {
        // Arrange
        let trie = build_trie_with_words(&["ab"]);
        let target_byte = b'a';
        let node = None;
        let is_root = false;

        // Act
        let actual = trie.dfs(target_byte, node, is_root);

        // Assert
        assert!(actual.is_none());
    }

    #[test]
    fn dfs_with_none_and_root_flag_traverses_from_root() {
        // Arrange
        let trie = build_trie_with_words(&["ab"]);
        let target_byte = b'a';
        let node = None;
        let is_root = true;

        // Act
        let actual = trie.dfs(target_byte, node, is_root);

        // Assert
        assert!(actual.is_some());
    }

    #[test]
    fn dfs_with_given_node_traverses_from_that_node() {
        // Arrange
        let trie = build_trie_with_words(&["ab"]);
        let root_byte = b'a';
        let child_byte = b'b';
        let is_root = false;
        let parent_node = trie.dfs(root_byte, None, true);

        // Act
        let actual = trie.dfs(child_byte, parent_node, is_root);

        // Assert
        assert!(actual.is_some());
    }

    #[test]
    fn dfs_nodes_expose_counters_for_duplicates_and_shared_prefixes() {
        // Arrange
        let trie = build_trie_with_words(&["a", "ab", "ab"]);

        let expected_a_node_counter_a_node_end_counter_right = (3, 1);
        let expected_b_node_counter_b_node_end_counter_right = (2, 2);

        let dfs_node = None;
        let dfs_is_root = true;
        let dfs_is_root_2 = false;

        // Act
        let a_node = trie.dfs(b'a', dfs_node, dfs_is_root).unwrap();
        let b_node = trie.dfs(b'b', Some(a_node), dfs_is_root_2).unwrap();

        let actual_a_node_counter_a_node_end_counter = (a_node.counter, a_node.end_counter);
        let actual_b_node_counter_b_node_end_counter = (b_node.counter, b_node.end_counter);

        // Assert
        assert_eq!(
            actual_a_node_counter_a_node_end_counter,
            expected_a_node_counter_a_node_end_counter_right
        );
        assert_eq!(
            actual_b_node_counter_b_node_end_counter,
            expected_b_node_counter_b_node_end_counter_right
        );
    }

    #[test]
    fn dfs_from_root_returns_none_for_missing_child() {
        // Arrange
        let trie = build_trie_with_words(&["ab"]);

        let dfs_node = None;
        let dfs_is_root = true;

        // Act
        let actual = trie.dfs(b'x', dfs_node, dfs_is_root);

        // Assert
        assert!(actual.is_none());
    }

    #[test]
    fn dfs_from_given_node_returns_none_for_missing_child() {
        // Arrange
        let trie = build_trie_with_words(&["ab"]);
        let a_node = trie.dfs(b'a', None, true).unwrap();

        let dfs_is_root = false;

        // Act
        let actual = trie.dfs(b'x', Some(a_node), dfs_is_root);

        // Assert
        assert!(actual.is_none());
    }

    #[test]
    fn dfs_with_given_node_and_root_flag_does_not_fall_back_to_root() {
        // Arrange
        let trie = build_trie_with_words(&["ab", "x"]);
        let a_node = trie.dfs(b'a', None, true).unwrap();

        let dfs_is_root = true;

        // Act
        let actual = trie.dfs(b'x', Some(a_node), dfs_is_root);

        // Assert
        assert!(actual.is_none());
    }

    #[test]
    fn lcp_returns_zero_when_first_character_mismatches() {
        // Arrange
        let trie = build_trie_with_words(&["abc"]);
        let target_word = "x";

        let expected_value_right = 0;

        // Act
        let actual = trie.lcp(target_word);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn lcp_returns_partial_and_full_lengths_correctly() {
        // Arrange
        let trie = build_trie_with_words(&["abc"]);
        let partial_word = "abx";
        let full_word = "abc";

        let expected_partial_right = 2;
        let expected_full_right = 3;

        // Act
        let actual_partial = trie.lcp(partial_word);
        let actual_full = trie.lcp(full_word);

        // Assert
        assert_eq!(actual_partial, expected_partial_right);
        assert_eq!(actual_full, expected_full_right);
    }

    #[test]
    fn lcp_for_empty_word_returns_zero() {
        // Arrange
        let trie = build_trie_with_words(&["abc"]);
        let target_word = "";

        let expected_value_right = 0;

        // Act
        let actual = trie.lcp(target_word);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn lcp_for_multibyte_words_follows_byte_based_behavior() {
        // Arrange
        let mut trie = Trie::new();
        let insert_word = "あい";
        let target_word = "あう";
        let expected = bytes_lcp_length(insert_word, target_word);

        trie.insert(insert_word.to_string());

        // Act
        let actual = trie.lcp(target_word);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn lcp_returns_query_length_when_query_is_shorter_than_registered_word() {
        // Arrange
        let trie = build_trie_with_words(&["abc"]);

        let expected_value_right = 2;

        let lcp_query = "ab";

        // Act
        let actual = trie.lcp(lcp_query);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn lcp_returns_registered_path_length_when_query_is_longer_than_registered_word() {
        // Arrange
        let trie = build_trie_with_words(&["abc"]);

        let expected_value_right = 3;

        let lcp_query = "abcd";

        // Act
        let actual = trie.lcp(lcp_query);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn substring_indices_returns_all_terminal_positions_for_prefix_matches() {
        // Arrange
        let trie = build_trie_with_words(&["a", "ab", "abc"]);
        let target_word = "abcx";

        let expected_value_right = vec![0, 1, 2];

        // Act
        let actual = trie.substring_indices(target_word);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn substring_indices_stops_at_mismatch_and_returns_previous_terminal_positions() {
        // Arrange
        let trie = build_trie_with_words(&["a", "ab", "abc"]);
        let target_word = "abx";

        let expected_value_right = vec![0, 1];

        // Act
        let actual = trie.substring_indices(target_word);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn substring_indices_returns_empty_when_first_character_mismatches() {
        // Arrange
        let trie = build_trie_with_words(&["ab"]);
        let target_word = "xab";

        // Act
        let actual = trie.substring_indices(target_word);

        // Assert
        assert!(actual.is_empty());
    }

    #[test]
    fn substring_indices_does_not_duplicate_positions_for_duplicated_word() {
        // Arrange
        let trie = build_trie_with_words(&["ab", "ab"]);
        let target_word = "ab";

        let expected_value_right = vec![1];

        // Act
        let actual = trie.substring_indices(target_word);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn substring_indices_for_empty_word_returns_empty_vector() {
        // Arrange
        let trie = build_trie_with_words(&["ab"]);
        let target_word = "";

        // Act
        let actual = trie.substring_indices(target_word);

        // Assert
        assert!(actual.is_empty());
    }

    #[test]
    fn substring_indices_returns_empty_for_existing_nonterminal_path() {
        // Arrange
        let trie = build_trie_with_words(&["abc"]);

        let substring_indices_word = "ab";

        // Act
        let actual = trie.substring_indices(substring_indices_word);

        // Assert
        assert!(actual.is_empty());
    }

    #[test]
    fn substring_indices_returns_utf8_terminal_byte_indices() {
        // Arrange
        let trie = build_trie_with_words(&["あ", "あい"]);

        let expected_value_right = vec![2, 5];

        let substring_indices_word = "あいう";

        // Act
        let actual = trie.substring_indices(substring_indices_word);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn substring_indices_ignores_registered_empty_word_for_empty_input() {
        // Arrange
        let trie = build_trie_with_words(&[""]);

        let substring_indices_word = "";

        // Act
        let actual = trie.substring_indices(substring_indices_word);

        // Assert
        assert!(actual.is_empty());
    }

    #[test]
    fn substring_indices_removes_only_deleted_terminal() {
        // Arrange
        let mut trie = build_trie_with_words(&["a", "ab"]);

        let expected_indices_right = vec![1];

        let remove_key = "a";
        let substring_indices_word = "ab";

        // Act
        let actual_removed = trie.remove(remove_key);
        let actual_indices = trie.substring_indices(substring_indices_word);

        // Assert
        assert!(actual_removed);
        assert_eq!(actual_indices, expected_indices_right);
    }

    #[test]
    fn substring_indices_tracks_duplicate_terminal_until_last_copy_is_removed() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab", "ab"]);

        let expected_after_one_remove_right = vec![1];
        let expected_remaining_removed_right = 1;

        let remove_key = "ab";
        let substring_indices_word = "ab";
        let remove_all_word = "ab";

        // Act
        let actual_first_removed = trie.remove(remove_key);
        let actual_after_one_remove = trie.substring_indices(substring_indices_word);
        let actual_remaining_removed = trie.remove_all(remove_all_word);
        let actual_after_all_removed = trie.substring_indices(substring_indices_word);

        // Assert
        assert!(actual_first_removed);
        assert_eq!(actual_after_one_remove, expected_after_one_remove_right);
        assert_eq!(actual_remaining_removed, expected_remaining_removed_right);
        assert!(actual_after_all_removed.is_empty());
    }

    #[test]
    fn smallest_word_returns_none_for_missing_prefix() {
        // Arrange
        let trie = build_trie_with_words(&["abc"]);
        let prefix = "x";

        let expected_value_right = None;

        // Act
        let actual = trie.smallest_word(prefix);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn smallest_word_returns_prefix_itself_when_prefix_is_registered_word() {
        // Arrange
        let trie = build_trie_with_words(&["ab", "abc"]);
        let prefix = "ab";

        let expected_value_right = Some("ab".to_string());

        // Act
        let actual = trie.smallest_word(prefix);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn smallest_word_returns_lexicographically_smallest_completion() {
        // Arrange
        let trie = build_trie_with_words(&["ac", "ab", "az"]);
        let prefix = "a";

        let expected_value_right = Some("ab".to_string());

        // Act
        let actual = trie.smallest_word(prefix);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn smallest_word_for_empty_prefix_returns_global_smallest_word() {
        // Arrange
        let trie = build_trie_with_words(&["dog", "cat", "apple"]);
        let prefix = "";

        let expected_value_right = Some("apple".to_string());

        // Act
        let actual = trie.smallest_word(prefix);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn smallest_word_on_empty_trie_and_empty_prefix_returns_none_without_panic() {
        // Arrange
        let trie = Trie::new();
        let prefix = "";

        let expected_value_right = None;

        // Act
        let actual = trie.smallest_word(prefix);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn smallest_word_returns_registered_empty_word_for_empty_prefix() {
        // Arrange
        let trie = build_trie_with_words(&["", "a"]);

        let expected_value_right = Some(String::new());

        let smallest_word_prefix = "";

        // Act
        let actual = trie.smallest_word(smallest_word_prefix);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn smallest_word_returns_valid_utf8_completion() {
        // Arrange
        let trie = build_trie_with_words(&["あい", "あう"]);

        let expected_value_right = Some("あい".to_string());

        let smallest_word_prefix = "あ";

        // Act
        let actual = trie.smallest_word(smallest_word_prefix);

        // Assert
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn smallest_word_moves_to_next_word_after_current_minimum_is_removed() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab", "ac"]);

        let expected_value_right = Some("ac".to_string());

        let remove_key = "ab";
        let smallest_word_prefix = "a";

        // Act
        let actual_removed = trie.remove(remove_key);
        let actual = trie.smallest_word(smallest_word_prefix);

        // Assert
        assert!(actual_removed);
        assert_eq!(actual, expected_value_right);
    }

    #[test]
    fn smallest_word_keeps_duplicate_minimum_until_all_copies_are_removed() {
        // Arrange
        let mut trie = build_trie_with_words(&["ab", "ab", "ac"]);

        let expected_after_one_remove_right = Some("ab".to_string());
        let expected_remaining_removed_right = 1;
        let expected_after_all_removed_right = Some("ac".to_string());

        let remove_key = "ab";
        let smallest_word_prefix = "a";
        let remove_all_word = "ab";

        // Act
        let actual_first_removed = trie.remove(remove_key);
        let actual_after_one_remove = trie.smallest_word(smallest_word_prefix);
        let actual_remaining_removed = trie.remove_all(remove_all_word);
        let actual_after_all_removed = trie.smallest_word(smallest_word_prefix);

        // Assert
        assert!(actual_first_removed);
        assert_eq!(actual_after_one_remove, expected_after_one_remove_right);
        assert_eq!(actual_remaining_removed, expected_remaining_removed_right);
        assert_eq!(actual_after_all_removed, expected_after_all_removed_right);
    }

    #[test]
    fn deterministic_operations_match_reference_for_all_query_apis() {
        #[derive(Clone, Copy, Debug)]
        enum Operation<'a> {
            Insert(&'a str),
            Remove(&'a str),
            RemoveAll(&'a str),
        }

        // Arrange
        let mut trie = Trie::new();
        let mut reference_word_counts = StdHashMap::<String, usize>::new();
        let operations = [
            Operation::Insert(""),
            Operation::Insert("a"),
            Operation::Insert("ab"),
            Operation::Insert("ab"),
            Operation::Insert("abc"),
            Operation::Insert("ax"),
            Operation::Insert("あ"),
            Operation::Insert("あい"),
            Operation::Insert("あい"),
            Operation::Insert("あう"),
            Operation::Remove("ab"),
            Operation::RemoveAll("あい"),
            Operation::Remove("未登録"),
            Operation::Insert("い"),
            Operation::Remove(""),
            Operation::RemoveAll("a"),
            Operation::RemoveAll("ab"),
            Operation::RemoveAll("あ"),
            Operation::RemoveAll("あう"),
            Operation::RemoveAll("い"),
        ];
        let words_to_check = [
            "",
            "a",
            "ab",
            "abc",
            "ax",
            "az",
            "b",
            "あ",
            "あい",
            "あう",
            "あえ",
            "い",
            "未登録",
        ];

        let expected_initial_state = reference_queries(&reference_word_counts, &words_to_check);

        // Act
        let actual_initial_state = actual_queries(&trie, &words_to_check);

        // Assert
        assert_eq!(actual_initial_state, expected_initial_state);

        for (stage, operation) in operations.iter().copied().enumerate() {
            // Arrange
            let expected_operation = match operation {
                Operation::Insert(word) => {
                    insert_word_to_reference_counts(&mut reference_word_counts, word);
                    None
                }
                Operation::Remove(word) => Some(usize::from(remove_word_from_reference_counts(
                    &mut reference_word_counts,
                    word,
                ))),
                Operation::RemoveAll(word) => Some(remove_all_words_from_reference_counts(
                    &mut reference_word_counts,
                    word,
                )),
            };
            let expected_state = reference_queries(&reference_word_counts, &words_to_check);

            // Act
            let actual_operation = match operation {
                Operation::Insert(word) => {
                    trie.insert(word.to_string());
                    None
                }
                Operation::Remove(word) => Some(usize::from(trie.remove(word))),
                Operation::RemoveAll(word) => Some(trie.remove_all(word)),
            };
            let actual_state = actual_queries(&trie, &words_to_check);

            // Assert
            assert_eq!(
                actual_operation, expected_operation,
                "stage {stage}, operation {operation:?}"
            );
            assert_eq!(
                actual_state, expected_state,
                "stage {stage}, operation {operation:?}"
            );
        }
    }

    #[test]
    fn mixed_insert_remove_and_remove_all_match_reference_hash_map() {
        // Arrange
        let mut trie = Trie::new();
        let words_to_check = ["", "a", "ab", "aba", "xyz", "zzz"];
        let inserted_words = ["a", "ab", "ab", "xyz"];
        let removed_word = "ab";
        let removed_all_word = "a";
        let empty_word = "";
        let next_word = "aba";
        let missing_word = "zzz";
        let mut reference = StdHashMap::<String, usize>::new();
        for word in inserted_words {
            insert_word_to_reference_counts(&mut reference, word);
        }
        let expected_removed = remove_word_from_reference_counts(&mut reference, removed_word);
        let expected_removed_all =
            remove_all_words_from_reference_counts(&mut reference, removed_all_word);
        insert_word_to_reference_counts(&mut reference, empty_word);
        let expected_removed_empty = remove_word_from_reference_counts(&mut reference, empty_word);
        insert_word_to_reference_counts(&mut reference, next_word);
        let expected_removed_missing =
            remove_all_words_from_reference_counts(&mut reference, missing_word);
        let expected_state = reference_queries(&reference, &words_to_check);

        // Act
        for word in inserted_words {
            trie.insert(word.to_string());
        }
        let actual_removed = trie.remove(removed_word);
        let actual_removed_all = trie.remove_all(removed_all_word);
        trie.insert(empty_word.to_string());
        let actual_removed_empty = trie.remove(empty_word);
        trie.insert(next_word.to_string());
        let actual_removed_missing = trie.remove_all(missing_word);
        let actual_state = actual_queries(&trie, &words_to_check);

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_removed_all, expected_removed_all);
        assert_eq!(actual_removed_empty, expected_removed_empty);
        assert_eq!(actual_removed_missing, expected_removed_missing);
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn reinserting_word_after_full_pruning_rebuilds_path_and_counts() {
        // Arrange
        let mut trie = build_trie_with_words(&["abc"]);
        assert!(trie.remove("abc"));
        assert!(trie.dfs(b'a', None, true).is_none());

        let expected_trie_count_word_abc_right = 1;
        let expected_trie_count_prefix_a_right = 1;
        let expected_trie_count_prefix_ab_right = 1;
        let expected_trie_count_prefix_abc_right = 1;

        let insert_value = "abc".to_string();
        let count_word_word = "abc";
        let count_prefix_prefix = "a";
        let count_prefix_prefix_2 = "ab";
        let count_prefix_prefix_3 = "abc";

        let search_key = "abc";

        // Act
        trie.insert(insert_value);

        let actual_trie_count_word_abc = trie.count_word(count_word_word);
        let actual_trie_count_prefix_a = trie.count_prefix(count_prefix_prefix);
        let actual_trie_count_prefix_ab = trie.count_prefix(count_prefix_prefix_2);
        let actual_trie_count_prefix_abc = trie.count_prefix(count_prefix_prefix_3);

        let actual_condition_1 = trie.search(search_key);

        // Assert
        assert!(actual_condition_1);
        assert_eq!(
            actual_trie_count_word_abc,
            expected_trie_count_word_abc_right
        );
        assert_eq!(
            actual_trie_count_prefix_a,
            expected_trie_count_prefix_a_right
        );
        assert_eq!(
            actual_trie_count_prefix_ab,
            expected_trie_count_prefix_ab_right
        );
        assert_eq!(
            actual_trie_count_prefix_abc,
            expected_trie_count_prefix_abc_right
        );
    }

    #[test]
    fn non_ascii_duplicate_word_counts_remain_consistent_after_remove() {
        // Arrange
        let mut trie = build_trie_with_words(&["あい", "あい", "あう"]);

        let expected_before_right = (true, true, 2, 3);
        let expected_after_right = (true, true, 1, 2);

        let search_key = "あい";
        let is_word_word = "あい";
        let count_word_word = "あい";
        let count_prefix_prefix = "あ";
        let remove_key = "あい";

        // Act
        let actual_before = (
            trie.search(search_key),
            trie.is_word(is_word_word),
            trie.count_word(count_word_word),
            trie.count_prefix(count_prefix_prefix),
        );
        let actual_removed = trie.remove(remove_key);
        let actual_after = (
            trie.search(search_key),
            trie.is_word(is_word_word),
            trie.count_word(count_word_word),
            trie.count_prefix(count_prefix_prefix),
        );

        // Assert
        assert_eq!(actual_before, expected_before_right);
        assert!(actual_removed);
        assert_eq!(actual_after, expected_after_right);
    }

    #[test]
    fn node_default_returns_empty_node_with_zero_counters() {
        // Arrange
        let expected_state = (true, 0usize, 0usize);

        // Act
        let node = Node::default();
        let actual_state = (node.children.is_empty(), node.counter, node.end_counter);

        // Assert
        assert_eq!(actual_state, expected_state);
    }
}

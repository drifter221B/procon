use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

/// A fixed-capacity cache that evicts the least recently used entry.
///
/// Successful [`get`](Self::get), [`get_mut`](Self::get_mut), and
/// [`put`](Self::put) operations mark the entry as most recently used. Peeking,
/// checking membership, and unsuccessful lookups leave the order unchanged.
/// A zero-capacity cache never stores entries.
///
/// Keys require `Eq + Hash` for lookups and removals, and additionally `Clone`
/// for insertion. Values have no trait requirements. Keys are stored both in a
/// hash map and in an indexed linked list, so inserting a new key clones it once.
///
/// Operations take expected amortized O(1) time, excluding the cost of hashing,
/// comparing, cloning, and dropping keys or values. [`clear`](Self::clear) takes
/// O(h) time, where h is the highest number of slots allocated since construction
/// or the previous clear, and h never exceeds the capacity. Storage is allocated
/// as needed and uses O(capacity) space.
///
/// # Examples
///
/// ```
/// use lru_cache::LruCache;
///
/// let mut cache = LruCache::new(2);
/// cache.put(String::from("first"), 10);
/// cache.put(String::from("second"), 20);
/// assert_eq!(cache.get("first"), Some(&10));
///
/// cache.put(String::from("third"), 30);
/// assert_eq!(cache.peek("second"), None);
/// assert_eq!(cache.pop_lru(), Some((String::from("first"), 10)));
/// ```
pub struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, usize>,
    nodes: Vec<Option<Node<K, V>>>,
    free: Vec<usize>,
    most_recent: Option<usize>,
    least_recent: Option<usize>,
}

struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

impl<K, V> LruCache<K, V> {
    /// Creates an empty cache without allocating entry storage.
    ///
    /// A capacity of zero is allowed; [`put`](Self::put) then discards every
    /// supplied key and value and returns `None`.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::new(),
            nodes: Vec::new(),
            free: Vec::new(),
            most_recent: None,
            least_recent: None,
        }
    }

    /// Returns the maximum number of entries the cache can store.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the number of stored entries.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns whether the cache contains no entries.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Removes all entries and releases their storage, keeping the capacity.
    ///
    /// Takes O(h) time in the highest number of slots allocated since the last
    /// clear or construction, with h at most the capacity.
    pub fn clear(&mut self) {
        let empty = Self::new(self.capacity);
        // Reset the cache before running user-defined key or value destructors.
        let old = std::mem::replace(self, empty);
        drop(old);
    }

    fn node(&self, index: usize) -> &Node<K, V> {
        self.nodes[index]
            .as_ref()
            .expect("a linked index points to an occupied slot")
    }

    fn node_mut(&mut self, index: usize) -> &mut Node<K, V> {
        self.nodes[index]
            .as_mut()
            .expect("a linked index points to an occupied slot")
    }

    fn detach(&mut self, index: usize) {
        let node = self.node(index);
        let (prev, next) = (node.prev, node.next);

        if let Some(prev) = prev {
            self.node_mut(prev).next = next;
        } else {
            self.most_recent = next;
        }
        if let Some(next) = next {
            self.node_mut(next).prev = prev;
        } else {
            self.least_recent = prev;
        }
    }

    fn attach_most_recent(&mut self, index: usize) {
        let most_recent = self.most_recent;
        let node = self.node_mut(index);
        node.prev = None;
        node.next = most_recent;

        if let Some(most_recent) = most_recent {
            self.node_mut(most_recent).prev = Some(index);
        } else {
            self.least_recent = Some(index);
        }
        self.most_recent = Some(index);
    }

    fn promote(&mut self, index: usize) {
        if self.most_recent != Some(index) {
            self.detach(index);
            self.attach_most_recent(index);
        }
    }

    fn take_node(&mut self, index: usize) -> Node<K, V> {
        self.detach(index);
        let node = self.nodes[index]
            .take()
            .expect("a linked index points to an occupied slot");
        self.free.push(index);
        node
    }
}

impl<K: Eq + Hash, V> LruCache<K, V> {
    /// Returns the value for `key`, marking a found entry as most recently used.
    ///
    /// Returns `None` without changing the order if the key is absent. A borrowed
    /// key may be used, such as `&str` for a cache with `String` keys.
    pub fn get<Q>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let index = *self.map.get(key)?;
        self.promote(index);
        Some(&self.node(index).value)
    }

    /// Returns a mutable value, marking a found entry as most recently used.
    ///
    /// Returns `None` without changing the order if the key is absent.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let index = *self.map.get(key)?;
        self.promote(index);
        Some(&mut self.node_mut(index).value)
    }

    /// Returns the value for `key` without changing the recency order.
    pub fn peek<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let index = *self.map.get(key)?;
        Some(&self.node(index).value)
    }

    /// Checks whether `key` is stored without changing the recency order.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.map.contains_key(key)
    }

    /// Removes `key` and returns its value, or `None` if the key is absent.
    ///
    /// The relative recency order of the remaining entries is unchanged.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let (map_key, index) = self.map.remove_entry(key)?;
        let node = self.take_node(index);
        // User-defined destructors run only after all indices are consistent.
        drop(map_key);
        Some(node.value)
    }

    /// Removes and returns the least recently used key and value.
    ///
    /// Returns `None` if the cache is empty.
    pub fn pop_lru(&mut self) -> Option<(K, V)> {
        let index = self.least_recent?;
        let (map_key, _) = self
            .map
            .remove_entry(
                &self.nodes[index]
                    .as_ref()
                    .expect("a linked index points to an occupied slot")
                    .key,
            )
            .expect("an occupied slot has a hash map entry");
        let node = self.take_node(index);
        drop(map_key);
        Some((node.key, node.value))
    }
}

impl<K: Eq + Hash + Clone, V> LruCache<K, V> {
    /// Inserts or updates an entry and marks it as most recently used.
    ///
    /// Updating an existing entry preserves its stored key and returns its old
    /// value. Inserting a new key returns `None` and first evicts the least
    /// recently used entry if the cache is full. With capacity zero, the supplied
    /// key and value are discarded and `None` is returned.
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        if self.capacity == 0 {
            return None;
        }

        if let Some(&index) = self.map.get(&key) {
            let old_value = std::mem::replace(&mut self.node_mut(index).value, value);
            self.promote(index);
            return Some(old_value);
        }

        // Clone before eviction so a failed clone does not remove an entry.
        let map_key = key.clone();
        if self.len() == self.capacity {
            drop(self.pop_lru());
        }

        let index = self.free.last().copied().unwrap_or(self.nodes.len());
        // Hashing and equality checks finish before changing slots or links.
        self.map.insert(map_key, index);
        let node = Node {
            key,
            value,
            prev: None,
            next: None,
        };
        if self.free.pop().is_some() {
            self.nodes[index] = Some(node);
        } else {
            self.nodes.push(Some(node));
        }
        self.attach_most_recent(index);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::LruCache;
    use std::cell::RefCell;
    use std::hash::{Hash, Hasher};
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::rc::Rc;

    fn drain<K: Eq + Hash + Clone, V>(cache: &mut LruCache<K, V>) -> Vec<(K, V)> {
        let mut entries = Vec::new();
        while let Some(entry) = cache.pop_lru() {
            entries.push(entry);
        }
        entries
    }

    #[test]
    fn zero_capacity_never_stores_entries() {
        // Arrange
        let capacity = 0;
        let key = 1;
        let value = 10;
        let expected_value = None;
        let expected_entry = None;
        let expected_state = (capacity, 0, true);
        let mut cache = LruCache::new(capacity);

        // Act
        let inserted = cache.put(key, value);
        let found = cache.get(&key).copied();
        let mutable = cache.get_mut(&key).copied();
        let peeked = cache.peek(&key).copied();
        let contains = cache.contains_key(&key);
        let removed = cache.remove(&key);
        let popped = cache.pop_lru();
        let state = (cache.capacity(), cache.len(), cache.is_empty());

        // Assert
        assert_eq!(inserted, expected_value);
        assert_eq!(found, expected_value);
        assert_eq!(mutable, expected_value);
        assert_eq!(peeked, expected_value);
        assert!(!contains);
        assert_eq!(removed, expected_value);
        assert_eq!(popped, expected_entry);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn capacity_one_updates_then_evicts_its_only_entry() {
        // Arrange
        let capacity = 1;
        let key = 1;
        let next_key = 2;
        let value = 10;
        let replacement = 11;
        let next_value = 20;
        let expected_old = Some(value);
        let expected_new = None;
        let expected_entries = vec![(next_key, next_value)];
        let mut cache = LruCache::new(capacity);
        let _ = cache.put(key, value);

        // Act
        let old = cache.put(key, replacement);
        let inserted = cache.put(next_key, next_value);
        let len = cache.len();
        let entries = drain(&mut cache);

        // Assert
        assert_eq!(old, expected_old);
        assert_eq!(inserted, expected_new);
        assert_eq!(len, capacity);
        assert_eq!(entries, expected_entries);
    }

    #[test]
    fn get_promotes_an_entry_and_changes_the_eviction_target() {
        // Arrange
        let capacity = 3;
        let initial = [(1, 10), (2, 20), (3, 30)];
        let accessed_key = initial[0].0;
        let next = (4, 40);
        let expected_found = Some(initial[0].1);
        let expected_entries = vec![initial[2], initial[0], next];
        let mut cache = LruCache::new(capacity);
        for (key, value) in initial {
            let _ = cache.put(key, value);
        }

        // Act
        let found = cache.get(&accessed_key).copied();
        let found_again = cache.get(&accessed_key).copied();
        let inserted = cache.put(next.0, next.1);
        let entries = drain(&mut cache);

        // Assert
        assert_eq!(found, expected_found);
        assert_eq!(found_again, expected_found);
        assert!(inserted.is_none());
        assert_eq!(entries, expected_entries);
    }

    #[test]
    fn get_mut_promotes_entries_even_without_changing_the_value() {
        // Arrange
        let capacity = 3;
        let initial = [(1, 10), (2, 20), (3, 30)];
        let changed_key = initial[0].0;
        let unchanged_key = initial[1].0;
        let increment = 5;
        let expected_changed = Some(initial[0].1 + increment);
        let expected_unchanged = Some(initial[1].1);
        let expected_entries = vec![
            initial[2],
            (changed_key, initial[0].1 + increment),
            initial[1],
        ];
        let mut cache = LruCache::new(capacity);
        for (key, value) in initial {
            let _ = cache.put(key, value);
        }

        // Act
        let changed = cache.get_mut(&changed_key).map(|value| {
            *value += increment;
            *value
        });
        let unchanged = cache.get_mut(&unchanged_key).copied();
        let entries = drain(&mut cache);

        // Assert
        assert_eq!(changed, expected_changed);
        assert_eq!(unchanged, expected_unchanged);
        assert_eq!(entries, expected_entries);
    }

    #[test]
    fn updating_an_existing_key_returns_its_value_and_promotes_it() {
        // Arrange
        let capacity = 2;
        let initial = [(1, 10), (2, 20)];
        let replacement = 11;
        let next = (3, 30);
        let expected_old = Some(initial[0].1);
        let expected_entries = vec![(initial[0].0, replacement), next];
        let mut cache = LruCache::new(capacity);
        for (key, value) in initial {
            let _ = cache.put(key, value);
        }

        // Act
        let old = cache.put(initial[0].0, replacement);
        let updated_len = cache.len();
        let inserted = cache.put(next.0, next.1);
        let entries = drain(&mut cache);

        // Assert
        assert_eq!(old, expected_old);
        assert_eq!(updated_len, capacity);
        assert!(inserted.is_none());
        assert_eq!(entries, expected_entries);
    }

    #[test]
    fn observations_and_missing_keys_preserve_recency() {
        // Arrange
        let capacity = 3;
        let initial = [(1, 10), (2, 20), (3, 30)];
        let observed_key = initial[0].0;
        let missing_key = 9;
        let expected_found = Some(initial[0].1);
        let expected_missing = None;
        let expected_entries = initial.to_vec();
        let mut cache = LruCache::new(capacity);
        for (key, value) in initial {
            let _ = cache.put(key, value);
        }

        // Act
        let peeked = cache.peek(&observed_key).copied();
        let contains = cache.contains_key(&observed_key);
        let found_missing = cache.get(&missing_key).copied();
        let mutable_missing = cache.get_mut(&missing_key).copied();
        let peeked_missing = cache.peek(&missing_key).copied();
        let contains_missing = cache.contains_key(&missing_key);
        let removed_missing = cache.remove(&missing_key);
        let entries = drain(&mut cache);

        // Assert
        assert_eq!(peeked, expected_found);
        assert!(contains);
        assert_eq!(found_missing, expected_missing);
        assert_eq!(mutable_missing, expected_missing);
        assert_eq!(peeked_missing, expected_missing);
        assert!(!contains_missing);
        assert_eq!(removed_missing, expected_missing);
        assert_eq!(entries, expected_entries);
    }

    #[test]
    fn removing_any_position_preserves_the_remaining_order() {
        // Arrange
        let capacity = 3;
        let initial = [(1, 10), (2, 20), (3, 30)];
        let expected = vec![
            (Some(10), vec![(2, 20), (3, 30)]),
            (Some(20), vec![(1, 10), (3, 30)]),
            (Some(30), vec![(1, 10), (2, 20)]),
        ];

        // Act
        let results: Vec<_> = initial
            .iter()
            .map(|&(removed_key, _)| {
                let mut cache = LruCache::new(capacity);
                for (key, value) in initial {
                    let _ = cache.put(key, value);
                }
                let removed = cache.remove(&removed_key);
                (removed, drain(&mut cache))
            })
            .collect();

        // Assert
        assert_eq!(results, expected);
    }

    #[test]
    fn removing_the_only_entry_allows_reinsertion() {
        // Arrange
        let capacity = 1;
        let initial = (1, 10);
        let next = (2, 20);
        let expected_removed = Some(initial.1);
        let expected_empty_state = (0, true);
        let expected_entries = vec![next];
        let mut cache = LruCache::new(capacity);
        let _ = cache.put(initial.0, initial.1);

        // Act
        let removed = cache.remove(&initial.0);
        let empty_state = (cache.len(), cache.is_empty());
        let inserted = cache.put(next.0, next.1);
        let entries = drain(&mut cache);

        // Assert
        assert_eq!(removed, expected_removed);
        assert_eq!(empty_state, expected_empty_state);
        assert!(inserted.is_none());
        assert_eq!(entries, expected_entries);
    }

    #[test]
    fn pop_lru_returns_entries_in_order_and_then_none() {
        // Arrange
        let capacity = 3;
        let initial = [(1, 10), (2, 20), (3, 30)];
        let expected_entries = initial.to_vec();
        let expected_empty = None;
        let expected_state = (0, true);
        let mut cache = LruCache::new(capacity);
        for (key, value) in initial {
            let _ = cache.put(key, value);
        }

        // Act
        let entries = drain(&mut cache);
        let empty = cache.pop_lru();
        let state = (cache.len(), cache.is_empty());

        // Assert
        assert_eq!(entries, expected_entries);
        assert_eq!(empty, expected_empty);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn clear_preserves_capacity_and_allows_reuse() {
        // Arrange
        let capacity = 2;
        let initial = [(1, 10), (2, 20)];
        let next = [(3, 30), (4, 40), (5, 50)];
        let expected_empty = (capacity, 0, true);
        let expected_entries = next[1..].to_vec();
        let mut cache = LruCache::new(capacity);
        for (key, value) in initial {
            let _ = cache.put(key, value);
        }

        // Act
        cache.clear();
        let empty_state = (cache.capacity(), cache.len(), cache.is_empty());
        cache.clear();
        for (key, value) in next {
            let _ = cache.put(key, value);
        }
        let entries = drain(&mut cache);
        let retained_capacity = cache.capacity();

        // Assert
        assert_eq!(empty_state, expected_empty);
        assert_eq!(entries, expected_entries);
        assert_eq!(retained_capacity, capacity);
    }

    #[test]
    fn string_keys_support_borrowed_str_for_every_lookup() {
        // Arrange
        let capacity = 2;
        let key = "alpha";
        let value = 10;
        let increment = 1;
        let expected_value = Some(value);
        let expected_changed = Some(value + increment);
        let mut cache = LruCache::new(capacity);
        let _ = cache.put(key.to_owned(), value);

        // Act
        let found = cache.get(key).copied();
        let peeked = cache.peek(key).copied();
        let contains = cache.contains_key(key);
        let changed = cache.get_mut(key).map(|stored| {
            *stored += increment;
            *stored
        });
        let removed = cache.remove(key);
        let empty = cache.is_empty();

        // Assert
        assert_eq!(found, expected_value);
        assert_eq!(peeked, expected_value);
        assert!(contains);
        assert_eq!(changed, expected_changed);
        assert_eq!(removed, expected_changed);
        assert!(empty);
    }

    #[derive(Clone, Debug)]
    struct CollisionKey {
        id: u8,
        label: &'static str,
    }

    impl PartialEq for CollisionKey {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl Eq for CollisionKey {}

    impl Hash for CollisionKey {
        fn hash<H: Hasher>(&self, state: &mut H) {
            0_u8.hash(state);
        }
    }

    #[test]
    fn colliding_keys_remain_distinct_and_updates_retain_the_original_key() {
        // Arrange
        let capacity = 3;
        let first = CollisionKey {
            id: 1,
            label: "original",
        };
        let same = CollisionKey {
            id: 1,
            label: "replacement",
        };
        let second = CollisionKey {
            id: 2,
            label: "second",
        };
        let third = CollisionKey {
            id: 3,
            label: "third",
        };
        let fourth = CollisionKey {
            id: 4,
            label: "fourth",
        };
        let values = [10, 20, 30, 40];
        let updated_value = 11;
        let expected_old = Some(values[0]);
        let expected_removed = Some(values[2]);
        let expected_entries = vec![
            (first.id, first.label, updated_value),
            (fourth.id, fourth.label, values[3]),
        ];
        let mut cache = LruCache::new(capacity);
        let _ = cache.put(first.clone(), values[0]);
        let _ = cache.put(second.clone(), values[1]);
        let _ = cache.put(third.clone(), values[2]);

        // Act
        let old = cache.put(same, updated_value);
        let inserted = cache.put(fourth, values[3]);
        let evicted = cache.contains_key(&second);
        let removed = cache.remove(&third);
        let entries: Vec<_> = drain(&mut cache)
            .into_iter()
            .map(|(key, value)| (key.id, key.label, value))
            .collect();

        // Assert
        assert_eq!(old, expected_old);
        assert!(inserted.is_none());
        assert!(!evicted);
        assert_eq!(removed, expected_removed);
        assert_eq!(entries, expected_entries);
    }

    struct TrackedValue {
        id: u8,
        drops: Rc<RefCell<Vec<u8>>>,
    }

    impl TrackedValue {
        fn new(id: u8, drops: &Rc<RefCell<Vec<u8>>>) -> Self {
            Self {
                id,
                drops: Rc::clone(drops),
            }
        }
    }

    impl Drop for TrackedValue {
        fn drop(&mut self) {
            self.drops.borrow_mut().push(self.id);
        }
    }

    #[test]
    fn non_clone_values_returned_to_the_caller_stay_alive_until_dropped() {
        // Arrange
        let capacity = 2;
        let keys = [1, 2];
        let ids = [10, 20, 30];
        let changed_id = 21;
        let expected_live_drops: Vec<u8> = Vec::new();
        let expected_drops = vec![ids[0], changed_id, ids[2]];
        let drops = Rc::new(RefCell::new(Vec::new()));
        let mut cache = LruCache::new(capacity);
        let _ = cache.put(keys[0], TrackedValue::new(ids[0], &drops));
        let _ = cache.put(keys[1], TrackedValue::new(ids[2], &drops));

        // Act
        let old = cache
            .put(keys[0], TrackedValue::new(ids[1], &drops))
            .unwrap();
        let changed = cache.get_mut(&keys[0]).map(|value| {
            value.id = changed_id;
            value.id
        });
        let removed = cache.remove(&keys[0]).unwrap();
        let popped = cache.pop_lru().unwrap();
        let live_drops = drops.borrow().clone();
        let returned_ids = [old.id, removed.id, popped.1.id];
        drop(old);
        drop(removed);
        drop(popped);
        let final_drops = drops.borrow().clone();

        // Assert
        assert_eq!(changed, Some(changed_id));
        assert_eq!(live_drops, expected_live_drops);
        assert_eq!(returned_ids.as_slice(), expected_drops.as_slice());
        assert_eq!(final_drops, expected_drops);
    }

    #[test]
    fn discarded_non_clone_values_are_dropped_once() {
        // Arrange
        let zero_capacity = 0;
        let capacity = 1;
        let keys = [1, 2, 3];
        let ids = [10, 20, 30, 40];
        let expected_rejected = vec![ids[0]];
        let expected_evicted = vec![ids[0], ids[1]];
        let expected_cleared = vec![ids[0], ids[1], ids[2]];
        let expected_final = ids.to_vec();
        let drops = Rc::new(RefCell::new(Vec::new()));
        let mut zero = LruCache::new(zero_capacity);
        let mut cache = LruCache::new(capacity);

        // Act
        let rejected = zero.put(keys[0], TrackedValue::new(ids[0], &drops));
        let rejected_drops = drops.borrow().clone();
        let inserted = cache.put(keys[0], TrackedValue::new(ids[1], &drops));
        let evicted = cache.put(keys[1], TrackedValue::new(ids[2], &drops));
        let evicted_drops = drops.borrow().clone();
        cache.clear();
        let cleared_drops = drops.borrow().clone();
        let reinserted = cache.put(keys[2], TrackedValue::new(ids[3], &drops));
        drop(cache);
        drop(zero);
        let final_drops = drops.borrow().clone();

        // Assert
        assert!(rejected.is_none());
        assert!(inserted.is_none());
        assert!(evicted.is_none());
        assert!(reinserted.is_none());
        assert_eq!(rejected_drops, expected_rejected);
        assert_eq!(evicted_drops, expected_evicted);
        assert_eq!(cleared_drops, expected_cleared);
        assert_eq!(final_drops, expected_final);
    }

    #[derive(Eq, PartialEq, Hash)]
    struct ClonePanicKey {
        id: u8,
        panic_on_clone: bool,
    }

    impl Clone for ClonePanicKey {
        fn clone(&self) -> Self {
            assert!(!self.panic_on_clone, "requested clone panic");
            Self {
                id: self.id,
                panic_on_clone: self.panic_on_clone,
            }
        }
    }

    #[test]
    fn a_new_key_clone_panic_does_not_evict_an_existing_entry() {
        // Arrange
        let capacity = 1;
        let initial_key = ClonePanicKey {
            id: 1,
            panic_on_clone: false,
        };
        let failing_key = ClonePanicKey {
            id: 2,
            panic_on_clone: true,
        };
        let initial_value = 10;
        let next_value = 20;
        let expected_found = Some(initial_value);
        let mut cache = LruCache::new(capacity);
        let _ = cache.put(initial_key.clone(), initial_value);

        // Act
        let result = catch_unwind(AssertUnwindSafe(|| cache.put(failing_key, next_value)));
        let found = cache.peek(&initial_key).copied();
        let len = cache.len();

        // Assert
        assert!(result.is_err());
        assert_eq!(found, expected_found);
        assert_eq!(len, capacity);
    }

    #[derive(Debug, PartialEq)]
    enum Outcome {
        Value(Option<i32>),
        Entry(Option<(i32, i32)>),
        Contains(bool),
        Cleared,
    }

    #[test]
    fn deterministic_operations_match_an_independent_vec_model() {
        // Arrange
        let capacities = [0, 1, 2, 5];
        let initial_seed = 0x6a09_e667_f3bc_c909_u64;
        let steps = 4_000;
        let key_count = 8;
        let drain_interval = 257;
        let increment = 1;

        for capacity in capacities {
            let mut cache = LruCache::new(capacity);
            // The model stores entries from LRU to MRU and uses only linear searches.
            let mut model: Vec<(i32, i32)> = Vec::new();
            let mut random = initial_seed;
            for step in 0..steps {
                // Arrange
                random ^= random << 13;
                random ^= random >> 7;
                random ^= random << 17;
                let operation = if step < key_count { 0 } else { random % 12 };
                let key = if step < key_count {
                    step
                } else {
                    (random >> 16) as usize % key_count
                } as i32;
                let value = step as i32;
                let position = model.iter().position(|&(stored, _)| stored == key);

                // Act
                let (actual, expected) = match operation {
                    0..=3 => {
                        let actual = cache.put(key, value);
                        let expected = position.map(|index| model.remove(index).1);
                        if capacity > 0 {
                            if model.len() == capacity {
                                model.remove(0);
                            }
                            model.push((key, value));
                        }
                        (Outcome::Value(actual), Outcome::Value(expected))
                    }
                    4 | 11 => {
                        let actual = cache.get(&key).copied();
                        let expected = position.map(|index| {
                            let entry = model.remove(index);
                            model.push(entry);
                            entry.1
                        });
                        (Outcome::Value(actual), Outcome::Value(expected))
                    }
                    5 => {
                        let actual = cache.get_mut(&key).map(|stored| {
                            *stored += increment;
                            *stored
                        });
                        let expected = position.map(|index| {
                            let mut entry = model.remove(index);
                            entry.1 += increment;
                            model.push(entry);
                            entry.1
                        });
                        (Outcome::Value(actual), Outcome::Value(expected))
                    }
                    6 => {
                        let actual = cache.peek(&key).copied();
                        let expected = position.map(|index| model[index].1);
                        (Outcome::Value(actual), Outcome::Value(expected))
                    }
                    7 => {
                        let actual = cache.contains_key(&key);
                        let expected = position.is_some();
                        (Outcome::Contains(actual), Outcome::Contains(expected))
                    }
                    8 => {
                        let actual = cache.remove(&key);
                        let expected = position.map(|index| model.remove(index).1);
                        (Outcome::Value(actual), Outcome::Value(expected))
                    }
                    9 => {
                        let actual = cache.pop_lru();
                        let expected = if model.is_empty() {
                            None
                        } else {
                            Some(model.remove(0))
                        };
                        (Outcome::Entry(actual), Outcome::Entry(expected))
                    }
                    10 => {
                        cache.clear();
                        model.clear();
                        (Outcome::Cleared, Outcome::Cleared)
                    }
                    _ => unreachable!(),
                };
                let actual_state = (cache.capacity(), cache.len(), cache.is_empty());
                let expected_state = (capacity, model.len(), model.is_empty());
                let actual_values: Vec<_> = (0..key_count as i32)
                    .map(|candidate| cache.peek(&candidate).copied())
                    .collect();
                let expected_values: Vec<_> = (0..key_count as i32)
                    .map(|candidate| {
                        model
                            .iter()
                            .find(|&&(stored, _)| stored == candidate)
                            .map(|&(_, stored)| stored)
                    })
                    .collect();
                let drained = if (step + 1) % drain_interval == 0 {
                    Some((drain(&mut cache), std::mem::take(&mut model)))
                } else {
                    None
                };

                // Assert
                assert_eq!(actual, expected, "seed={initial_seed:#x}, capacity={capacity}, step={step}, operation={operation}");
                assert_eq!(
                    actual_state, expected_state,
                    "capacity={capacity}, step={step}"
                );
                assert_eq!(
                    actual_values, expected_values,
                    "capacity={capacity}, step={step}"
                );
                if let Some((actual_order, expected_order)) = drained {
                    assert_eq!(
                        actual_order, expected_order,
                        "capacity={capacity}, step={step}"
                    );
                }
            }

            // Act
            let final_entries = drain(&mut cache);

            // Assert
            assert_eq!(
                final_entries, model,
                "seed={initial_seed:#x}, capacity={capacity}, final order"
            );
        }
    }
}

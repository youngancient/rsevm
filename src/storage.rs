use std::collections::{HashMap, HashSet};

use alloy_primitives::U256;

pub struct Storage {
    pub storage: HashMap<U256, U256>,
    cache: HashSet<U256>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            storage: HashMap::new(),
            cache: HashSet::new(),
        }
    }

    fn load_raw(&self, key: U256) -> U256 {
        *self.storage.get(&key).unwrap_or(&U256::ZERO)
    }
    pub fn load(&mut self, key: U256) -> (bool, U256) {
        // .insert() returns true if the value was NEW (Cold).
        // It returns false if the value was ALREADY THERE (Warm).
        let is_warm = !self.cache.insert(key);
        return (is_warm, self.load_raw(key));
    }

    pub fn store(&mut self, key: U256, value: U256) -> (bool, U256) {
        let is_warm = !self.cache.insert(key);
        let old_value = self.load_raw(key);
        if value == U256::ZERO {
            self.storage.remove(&key);
        } else {
            self.storage.insert(key, value);
        }
        (is_warm, old_value)
    }
    // read-only helper to calculate SSTORE gas requirements
    // does not modify storage state
    pub fn peek(&self, key: &U256) -> (bool, U256) {
        let is_warm = self.cache.contains(key);

        let value = *self.storage.get(key).unwrap_or(&U256::ZERO);
        (is_warm, value)
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    fn create_storage() -> Storage {
        Storage::new()
    }
    #[test]
    fn test_storage_init() {
        let mut evm_storage = create_storage();
        assert_eq!(evm_storage.cache.len(), 0);
        assert_eq!(evm_storage.load(U256::from(3)), (false, U256::ZERO))
    }

    #[test]
    fn test_storage_store_and_load() {
        let mut evm_storage = create_storage();
        let key = U256::from(1);
        let val = U256::from(420);
        evm_storage.store(key, val);
        assert_eq!(evm_storage.load(key), (false, val));
    }

    #[test]
    fn test_storage_load_cool_and_warm() {
        let mut evm_storage = create_storage();
        let key = U256::from(1);
        let val = U256::from(420);
        evm_storage.store(key, val);
        assert_eq!(evm_storage.load(key), (false, val)); // cold
        assert_eq!(evm_storage.cache.len(), 1); // key cached
        assert_eq!(evm_storage.load(key), (true, val)); // warm after first access
    }
}

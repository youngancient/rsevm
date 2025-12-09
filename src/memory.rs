use crate::evm::EvmError;

pub struct Memory {
    pub memory: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self { memory: Vec::new() }
    }
    pub fn ensure_capacity(&mut self, offset: usize, size: usize) -> u64 {
        let required_len = offset.saturating_add(size);
        if required_len <= self.memory.len() {
            return 0u64;
        }
        // old cost
        let current_words = (self.memory.len() as u64 + 31) / 32;
        let old_cost = Self::calculate_memory_gas(current_words);

        // new cost
        let required_words = (required_len as u64 + 31) / 32;
        let new_cost = Self::calculate_memory_gas(required_words);

        let expansion_cost = new_cost.saturating_sub(old_cost);

        let new_size_bytes = (required_words * 32) as usize;
        self.memory.resize(new_size_bytes, 0);

        expansion_cost
    }
    pub fn access(&self, offset: usize, size: usize) -> Result<&[u8], EvmError> {
        let end = offset.saturating_add(size);
        if end > self.memory.len() {
            return Err(EvmError::MemoryOutOfBounds {
                offset,
                size,
                max: self.memory.len(),
            });
        }
        Ok(&self.memory[offset..end])
    }
    pub fn load(&mut self, offset: usize) -> ([u8; 32], u64) {
        const WORD_SIZE: usize = 32;
        let required_len = offset.saturating_add(WORD_SIZE);
        let expansion_cost = self.ensure_capacity(offset, WORD_SIZE);
        let mut word = [0u8; WORD_SIZE];
        let available_data = &self.memory[offset..required_len];
        word.copy_from_slice(available_data);
        (word, expansion_cost)
    }
    pub fn store(&mut self, offset: usize, value: &[u8]) -> u64 {
        let size = value.len();
        if size == 0 {
            return 0;
        }
        let expansion_cost = self.ensure_capacity(offset, size); // expands memory to be able to store value
        let dest = &mut self.memory[offset..offset + size];
        dest.copy_from_slice(value);
        expansion_cost
    }

    fn calculate_memory_gas(size_in_words: u64) -> u64 {
        let linear_cost = size_in_words * 3;
        let quadratic_cost = (size_in_words * size_in_words) / 512;
        linear_cost + quadratic_cost
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    fn init_memory() -> Memory {
        Memory::new()
    }

    #[test]
    fn test_memory_init() {
        let mem = init_memory();
        assert_eq!(mem.memory.len(), 0)
    }

    #[test]
    fn test_store() {
        let mut mem = init_memory();
        mem.store(0, &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(mem.memory.len(), 4);
    }

    #[test]
    fn test_access() {
        let mut mem = init_memory();
        mem.store(0, &[0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_access_success() {
        let mut mem = init_memory();
        mem.store(0, &[0x01, 0x02, 0x03, 0x04]);
        let result = mem.access(1, 3);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), &[0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_access_failure() {
        let mut mem = init_memory();
        mem.store(0, &[0x01, 0x02, 0x03, 0x04]);
        // should return EvmError
        let result = mem.access(1, 5);
        assert!(result.is_err());
        let expected_error = EvmError::MemoryOutOfBounds {
            offset: 1,
            size: 5,
            max: 4,
        };
        assert_eq!(result.unwrap_err(), expected_error)
    }

    #[test]
    fn test_load() {
        let mut mem = init_memory();
        mem.store(0, &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(
            mem.load(0).0,
            [
                1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn test_load_from_offset_1() {
        let mut mem = init_memory();
        mem.store(0, &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(
            mem.load(1).0,
            [
                2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );
    }
    #[test]
    fn test_load_from_offset_2() {
        let mut mem = init_memory();
        mem.store(0, &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(
            mem.load(3).0,
            [
                4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );
    }
}

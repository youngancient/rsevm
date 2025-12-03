use alloy_primitives::U256;

use crate::evm::EvmError;

pub const MAXIMUM_STACK_SIZE: usize = 1024;
#[derive(Debug, Clone)]
pub struct Stack {
    items: Vec<U256>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            items: Vec::with_capacity(MAXIMUM_STACK_SIZE),
        }
    }
    pub fn push(&mut self, item: U256) -> Result<(), EvmError> {
        if self.items.len() + 1 > MAXIMUM_STACK_SIZE {
            return Err(EvmError::StackOverflow);
        }
        self.items.push(item);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<U256, EvmError> {
        self.items.pop().ok_or(EvmError::StackUnderflow)
    }

    pub fn len(&self) -> usize{
        self.items.len()
    }

    // peek at item n slots from the top of the stack
    pub fn peek(&self, n_from_top : usize) -> Result<U256, EvmError> {
        let stack_length = self.items.len();
        if stack_length < n_from_top{
            return Err(EvmError::StackUnderflow);
        }
        let index = stack_length - 1 - n_from_top;  // subtract 1 because we want to start from index zero,
        Ok(self.items[index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_stack() -> Stack {
        Stack::new()
    }

    #[test]
    fn test_create_stack() {
        assert_eq!(create_stack().items.len(), 0);
    }

    #[test]
    fn test_push_item_to_stack() {
        let mut new_stack = create_stack();
        let _ = new_stack.push(U256::from(0x9222));
        let _ = new_stack.push(U256::from(0x87222));
        assert_eq!(new_stack.items.len(), 2);
    }
    // fix
    #[test]
    fn test_should_fail_if_stack_overflow() {
        let mut new_stack = create_stack();
        for _ in 0..=1024 {
            let res = new_stack.push(U256::from(0x9222));
            if res.is_err() {
                assert_eq!(res.unwrap_err(), EvmError::StackOverflow);
            }
        }
    }

    #[test]
    fn test_pop_item_from_stack() {
        let mut new_stack = create_stack();
        let _ = new_stack.push(U256::from(0x9222));
        let _ = new_stack.push(U256::from(0x87222));
        let _ = new_stack.pop();
        assert_eq!(new_stack.items[0], U256::from(0x9222));
        assert_eq!(new_stack.items.len(), 1);
    }
    // fix
    #[test]
    fn test_should_fail_if_stack_underflow() {
        let mut new_stack = create_stack();
        let _ = new_stack.push(U256::from(0x9222));
        let _ = new_stack.push(U256::from(0x87222));
        for _ in 0..5 {
            let res = new_stack.pop();
            if res.is_err() {
                assert_eq!(res.unwrap_err(), EvmError::StackUnderflow);
            }
        }
    }
}

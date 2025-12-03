use alloy_primitives::{Address, U256};

use crate::{memory::Memory, stack::Stack, storage::Storage};

#[derive(Debug,PartialEq)]
pub enum EvmError {
    OutOfGas,
    StackUnderflow,
    StackOverflow,
    MemoryOutOfBounds { offset: usize, size: usize, max: usize },
    ReturnDataOutOfBounds { offset: usize, size: usize, max: usize },
    BadJumpDestination {dest : usize, reason : String}
}
pub struct Log {
    pub topics: Vec<U256>,
    pub data: Vec<u8>,
}

pub struct EVM {
    pub pc: usize,
    pub value: U256,
    pub calldata: Vec<u8>,
    pub gas: u64,
    pub refund: u64,    // refunds can not pay for transactions themselves, they like vouchers given on transaction execution
    pub sender: Address,
    // sub components
    pub program: Vec<u8>,
    pub stack: Stack,
    pub memory: Memory,
    pub storage: Storage,
    // flags
    pub stop_flag: bool,
    pub revert_flag: bool,
    // output
    pub return_data: Vec<u8>,
    pub logs: Vec<Log>,
}

// todos:
// opcodes left: Push, Swap and Log
// EVM functions left: run 

impl EVM {
    pub fn new(
        sender: Address,
        program: Vec<u8>,
        gas: u64,
        value: U256,
        calldata: Vec<u8>,
    ) -> Self {
        Self {
            pc: 0,
            value,
            sender,
            calldata,
            program,
            gas,
            refund : 0,
            stop_flag: false,
            revert_flag: false,
            stack: Stack::new(),
            memory: Memory::new(),
            storage: Storage::new(),
            return_data: Vec::new(),
            logs: Vec::new(),
        }
    }

    pub fn gas_dec(&mut self, amount: u64) -> Result<(), EvmError> {
        if amount > self.gas{
            return Err(EvmError::OutOfGas);
        }
        self.gas -= amount;
        Ok(())
    }
    pub fn peek(&self) -> u8{
        self.program[self.pc]
    }
    pub fn reset(&mut self){
        self.pc = 0;
        self.stack = Stack::new();
        self.memory = Memory::new();
        self.storage = Storage::new();
    }
    pub fn should_execute_next_opcode(self) -> bool{
        if self.pc > (self.program.len() - 1){  // means pc has reached the max program length
            return false;
        }
        if self.stop_flag || self.revert_flag{
            return false;
        }
        true
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_state() {}
}

use alloy_primitives::{U256, Address};

use crate::{memory::Memory, stack::Stack, storage::Storage};

pub struct Log{
    pub topics : Vec<U256>,
    pub data : Vec<u8>
}

pub struct State {
    pc: usize,
    value: U256,
    calldata: Vec<u8>,
    gas : u64,
    sender : Address,
    // sub components
    program : Vec<u8>,
    stack: Stack,
    memory: Memory,
    storage: Storage,
    // flags
    stop_flag: bool,
    revert_flag: bool,
    // output
    return_data: Vec<u8>,
    logs: Vec<Log>,
}

impl State {
    pub fn new(sender : Address, program : Vec<u8>, gas : u64, value : U256, calldata : Vec<u8>) -> Self {
        Self {
            pc: 0,
            value,
            sender,
            calldata,
            program,
            gas,
            stop_flag: false,
            revert_flag: false,
            stack: Stack::new(),
            memory: Memory::new(),
            storage: Storage::new(),
            return_data: Vec::new(),
            logs: Vec::new(),
        }
    }
}

#[cfg(test)]

mod tests{
    use super::*;

    #[test]
    fn test_state(){
        
    }
}
use std::{collections::HashMap, fmt::Debug};

use alloy_primitives::{Address, U256};

use crate::{
    memory::Memory,
    opcodes::{
        bit::{byte, sar, shl, shr},
        comparisons::{eq, gt, is_zero, lt, sgt, slt},
        contract::revert,
        dup::dup,
        environment::{
            address, balance, call_data_copy, call_data_load, call_data_size, call_value,
            code_copy, code_size, ext_code_copy, ext_code_hash, gas_price, return_data_copy,
            return_data_size,
        },
        jump::{jump, jump_dest, jumpi, pc},
        log::log,
        logic::{and, not, or, xor},
        math::{add, add_mod, div, mul, mul_mod, sdiv, signextend, smod, sub, vm_mod},
        memory::{mload, mstore, mstore8},
        misc::sha3,
        opcodes::{
            ADD, ADDMOD, ADDRESS, AND, BALANCE, BYTE, CALLDATACOPY, CALLDATALOAD, CALLDATASIZE,
            CALLVALUE, CODECOPY, CODESIZE, DIV, DUP1, DUP16, EQ, EXTCODECOPY, EXTCODEHASH,
            GASPRICE, GT, ISZERO, JUMP, JUMPDEST, JUMPI, LOG0, LOG4, LT, MLOAD, MOD, MSTORE,
            MSTORE8, MUL, MULMOD, NOT, OR, ORIGIN, PC, POP, PUSH1, PUSH32, RETURNDATACOPY,
            RETURNDATASIZE, REVERT, SAR, SDIV, SGT, SHA3, SHL, SHR, SIGNEXTEND, SLOAD, SLT, SMOD,
            SSTORE, STOP, SUB, SWAP1, SWAP16, TLOAD, TSTORE, XOR,
        },
        pop::pop,
        push::push,
        stop::stop,
        storage::{s_store, sload},
        swap::swap,
        transient::{tload, tstore},
    },
    stack::Stack,
    storage::Storage,
};

#[derive(Debug, PartialEq)]
pub enum EvmError {
    OutOfGas,
    StackUnderflow,
    StackOverflow,
    MemoryOutOfBounds {
        offset: usize,
        size: usize,
        max: usize,
    },
    ReturnDataOutOfBounds {
        offset: usize,
        size: usize,
        max: usize,
    },
    BadJumpDestination {
        dest: usize,
        reason: String,
    },
    UnknownOpcode {
        opcode: String,
    },
}
pub struct Log {
    pub topics: Vec<U256>,
    pub data: Vec<u8>,
}
impl Log {
    pub fn new(data: Vec<u8>, topics: Vec<U256>) -> Self {
        Self { topics, data }
    }
}

impl Debug for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Log: Topics: {:?} | Data: {:?}", self.topics, self.data)
    }
}

pub struct EVM {
    // The evm is dumb, it cannot different between hex values that are opcodes and those that are just values
    // Its train of execution is directed by the program counter. 
    // e.g given program -> [0x60, 0x69,0x60, 0x01, 0x55]. program[0] = 0x60 => PUSH1 
    // Initially PC = 0, the EVM executes PUSH1
    // PUSH1 updates PC by adding 2 bytes to it (one for itself, and one for the value pushed to the Stack)
    // now PC = 2   program[2] = 0x60 => PUSH1, the EVM executes PUSH1
    // As seen above, the EVM was dumb and only followed the directive of PC
    pub pc: usize,
    pub value: U256,
    pub calldata: Vec<u8>,
    pub gas: u64,
    pub refund: u64, // refunds can not pay for transactions themselves, they like vouchers given on transaction execution
    pub sender: Address,
    // sub components
    pub program: Vec<u8>,
    pub stack: Stack,
    pub memory: Memory,
    pub storage: Storage,
    pub transient_storage: HashMap<U256, U256>,
    // flags
    pub stop_flag: bool,
    pub revert_flag: bool,
    // output
    pub return_data: Vec<u8>,
    pub logs: Vec<Log>,
}

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
            refund: 0,
            stop_flag: false,
            revert_flag: false,
            stack: Stack::new(),
            memory: Memory::new(),
            storage: Storage::new(),
            transient_storage: HashMap::new(),
            return_data: Vec::new(),
            logs: Vec::new(),
        }
    }

    pub fn gas_dec(&mut self, amount: u64) -> Result<(), EvmError> {
        if amount > self.gas {
            return Err(EvmError::OutOfGas);
        }
        self.gas -= amount;
        Ok(())
    }
    pub fn peek(&self) -> u8 {
        self.program[self.pc]
    }
    pub fn reset(&mut self) {
        self.pc = 0;
        self.stack = Stack::new();
        self.memory = Memory::new();
        self.storage = Storage::new();
        self.transient_storage = HashMap::new()
    }
    pub fn should_execute_next_opcode(&self) -> bool {
        if self.pc > (self.program.len() - 1) {
            // means pc has reached the max program length
            return false;
        }
        if self.stop_flag || self.revert_flag {
            return false;
        }
        true
    }
    pub fn run(&mut self) -> Result<(), EvmError> {
        while self.should_execute_next_opcode() {
            let opcode = self.program[self.pc];
            match opcode {
                // STOP
                STOP => stop(self)?,
                // MATH
                ADD => add(self)?,
                SUB => sub(self)?,
                MUL => mul(self)?,
                SMOD => smod(self)?,
                DIV => div(self)?,
                SDIV => sdiv(self)?,
                MOD => vm_mod(self)?,
                ADDMOD => add_mod(self)?,
                MULMOD => mul_mod(self)?,
                SIGNEXTEND => signextend(self)?,
                // BIT
                BYTE => byte(self)?,
                SHL => shl(self)?,
                SHR => shr(self)?,
                SAR => sar(self)?,
                // LOGIC
                LT => lt(self)?,
                SLT => slt(self)?,
                GT => gt(self)?,
                SGT => sgt(self)?,
                EQ => eq(self)?,
                ISZERO => is_zero(self)?,
                // DUP
                DUP1..=DUP16 => {
                    // calculate n dynamically
                    let n = (opcode - DUP1 + 1) as usize; // 1 is added because DUP is 1-indexed
                    dup(self, n)?;
                }
                // ENVIRONMENT
                ADDRESS => address(self)?,
                BALANCE => balance(self)?,
                ORIGIN => balance(self)?,
                CALLVALUE => call_value(self)?,
                CALLDATALOAD => call_data_load(self)?,
                CALLDATASIZE => call_data_size(self)?,
                CALLDATACOPY => call_data_copy(self)?,
                CODESIZE => code_size(self)?,
                CODECOPY => code_copy(self)?,
                GASPRICE => gas_price(self)?,
                EXTCODECOPY => ext_code_copy(self)?,
                EXTCODEHASH => ext_code_hash(self)?,
                RETURNDATACOPY => return_data_copy(self)?,
                RETURNDATASIZE => return_data_size(self)?,
                // JUMP
                JUMP => jump(self)?,
                JUMPI => jumpi(self)?,
                JUMPDEST => jump_dest(self)?,
                PC => pc(self)?,
                // LOG
                LOG0..=LOG4 => {
                    // calculate n dynamically
                    let n = (opcode - LOG0) as usize; // there's no need to add 1, since LOG is 0-indexed
                    log(self, n)?;
                }
                // LOGIC
                AND => and(self)?,
                OR => or(self)?,
                XOR => xor(self)?,
                NOT => not(self)?,
                // MEMORY
                MLOAD => mload(self)?,
                MSTORE => mstore(self)?,
                MSTORE8 => mstore8(self)?,
                // MISC
                SHA3 => sha3(self)?,
                // POP
                POP => pop(self)?,
                // PUSH
                PUSH1..=PUSH32 => {
                    // calculate n dynamically
                    let n = (opcode - PUSH1 + 1) as usize; // 1 is added because PUSH is 1-indexed
                    push(self, n)?;
                }
                // STORAGE
                SLOAD => sload(self)?,
                SSTORE => s_store(self)?,
                // SWAP
                SWAP1..=SWAP16 => {
                    // calculate n dynamically
                    // 0x90 - 0x90 + 1 = SWAP 1
                    // 0X91 - 0X90 + 1 = SWAP 2
                    let n = (opcode - SWAP1 + 1) as usize; // 1 is added because SWAP is 1-indexed
                    swap(self, n)?;
                }
                // TRANSIENT
                TLOAD => tload(self)?,
                TSTORE => tstore(self)?,
                REVERT => revert(self)?,
                _ => {
                    return Err(EvmError::UnknownOpcode {
                        opcode: format!("0x{:02x}", opcode),
                    });
                }
            }
        }
        Ok(())
    }

    // for terminal UI
    pub fn step(&mut self) -> Result<bool, EvmError> {
        if !self.should_execute_next_opcode() {
            return Ok(false);
        }
        let opcode = self.program[self.pc];
        match opcode {
            // STOP
            STOP => stop(self)?,
            // MATH
            ADD => add(self)?,
            SUB => sub(self)?,
            MUL => mul(self)?,
            SMOD => smod(self)?,
            DIV => div(self)?,
            SDIV => sdiv(self)?,
            MOD => vm_mod(self)?,
            ADDMOD => add_mod(self)?,
            MULMOD => mul_mod(self)?,
            SIGNEXTEND => signextend(self)?,
            // BIT
            BYTE => byte(self)?,
            SHL => shl(self)?,
            SHR => shr(self)?,
            SAR => sar(self)?,
            // LOGIC
            LT => lt(self)?,
            SLT => slt(self)?,
            GT => gt(self)?,
            SGT => sgt(self)?,
            EQ => eq(self)?,
            ISZERO => is_zero(self)?,
            // DUP
            DUP1..=DUP16 => {
                // calculate n dynamically
                let n = (opcode - DUP1 + 1) as usize; // 1 is added because DUP is 1-indexed
                dup(self, n)?;
            }
            // ENVIRONMENT
            ADDRESS => address(self)?,
            BALANCE => balance(self)?,
            ORIGIN => balance(self)?,
            CALLVALUE => call_value(self)?,
            CALLDATALOAD => call_data_load(self)?,
            CALLDATASIZE => call_data_size(self)?,
            CALLDATACOPY => call_data_copy(self)?,
            CODESIZE => code_size(self)?,
            CODECOPY => code_copy(self)?,
            GASPRICE => gas_price(self)?,
            EXTCODECOPY => ext_code_copy(self)?,
            EXTCODEHASH => ext_code_hash(self)?,
            RETURNDATACOPY => return_data_copy(self)?,
            RETURNDATASIZE => return_data_size(self)?,
            // JUMP
            JUMP => jump(self)?,
            JUMPI => jumpi(self)?,
            JUMPDEST => jump_dest(self)?,
            PC => pc(self)?,
            // LOG
            LOG0..=LOG4 => {
                // calculate n dynamically
                let n = (opcode - LOG0) as usize; // there's no need to add 1, since LOG is 0-indexed
                log(self, n)?;
            }
            // LOGIC
            AND => and(self)?,
            OR => or(self)?,
            XOR => xor(self)?,
            NOT => not(self)?,
            // MEMORY
            MLOAD => mload(self)?,
            MSTORE => mstore(self)?,
            MSTORE8 => mstore8(self)?,
            // MISC
            SHA3 => sha3(self)?,
            // POP
            POP => pop(self)?,
            // PUSH
            PUSH1..=PUSH32 => {
                // calculate n dynamically
                let n = (opcode - PUSH1 + 1) as usize; // 1 is added because PUSH is 1-indexed
                push(self, n)?;
            }
            // STORAGE
            SLOAD => sload(self)?,
            SSTORE => s_store(self)?,
            // SWAP
            SWAP1..=SWAP16 => {
                // calculate n dynamically
                // 0x90 - 0x90 + 1 = SWAP 1
                // 0X91 - 0X90 + 1 = SWAP 2
                let n = (opcode - SWAP1 + 1) as usize; // 1 is added because SWAP is 1-indexed
                swap(self, n)?;
            }
            // TRANSIENT
            TLOAD => tload(self)?,
            TSTORE => tstore(self)?,
            REVERT => revert(self)?,
            _ => {
                return Err(EvmError::UnknownOpcode {
                    opcode: format!("0x{:02x}", opcode),
                });
            }
        }
        Ok(true)
    }
}

use alloy_primitives::{Address, U256};

use evm::evm::{EVM, EvmError};

fn init_evm() -> EVM {
    EVM::new(Address::ZERO, vec![], 1000, U256::ZERO, vec![])
}

#[test]
fn test_simple_add() {
    let mut my_evm = init_evm();
    let program = [0x60, 0x42, 0x60, 0xFF, 0x01];
    // 0x60 opcode -> PUSH1 -> push the next 1 byte onto the stack
    // 0x42 -> 66
    // 0xFF -> 255
    // 0x01 opcode -> ADD

    // load program
    my_evm.program = program.to_vec();
    let output = my_evm.run();
    assert!(output.is_ok());
    assert_eq!(my_evm.stack.len(), 1);
    assert_eq!(my_evm.stack.peek(0).unwrap(), U256::from(321));
    assert_eq!(my_evm.pc, 5);
}

#[test]
fn test_memory_store_load() {
    let mut my_evm = init_evm();
    let program = vec![
        0x60, 0xFF, // PUSH1 0xFF
        0x60, 0x00, // PUSH1 0x00
        0x52, // MSTORE
        0x60, 0x00, // PUSH1 0x00
        0x51, // MLOAD
    ];
    // load program
    my_evm.program = program;
    let output = my_evm.run();
    assert!(output.is_ok());
    assert_eq!(my_evm.stack.peek(0).unwrap(), U256::from(0xFF));
    assert_eq!(my_evm.pc, 8);
}

#[test]
fn test_storage_persistence() {
    let mut my_evm = init_evm();
    let program = vec![
        0x60, 0x69, // PUSH1 0X69
        0x60, 0x01, // PUSH1 0X01
        0x55,
    ]; // SSTORE
    // load bytecode
    my_evm.program = program;
    // increase gas
    my_evm.gas = 30000;

    let output = my_evm.run();
    assert!(output.is_ok());
    let key = U256::from(0x01);
    let value = U256::from(0x69);
    let retrieved_val = my_evm.storage.load(key);
    assert_eq!(retrieved_val.1, value);
}
// check for storage persistence and warmness of value

// Error handling
#[test]
fn test_out_of_gas() {
    let mut my_evm = init_evm();
    let program = vec![
        0x60, 0x69, // PUSH1 0X69
        0x60, 0x01, // PUSH1 0X01
        0x55,
    ]; // SSTORE
    // load bytecode
    my_evm.program = program;
    let output = my_evm.run();
    assert!(output.is_err());
    assert_eq!(output, Err(EvmError::OutOfGas));
}

#[test]
fn test_stack_underflow() {
    let mut my_evm = init_evm();
    let program = vec![
        0x50, // POP on an empty stack
    ];
    // load bytecode
    my_evm.program = program;
    let output = my_evm.run();
    assert!(output.is_err());
    assert_eq!(output, Err(EvmError::StackUnderflow));
}

#[test]
fn test_stack_overflow() {
    let mut my_evm = init_evm();
    let mut program = Vec::new();
    for _ in 0..1025 {
        program.push(0x60); // PUSH1
        program.push(0x00); // 0
    }
    // load bytecode
    my_evm.program = program;
    // since each PUSH1 costs 3 gas, we need to upgrade gas to 1025 * 3
    my_evm.gas = 1025 * 3;
    let output = my_evm.run();
    assert!(output.is_err());
    assert_eq!(output, Err(EvmError::StackOverflow));
}

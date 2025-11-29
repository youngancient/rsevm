use alloy_primitives::U256;

use crate::evm::{EVM, EvmError};

// loads one word (32 bytes) from memory onto the stack
pub fn mload(vm: &mut EVM) -> Result<(), EvmError> {
    let offset_raw = vm.stack.pop()?;
    let offset = offset_raw.saturating_to::<usize>();

    let static_gas = 3;
    let (word, _expansion_cost) = vm.memory.load(offset);
    vm.gas_dec(static_gas + _expansion_cost)?;

    vm.stack.push(U256::from_be_bytes(word))?;
    vm.pc += 1;
    Ok(())
}

// stores one word (32 bytes) in memory
pub fn mstore(vm: &mut EVM) -> Result<(), EvmError> {
    let offset_raw = vm.stack.pop()?;
    let word = vm.stack.pop()?;

    let offset = offset_raw.saturating_to::<usize>();
    let word_in_bytes: [u8; 32] = word.to_be_bytes();
    let expansion_cost = vm.memory.store(offset, &word_in_bytes);

    let static_gas = 3;
    vm.gas_dec(expansion_cost + static_gas)?;
    vm.pc += 1;
    Ok(())
}

// stores 1byte of a Word in memory
pub fn mstore8(vm: &mut EVM) -> Result<(), EvmError> {
    let offset_raw = vm.stack.pop()?;
    let word = vm.stack.pop()?;

    let offset = offset_raw.saturating_to::<usize>();
    let word_in_bytes: [u8; 32] = word.to_be_bytes();
    let single_byte_of_word = word_in_bytes[31];
    let expansion_cost = vm.memory.store(offset, &[single_byte_of_word]);

    let static_gas = 3;
    vm.gas_dec(expansion_cost + static_gas)?;
    vm.pc += 1;
    Ok(())
}

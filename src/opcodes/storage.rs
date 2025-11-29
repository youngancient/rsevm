// use alloy_primitives::U256;

use crate::evm::{EVM, EvmError};

// loads one word (32 bytes) from storage by a `key`` onto the stack
pub fn sload(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(2100)?;

    let key = vm.stack.pop()?;
    let (_is_warm, word) = vm.storage.load(key);
    vm.stack.push(word)?;

    vm.pc += 1;
    Ok(())
}

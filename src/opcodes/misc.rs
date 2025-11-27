use alloy_primitives::keccak256;

use crate::evm::{EVM, EvmError};

pub fn sha3(vm: &mut EVM) -> Result<(), EvmError> {
    let offset = vm.stack.pop()?;
    let size = vm.stack.pop()?;
    let size_u64 = size.saturating_to::<usize>();
    let min_word_size = (size_u64 + 31) / 32;
    let dynamic_gas = 6 * min_word_size;
    let static_gas = 30;
    vm.gas_dec((dynamic_gas + static_gas) as u64)?;

    let value = vm.memory.access(offset.to::<usize>(), size.to::<usize>())?;
    vm.stack.push(keccak256(value).into())?;

    vm.pc += 1;
    Ok(())
}

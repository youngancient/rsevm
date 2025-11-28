use crate::evm::{EVM, EvmError};

// pops first item off stack
pub fn pop(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(2)?;
    vm.stack.pop()?;
    vm.pc += 1;
    Ok(())
}

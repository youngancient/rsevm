use crate::evm::{EVM, EvmError};

pub fn revert(vm: &mut EVM) -> Result<(), EvmError> {
    let offset_raw = vm.stack.pop()?;
    let size_raw = vm.stack.pop()?;
    let offset = offset_raw.saturating_to::<usize>();
    let size = size_raw.saturating_to::<usize>();
    
    let expansion_cost = vm.memory.ensure_capacity(offset, size);
    // static gas cost is zero, only memory epansion cost is paid
    vm.gas_dec(expansion_cost)?;

    vm.return_data = vm.memory.access(offset, size)?.to_vec();

    vm.revert_flag = true;
    vm.stop_flag = true;

    vm.pc += 1;
    Ok(())
}

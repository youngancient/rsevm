use crate::evm::{EVM, EvmError};

// Duplicate a stack item by putting it on top of the stack.
// DUP1 means duplicate the first item on the stack; so we peek at the offset 0
// DUP2 means duplicate the first item on the stack; so we peek at the offset 1
pub fn dup(vm: &mut EVM, n: usize) -> Result<(), EvmError> {
    vm.gas_dec(3)?;
    let value = vm.stack.peek(n - 1)?;
    vm.stack.push(value)?;
    
    vm.pc += 1;

    Ok(())
}


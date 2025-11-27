// these opcodes give access to the ethereum environment

use alloy_primitives::U256;

use crate::evm::{EVM, EvmError};

pub fn address(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(2)?;
    vm.stack.push(vm.sender.into_word().into())?;
    vm.pc += 1;
    Ok(())
}

pub fn balance(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(2600)?;
    let _address = vm.stack.pop()?;
    vm.stack.push(U256::from(9999999))?;
    vm.pc += 1;
    Ok(())
}

pub fn origin(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(2)?;
    vm.stack.push(vm.sender.into_word().into())?;
    vm.pc += 1;
    Ok(())
}

// pub fn caller(vm: &mut EVM) -> Result<(), EvmError> {
//     vm.gas_dec(2)?;
//     vm.stack.push("0x414b60745072088d013721b4a28a0559b1A9d213")?;
//     vm.pc += 1;
//     Ok(())
// }

pub fn callvalue(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(2)?;
    vm.stack.push(vm.value)?;
    vm.pc += 1;
    Ok(())
}
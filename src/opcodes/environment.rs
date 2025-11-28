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

pub fn call_value(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(2)?;
    vm.stack.push(vm.value)?;
    vm.pc += 1;
    Ok(())
}

pub fn call_data_load(vm: &mut EVM) -> Result<(), EvmError> {
    todo!()
}

pub fn call_data_size(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(2)?;
    vm.stack.push(U256::from(vm.calldata.len()))?;
    vm.pc += 1;

    Ok(())
}

pub fn call_data_copy(vm: &mut EVM) -> Result<(), EvmError> {
    todo!()
}

pub fn code_size(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(2)?;
    vm.stack.push(U256::from(vm.program.len()))?;
    vm.pc += 1;

    Ok(())
}

pub fn code_copy(vm: &mut EVM) -> Result<(), EvmError> {
    todo!()
}

pub fn gas_price(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(2)?;
    vm.stack.push(U256::ZERO)?;
    vm.pc += 1;
    Ok(())
}


pub fn ext_code_copy(vm: &mut EVM) -> Result<(), EvmError> {
    todo!()
}

pub fn return_data_size(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(2)?;
    vm.stack.push(U256::ZERO)?;
    vm.pc += 1;
    Ok(())
}

pub fn return_data_copy(vm: &mut EVM) -> Result<(), EvmError> {
    todo!()
}

pub fn ext_code_hash(vm: &mut EVM) -> Result<(), EvmError> {
    let _ = vm.stack.pop()?;
    vm.gas_dec(2600)?;
    vm.stack.push(U256::ZERO)?;
    vm.pc += 1;
    Ok(())
}


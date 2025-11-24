use alloy_primitives::{I256, U256};

use crate::evm::{EVM, EvmError};

pub fn lt(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(3)?;
    let a = vm.stack.pop()?;
    let b = vm.stack.pop()?;
    let result = if a < b { U256::ONE } else { U256::ZERO };
    vm.stack.push(result)?;
    vm.pc += 1;
    Ok(())
}

pub fn slt(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(3)?;
    let a_raw = vm.stack.pop()?;
    let b_raw = vm.stack.pop()?;
    let a = I256::from_raw(a_raw);
    let b = I256::from_raw(b_raw);
    let result = if a < b { U256::ONE } else { U256::ZERO };
    vm.stack.push(result)?;
    vm.pc += 1;
    Ok(())
}

pub fn gt(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(3)?;
    let a = vm.stack.pop()?;
    let b = vm.stack.pop()?;
    let result = if a > b { U256::ONE } else { U256::ZERO };
    vm.stack.push(result)?;
    vm.pc += 1;
    Ok(())
}

pub fn sgt(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(3)?;
    let a_raw = vm.stack.pop()?;
    let b_raw = vm.stack.pop()?;
    let a = I256::from_raw(a_raw);
    let b = I256::from_raw(b_raw);
    let result = if a > b { U256::ONE } else { U256::ZERO };
    vm.stack.push(result)?;
    vm.pc += 1;
    Ok(())
}

pub fn eq(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(3)?;
    let a = vm.stack.pop()?;
    let b = vm.stack.pop()?;
    let result = if a == b { U256::ONE } else { U256::ZERO };
    vm.stack.push(result)?;
    vm.pc += 1;
    Ok(())
}

pub fn is_zero(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(3)?;
    let value = vm.stack.pop()?;
    let result = if value == U256::ZERO {
        U256::ONE
    } else {
        U256::ZERO
    };
    vm.stack.push(result)?;
    vm.pc += 1;
    Ok(())
}

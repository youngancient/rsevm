use alloy_primitives::{I256, U256};

use crate::evm::{EVM, EvmError};

pub fn add(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(3)?;

    // pop the 2 values
    let a = vm.stack.pop()?;
    let b = vm.stack.pop()?;
    // add them
    vm.stack.push(a.wrapping_add(b))?;
    // increase pc
    vm.pc += 1;
    Ok(())
}

pub fn mul(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(5)?;

    // pop the 2 values
    let a = vm.stack.pop()?;
    let b = vm.stack.pop()?;
    // multiply and push to the stack
    vm.stack.push(a.wrapping_mul(b))?;
    // increase pc
    vm.pc += 1;
    Ok(())
}

pub fn sub(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(3)?;

    // pop the 2 values
    let a = vm.stack.pop()?;
    let b = vm.stack.pop()?;
    // add them
    vm.stack.push(a.wrapping_sub(b))?;
    // increase pc
    vm.pc += 1;
    Ok(())
}

pub fn div(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(5)?;

    // pop the 2 values
    let a = vm.stack.pop()?;
    let b = vm.stack.pop()?;
    // divide and push to the stack
    let result = if b == U256::ZERO { U256::ZERO } else { a / b };
    vm.stack.push(result)?;
    // increase pc
    vm.pc += 1;
    Ok(())
}

pub fn sdiv(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(5)?;

    // pop the 2 values
    let a_raw = vm.stack.pop()?;
    let b_raw = vm.stack.pop()?;

    let a = I256::from_raw(a_raw);
    let b = I256::from_raw(b_raw);

    // multiply and push to the stack
    let result = if b == I256::ZERO {
        I256::ZERO
    } else if a == I256::MIN && b == I256::MINUS_ONE {
        I256::MIN
    } else {
        a / b
    };

    vm.stack.push(result.into_raw())?;
    // // increase pc
    vm.pc += 1;
    Ok(())
}

pub fn vm_mod(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(5)?;
    // pop the 2 values
    let a = vm.stack.pop()?;
    let b = vm.stack.pop()?;
    let result = if b == U256::ZERO { U256::ZERO } else { a % b };
    // push result to Evm stack
    vm.stack.push(result)?;
    // increase pc
    vm.pc += 1;
    Ok(())
}

pub fn smod(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(5)?;
    // pop the 2 values
    let a_raw = vm.stack.pop()?;
    let b_raw = vm.stack.pop()?;
    //  to signed
    let a = I256::from_raw(a_raw);
    let b = I256::from_raw(b_raw);

    if b == I256::ZERO {
        vm.stack.push(U256::ZERO)?;
        return Ok(());
    }

    let result = a % b;
    vm.stack.push(result.into_raw())?;
    Ok(())
}

pub fn add_mod(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(8)?;
    // pop the 2 values
    let a = vm.stack.pop()?;
    let b = vm.stack.pop()?;
    let n = vm.stack.pop()?;
    let addition_result = a.wrapping_add(b);
    let result = if n == U256::ZERO {
        U256::ZERO
    } else {
        addition_result % n
    };

    vm.stack.push(result)?;
    vm.pc += 1;
    Ok(())
}

pub fn mul_mod(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(8)?;
    // pop the 2 values
    let a = vm.stack.pop()?;
    let b = vm.stack.pop()?;
    let n = vm.stack.pop()?;
    let mul_result = a.wrapping_mul(b);
    let result = if n == U256::ZERO {
        U256::ZERO
    } else {
        mul_result % n
    };

    vm.stack.push(result)?;
    vm.pc += 1;
    Ok(())
}

pub fn size_in_bytes(num: &U256) -> u64 {
    let bits = num.bit_len() as u64;
    (bits + 7) / 8
}

pub fn exp(vm: &mut EVM) -> Result<(), EvmError> {
    // pop the 2 values
    let base = vm.stack.pop()?;
    let exponent = vm.stack.pop()?;

    // calculate gas
    let exponent_byte_len = size_in_bytes(&exponent);
    let gas_cost = 10 +  (50 * exponent_byte_len);
    vm.gas_dec(gas_cost)?;

    let result = base.pow(exponent);

    vm.stack.push(result)?;

    vm.pc += 1;

    Ok(())
}

pub fn signextend(vm: &mut EVM) -> Result<(), EvmError> {
    todo!()
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_size_in_bytes() {
        assert_eq!(size_in_bytes(&U256::from(12345)), 2);
        assert_eq!(size_in_bytes(&U256::ZERO), 0);
    }
}

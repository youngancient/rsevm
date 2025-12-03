use alloy_primitives::U256;

use crate::evm::{EVM, EvmError};

const OP_JUMPDEST: u8 = 0x5B;

pub fn jump(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(8)?;
    let dest_raw = vm.stack.pop()?;
    let dest = dest_raw.saturating_to::<usize>();

    if dest >= vm.program.len() {
        return Err(EvmError::BadJumpDestination {
            dest,
            reason: "Out of bound".to_string(),
        });
    }
    // We look at the byte at that specific index to see if it is a JUMPDEST
    if vm.program[dest] != OP_JUMPDEST {
        return Err(EvmError::BadJumpDestination {
            dest,
            reason: "Not a JUMPDEST opcode".to_string(),
        });
    }
    vm.pc = dest;
    Ok(())
}

// this is a conditional jump, known as jump-if
pub fn jumpi(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(10)?;

    let dest_raw = vm.stack.pop()?;
    let dest = dest_raw.saturating_to::<usize>();
    let condition = vm.stack.pop()?;
    // if condition  is not zero, that means true => jump to dest
    if condition != U256::ZERO {
        // in jumping to dest, we must check that it is safe, e.g dest is not greater than vm.program
        if dest >= vm.program.len() {
            return Err(EvmError::BadJumpDestination {
                dest,
                reason: "Out of bound".to_string(),
            });
        }
        if vm.program[dest] != OP_JUMPDEST {
            return Err(EvmError::BadJumpDestination {
                dest,
                reason: "Not a JUMPDEST opcode".to_string(),
            });
        }
        vm.pc = dest;
    } else {
        // if condition  is zero, that means false => don't just, progress normally to the next instruction
        vm.pc += 1;
    }

    Ok(())
}

// this opcode bookmarks the last instruction execution line so the EVM can find its way back
// suppose that there's a function A which calls another function B within it, the line where B call is initiated is saved on the stack
// sortof like saving its frame of reference, so after B is executed , it has a record of where it was and simple Jumps back
pub fn pc(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(2)?;
    vm.stack.push(U256::from(vm.pc))?;
    vm.pc += 1;
    Ok(())
}

// this opcode does nothing, it's a no-operation opcode.
// It doesn't pop anything, it doesn't push anything, and it doesn't change memory.
// it exists solely for static analysis
// it's the opcode that clearly indicates which part of an instruction the EVM should jump to
// JUMP and JUMPI are dependent on it
pub fn jump_dest(vm: &mut EVM) -> Result<(), EvmError> {
    vm.gas_dec(1)?;
    vm.pc += 1;
    Ok(())
}
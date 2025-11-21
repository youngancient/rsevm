use crate::evm::EVM;

pub fn stop(machine: &mut EVM) {
    machine.stop_flag = true;
}

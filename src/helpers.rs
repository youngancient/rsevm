pub fn get_supported_opcode_name(op: u8) -> String {
    match op {
        // Stop & Arithmetic
        0x00 => "STOP".to_string(),
        0x01 => "ADD".to_string(),
        0x02 => "MUL".to_string(),
        0x03 => "SUB".to_string(),
        0x04 => "DIV".to_string(),
        0x05 => "SDIV".to_string(),
        0x06 => "MOD".to_string(),
        0x07 => "SMOD".to_string(),
        0x08 => "ADDMOD".to_string(),
        0x09 => "MULMOD".to_string(),
        0x0A => "EXP".to_string(),
        0x0B => "SIGNEXTEND".to_string(),

        // Comparison
        0x10 => "LT".to_string(),
        0x11 => "GT".to_string(),
        0x12 => "SLT".to_string(),
        0x13 => "SGT".to_string(),
        0x14 => "EQ".to_string(),
        0x15 => "ISZERO".to_string(),

        // Logical & Bitwise
        0x16 => "AND".to_string(),
        0x17 => "OR".to_string(),
        0x18 => "XOR".to_string(),
        0x19 => "NOT".to_string(),
        0x1A => "BYTE".to_string(),
        0x1B => "SHL".to_string(),
        0x1C => "SHR".to_string(),
        0x1D => "SAR".to_string(),

        // SHA3
        0x20 => "SHA3".to_string(),

        // Environment
        0x30 => "ADDRESS".to_string(),
        0x31 => "BALANCE".to_string(),
        0x32 => "ORIGIN".to_string(),
        0x33 => "CALLER".to_string(),
        0x34 => "CALLVALUE".to_string(),
        0x35 => "CALLDATALOAD".to_string(),
        0x36 => "CALLDATASIZE".to_string(),
        0x37 => "CALLDATACOPY".to_string(),
        0x38 => "CODESIZE".to_string(),
        0x39 => "CODECOPY".to_string(),
        0x3A => "GASPRICE".to_string(),
        0x3B => "EXTCODESIZE".to_string(),
        0x3C => "EXTCODECOPY".to_string(),
        0x3D => "RETURNDATASIZE".to_string(),
        0x3E => "RETURNDATACOPY".to_string(),
        0x3F => "EXTCODEHASH".to_string(),

        // Commented out for now
        // 0x40 => "BLOCKHASH".to_string(),
        // 0x41 => "COINBASE".to_string(),
        // 0x42 => "TIMESTAMP".to_string(),
        // 0x43 => "NUMBER".to_string(),
        // 0x44 => "DIFFICULTY".to_string(),
        // 0x45 => "GASLIMIT".to_string(),
        // 0x46 => "CHAINID".to_string(),
        // 0x47 => "SELFBALANCE".to_string(),
        // 0x48 => "BASEFEE".to_string(),

        // Stack Memory Storage Flow
        0x50 => "POP".to_string(),
        0x51 => "MLOAD".to_string(),
        0x52 => "MSTORE".to_string(),
        0x53 => "MSTORE8".to_string(),
        0x54 => "SLOAD".to_string(),
        0x55 => "SSTORE".to_string(),
        0x56 => "JUMP".to_string(),
        0x57 => "JUMPI".to_string(),
        0x58 => "PC".to_string(),
        // 0x59 => "MSIZE".to_string(),
        0x5A => "GAS".to_string(),
        0x5B => "JUMPDEST".to_string(),
        0x5C => "TLOAD".to_string(),
        0x5D => "TSTORE".to_string(),

        // PUSH 1..32
        0x60..=0x7F => format!("PUSH{}", op - 0x60 + 1),

        // DUP 1..16
        0x80..=0x8F => format!("DUP{}", op - 0x80 + 1),

        // SWAP 1..16
        0x90..=0x9F => format!("SWAP{}", op - 0x90 + 1),

        // LOG 0..4
        0xA0..=0xA4 => format!("LOG{}", op - 0xA0),

        // System
        // 0xF0 => "CREATE".to_string(),
        // 0xF1 => "CALL".to_string(),
        // 0xF2 => "CALLCODE".to_string(),
        // 0xF3 => "RETURN".to_string(),
        // 0xF4 => "DELEGATECALL".to_string(),
        // 0xF5 => "CREATE2".to_string(),
        // 0xFA => "STATICCALL".to_string(),
        // 0xFD => "REVERT".to_string(),
        // 0xFE => "INVALID".to_string(),
        // 0xFF => "SELFDESTRUCT".to_string(),

        _ => format!("UNKNOWN(0x{:02x})", op),
    }
}
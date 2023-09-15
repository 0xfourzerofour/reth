//! Opcount tracing inspector that simply counts all opcodes.
//!
//! See also <https://geth.ethereum.org/docs/developers/evm-tracing/built-in-tracers>

use reth_primitives::Address;
use revm::{
    interpreter::{InstructionResult, Interpreter},
    Database, EVMData, Inspector,
};

const DEPOSIT_TO_SELECTOR: &str = "0xb760faf9";

const FORBIDDEN_OPCODES: [&str; 13] = [
    "GASPRICE",
    "GASLIMIT",
    "DIFFICULTY",
    "TIMESTAMP",
    "BASEFEE",
    "BLOCKHASH",
    "NUMBER",
    "SELFBALANCE",
    "BALANCE",
    "ORIGIN",
    "CREATE",
    "COINBASE",
    "SELFDESTRUCT",
];

// If you add any opcodes to this list, make sure they take the contract
// address as their *second* argument, or modify the handling below.
const CALL_OPCODES: [&str; 4] = ["CALL", "CALLCODE", "DELEGATECALL", "STATICCALL"];

// If you add any opcodes to this list, make sure they take the contract
// address as their *first* argument, or modify the handling below.
const EXT_OPCODES: [&str; 3] = ["EXTCODECOPY", "EXTCODEHASH", "EXTCODELENGTH"];

const PRECOMPILE_WHILTELIST: [&str; 9] = [
    "0x0000000000000000000000000000000000000001", // ecRecover
    "0x0000000000000000000000000000000000000002", // SHA2-256
    "0x0000000000000000000000000000000000000003", // RIPEMD-160
    "0x0000000000000000000000000000000000000004", // identity
    "0x0000000000000000000000000000000000000005", // modexp
    "0x0000000000000000000000000000000000000006", // ecAdd
    "0x0000000000000000000000000000000000000007", // ecMul
    "0x0000000000000000000000000000000000000008", // ecPairing
    "0x0000000000000000000000000000000000000009", // black2f
];

/// An inspector that counts all opcodes.
#[derive(Debug, Clone, Copy, Default)]
pub struct AAInspector {
    /// opcode counter
    entrypoint: Address,
    out_of_gas: bool,
}

impl AAInspector {
    /// Returns the opcode counter
    pub fn entrypoint(&self) -> Address {
        self.entrypoint
    }
}

impl<DB> Inspector<DB> for AAInspector
where
    DB: Database,
{
    fn initialize_interp(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> InstructionResult {
        self.entrypoint = Address::zero();
        self.out_of_gas = false;

        InstructionResult::Continue
    }

    fn step(
        &mut self,
        interp: &mut Interpreter,
        data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> InstructionResult {
        if self.entrypoint.is_zero() {
            self.entrypoint = interp.contract.address;
        }

        if interp.gas.remaining() < interp.gas().spend() {
            self.out_of_gas = true;
        }

        let opcode = interp.current_opcode();

        InstructionResult::Continue
    }
}

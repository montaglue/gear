//! A simple example of `create_program` sys-call.
//!
//! The program is mainly used for testing the sys-call logic in pallet `gear` tests.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
#[cfg(not(feature = "std"))]
use gstd::prelude::*;

#[cfg(feature = "std")]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}

#[cfg(feature = "std")]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub enum CreateProgram {
    Default,
    // code hash, salt, gas limit
    Custom(Vec<([u8; 32], Vec<u8>, u64)>),
}

#[allow(unused)]
const CHILD_CODE_HASH: [u8; 32] = hex_literal::hex!(
    "abf3746e72a6e8740bd9e12b879fbdd59e052cb390f116454e9116c22021ae4a"
);

#[cfg(not(feature = "std"))]
mod wasm {
    use gstd::{debug, msg, prog, CodeHash};

    use super::{CreateProgram, CHILD_CODE_HASH};

    static mut COUNTER: i32 = 0;

    #[no_mangle]
    pub unsafe extern "C" fn handle() {
        match msg::load().expect("provided invalid payload") {
            CreateProgram::Default => {
                let submitted_code = CHILD_CODE_HASH.into();
                let new_program_id = prog::create_program_with_gas(
                    submitted_code,
                    COUNTER.to_le_bytes(),
                    [],
                    100_000,
                    0,
                );
                msg::send_with_gas(new_program_id, b"", 100_001, 0);

                COUNTER += 1;
            }
            CreateProgram::Custom(custom_child_data) => {
                for (code_hash, salt, gas_limit) in custom_child_data {
                    let submitted_code = code_hash.into();
                    let new_program_id =
                        prog::create_program_with_gas(submitted_code, &salt, [], gas_limit, 0);
                    let msg_id = msg::send_with_gas(new_program_id, b"", 100_001, 0);
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use gtest::{System, Program};

    use super::*;

    // Creates a new factory and initializes it.
    fn prepare_factory<'a>(sys: &'a System) -> Program<'a> {
        // Store child
        let code_hash_stored = sys.submit_code("./child_contract.wasm");
        assert_eq!(code_hash_stored.inner(), CHILD_CODE_HASH);

        // Instantiate factory
        let factory = Program::current_with_id(sys, 100);
        // Send `init` msg to factory
        let res = factory.send_bytes(10001, "EMPTY");
        assert!(!res.main_failed());
        assert_eq!(res.initialized_programs().len(), 1);

        factory
    }

    #[test]
    fn test_simple() {
        let sys = System::new();
        let factory = prepare_factory(&sys);

        let child_code_hash = CHILD_CODE_HASH.into();
        let child_id_expected = Program::calculate_program_id(child_code_hash, &0i32.to_le_bytes());

        // Send `handle` msg to factory to create a new child
        let res = factory.send_bytes(10001, CreateProgram::Default.encode());
        assert!(!res.main_failed());
        assert!(!res.others_failed());
        assert_eq!(res.initialized_programs().len(), 2);

        let (new_actor_id_actual, new_actor_code_hash) = res.initialized_programs().last().copied().unwrap();
        assert_eq!(child_id_expected, new_actor_id_actual);
        assert_eq!(Some(child_code_hash), new_actor_code_hash);
    }

    #[test]
    fn test_duplicate() {
        let sys = System::new();
        let factory = prepare_factory(&sys);

        let first_call_salt = 0i32.to_le_bytes();

        // Send `handle` msg to factory to create a new child
        let res = factory.send_bytes(10001, CreateProgram::Default.encode());
        assert_eq!(res.initialized_programs().len(), 2);

        // Duplicate
        let payload = CreateProgram::Custom(vec![(CHILD_CODE_HASH, first_call_salt.to_vec(), 100_000)]);
        let res = factory.send_bytes(10001, payload.encode());
        assert!(!res.main_failed());
        assert!(!res.others_failed());
        // No new programs!
        assert_eq!(res.initialized_programs().len(), 2);
    }

    #[test]
    fn test_non_existing_code_hash() {
        let sys = System::new();
        let factory = prepare_factory(&sys);

        // Send `handle` msg to factory to create a new child
        factory.send_bytes(10001, CreateProgram::Default.encode());

        env_logger::init();

        // Non existing code hash provided
        let non_existing_code_hash = [10u8; 32];
        let salt = b"some_salt";
        let fictional_program_id = Program::calculate_program_id(non_existing_code_hash.into(), salt);
        let payload = CreateProgram::Custom(
            vec![(non_existing_code_hash, salt.to_vec(), 100_000)]
        );
        let res = factory.send_bytes(10001, payload.encode());
        assert!(!res.main_failed());
        // No new program with fictional id
        assert!(!res.initialized_programs().iter().any(|(p_id, _)| p_id == &fictional_program_id));
    }
}

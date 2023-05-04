#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

// EXPORTS
// ================================================================================================
use crate::utils::string::String;
use crate::utils::collections::Vec;
pub use assembly::{Assembler, AssemblyError, ParsingError};
pub use processor::{
    crypto, execute, execute_iter, utils, AdviceInputs, AdviceProvider, AsmOpInfo, ExecutionError,
    ExecutionTrace, Kernel, MemAdviceProvider, Operation, ProgramInfo, StackInputs, VmState,
    VmStateIterator,
};
pub use prover::{
    math, prove, Digest, ExecutionProof, FieldExtension, HashFunction, InputError, Program,
    ProofOptions, StackOutputs, StarkProof, Word,
};
use serde::{Deserialize, Serialize};
pub use verifier::{verify, VerificationError};
extern crate wasm_bindgen;
use vm_core::Felt;
// use wasm_bindgen::prelude::*;
// use wasm_bindgen_test::console_log;
use winterfell::{Deserializable, SliceReader};
// extern crate console_error_panic_hook;

#[derive(Debug, Serialize, Deserialize)]
pub struct NormalInput {
    pub stack_inputs: StackInputs,
    pub advice_provider: MemAdviceProvider,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VMResult {
    pub outputs: StackOutputs,
    pub starkproof: ExecutionProof,
}

// #[wasm_bindgen]
pub fn execute_zk_program(program_code: String, stack_init: String, advice_tape: String) -> String {
    let options = ProofOptions::with_96_bit_security();

    let assembler = Assembler::default().with_library(&stdlib::StdLibrary::default()).unwrap();

    let program = assembler.compile(&program_code).unwrap();

    let inputs: NormalInput = convert_stackinputs(stack_init, advice_tape);

    let res = prove(&program, inputs.stack_inputs, inputs.advice_provider, options);

    assert!(res.is_ok(), "The proof generation fails: {:?}", res);

    let (outputs, proof) = res.unwrap();

    let result = VMResult {
        outputs,
        starkproof: proof,
    };

    let final_result: String = serde_json::to_string(&result).unwrap();
    return final_result;
}

// #[wasm_bindgen]
pub fn generate_program_hash(program_in_assembly: String) -> String {
    let assembler = Assembler::default().with_library(&stdlib::StdLibrary::default()).unwrap();
    let program = assembler.compile(&program_in_assembly).unwrap();
    use vm_core::utils::Serializable;
    let program_hash = program.hash().to_bytes();
    let ph = hex::encode(program_hash);
    return ph;
}

pub fn convert_stackinputs(stack_init: String, advice_tape: String) -> NormalInput {
    let mut stack_inita = Vec::new();
    let mut advice_tapea = Vec::new();
    if stack_init.len() != 0 {
        let stack_init: Vec<&str> = stack_init.split(',').collect();
        stack_inita = stack_init
            .iter()
            .map(|stack_init| Felt::new(stack_init.parse::<u64>().unwrap()))
            .collect();
    };

    if advice_tape.len() != 0 {
        let advice_tape: Vec<&str> = advice_tape.split(',').collect();
        advice_tapea = advice_tape
            .iter()
            .map(|advice_tape| advice_tape.parse::<u64>().unwrap())
            .collect();
    };

    let stack_input: StackInputs = StackInputs::new(stack_inita);

    let advice_inputs = AdviceInputs::default().with_stack_values(advice_tapea).unwrap();

    let mem_advice_provider: MemAdviceProvider = MemAdviceProvider::from(advice_inputs);

    let inputs = NormalInput {
        stack_inputs: stack_input,
        advice_provider: mem_advice_provider,
    };

    return inputs;
}

// #[wasm_bindgen]
pub fn verify_zk_program(program_hash: String, stack_inputs: String, zk_outputs: VMResult) -> u32 {
    let mut stack_inita = Vec::new();
    if stack_inputs.len() != 0 {
        let stack_init: Vec<&str> = stack_inputs.split(',').collect();
        stack_inita = stack_init
            .iter()
            .map(|stack_init| Felt::new(stack_init.parse::<u64>().unwrap()))
            .collect();
    };
    let stack_input: StackInputs = StackInputs::new(stack_inita);
    // let zk_outputs: VMResult = serde_json::from_str(&final_result).unwrap();

    let bytes = hex::decode(program_hash).unwrap();
    assert_eq!(32, bytes.len());

    let mut reader = SliceReader::new(&bytes);
    let program_digest = Digest::read_from(&mut reader).unwrap();

    let kernel = Kernel::default();
    let program_info = ProgramInfo::new(program_digest, kernel);

    let security_level =
        verify(program_info, stack_input, zk_outputs.outputs, zk_outputs.starkproof).unwrap();
    return security_level;
}

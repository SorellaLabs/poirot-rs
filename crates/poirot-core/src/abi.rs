use std::collections::HashMap;
use std::path::PathBuf;
use alloy_sol_types::decode;
use alloy_json_abi::Function;
use serde_json::Value;
use reth_rpc_types::trace::parity::{Action as RethAction, LocalizedTransactionTrace};
use revm_primitives::bits::B160;

pub struct ContractAbiStorage<'a> {
    mapping: HashMap<&'a B160, PathBuf>,
}

impl<'a> ContractAbiStorage<'a> {
    pub fn new() -> Self {
        Self {
            mapping: HashMap::new(),
        }
    }

    pub fn add_abi(&mut self, contract_address: &'a B160, abi_path: PathBuf) {
        self.mapping.insert(contract_address, abi_path);
    }

    pub fn get_abi(&self, contract_address: &'a B160) -> Option<&PathBuf> {
        self.mapping.get(contract_address)
    }
}

pub fn sleuth<'a>(
    storage: &'a ContractAbiStorage,
    trace: LocalizedTransactionTrace,
) -> Result<String, Box<dyn std::error::Error>> {
    let action = trace.trace.action;

    let (contract_address, input) = match action {
        RethAction::Call(call_action) => (call_action.to, call_action.input.to_vec()),
        _ => return Err(From::from("The action in the transaction trace is not Call(CallAction)")),
    };

    let contract_address = contract_address;
    let abi_path = storage
        .get_abi(&contract_address)
        .ok_or("No ABI found for this contract")?;


    // Read the JSON ABI file
    let file = std::fs::File::open(abi_path)?;
    let reader = std::io::BufReader::new(file);
    
    // Parse the JSON ABI
    let json_abi: Value = serde_json::from_reader(reader)?;

    // Extract function selectors from the ABI
    let mut function_selectors = HashMap::new();
    if let Value::Array(functions) = json_abi.get("functions").unwrap() {
        for function in functions {
            let function: Function = serde_json::from_value(function.clone())?;
            function_selectors.insert(function.selector(), function);
        }
    }

    // Extract the function selector from the input
    let function_selector = &input[..4];

    // Find the matching function in the ABI
    let function = function_selectors
        .get(function_selector)
        .ok_or("No matching function found in the ABI")?;

    // Decode the input data
    let decoded = decode::<Vec<Value>>(&input, true)?;

    // Convert the decoded input to a string for printing
    let printout = format!("{:?}", decoded);

    Ok(printout)
}

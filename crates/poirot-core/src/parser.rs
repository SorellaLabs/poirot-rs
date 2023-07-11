use crate::action::{Action, ActionType, Deposit, PoolCreation, Swap, Transfer, Withdrawal};

use reth_rpc_types::trace::parity::{Action as RethAction, LocalizedTransactionTrace};

use alloy_json_abi::JsonAbi;
use alloy_sol_types::{sol, SolCall};
use reth_primitives::{hex_literal::hex, H160};
use reth_revm::precompile::primitives::ruint::Uint;

use std::collections::HashMap;
use std::path::PathBuf;

use ethers::abi::{Abi, Token};

pub struct Parser {
    block_trace: Vec<LocalizedTransactionTrace>,
    pub store: HashMap<H160, PathBuf>,
}

impl Parser {
    pub fn new(block_trace: Vec<LocalizedTransactionTrace>) -> Self {
        Self { block_trace, store: HashMap::new() }
    }

    pub fn parse(&self) -> Vec<Result<Vec<Token>, ()>> {
        let mut actions = vec![];

        for trace in self.block_trace {
            actions.push(self.parse_trace(&trace));
        }

        actions
    }

   pub fn parse_trace(&self, trace: &LocalizedTransactionTrace) -> Result<Vec<Token>, ()>{
        let action = match trace.trace.action {
            RethAction::Call(call) => call,
            _ => return Err(()),
        };
 
        let file = std::fs::File::open(self.store.get(&action.to)?)?;
        let abi = Abi::load(std::io::BufReader::new(file));

        let mut function_selectors = HashMap::new();

        for function in abi.unwrap().functions() {
            function_selectors.insert(function.short_signature(), function);
        }

        let input_selector = &action.input[..4];

        let function = function_selectors
            .get(input_selector);

        Ok(function?.decode_input(&action.input.to_vec())?)
   }
}

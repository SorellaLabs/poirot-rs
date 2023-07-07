use crate::action::{Action, ActionType, Transfer, PoolCreation};

use reth_rpc_types::trace::parity::{Action as RethAction, LocalizedTransactionTrace};

use alloy_sol_types::{sol, SolCall};
use reth_primitives::{hex_literal::hex, H160};

use std::cell::Cell;

sol! {
    /// Interface of the ERC20 standard as defined in [the EIP].
    ///
    /// [the EIP]: https://eips.ethereum.org/EIPS/eip-20
    #[derive(Debug, PartialEq)]
    interface IERC20 {
        function transfer(address to, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
    }
}

sol! {
    interface IUniswapV3Factory {
        function createPool(address tokenA, address tokenB, uint24 fee) external returns (address pool);
    }
}

pub struct Parser {
    block_trace: Vec<LocalizedTransactionTrace>,
}

impl Parser {
    pub fn new(block_trace: Vec<LocalizedTransactionTrace>) -> Self {
        Self { block_trace }
    }

    pub fn parse(&self) -> Vec<Action> {
        let mut actions = vec![];

        for i in self.block_trace.clone() {
            let parsed = self.parse_trace(&i);

            if parsed.is_some() {
                actions.push(parsed.unwrap());
            } else {
                actions.push(Action {
                    ty: ActionType::Unclassified(i.clone()),
                    hash: i.clone().transaction_hash.unwrap(),
                    block: i.clone().block_number.unwrap(),
                });
            }
        }

        actions
    }

    /// Parse a single transaction trace.
    pub fn parse_trace(&self, curr: &LocalizedTransactionTrace) -> Option<Action> {
        self.parse_transfer(curr)
            .or_else(|| self.parse_pool_creation(curr))
    }

    pub fn parse_transfer(&self, curr: &LocalizedTransactionTrace) -> Option<Action> {
        match &curr.trace.action {
            RethAction::Call(call) => {
                let mut decoded = match IERC20::IERC20Calls::decode(&call.input.to_vec(), true) {
                    Ok(decoded) => decoded,
                    Err(_) => return None,
                };

                match decoded {
                    IERC20::IERC20Calls::transfer(transfer_call) => {
                        return Some(Action {
                            ty: ActionType::Transfer(Transfer::new(transfer_call.to, transfer_call.amount.into(), call.to)),
                            hash: curr.transaction_hash.unwrap(),
                            block: curr.block_number.unwrap(),
                        })
                    }
                    IERC20::IERC20Calls::transferFrom(transfer_from_call) => {
                        return Some(Action {
                            ty: ActionType::Transfer(Transfer::new(transfer_from_call.to, transfer_from_call.amount.into(), call.to)),
                            hash: curr.transaction_hash.unwrap(),
                            block: curr.block_number.unwrap(),
                        })
                    }
                    _ => return None,
                }
            }
            _ => None,
        }
    }

    pub fn parse_pool_creation(&self, curr: &LocalizedTransactionTrace) -> Option<Action> {
        match &curr.trace.action {
            RethAction::Call(call) => {
                let mut decoded = match IUniswapV3Factory::decode(&call.input.to_vec(), true) {
                    Ok(decoded) => decoded,
                    Err(_) => return None,
                };

                match decoded {
                    IUniswapV3Factory::createPool(create_pool_call) => {
                        return Some(Action {
                            ty: ActionType::PoolCreation(PoolCreation::new(create_pool_call.tokenA, create_pool_call.tokenB, create_pool_call.fee)),
                            hash: curr.transaction_hash.unwrap(),
                            block: curr.block_number.unwrap(),
                        })
                    }
                    _ => return None,
                }
            }
            _ => None,
        }
    }
}

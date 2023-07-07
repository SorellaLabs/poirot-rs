use crate::action::{Action, ActionType, Deposit, PoolCreation, Transfer, Withdrawal};

use reth_rpc_types::trace::parity::{Action as RethAction, LocalizedTransactionTrace};

use alloy_sol_types::{sol, SolCall};
use reth_primitives::{hex_literal::hex, H160};

use std::cell::Cell;

sol! {
    #[derive(Debug, PartialEq)]
    interface IERC20 {
        function transfer(address to, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
    }
}

sol! {
    #[derive(Debug, PartialEq)]
    interface IUniswapV3Factory {
        function createPool(address tokenA, address tokenB, uint24 fee) external returns (address);
    }
}

sol! {
    #[derive(Debug, PartialEq)]
    interface WETH9 {
        function deposit() public payable;
        function withdraw(uint wad) public;
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
            .or_else(|| self.parse_weth(curr))
    }

    pub fn parse_weth(&self, curr: &LocalizedTransactionTrace) -> Option<Action> {
        match &curr.trace.action {
            RethAction::Call(call) => {
                let mut decoded = match WETH9::WETH9Calls::decode(&call.input.to_vec(), true) {
                    Ok(decoded) => decoded,
                    Err(_) => return None,
                };

                match decoded {
                    WETH9::WETH9Calls::deposit(deposit_call) => {
                        return Some(Action {
                            ty: ActionType::WethDeposit(Deposit::new(call.from, call.value.into())),
                            hash: curr.transaction_hash.unwrap(),
                            block: curr.block_number.unwrap(),
                        })
                    }
                    WETH9::WETH9Calls::withdraw(withdraw_call) => {
                        return Some(Action {
                            ty: ActionType::WethWithdraw(Withdrawal::new(
                                call.from,
                                withdraw_call.wad,
                            )),
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
                            ty: ActionType::Transfer(Transfer::new(
                                transfer_call.to,
                                transfer_call.amount.into(),
                                call.to,
                            )),
                            hash: curr.transaction_hash.unwrap(),
                            block: curr.block_number.unwrap(),
                        })
                    }
                    IERC20::IERC20Calls::transferFrom(transfer_from_call) => {
                        return Some(Action {
                            ty: ActionType::Transfer(Transfer::new(
                                transfer_from_call.to,
                                transfer_from_call.amount.into(),
                                call.to,
                            )),
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
                let mut decoded =
                    match IUniswapV3Factory::createPoolCall::decode(&call.input.to_vec(), true) {
                        Ok(decoded) => decoded,
                        Err(_) => return None,
                    };

                return Some(Action {
                    ty: ActionType::PoolCreation(PoolCreation::new(
                        decoded.tokenA,
                        decoded.tokenB,
                        decoded.fee,
                    )),
                    hash: curr.transaction_hash.unwrap(),
                    block: curr.block_number.unwrap(),
                })
            }
            _ => None,
        }
    }
}

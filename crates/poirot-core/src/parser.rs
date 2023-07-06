use crate::action::{Action, ActionType, Transfer};

use reth_rpc_types::trace::parity::{Action as RethAction, LocalizedTransactionTrace};

use alloy_sol_types::{sol, SolCall};
use alloy_json_abi::JsonAbi;
use reth_primitives::{hex_literal::hex, H160};

use std::cell::Cell;

sol! {
    /// Interface of the ERC20 standard as defined in [the EIP].
    ///
    /// [the EIP]: https://eips.ethereum.org/EIPS/eip-20
    #[derive(Debug, PartialEq)]
    interface IERC20 {
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);

        function totalSupply() external view returns (uint256);
        function balanceOf(address account) external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
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
            let parsed = self.parse_transfer(&i);

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

    /// Parse a token transfer.
    pub fn parse_transfer(&self, curr: &LocalizedTransactionTrace) -> Option<Action> {
        match &curr.trace.action {
            RethAction::Call(call) => {
                let mut decoded = match IERC20::IERC20Calls::decode(&call.input.to_vec(), true) {
                    Ok(decoded) => decoded,
                    Err(_) => return None,
                };

                match decoded {
                    IERC20::IERC20Calls::transfer(transfer_call) => {
                        let transfer = Transfer {
                            to: transfer_call.to,
                            amount: transfer_call.amount.into(),
                            token: call.to,
                        };
                    }
                    IERC20::IERC20Calls::transferFrom(transfer_from_call) => {
                        let transfer = Transfer {
                            to: transfer_from_call.to,
                            amount: transfer_from_call.amount.into(),
                            token: call.to,
                        };
                    }
                    _ => return None,
                }

                return Some(Action {
                    ty: ActionType::None,
                    hash: curr.transaction_hash.unwrap(),
                    block: curr.transaction_position.unwrap(),
                })
            }
            _ => None,
        }
    }


    


}



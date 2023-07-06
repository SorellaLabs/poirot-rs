use crate::action::Action;
use crate::action::ActionType;
use crate::action::Transfer;

use reth_rpc_types::trace::parity::{Action as RethAction};
use reth_rpc_types::trace::parity::LocalizedTransactionTrace;

use alloy_sol_types::{sol, SolCall};
use reth_primitives::hex_literal::hex;
use reth_primitives::H160;

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
    cursor: Cell<usize>,
    block_trace: Vec<LocalizedTransactionTrace>,
}

impl Parser {
    pub fn new(block_trace: Vec<LocalizedTransactionTrace>) -> Self {
        Self { cursor: Cell::new(0), block_trace }
    }

    pub fn parse(&self) -> Vec<Action> {
        let mut actions = vec![];

        let parsed = self.parse_transfer();

        if parsed.is_some() {
            actions.push(parsed.unwrap());
        } else {
            actions.push(Action {
                ty: ActionType::Unclassified(self.current().clone()),
                hash: self.current().transaction_hash.unwrap(),
                block: self.current().block_number.unwrap(),
            });
        }

        actions
    }

    /// Advance the parser forwards one step, ready to parse the next token.
    pub fn advance(&self) {
        let mut curr = self.cursor.get();
        curr += 1;
        self.cursor.set(curr);
    }

    /// Collect the current transaction from the parser.
    pub fn current(&self) -> &LocalizedTransactionTrace {
        &self.block_trace[self.cursor.get()]
    }

    /// Parse a token transfer.
    pub fn parse_transfer(&self) -> Option<Action> {
        let curr = self.current().clone();

        self.advance();

        match curr.trace.action {
            RethAction::Call(call) => {
                let mut decoded = match IERC20::IERC20Calls::decode(&call.input.to_vec(), true) {
                    Ok(decoded) => decoded,
                    Err(_) => return None,
                };

                match decoded {
                    IERC20::IERC20Calls::transfer(transfer_call) => {
                        transfer_call.to.hello;
                        let transfer = Transfer {
                            to: transfer_call.to.into(),
                            amount: transfer_call.amount.into(),
                            token: call.to,
                        };
                    },
                    IERC20::IERC20Calls::transferFrom(transfer_from_call) => {
                        let transfer = Transfer {
                            to: transfer_from_call.to.into(),
                            amount: transfer_from_call.amount.into(),
                            token: call.to,
                        };
                    },
                    _ => return None,
                }

                return Some(Action {
                    ty: ActionType::None,
                    hash: curr.transaction_hash.unwrap(),
                    block: curr.transaction_position.unwrap(),
                });
            }
            _ => None,
        }
    }
}
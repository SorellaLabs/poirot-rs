use crate::action::Action;
use crate::action::ActionType;
use crate::action::Transfer;

use reth_rpc_types::trace::parity::{Action as RethAction};
use reth_rpc_types::trace::parity::LocalizedTransactionTrace;

use alloy_sol_types::{sol, SolCall};
use reth_primitives::hex_literal::hex;

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
                ty: self.current().clone(),
                hash: self.current().transaction_hash,
                block: self.current().block_number,
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
                let decoded = match IERC20::IERC20Calls::decode(&call.input.to_vec(), true) {
                    Ok(decoded) => decoded,
                    Err(_) => return None,
                };

                let transfer = Transfer {
                    to: decoded.to,
                    amount: decoded.amount,
                    token: call.to,
                };

                return Some(Action {
                    ty: ActionType::Transfer(transfer),
                    hash: curr.transaction_hash.unwrap(),
                    block: curr.transaction_position.unwrap(),
                });
            }
            _ => None,
        }
    }
}
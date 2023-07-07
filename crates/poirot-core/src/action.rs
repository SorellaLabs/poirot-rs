use alloy_primitives::{Address, U256};
use reth_primitives::{Bytes, H160, H256};
use reth_revm::precompile::primitives::ruint::Uint;
use reth_rpc_types::trace::parity::LocalizedTransactionTrace;

#[derive(Debug, Clone)]
pub struct Action {
    pub ty: ActionType,
    pub hash: H256,
    pub block: u64,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    Transfer(Transfer),
    PoolCreation(PoolCreation),
    Swap(Swap),
    WethDeposit(Deposit),
    WethWithdraw(Withdrawal),
    Unclassified(LocalizedTransactionTrace),
}

#[derive(Debug, Clone)]
pub struct Withdrawal {
    pub to: H160,
    pub amount: U256,
}

#[derive(Debug, Clone)]
pub struct Deposit {
    pub from: H160,
    pub tokens: Vec<H160>,
    pub amounts: Vec<U256>,
}

#[derive(Debug, Clone)]
pub struct Transfer {
    pub to: H160,
    pub amount: U256,
    pub token: H160,
}

#[derive(Debug, Clone)]
pub struct PoolCreation {
    pub token_0: H160,
    pub token_1: H160,
    pub fee: u32,
}

#[derive(Debug, Clone)]
pub struct Swap {
    pub recipient: H160,
    pub direction: bool,
    pub amount_specified: U256,
    pub price_limit: U256,
    pub data: Vec<u8>,
}

impl Transfer {
    /// Public constructor function to instantiate a [`Transfer`].
    pub fn new(to: H160, amount: U256, token: H160) -> Self {
        Self { to, amount, token }
    }
}

impl PoolCreation {
    /// Public constructor function to instantiate a [`PoolCreation`].
    pub fn new(token_0: H160, token_1: H160, fee: u32) -> Self {
        Self { token_0, token_1, fee }
    }
}

impl Deposit {
    /// Public constructor function to instantiate a [`Deposit`].
    pub fn new(from: H160, tokens: Vec<H160>, amounts: Vec<U256>) -> Self {
        Self { from, tokens, amounts }
    }
}

impl Withdrawal {
    /// Public constructor function to instantiate a [`Withdrawal`].
    pub fn new(to: H160, amount: U256) -> Self {
        Self { to, amount }
    }
}

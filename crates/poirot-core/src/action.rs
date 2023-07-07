use alloy_primitives::Address;
use reth_primitives::{Bytes, H160, H256, U256};
use reth_rpc_types::trace::parity::LocalizedTransactionTrace;

#[derive(Debug, Clone)]
pub struct Action {
    pub ty: ActionType,
    pub hash: H256,
    pub block: u64,
    pub protocol: String,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    Transfer(Transfer),
    PoolCreation(PoolCreation),

    Unclassified(LocalizedTransactionTrace),
    None,
}

#[derive(Debug, Clone)]
pub struct Transfer {
    // pub from: Address,
    pub to: Address,
    pub amount: ruint2::Uint<256, 4>,
    pub token: H160,
}

#[derive(Debug, Clone)]
pub struct PoolCreation {
    pub token_0: Address,
    pub token_1: Address,
    pub fee: u32,
}

impl Transfer {
    /// Public constructor function to instantiate a [`Transfer`].
    pub fn new(to: Address, amount: ruint2::Uint<256, 4>, token: H160) -> Self {
        Self { to, amount, token }
    }
}

impl PoolCreation {
    /// Public constructor function to instantiate a [`PoolCreation`].
    pub fn new(token_0: Address, token_1: Address, fee: u32) -> Self {
        Self { token_0, token_1, fee }
    }
}

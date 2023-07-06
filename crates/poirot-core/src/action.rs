use alloy_primitives::Address;
use reth_primitives::{Bytes, H160, H256, U256};
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

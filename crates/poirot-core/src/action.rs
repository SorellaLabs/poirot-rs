use reth_primitives::{Bytes, Address, H256, U256};
use reth_rpc_types::trace::parity::LocalizedTransactionTrace;

pub struct Action {
    ty: ActionType,
    hash: H256,
    block: u64,    
}

pub enum ActionType {
    Transfer(Transfer),
    Unclassified(LocalizedTransactionTrace),
}

pub struct Transfer {
    // pub from: Address,
    pub to: Address,
    pub amount: U256,
    pub token: Address,
}
use reth_primitives::{Bytes, Address};

pub enum Action {
    Transfer(Transfer),
    Trade(Trade),

    Unclassified(Bytes),
}

pub struct Transfer {
    pub from: Address,
    pub to: Address,
    pub amount: U256,
    pub token: Address,
}

pub struct Trade {
    pub t1: Transfer,
    pub t2: Transfer,
}
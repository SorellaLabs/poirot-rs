use alloy_primitives::{Address, U256};
use reth_primitives::{H160, H256};
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
pub enum Protocol {
    UniswapV2,
    Sushiswap,
    Balancer,
    Curve,
    UniswapV3,
    SushiswapV3,
    Bancor,
    Kyber,
    Mooniswap,
    Dodo,
    DodoV2,
    DodoV3,
}

#[derive(Debug, Clone)]
pub struct Withdrawal {
    pub to: H160,
    pub amount: alloy_primitives::Uint<256, 4>,
}

#[derive(Debug, Clone)]
pub struct Deposit {
    pub from: H160,
    pub amount: Uint<256, 4>,
}

#[derive(Debug, Clone)]
pub struct Transfer {
    pub to: Address,
    pub amount: U256,
    pub token: H160,
}

#[derive(Debug, Clone)]
pub struct PoolCreation {
    pub token_0: Address,
    pub token_1: Address,
    pub fee: u32,
}

#[derive(Debug, Clone)]
pub struct Swap {
    pub recipient: Address,
    pub direction: bool,
    pub amount_specified: alloy_primitives::Signed<256, 4>,
    pub price_limit: alloy_primitives::Uint<256, 4>,
    pub data: Vec<u8>,
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

impl Deposit {
    /// Public constructor function to instantiate a [`Deposit`].
    pub fn new(from: H160, amount: Uint<256, 4>) -> Self {
        Self { from, amount }
    }
}

impl Withdrawal {
    /// Public constructor function to instantiate a [`Withdrawal`].
    pub fn new(to: H160, amount: alloy_primitives::Uint<256, 4>) -> Self {
        Self { to, amount }
    }
}

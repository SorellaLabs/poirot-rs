use crate::action::{Action, ActionType, Deposit, PoolCreation, Swap, Transfer, Withdrawal};

use reth_rpc_types::trace::parity::{Action as RethAction, LocalizedTransactionTrace};

use alloy_json_abi::JsonAbi;
use alloy_sol_types::{sol, SolCall};
use reth_primitives::{hex_literal::hex, H160};
use reth_revm::precompile::primitives::ruint::Uint;

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
        function getPool(
            address tokenA,
            address tokenB,
            uint24 fee
        ) external view returns (address pool);
        function feeAmountTickSpacing(uint24 fee) external view returns (int24);
        function setOwner(address _owner) external;
        function enableFeeAmount(uint24 fee, int24 tickSpacing) external;

    }
}

sol! {
    interface IUniswapV3FlashCallback {
        function uniswapV3FlashCallback(
            uint256 fee0,
            uint256 fee1,
            bytes calldata data
        ) external;
    }
}

sol! {
    interface IUniswapV3MintCallback {
        function uniswapV3MintCallback(
            uint256 amount0Owed,
            uint256 amount1Owed,
            bytes calldata data
        ) external;
    }
}

sol! {
    interface IUniswapV3SwapCallback {
        function uniswapV3SwapCallback(
            int256 amount0Delta,
            int256 amount1Delta,
            bytes calldata data
        ) external;
    }
}

sol! {
    #[derive(Debug, PartialEq)]
    interface IUniswapV3PoolDeployer {
        function parameters()
            external
            view
            returns (
                address factory,
                address token0,
                address token1,
                uint24 fee,
                int24 tickSpacing
            );
    }
}

sol! {
    #[derive(Debug, PartialEq)]
    interface WETH9 {
        function deposit() public payable;
        function withdraw(uint wad) public;
    }
}

sol! {
    #[derive(Debug, PartialEq)]
    interface IUniswapV3Pool {
        function swap(address recipient, bool zeroForOne, int256 amountSpecified, uint160 sqrtPriceLimitX96, bytes data) external override returns (int256, int256);
        function mint(
            address recipient,
            int24 tickLower,
            int24 tickUpper,
            uint128 amount,
            bytes calldata data
        ) external returns (uint256 amount0, uint256 amount1);
        function collect(
            address recipient,
            int24 tickLower,
            int24 tickUpper,
            uint128 amount0Requested,
            uint128 amount1Requested
        ) external returns (uint128 amount0, uint128 amount1);
        function burn(
            int24 tickLower,
            int24 tickUpper,
            uint128 amount
        ) external returns (uint256 amount0, uint256 amount1);
        function flash(
            address recipient,
            uint256 amount0,
            uint256 amount1,
            bytes calldata data
        ) external;
    }
}

pub struct Parser {
    block_trace: Vec<LocalizedTransactionTrace>,
}
//TODO: Instead of directly going from trace to action we should have an intermediatary filter step
//TODO: This step could be used to filter known contract interactions & directly match on the
// appropriate decoder instead of naively looping thorugh all of them

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

    //TODO: Note, because a transaction can be a swap -> transfer -> transfer we would have to
    // avoid double counting the transfer & essentially create a higher TODO: level swap action
    // that contains its subsequent transfers
    /// Parse a single transaction trace.
    pub fn parse_trace(&self, curr: &LocalizedTransactionTrace) -> Option<Action> {
        self.parse_transfer(curr)
            .or_else(|| self.parse_pool_creation(curr))
            .or_else(|| self.parse_weth(curr))
            .or_else(|| self.parse_swap(curr))
    }
    // TODO: So here we kind of have to create a type for each contract abi, and we can automate that 
    // by using the alloy json abi, so we can just decode them for any function, & then after that 
    // we sort for actions of interest, so even if we don't care about it for mev we still know wtf is going on 
    // so we can eaisly see what we are missing that could potentially be interesting 
    // because we have our db that is going to have all contract address so we want to do something where we are automating new integrations
    // & can do something like this collect contract addr classified by (type, so like dex etc..) -> abi -> function -> action type -> mev filtering if of interest
    pub fn parse_swap(&self, curr: &LocalizedTransactionTrace) -> Option<Action> {
        match &curr.trace.action {
            RethAction::Call(call) => {
                let mut decoded = match IUniswapV3Pool::swapCall::decode(&call.input.to_vec(), true)
                {
                    Ok(decoded) => decoded,
                    Err(_) => return None,
                };

                return Some(Action {
                    ty: ActionType::Swap(Swap {
                        recipient: decoded.recipient,
                        direction: decoded.zeroForOne,
                        amount_specified: decoded.amountSpecified,
                        price_limit: decoded.sqrtPriceLimitX96,
                        data: decoded.data,
                    }),
                    hash: curr.transaction_hash.unwrap(),
                    block: curr.block_number.unwrap(),
                })
            }
            _ => None,
        }
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
                            ty: ActionType::WethDeposit(Deposit::new(call.from, call.value)),
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

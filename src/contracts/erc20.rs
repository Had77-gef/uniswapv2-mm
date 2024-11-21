use std::sync::Arc;

use bindings_uniswapv2::ierc20::IERC20;
use ethers::{
    providers::Middleware,
    types::{Address, U256},
};

pub async fn allowance<M: Middleware + 'static>(
    client: Arc<M>,
    token_address: Address,
    owner: Address,
    spender: Address,
) -> U256 {
    let token = IERC20::new(token_address, client);
    let allowance = token
        .allowance(owner, spender)
        .await
        .unwrap_or(U256::zero());
    allowance
}

pub async fn balance_of<M: Middleware + 'static>(
    client: Arc<M>,
    token_address: Address,
    owner: Address,
) -> U256 {
    let token = IERC20::new(token_address, client);
    let balance = token.balance_of(owner).await.unwrap_or(U256::zero());
    balance
}

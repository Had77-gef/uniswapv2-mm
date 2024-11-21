use std::sync::Arc;

use bindings_uniswapv2::weth::WETH;
use ethers::{abi::Address, providers::Middleware, types::U256};

use crate::{config::Config, BASIS_POINT};

const DEFAULT_DEPOSIT_CALL_GAS: u64 = 50000u64;
const DEFAULT_GAS_PRICE: u64 = 600000u64;

pub async fn balance_of<M: Middleware + 'static>(
    client: Arc<M>,
    weth_address: Address,
    account: Address,
) -> U256 {
    let weth = WETH::new(weth_address, client.clone());
    let balance = weth.balance_of(account).await.unwrap_or(U256::zero());
    balance
}

pub async fn deposit<M: Middleware + 'static>(
    client: Arc<M>,
    weth_address: Address,
    amount: U256,
    config: Config,
) {
    let weth = WETH::new(weth_address, client.clone());
    let weth_call = weth.deposit().value(amount);

    let gas = weth_call
        .estimate_gas()
        .await
        .unwrap_or(U256::from(DEFAULT_DEPOSIT_CALL_GAS));

    let gas_price = client
        .get_gas_price()
        .await
        .unwrap_or(U256::from(DEFAULT_GAS_PRICE));
    let gas_price = gas_price
        .checked_mul(U256::from(config.network.gas_price_multipler))
        .unwrap()
        .checked_div(U256::from(BASIS_POINT))
        .unwrap();

    weth_call
        .gas(gas)
        .gas_price(gas_price)
        .send()
        .await
        .unwrap();
}

pub async fn withdraw<M: Middleware + 'static>(
    client: Arc<M>,
    weth_address: Address,
    amount: U256,
    config: Config,
) {
    let weth = WETH::new(weth_address, client.clone());
    let weth_call = weth.withdraw(amount);

    let gas = weth_call
        .estimate_gas()
        .await
        .unwrap_or(U256::from(DEFAULT_DEPOSIT_CALL_GAS));

    let gas_price = client
        .get_gas_price()
        .await
        .unwrap_or(U256::from(DEFAULT_GAS_PRICE));
    let gas_price = gas_price
        .checked_mul(U256::from(config.network.gas_price_multipler))
        .unwrap()
        .checked_div(U256::from(BASIS_POINT))
        .unwrap();

    weth_call
        .gas(gas)
        .gas_price(gas_price)
        .send()
        .await
        .unwrap();
}

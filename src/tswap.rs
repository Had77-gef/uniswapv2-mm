use crate::{params::config::Config, BASIS_POINT};
use bindings_uniswapv2::{ierc20::IERC20, uniswapv2_router02::UniswapV2Router02};
use chrono::Utc;
use ethers::{abi::Address, contract::ContractError, providers::Middleware, types::U256};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

const GAS_MULTIPLIER: u64 = 13000u64;
const DEFAULT_GAS_PRICE: u64 = 50000000000u64; // 50 gwei
const DEFAULT_APPROVE_CALL_GAS: u64 = 40000u64;
const DEFAULT_SWAP_CALL_GAS: u64 = 200000u64;

pub const SWAP_DEADLINE: u128 = 120000u128; // 120 seconds

#[derive(Debug)]
#[allow(dead_code)]
pub struct Tswap<M> {
    /// Ethers client.
    client: Arc<M>,
    owner: Address,
    config: Config,
}

impl<M: Middleware + 'static> Tswap<M> {
    pub fn new(client: Arc<M>, owner: Address, config: Config) -> Self {
        Self {
            client,
            owner,
            config,
        }
    }

    pub async fn approve_token(
        &self,
        token_address: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), ContractError<M>> {
        let token = IERC20::new(token_address, self.client.clone());

        let approve_token_call = token.approve(spender, value);

        let gas = approve_token_call
            .estimate_gas()
            .await
            .unwrap_or(U256::from(DEFAULT_APPROVE_CALL_GAS));
        let gas = gas
            .checked_mul(U256::from(GAS_MULTIPLIER))
            .unwrap()
            .checked_div(U256::from(BASIS_POINT))
            .unwrap();

        let gas_price = self
            .client
            .get_gas_price()
            .await
            .unwrap_or(U256::from(DEFAULT_GAS_PRICE));
        let gas_price = gas_price
            .checked_mul(U256::from(self.config.network.gas_price_multipler))
            .unwrap()
            .checked_div(U256::from(BASIS_POINT))
            .unwrap();

        approve_token_call
            .gas(gas)
            .gas_price(gas_price)
            .send()
            .await?;

        Ok(())
    }

    async fn swap(
        &self,
        client: Arc<M>,
        router_address: Address,
        owner: Address,
        token_a_address: Address,
        token_b_address: Address,
        amount: U256,
    ) -> Result<(), ContractError<M>> {
        let route02 = UniswapV2Router02::new(router_address, client);

        let amounts = route02
            .get_amounts_out(amount, vec![token_a_address, token_b_address])
            .await
            .unwrap_or(vec![U256::zero()]);
        let zero = U256::zero();
        let amount_out = amounts.last().unwrap_or(&zero);
        let amount_out_min = amount_out
            .checked_mul(U256::from(self.config.trade.slippage))
            .unwrap()
            .checked_div(U256::from(BASIS_POINT))
            .unwrap();

        let valid_timestamp = get_valid_timestamp(SWAP_DEADLINE);
        let swap_call = route02.swap_exact_tokens_for_tokens(
            amount,
            amount_out_min,
            vec![token_a_address, token_b_address],
            owner,
            U256::from_dec_str(&valid_timestamp.to_string()).unwrap(),
        );

        let gas = swap_call
            .estimate_gas()
            .await
            .unwrap_or(U256::from(DEFAULT_SWAP_CALL_GAS));
        let gas = gas
            .checked_mul(U256::from(GAS_MULTIPLIER))
            .unwrap()
            .checked_div(U256::from(BASIS_POINT))
            .unwrap();

        let gas_price = self
            .client
            .get_gas_price()
            .await
            .unwrap_or(U256::from(DEFAULT_GAS_PRICE));
        let gas_price = gas_price
            .checked_mul(U256::from(self.config.network.gas_price_multipler))
            .unwrap()
            .checked_div(U256::from(BASIS_POINT))
            .unwrap();

        swap_call.gas(gas).gas_price(gas_price).send().await?;

        Ok(())
    }

    pub async fn tswap(
        &self,
        router_address: Address,
        token_a_address: Address,
        token_b_address: Address,
        amount: U256,
        max_step: u64,
    ) {
        let step = self.config.trade.tswap_step as i64;
        let mut step_count = 0u64;
        let mut last_tick = Utc::now();
        loop {
            let now = Utc::now();
            if now.timestamp() - last_tick.timestamp() > step || step_count == 0 {
                last_tick = now;
                step_count += 1;

                let samount = if step_count == max_step {
                    amount
                        .checked_sub(
                            amount
                                .checked_div(U256::from(max_step))
                                .unwrap()
                                .checked_mul(U256::from(max_step - 1))
                                .unwrap(),
                        )
                        .unwrap()
                } else {
                    amount.checked_div(U256::from(max_step)).unwrap()
                };

                let result = self
                    .swap(
                        self.client.clone(),
                        router_address,
                        self.owner.clone(),
                        token_a_address,
                        token_b_address,
                        samount,
                    )
                    .await;

                if result.is_err() {
                    println!("step {} fail", step_count);
                } else {
                    println!("step {} success", step_count);
                }
            }

            if step_count == max_step {
                break;
            }
        }
    }
}

fn get_valid_timestamp(future_millis: u128) -> u128 {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    let time_millis = since_epoch.as_millis().checked_add(future_millis).unwrap();

    time_millis
}

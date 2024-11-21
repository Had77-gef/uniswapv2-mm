use config_file::{ConfigFileError, FromConfigFile};
use serde::Deserialize;

const CONFIG_PATH: &str = "./config.json";

#[derive(Debug, Deserialize, Clone)]
pub struct PoolConfig {
    pub router_address: String,
    pub weth_address: String,
    pub token_a_address: String,
    pub token_a_symbol: String,
    pub token_a_decimal: u32,
    pub token_b_address: String,
    pub token_b_symbol: String,
    pub token_b_decimal: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TradeConfig {
    pub tswap_buy: u64,
    pub tswap_sell: u64,
    pub tswap_step: u64,
    pub slippage: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    /// Chain ID.
    pub chain_id: u64,
    /// Network rpc.
    pub rpc: String,
    pub gas_price_multipler: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub pool: PoolConfig,
    pub trade: TradeConfig,
    pub network: NetworkConfig,
}

pub fn load_config() -> Result<Config, ConfigFileError> {
    let config = Config::from_config_file(CONFIG_PATH)?;
    Ok(config)
}

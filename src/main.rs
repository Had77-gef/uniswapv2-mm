use ethers::providers::Middleware;
use ethers::utils::{format_units, parse_units};
use ethers::{
    prelude::MiddlewareBuilder,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use text_io::try_read;
use tswap::{Tswap, SWAP_DEADLINE};

pub mod params;
pub use params::*;

pub mod contracts;
pub use contracts::*;

pub mod tswap;

const TRADING_ACTION: i32 = 1i32;
const CONVERTING_ACTION: i32 = 2i32;

const BASIS_POINT: u64 = 10000u64;

const GAS_MULTIPLIER: u64 = 13000u64;
const DEFAULT_GAS_PRICE: u64 = 50000000000u64; // 50 gwei

#[tokio::main]
async fn main() -> Result<(), ()> {
    // LOAD: private keys
    let env_vars = env::load_env().unwrap();
    // LOAD: bot config
    let config = config::load_config().unwrap();

    // Set up ethers provider.
    let rpc = Http::from_str(&config.network.rpc).unwrap();
    let provider = Provider::new(rpc);

    let router_address = Address::from_str(&config.pool.router_address).unwrap();
    let weth_address = Address::from_str(&config.pool.weth_address).unwrap();
    let token_a_address = Address::from_str(&config.pool.token_a_address).unwrap();
    let token_b_address = Address::from_str(&config.pool.token_b_address).unwrap();

    // Derive wallet from private key
    let wallet: LocalWallet = env_vars
        .private_key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(config.network.chain_id);
    let address = wallet.address();

    let provider = Arc::new(provider.nonce_manager(address).with_signer(wallet.clone()));

    let tswap = Tswap::new(provider.clone(), address, config.clone());

    loop {
        print!(
            "\n{}\n{}\n{}\n{}\n",
            "########################\n",
            "#    1 is trading      #\n",
            "#    2 is converting   #\n",
            "########################\n"
        );
        print!("Action: ");
        let option: i32 = try_read!("{}\n").unwrap_or(0);
        match option {
            TRADING_ACTION => {
                // log account balance
                let token_a_balance =
                    erc20::balance_of(provider.clone(), token_a_address, address).await;
                let token_b_balance =
                    erc20::balance_of(provider.clone(), token_b_address, address).await;

                println!(
                    "Account {}: {} ({}) - {} ({})\n",
                    address,
                    format_units(token_a_balance, config.pool.token_a_decimal).unwrap(),
                    config.pool.token_a_symbol,
                    format_units(token_b_balance, config.pool.token_b_decimal).unwrap(),
                    config.pool.token_b_symbol
                );
                // end of log

                print!(
                    "Choose Token (1 is {}, 2 is {}): ",
                    config.pool.token_a_symbol, config.pool.token_b_symbol
                );
                let token_id: u32 = try_read!("{}\n").unwrap_or(1);

                let (
                    token_a_address,
                    token_a_symbol,
                    token_a_decimal,
                    token_b_address,
                    token_b_symbol,
                    token_b_decimal,
                ) = if token_id == 1 {
                    (
                        token_a_address,
                        config.pool.token_a_symbol.clone(),
                        config.pool.token_a_decimal,
                        token_b_address,
                        config.pool.token_b_symbol.clone(),
                        config.pool.token_b_decimal,
                    )
                } else {
                    (
                        token_b_address,
                        config.pool.token_b_symbol.clone(),
                        config.pool.token_b_decimal,
                        token_a_address,
                        config.pool.token_a_symbol.clone(),
                        config.pool.token_a_decimal,
                    )
                };

                print!("Amount Sell ({}): ", token_a_symbol);
                let eth: String = try_read!("{}\n").unwrap_or("0".to_string());
                let gwei: U256 = parse_units(eth.clone(), token_a_decimal).unwrap().into();

                // approve router contract to use token for trading
                check_allowance(
                    provider.clone(),
                    &tswap,
                    token_a_address,
                    address,
                    router_address,
                    gwei,
                )
                .await;

                //// log
                println!("setup done, start trading");

                let before_selling_a_balance =
                    erc20::balance_of(provider.clone(), token_a_address, address).await;
                let before_selling_b_balance =
                    erc20::balance_of(provider.clone(), token_b_address, address).await;

                println!(
                    "------------ selling stage ({} steps) ------------",
                    config.trade.tswap_sell
                );
                println!(
                    "sell {} ({}) to buy {}",
                    eth, token_a_symbol, token_b_symbol
                );
                //// end of log

                // sell token A
                tswap
                    .tswap(
                        router_address,
                        token_a_address,
                        token_b_address,
                        gwei,
                        config.trade.tswap_sell,
                    )
                    .await;

                // wait for last swap transaction finalized
                thread::sleep(Duration::from_millis(SWAP_DEADLINE as u64));

                // recheck balance
                let after_selling_a_balance =
                    erc20::balance_of(provider.clone(), token_a_address, address).await;
                let after_selling_b_balance =
                    erc20::balance_of(provider.clone(), token_b_address, address).await;

                //// log
                println!(
                    "\nsold {} ({}) and received {} ({})\n",
                    format_units(
                        before_selling_a_balance
                            .checked_sub(after_selling_a_balance)
                            .unwrap(),
                        token_a_decimal
                    )
                    .unwrap(),
                    token_a_symbol,
                    format_units(
                        after_selling_b_balance
                            .checked_sub(before_selling_b_balance)
                            .unwrap(),
                        token_b_decimal
                    )
                    .unwrap(),
                    token_b_symbol
                );
                //// end of log

                if after_selling_b_balance <= before_selling_b_balance {
                    println!(
                        "\nnumber of token after purchase does not increase, something is wrong\n"
                    );
                    continue;
                }

                let token_b_gwei = after_selling_b_balance
                    .checked_sub(before_selling_b_balance)
                    .unwrap();

                // approve router contract to use token for trading
                check_allowance(
                    provider.clone(),
                    &tswap,
                    token_b_address,
                    address,
                    router_address,
                    token_b_gwei,
                )
                .await;

                //// log
                println!(
                    "------------ buying stage ({} steps) ------------",
                    config.trade.tswap_buy
                );
                println!(
                    "sell {} ({}) to buy back {}",
                    format_units(token_b_gwei, token_b_decimal).unwrap(),
                    token_b_symbol,
                    token_a_symbol
                );
                //// end of log

                tswap
                    .tswap(
                        router_address,
                        token_b_address,
                        token_a_address,
                        token_b_gwei,
                        config.trade.tswap_buy,
                    )
                    .await;
                // wait for last swap transaction finalized
                thread::sleep(Duration::from_millis(SWAP_DEADLINE as u64));

                let after_buying_a_balance =
                    erc20::balance_of(provider.clone(), token_a_address, address).await;
                let after_buying_b_balance =
                    erc20::balance_of(provider.clone(), token_b_address, address).await;

                //// log
                println!(
                    "\nsold {} ({}) and received {} ({})\n",
                    format_units(
                        after_selling_b_balance
                            .checked_sub(after_buying_b_balance)
                            .unwrap(),
                        token_b_decimal
                    )
                    .unwrap(),
                    token_b_symbol,
                    format_units(
                        after_buying_a_balance
                            .checked_sub(after_selling_a_balance)
                            .unwrap(),
                        token_a_decimal
                    )
                    .unwrap(),
                    token_a_symbol
                );
                //// end of log

                //// log
                println!("------------ result ------------");
                println!(
                    "{}: {}",
                    token_a_symbol,
                    if after_buying_a_balance >= before_selling_a_balance {
                        format!(
                            "{}{}",
                            "+",
                            format_units(
                                after_buying_a_balance
                                    .checked_sub(before_selling_a_balance)
                                    .unwrap(),
                                token_a_decimal
                            )
                            .unwrap()
                        )
                    } else {
                        format!(
                            "{}{}",
                            "-",
                            format_units(
                                before_selling_a_balance
                                    .checked_sub(after_buying_a_balance)
                                    .unwrap(),
                                token_a_decimal
                            )
                            .unwrap()
                        )
                    }
                );
                println!(
                    "{}: {}",
                    token_b_symbol,
                    if after_buying_b_balance >= before_selling_b_balance {
                        format!(
                            "{}{}",
                            "+",
                            format_units(
                                after_buying_b_balance
                                    .checked_sub(before_selling_b_balance)
                                    .unwrap(),
                                token_b_decimal
                            )
                            .unwrap()
                        )
                    } else {
                        format!(
                            "{}{}",
                            "-",
                            format_units(
                                before_selling_b_balance
                                    .checked_sub(after_buying_b_balance)
                                    .unwrap(),
                                token_b_decimal
                            )
                            .unwrap()
                        )
                    }
                );
                //// end of log
            }
            CONVERTING_ACTION => {
                // log account balance
                let eth_balance = provider
                    .get_balance(address, None)
                    .await
                    .unwrap_or(U256::zero());
                let weth_balance = weth::balance_of(provider.clone(), weth_address, address).await;

                println!(
                    "Account {}: {} (ETH) - {} (WETH)\n",
                    address,
                    format_units(eth_balance, 18).unwrap(),
                    format_units(weth_balance, 18).unwrap(),
                );
                // end of log

                print!("Choose Path (1 is ETH -> WETH, 2 is WETH -> ETH): ");
                let path: u32 = try_read!("{}\n").unwrap_or(1);

                print!("Amount ({}): ", if path == 1 { "ETH" } else { "WETH" });
                let eth: String = try_read!("{}\n").unwrap_or("0".to_string());
                let gwei: U256 = parse_units(eth, 18).unwrap().into();

                match path {
                    1 => {
                        weth::deposit(provider.clone(), weth_address, gwei, config.clone()).await;

                        // wait for balance update
                        while weth::balance_of(provider.clone(), weth_address, address).await
                            <= weth_balance
                        {
                            thread::sleep(Duration::from_secs(5));
                            println!("waiting tx ...");
                        }

                        println!("success");
                    }
                    2 => {
                        weth::withdraw(provider.clone(), weth_address, gwei, config.clone()).await;

                        // wait for balance update
                        while provider
                            .get_balance(address, None)
                            .await
                            .unwrap_or(U256::zero())
                            <= eth_balance
                        {
                            thread::sleep(Duration::from_secs(5));
                            println!("waiting tx ...");
                        }

                        println!("success");
                    }
                    _ => println!("unsupported option"),
                }
            }
            _ => println!("unsupported option"),
        }
    }
}

async fn check_allowance<M: Middleware + 'static>(
    provider: Arc<M>,
    tswap: &Tswap<M>,
    token_address: Address,
    owner: Address,
    spender: Address,
    expected: U256,
) {
    let mut check = false;
    while erc20::allowance(provider.clone(), token_address, owner, spender).await < expected {
        if !check {
            tswap
                .approve_token(token_address, spender, expected)
                .await
                .unwrap();

            check = !check;
        }
        thread::sleep(Duration::from_secs(5));
        println!("prev setup ...");
    }
}

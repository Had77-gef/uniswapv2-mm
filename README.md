# UNISWAPV2-MM
bot doing tswap on uniswapv2 pool

automatically sell token A in time X then sell received token B to buy back token A in time Y

E.g. sell 10 ETH in 10 minutes then buy back in 30 minutes

TODO: run swap in another thread and wait for transaction confirmation
### Config
file path `.env`
- **private_key**: your wallet private key

<br>

file path `config.json`

- **pool**:
    - router_address: address of uniswap router02 contract,
    - weth_address: address of weth contract,
    - token_a_address: address of token A contract,
    - token_a_symbol: token A symbol,
    - token_a_decimal: token A decimal,
    - token_b_address: address of token B contract,
    - token_b_symbol: token B symbol,
    - token_b_decimal: token B decimal
- **trade**:
    - tswap_buy: number of steps of the buying stage,
    - tswap_sell: number of steps of the selling stage,
    - tswap_step: step duration in second (e.g. 60 ~ 60 seconds),
    - slippage: swap slippage with decimal 4 (e.g. 9500 means when selling input_token, accepting output token received ~ 95% of output token when simulating)
- **network**:
    - rpc: node rpc
    - chain_id: chain id
    - gas_multipler: multiply gas price to increase the chance of being processed, 4 decimal (e.g. 15000 ~ x1.5)

### Run

linux system

```
cp .env.example .env

cargo build --release

./target/release/uniswapv2-mm
```
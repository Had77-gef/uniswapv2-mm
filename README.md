# UNISWAPV2-MM
bot doing tswap on uniswapv2 pool

Gradually sell token A until a predetermined amount is sold out, then use the received token B to gradually buy back token A

E.g. sell 10 ETH for 15 SOL within 10 minutes, then use the 15 SOL to buy back ETH within 30 minutes

TODO: run swaps in multiple threads and wait for transaction confirmation
### Config
file path `.env`
- **private_key**: your wallet's private key in hex format (e.g. 8da4ef21b864d2cc526dbdb2a120bd2874c36c9d0a1fb7f8c63d7f7a8b41de8f)

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
    - tswap_step: step duration in second, must be greater than the average block time for the target chain (e.g. 60 ~ 60 seconds),
    - slippage: swap slippage with decimal 4 (e.g. 9500 means that when selling the input token, only accept the transaction if the output token received is at least 95% of the output token predicted by the simulation.)
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
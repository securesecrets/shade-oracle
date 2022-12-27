## Oracles

The main point of interaction will be the [Oracle Router](./contracts/oracle_router/). This contract serves as a global key value store linking keys to oracles. This simplifies dependency management because an oracle used to get the price of some asset can be swapped out for another without affecting its consumers.

## Router Key Naming Conventions
This describes how the keys should be named for all assets supported by the oracle router.

Standard assets will have their regular token symbol name as their key, i.e. `SCRT` will be `SCRT`.

Generally, assets will be formatted as `{source} {name} ({extra information})` where source can be the issuing protocol and name is the asset itself.

### SiennaSwap & ShadeSwap
The spot prices for LP tokens will be prefixed by their dex and appended `LP`. Any other method for calculating an LP token's price be appended to the key.

- SiennaSwap spot oracle for USDT/ETH LP - `SiennaSwap USDT/ETH LP`.
- ShadeSwap USDT/ETH LP based off inferred reserves - `ShadeSwap USDT/ETH LP (Inferred Reserves)`.

For market oracles that derive the price of one asset based off the other in a pair, a sample key format would be `SHD (ShadeSwap SHD/ETH)`.

### Shade Staking Derivatives
Key will be their regular token name, i.e. `stkd-SCRT`.

### Stride
Stride liquid staking assets will be prefixed by `Stride`, i.e. `Stride OSMO`, `Stride JUNO`.

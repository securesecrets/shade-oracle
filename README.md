# Shade Oracles

This repository contains all of the oracle contracts used by Shade Protocol.



## Getting Started

Run `cargo update` and then `cargo check`. If `cargo update` fails, execute `export CARGO_NET_GIT_FETCH_WITH_CLI=true` and then run the update command again. This is needed because we are pulling a dependency from one of our private repos.

To test the contracts, run `cargo test` in `packages/ensemble_tests/src`.

## Using the oracles in your code

These oracles should be used by the whole Shade Engineering organization. To use them, include this repo as a submodule in your repository. You should be able to deploy them to your local environment the same way you deploy any submodule. We all use different deployment scripts which is why there is probably no general instruction to give here.

In Lend, we do the following from a Makefile we execute in the root our of repository:


```
oracle_submodule_dir=external_projects/shade-oracle/contracts
(cd ${oracle_submodule_dir}; RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown)
```

## Contract Summary

#### oracle_router
This is a router contract that should be the entry point for any consumer of oracle data. This contract will map an "asset ID" to a contract that has a `GetPrice` query method that will return a value that the consumer should interpret as the price of that asset. Some examples:

**SecretSwap-sSCRT-sUSDC-LP**: This could be the asset ID of an sSCRT/sUSDC LP on SecretSwap that maps to an `lp_oracle` deployment that gets the price of this LP token.
**SCRT**: This could be the asset ID that simply gets the price of SCRT

This implementation allows us to deploy new oracles to replace existing ones without requiring us to update configuration across all of our applications. For example, without an oracle router, if we had five different teams and thirteen different applications using the SCRT oracle and we had to update the SCRT oracle contract to add Chainlink feed support, those five separate teams need to independently update configuration of their applications to point to the new oracle and hopefully not make any mistakes. Having this update happen in one place and seamlessly propogate through the system greatly reduces our risk at the cost of a slightly more expensive query.

#### mock_band
This is not an oracle, this is a mock band contract that can be used by other oracles to get Band prices during local testing. **This contract should not be deployed or used in testnet or mainnet.**

#### proxy_band_oracle
This oracle is a simple single asset oracle that gets the price of an asset from Band. In the test environment, this contract should be configured to contact the locally deployed `mock_band` contract. In a real deployment, it should use the Band testnet and mainnet contracts.

#### lp_oracle
This oracle is for SecretSwap and SiennaSwap LP tokens. This oracle should also work for any Uniswap V2-style AMM. This oracle does _not_ use the live data from the AMM. This oracle instead gets the prices of either side of the LP though a price feed like Band, and then uses that to infer the reserves of the LP and is generally within 0.3% of the live price on the exchange. https://blog.alphafinance.io/fair-lp-token-pricing/

#### earn_v1_oracle
This oracle is for Shade Earn Receipt Tokens. These tokens are rebase tokens that represent a share of underlying assets. This oracle must be configured with the contract address of another oracle for the underlying asset, and will return the price of the receipt token. For example:

1 SCRT = $10
1 stkd-SCRT = 1.25 SCRT

For a stkd-SCRT oracle, we would configure the contract with the oracle for SCRT, and `GetPrice` for the stkd-SCRT oracle would return $12.50. This implementation allows a receipt token to be made for any asset class with any underlying oracle implemention (e.g. LP tokens and single assets use different oracle implementations, but can use the same receipt token implementation and same receipt token oracle implementation)

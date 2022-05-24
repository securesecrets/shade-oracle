
# Shade Staking Derivative Oracle
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Owner](#Owner)
        * Messages
            * [UpdateConfig](#UpdateConfig)
    * [User](#User)
        * Queries
            * [GetConfig](#GetConfig)
            * [GetPrice](#GetPrice)
# Introduction
Contract responsible for retrieving the price of a Shade staking derivative.

# Sections

## Init
##### Request
| Name               | Type     | Description                                            | optional |
|--------------------|----------|--------------------------------------------------------|----------|
| owner              | String   | Contract owner, has ability to adjust config           | no       |
| symbol             | String   | Key used to query price of underlying asset via router | no       |
| staking_derivative | Contract | Staking derivative contract                            | no       |
| router             | Contract | Oracle router contract                                 | no       |

## Owner

### Messages
#### UpdateConfig
##### Request
Updates config of the oracle contract.
| Name               | Type     | Description                                            | optional |
|--------------------|----------|--------------------------------------------------------|----------|
| owner              | String   | Contract owner, has ability to adjust config           | no       |
| symbol             | String   | Key used to query price of underlying asset via router | no       |
| staking_derivative | Contract | Staking derivative contract                            | no       |
| router             | Contract | Oracle router contract                                 | no       |

## User

### Queries

#### GetConfig
Gets the contract's config data.
##### Response
```json
{
  "ConfigResponse": {
    "owner": "String of owner's address",
    "symbol": "Key used to query price of underlying asset via router",
    "staking_derivative": "Staking derivative contract",
    "router": "Oracle router contract",
  }
}
```

#### GetPrice
Gets the asset price from the band contract.
##### Response
```json
{
  "ReferenceData": {
    "rate": "Uint128 of the queried asset rate",
    "last_updated_base": "u64 of the block time",
    "last_updated_quote": "u64 of the block time"
  }
}
```

## Contract
Type used in many of the configuration variables and messages
```json
{
  "contract": {
    "address": "Contract address",
    "code_hash": "Callback code hash"
  }
}
```

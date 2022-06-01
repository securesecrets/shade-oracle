# Index Oracle
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Owner](#Owner)
        * Messages
            * [UpdateConfig](#UpdateConfig)
            * [ModBasket](#ModBasket)
    * [User](#User)
        * Queries
            * [GetConfig](#GetConfig)
            * [GetPrice](#GetPrice)
            * [GetPrices](#GetPrices)
# Introduction
Contract responsible for aggregating a list of assets (denom/weight) into a single price feed (SILK)

# Sections

## Init
##### Request
| Name         | Type     | Description                                              | optional |
|--------------|----------|----------------------------------------------------------|----------|
| owner        | String   | Contract owner, has ability to adjust config             | no       |
| router       | Contract | Oracle Router contract                                   | no       |
| symbol       | String   | Symbol of representing this basket of assets             | no       |
| basket       | HashMap  | Map of `{symbol: weight}`                                | no       |
| target       | Uint128  | Initial price target                                     | no       |

## Owner

### Messages
#### UpdateConfig
##### Request
Updates config of proxy band oracle contract.
| Name         | Type     | Description                                              | optional |
|--------------|----------|----------------------------------------------------------|----------|
| admins       | Vec<HumanAddr> | List of admins has ability to adjust config        | yes      |
| router       | Contract    | Oracle router contract                                | yes      |

## User

### Queries

#### GetConfig
Gets the contract's config data.
##### Response
```json
{
  "Config": {
    "admins": "List of admin addresses",
    "router": "Oracle Router contract",
  }
}
```

#### GetPrice
Gets the asset price from the band contract.
##### Response
```json
{
  "key": "String price feed symbol",
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

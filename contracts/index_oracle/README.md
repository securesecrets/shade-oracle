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

## Owner

### Messages
#### UpdateConfig
##### Request
Updates config of proxy band oracle contract.
| Name         | Type     | Description                                              | optional |
|--------------|----------|----------------------------------------------------------|----------|
| owner        | String   | Contract owner, has ability to adjust config             | yes      |
| band         | Contract | Band contract to retrieve prices from                    | yes      |
| base_symbol  | String   | Symbol of asset to retrieve price of                     | yes      |
| quote_symbol | String   | Symbol of the asset which the desired asset is quoted in | yes      |

## User

### Queries

#### GetConfig
Gets the contract's config data.
##### Response
```json
{
  "ConfigResponse": {
    "owner": "String of owner's address",
    "band": "Band contract",
    "base_symbol": "String of the asset symbol to retrieve price of",
    "quote_symbol": "String of the asset symbol of which the desired asset is quoted in"
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

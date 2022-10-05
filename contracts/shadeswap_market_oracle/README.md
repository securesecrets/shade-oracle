# Sienna Swap Open Market Price Oracle
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
Contract responsible for retrieving the price of a Secretswap or Siennaswap LP.

# Sections

## Init
##### Request
| Name     | Type     | Description                                  | optional |
|----------|----------|----------------------------------------------|----------|
| owner    | String   | Contract owner, has ability to adjust config | no       |
| symbol   | String   | Symbol to evaluate to a USD price      | no       |
| base_symbol    | String   | Symbol with alternative price feed to evaluate by       | no       |
| pair     | Contract | Contract of the of an oracle for 1st asset of LP    | no       |
| base_peg | String   | Symbol to query for base asset price         | yes      |


## Owner

### Messages
#### UpdateConfig
##### Request
Updates config of the oracle contract.
| Name     | Type     | Description                                  | optional |
|----------|----------|----------------------------------------------|----------|
| owner    | String   | Contract owner, has ability to adjust config | no       |
| oracle1  | Contract | Contract of an oracle for 1st asset of LP    | no       |
| oracle2  | Contract | Contract of an oracle for 2nd asset of LP    | no       |
| lp_token | Contract | Contract of the lp token                     | no       |

## User

### Queries

#### GetConfig
Gets the contract's config data.
##### Response
```json
{
  "ConfigResponse": {
    "owner": "String of owner's address",
    "oracle1": "Contract of an oracle for 1st asset of LP",
    "oracle2": "Contract of an oracle for 2nd asset of LP",
    "lp_token": "Contract of the lp token",
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

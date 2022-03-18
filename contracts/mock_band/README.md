
# Mock Band Oracle Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [UpdateConfig](#UpdateConfig)
            * [UpdateMintLimit](#UpdateMintLimit)
            * [RegisterAsset](#RegisterAsset)
            * [RemoveAsset](#RemoveAsset)
    * [User](#User)
        * Messages
          * [UpdateSymbolPrice](#UpdateSymbolPrice)
        * Queries
            * [GetReferenceData](#GetReferenceData)
            * [GetReferenceDataBulk](#GetReferenceDataBulk)
# Introduction
Contract responsible for mimicking the desired behavior of a band oracle contract.

# Sections

## User

### Messages
#### UpdateSymbolPrice
##### Request
Updates current data of asset.
| Name         | Type    | Description                                              | optional |
|--------------|---------|----------------------------------------------------------|----------|
| base_symbol  | String  | Symbol of asset to retrieve price of                     | no       |
| quote_symbol | String  | Symbol of the asset which the desired asset is quoted in | no       |
| rate         | Uint128 | Current rate of asset                                    | no       |
| last_updated | u64     | Timestamp of time when price is last updated             | yes      |

### Queries

#### GetReferenceData
Gets the price data of the queried asset.
##### Request
| Name         | Type    | Description                                              | optional |
|--------------|---------|----------------------------------------------------------|----------|
| base_symbol  | String  | Symbol of asset to retrieve price of                     | no       |
| quote_symbol | String  | Symbol of the asset which the desired asset is quoted in | no       |
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

#### GetReferenceDataBulk
Gets a vector of the the price data of the queried asset.
##### Request
| Name         | Type    | Description                                              | optional |
|--------------|---------|----------------------------------------------------------|----------|
| base_symbol  | String  | Symbol of asset to retrieve price of                     | no       |
| quote_symbol | String  | Symbol of the asset which the desired asset is quoted in | no       |
##### Response
```json
{
  "ReferenceData": {
    "rate": "Uint128 of the queried asset rate",
    "last_updated_base": "u64 of the block time",
    "last_updated_quote": "u64 of the block time"
  }
  ...
}
```

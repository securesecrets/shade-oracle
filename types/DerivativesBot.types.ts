/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.17.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export interface InstantiateMsg {
  router: RawContract;
}
export interface RawContract {
  address: string;
  code_hash: string;
}
export type ExecuteMsg = {
  set_derivatives: RawDerivativeData[];
} | {
  remove_derivatives: string[];
} | {
  update_config: RawContract;
} | {
  update_derivatives: DerivativeUpdates;
} | {
  set_status: boolean;
};
export type Decimal256 = string;
export type DerivativeUpdates = {
  rates: [string, Decimal256][];
} | {
  config: [string, DerivativeDataConfigUpdate][];
};
export interface RawDerivativeData {
  initial_rate: Decimal256;
  key: string;
  rate_max_change: Decimal256;
  rate_timeout: number;
  underlying_key: string;
}
export interface DerivativeDataConfigUpdate {
  rate_max_change?: Decimal256 | null;
  rate_timeout?: number | null;
  underlying_key?: string | null;
}
export type QueryMsg = {
  get_price: {
    key: string;
  };
} | {
  get_prices: {
    keys: string[];
  };
} | {
  get_config: {};
} | {
  get_derivatives: {};
};
export type Addr = string;
export interface CommonConfigResponse {
  config: CommonConfig;
  supported_keys: string[];
}
export interface CommonConfig {
  enabled: boolean;
  router: Contract;
}
export interface Contract {
  address: Addr;
  code_hash: string;
}
export type ArrayOfDerivativeData = DerivativeData[];
export interface DerivativeData {
  key: string;
  rate: DerivativeRate;
  underlying_key: string;
}
export interface DerivativeRate {
  last_updated: number;
  max_change: Decimal256;
  timeout: number;
  value: Decimal256;
}
export type Uint256 = string;
export interface OraclePrice {
  data: ReferenceData;
  key: string;
}
export interface ReferenceData {
  last_updated_base: number;
  last_updated_quote: number;
  rate: Uint256;
}
export type ArrayOfOraclePrice = OraclePrice[];
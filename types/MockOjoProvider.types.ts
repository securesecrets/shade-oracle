/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.17.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Uint256 = string;
export interface InstantiateMsg {
  admin_auth: RawContract;
  initial_prices: [string, string, Uint256][];
  quote_symbol?: string | null;
}
export interface RawContract {
  address: string;
  code_hash: string;
}
export type ExecuteMsg = {
  set_status: boolean;
} | {
  update_config: {
    admin_auth?: RawContract | null;
    quote_symbol?: string | null;
  };
} | {
  set_price: MockPrice;
} | {
  set_prices: MockPrice[];
};
export interface MockPrice {
  base_symbol: string;
  last_updated?: number | null;
  quote_symbol: string;
  rate: Uint256;
}
export type QueryMsg = {
  get_reference_data: {
    symbol_pair: [string, string];
  };
} | {
  get_reference_data_bulk: {
    symbol_pairs: [string, string][];
  };
} | {
  get_median_reference_data: {
    symbol_pair: [string, string];
  };
} | {
  get_median_reference_data_bulk: {
    symbol_pairs: [string, string][];
  };
} | {
  get_price: {
    key: string;
  };
} | {
  get_prices: {
    keys: string[];
  };
} | {
  get_config: {};
};
export type Addr = string;
export interface Config {
  admin_auth: Contract;
  enabled: boolean;
  quote_symbol: string;
}
export interface Contract {
  address: Addr;
  code_hash: string;
}
export type Uint64 = string;
export interface OjoReferenceData {
  last_updated_base: Uint64;
  last_updated_quote: Uint64;
  rate: Uint256;
}
export type ArrayOfOjoReferenceData = OjoReferenceData[];
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
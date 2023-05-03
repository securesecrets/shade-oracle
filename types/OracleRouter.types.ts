/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.17.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type RawProvider = {
  band: RawContract;
} | {
  ojo: RawContract;
};
export interface InstantiateMsg {
  admin_auth: RawContract;
  provider: RawProvider;
  quote_symbol: string;
}
export interface RawContract {
  address: string;
  code_hash: string;
}
export type ExecuteMsg = {
  set_status: ContractStatus;
} | {
  update_protected_keys: [string, Uint256][];
} | {
  update_config: UpdateConfig;
} | {
  update_registry: RegistryOperation;
} | {
  batch_update_registry: RegistryOperation[];
};
export type ContractStatus = "normal" | "deprecated" | "frozen";
export type Uint256 = string;
export type RegistryOperation = {
  remove_keys: {
    keys: string[];
  };
} | {
  set_keys: {
    keys: string[];
    oracle: RawContract;
  };
} | {
  set_protection: {
    infos: ProtectedKeyInfo[];
  };
} | {
  remove_protection: {
    keys: string[];
  };
};
export type Decimal256 = string;
export interface UpdateConfig {
  admin_auth?: RawContract | null;
  provider?: RawProvider | null;
  quote_symbol?: string | null;
}
export interface ProtectedKeyInfo {
  deviation: Decimal256;
  key: string;
  price: Uint256;
}
export type QueryMsg = {
  get_config: {};
} | {
  get_oracle: {
    key: string;
  };
} | {
  get_price: {
    key: string;
  };
} | {
  get_oracles: {
    keys: string[];
  };
} | {
  get_prices: {
    keys: string[];
  };
} | {
  get_keys: {};
} | {
  get_protected_keys: {};
};
export type Addr = string;
export type Provider = {
  band: Contract;
} | {
  ojo: Contract;
};
export interface ConfigResponse {
  config: Config;
  status: ContractStatus;
}
export interface Config {
  admin_auth: Contract;
  provider: Provider;
  quote_symbol: string;
  this: Contract;
}
export interface Contract {
  address: Addr;
  code_hash: string;
}
export type ArrayOfString = string[];
export interface OracleResponse {
  key: string;
  oracle: Contract;
}
export type ArrayOfOracleResponse = OracleResponse[];
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
export type ArrayOfProtectedKeyInfo = ProtectedKeyInfo[];
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PREFIX_BASE_ASSET_PRICE_DATA: &[u8] = b"pricedata";

#[derive(Serialize, Deserialize, Default, JsonSchema)]
pub struct SavedBandData {
    pub rate: u128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

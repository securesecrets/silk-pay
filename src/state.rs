use cosmwasm_std::{HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub admin: HumanAddr,
    pub fee: Uint128,
    pub new_admin_nomination: Option<HumanAddr>,
    pub registered_tokens: Option<HashMap<HumanAddr, String>>,
    pub shade_token: SecretContract,
    pub treasury_address: HumanAddr,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
pub struct SecretContract {
    pub address: HumanAddr,
    pub contract_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Token {
    Snip20(SecretContract),
    Native(SecretContract),
}

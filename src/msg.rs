use crate::state::SecretContract;
use crate::transaction_history::HumanizedTx;
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub fee: Uint128,
    pub shade: SecretContract,
    pub sscrt: SecretContract,
    pub treasury_address: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    AcceptNewAdminNomination {},
    Cancel {
        position: u32,
    },
    CreateReceiveRequest {
        address: HumanAddr,
        amount: Uint128,
        description: Option<String>,
        token_address: HumanAddr,
    },
    CreateSendRequest {
        address: HumanAddr,
        amount: Uint128,
        description: Option<String>,
        token_address: HumanAddr,
    },
    NominateNewAdmin {
        address: HumanAddr,
    },
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        msg: Binary,
    },
    RegisterTokens {
        tokens: Vec<SecretContract>,
    },
    UpdateFee {
        fee: Uint128,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    ConfirmAddress { position: u32 },
    SendPayment { position: u32 },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Txs {
        txs: Vec<HumanizedTx>,
        total: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Txs {
        address: HumanAddr,
        key: String,
        page: u32,
        page_size: u32,
    },
}

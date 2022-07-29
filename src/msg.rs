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
    NominateNewAdmin {
        address: HumanAddr,
    },
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        msg: Binary,
    },
    UpdateFee {
        fee: Uint128,
    },
    UpdateTreasuryAddress {
        address: HumanAddr,
    },
}
/**
 * Tx status enumeration:
 * 0 - Single Send Request
 * 1 - Single Receive Request
 * 2 - Cancelled
 * 3 - Completed
 * 4 - Active Recurring Send Request
 * 5 - Active Recurring Receive Request
 */
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Cancel {
        position: u32,
    },
    ConfirmAddress {
        position: u32,
    },
    CreateReceiveRequest {
        address: HumanAddr,
        send_amount: Uint128,
        description: Option<String>,
        token: SecretContract,
    },
    CreateSendRequest {
        address: HumanAddr,
        send_amount: Uint128,
        description: Option<String>,
        token: SecretContract,
    },
    SendPayment {
        position: u32,
    },
    CreateRecurringSendRequest {
        address: HumanAddr,
        description: Option<String>,
        token: SecretContract,
        send_amount: Uint128,
        start_time: u64,
        interval: u64,
        end_time: u64,
    },
    CreateRecurringReceiveRequest {
        address: HumanAddr,
        description: Option<String>,
        token: SecretContract,
        receive_amount: Uint128,
        start_time: u64,
        interval: u64,
        end_time: u64,
    },
    FulfillRecurringPayment {
        position: u32,
    },
    AcceptRecurringPayment {
        position: u32,
    },
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

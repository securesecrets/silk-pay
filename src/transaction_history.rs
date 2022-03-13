use crate::constants::PREFIX_TXS;
use crate::state::SecretContract;
use cosmwasm_std::{
    Api, CanonicalAddr, HumanAddr, ReadonlyStorage, StdError, StdResult, Storage, Uint128,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HumanizedTx {
    pub position: u32,
    pub from: HumanAddr,
    pub to: HumanAddr,
    pub amount: Uint128,
    pub token: SecretContract,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: u8,
    pub block_time: u64,
    pub block_height: u64,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Tx {
    pub position: u32,
    pub other_storage_position: u32,
    pub from: CanonicalAddr,
    pub to: CanonicalAddr,
    pub amount: Uint128,
    pub token: SecretContract,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: u8,
    pub block_time: u64,
    pub block_height: u64,
}
impl Tx {
    fn into_humanized<A: Api>(self, api: &A) -> StdResult<HumanizedTx> {
        Ok(HumanizedTx {
            position: self.position,
            from: api.human_address(&self.from)?,
            to: api.human_address(&self.to)?,
            amount: self.amount,
            token: self.token,
            description: self.description,
            status: self.status,
            block_time: self.block_time,
            block_height: self.block_height,
        })
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum StatusCode {
    PendingAddressConfirmation = 0,
    PendingPayment = 1,
    Cancelled = 2,
    Finalized = 3,
}

impl StatusCode {
    fn to_u8(self) -> u8 {
        self as u8
    }

    fn from_u8(n: u8) -> StdResult<Self> {
        use StatusCode::*;
        match n {
            0 => Ok(PendingAddressConfirmation),
            1 => Ok(PendingPayment),
            2 => Ok(Cancelled),
            3 => Ok(Finalized),
            other => Err(StdError::generic_err(format!(
                "Unexpected Status code in transaction history: {} Storage is corrupted.",
                other
            ))),
        }
    }
}

// Storage functions:

#[allow(clippy::too_many_arguments)] // We just need them
pub fn store_tx<S: Storage>(
    store: &mut S,
    from: &CanonicalAddr,
    to: &CanonicalAddr,
    amount: Uint128,
    token: SecretContract,
    description: Option<String>,
    status: u8,
    block: &cosmwasm_std::BlockInfo,
) -> StdResult<()> {
    if from == to {
        return Err(StdError::generic_err(
            "From and to addresses must be different.",
        ));
    }

    let from_position = get_next_position(store, from)?;
    let to_position = get_next_position(store, to)?;
    let from_tx = Tx {
        position: from_position,
        other_storage_position: to_position,
        from: from.clone(),
        to: to.clone(),
        amount: amount,
        token: token,
        description: description,
        status: status,
        block_time: block.time,
        block_height: block.height,
    };
    append_tx(store, &from_tx, from)?;
    let mut to_tx = from_tx;
    to_tx.position = to_position;
    to_tx.other_storage_position = from_position;
    append_tx(store, &to_tx, to)?;

    Ok(())
}

fn append_tx<S: Storage>(store: &mut S, tx: &Tx, for_address: &CanonicalAddr) -> StdResult<()> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_TXS, for_address.as_slice()], store);
    let mut store = AppendStoreMut::attach_or_create(&mut store)?;
    store.push(tx)
}

fn get_next_position<S: Storage>(store: &mut S, for_address: &CanonicalAddr) -> StdResult<u32> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_TXS, for_address.as_slice()], store);
    let store = AppendStoreMut::<Tx, _>::attach_or_create(&mut store)?;
    Ok(store.len())
}

pub fn get_txs<A: Api, S: ReadonlyStorage>(
    api: &A,
    storage: &S,
    for_address: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<(Vec<HumanizedTx>, u64)> {
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_TXS, for_address.as_slice()], storage);

    // Try to access the storage of txs for the account.
    // If it doesn't exist yet, return an empty list of transfers.
    let store = AppendStore::<Tx, _, _>::attach(&store);
    let store = if let Some(result) = store {
        result?
    } else {
        return Ok((vec![], 0));
    };

    // Take `page_size` txs starting from the latest tx, potentially skipping `page * page_size`
    // txs from the start.
    let tx_iter = store
        .iter()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);

    // The `and_then` here flattens the `StdResult<StdResult<RichTx>>` to an `StdResult<RichTx>`
    let txs: StdResult<Vec<HumanizedTx>> = tx_iter
        .map(|tx| tx.map(|tx| tx.into_humanized(api)).and_then(|x| x))
        .collect();
    txs.map(|txs| (txs, store.len() as u64))
}

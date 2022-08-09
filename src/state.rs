use crate::constants::PREFIX_REGISTERED_TOKENS;
use cosmwasm_std::{HumanAddr, ReadonlyStorage, StdResult, Storage};
use cosmwasm_math_compat::Uint128;
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use secret_toolkit::serialization::{Bincode2, Serde};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub admin: HumanAddr,
    pub fee: Uint128,
    pub new_admin_nomination: Option<HumanAddr>,
    pub shade: SecretContract,
    pub sscrt: SecretContract,
    pub treasury_address: HumanAddr,
    pub end_time_limit: u64,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
pub struct SecretContract {
    pub address: HumanAddr,
    pub contract_hash: String,
}

// === RegisteredTokens Storage ===
pub struct RegisteredTokensReadonlyStorage<'a, S: Storage> {
    storage: ReadonlyPrefixedStorage<'a, S>,
}
impl<'a, S: Storage> RegisteredTokensReadonlyStorage<'a, S> {
    pub fn from_storage(storage: &'a S) -> Self {
        Self {
            storage: ReadonlyPrefixedStorage::new(PREFIX_REGISTERED_TOKENS, storage),
        }
    }

    pub fn get_contract_hash(&mut self, key: HumanAddr) -> Option<String> {
        let key = key.to_string();
        self.as_readonly().get(&key)
    }

    // private

    fn as_readonly(&self) -> ReadonlyRegisteredTokensStorageImpl<ReadonlyPrefixedStorage<S>> {
        ReadonlyRegisteredTokensStorageImpl(&self.storage)
    }
}

pub struct RegisteredTokensStorage<'a, S: Storage> {
    storage: PrefixedStorage<'a, S>,
}
impl<'a, S: Storage> RegisteredTokensStorage<'a, S> {
    pub fn from_storage(storage: &'a mut S) -> Self {
        Self {
            storage: PrefixedStorage::new(PREFIX_REGISTERED_TOKENS, storage),
        }
    }

    pub fn get_contract_hash(&mut self, key: HumanAddr) -> Option<String> {
        let key = key.to_string();
        self.as_readonly().get(&key)
    }

    pub fn set_contract_hash(&mut self, key: HumanAddr, value: &String) {
        let key = key.0.as_bytes();
        save(&mut self.storage, key, value).ok();
    }

    // private

    fn as_readonly(&self) -> ReadonlyRegisteredTokensStorageImpl<PrefixedStorage<S>> {
        ReadonlyRegisteredTokensStorageImpl(&self.storage)
    }
}

struct ReadonlyRegisteredTokensStorageImpl<'a, S: ReadonlyStorage>(&'a S);
impl<'a, S: ReadonlyStorage> ReadonlyRegisteredTokensStorageImpl<'a, S> {
    pub fn get(&self, key: &String) -> Option<String> {
        let contract_hash: Option<String> = may_load(self.0, &key.as_bytes()).ok().unwrap();
        contract_hash
    }
}

// === FUNCTIONS ===
fn may_load<T: DeserializeOwned, S: ReadonlyStorage>(
    storage: &S,
    key: &[u8],
) -> StdResult<Option<T>> {
    match storage.get(key) {
        Some(value) => Bincode2::deserialize(&value).map(Some),
        None => Ok(None),
    }
}

fn save<T: Serialize, S: Storage>(storage: &mut S, key: &[u8], value: &T) -> StdResult<()> {
    storage.set(key, &Bincode2::serialize(value)?);
    Ok(())
}

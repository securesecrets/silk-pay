use crate::constants::{BLOCK_SIZE, CONFIG_KEY};
use crate::{
    msg::{HandleMsg, InitMsg, QueryMsg},
    state::{Config, SecretContract, Token},
};
use cosmwasm_std::{
    from_binary, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdResult, Storage,
};
use secret_toolkit::snip20;
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use std::collections::HashMap;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let mut config_store = TypedStoreMut::attach(&mut deps.storage);
    let config: Config = Config {
        admin: env.message.sender,
        fee: msg.fee,
        registered_tokens: None,
        treasury_address: msg.treasury_address,
    };
    config_store.store(CONFIG_KEY, &config)?;

    Ok(InitResponse {
        messages: vec![],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::RegisterTokens { tokens } => register_tokens(deps, &env, tokens),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY)?;
            Ok(to_binary(&config)?)
        }
    }
}

fn register_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    tokens: Vec<SecretContract>,
) -> StdResult<HandleResponse> {
    let mut config: Config = TypedStoreMut::attach(&mut deps.storage)
        .load(CONFIG_KEY)
        .unwrap();
    let mut registered_tokens: HashMap<HumanAddr, String> = if config.registered_tokens.is_some() {
        config.registered_tokens.unwrap()
    } else {
        HashMap::new()
    };
    let mut messages = vec![];
    for token in tokens {
        if !registered_tokens.contains_key(&token.address) {
            let address = token.address;
            let contract_hash = token.contract_hash;
            messages.push(snip20::register_receive_msg(
                env.contract_code_hash.clone(),
                None,
                BLOCK_SIZE,
                contract_hash.clone(),
                address.clone(),
            )?);
            registered_tokens.insert(address, contract_hash);
        }
    }
    config.registered_tokens = Some(registered_tokens);
    TypedStoreMut::attach(&mut deps.storage).store(CONFIG_KEY, &config)?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage};

    // === HELPERS ===
    fn init_helper() -> (
        StdResult<InitResponse>,
        Extern<MockStorage, MockApi, MockQuerier>,
    ) {
        let env = mock_env(mock_contract_initiator_address(), &[]);
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            treasury_address: mock_treasury_address(),
        };
        (init(&mut deps, env, msg), deps)
    }

    fn mock_silk() -> SecretContract {
        SecretContract {
            address: HumanAddr::from("mock-silk-address"),
            contract_hash: "mock-silk-contract-hash".to_string(),
        }
    }

    fn mock_treasury_address() -> HumanAddr {
        HumanAddr::from("mock-treasury-address")
    }

    fn mock_contract() -> SecretContract {
        let env = mock_env(mock_user_address(), &[]);
        SecretContract {
            address: env.contract.address,
            contract_hash: env.contract_code_hash,
        }
    }

    fn mock_contract_initiator_address() -> HumanAddr {
        HumanAddr::from("shade-protocol")
    }

    fn mock_pair_contract() -> SecretContract {
        SecretContract {
            address: HumanAddr::from("pair-contract-address"),
            contract_hash: "pair-contract-contract-hash".to_string(),
        }
    }

    fn mock_pair_contract_two() -> SecretContract {
        SecretContract {
            address: HumanAddr::from("pair-contract-two-address"),
            contract_hash: "pair-contract-two-hash".to_string(),
        }
    }

    fn mock_sscrt() -> SecretContract {
        SecretContract {
            address: HumanAddr::from("mock-sscrt-address"),
            contract_hash: "mock-sscrt-contract-hash".to_string(),
        }
    }

    fn mock_shade() -> SecretContract {
        SecretContract {
            address: HumanAddr::from("mock-token-address"),
            contract_hash: "mock-token-contract-hash".to_string(),
        }
    }

    fn mock_token_native() -> Token {
        Token::Native(mock_sscrt())
    }

    fn mock_token_snip20() -> Token {
        Token::Snip20(mock_sscrt())
    }

    fn mock_user_address() -> HumanAddr {
        HumanAddr::from("gary")
    }

    // === QUERY TESTS ===
    #[test]
    fn test_query_config() {
        let (_init_result, deps) = init_helper();
        let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY).unwrap();
        let query_result = query(&deps, QueryMsg::Config {}).unwrap();
        let query_answer_config: Config = from_binary(&query_result).unwrap();
        assert_eq!(query_answer_config, config);
    }

    // === HANDLE TESTS ===
    #[test]
    fn test_register_tokens() {
        let (_init_result, mut deps) = init_helper();
        let env = mock_env(mock_user_address(), &[]);

        // when no tokens are sent in
        let handle_msg = HandleMsg::RegisterTokens { tokens: vec![] };
        let handle_result = handle(&mut deps, env.clone(), handle_msg);
        let handle_result_unwrapped = handle_result.unwrap();
        // * no messages are sent
        assert_eq!(handle_result_unwrapped.messages, vec![]);

        // When tokens are in the parameter
        let handle_msg = HandleMsg::RegisterTokens {
            tokens: vec![mock_silk(), mock_shade()],
        };
        let handle_result = handle(&mut deps, env.clone(), handle_msg);
        let handle_result_unwrapped = handle_result.unwrap();
        // * it sends a message to register receive for the token
        assert_eq!(
            handle_result_unwrapped.messages,
            vec![
                snip20::register_receive_msg(
                    mock_contract().contract_hash.clone(),
                    None,
                    BLOCK_SIZE,
                    mock_silk().contract_hash,
                    mock_silk().address,
                )
                .unwrap(),
                snip20::register_receive_msg(
                    mock_contract().contract_hash,
                    None,
                    BLOCK_SIZE,
                    mock_shade().contract_hash,
                    mock_shade().address,
                )
                .unwrap(),
            ]
        );

        // * it records the registered tokens in the config
        let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY).unwrap();
        let registered_tokens = config.registered_tokens.unwrap();
        assert_eq!(registered_tokens.contains_key(&mock_silk().address), true);
        assert_eq!(registered_tokens.contains_key(&mock_shade().address), true);

        // = When tokens already exist
        let handle_msg = HandleMsg::RegisterTokens {
            tokens: vec![mock_silk(), mock_shade()],
        };
        let handle_result = handle(&mut deps, env.clone(), handle_msg);
        let handle_result_unwrapped = handle_result.unwrap();
        // = * it doesn't send any messages
        assert_eq!(handle_result_unwrapped.messages, vec![]);
    }
}

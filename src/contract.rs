use crate::authorize::authorize;
use crate::constants::{BLOCK_SIZE, CONFIG_KEY};
use crate::transaction_history::{
    get_txs, store_txs, update_tx, verify_txs, verify_txs_for_cancel,
    verify_txs_for_confirm_address,
};
use crate::{
    msg::{HandleMsg, InitMsg, QueryAnswer, QueryMsg, ReceiveMsg},
    state::{Config, RegisteredTokensStorage, SecretContract},
};
use cosmwasm_std::{
    from_binary, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    InitResponse, Querier, StdError, StdResult, Storage, Uint128,
};
use secret_toolkit::snip20;
use secret_toolkit::storage::{TypedStore, TypedStoreMut};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let mut config_store = TypedStoreMut::attach(&mut deps.storage);
    let config: Config = Config {
        admin: env.message.sender,
        fee: msg.fee,
        new_admin_nomination: None,
        shade: msg.shade.clone(),
        sscrt: msg.sscrt.clone(),
        treasury_address: msg.treasury_address,
    };
    config_store.store(CONFIG_KEY, &config)?;

    Ok(InitResponse {
        messages: vec![
            register_token(&mut deps.storage, env.contract_code_hash.clone(), msg.sscrt)?.unwrap(),
            register_token(&mut deps.storage, env.contract_code_hash, msg.shade)?.unwrap(),
        ],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::AcceptNewAdminNomination {} => accept_new_admin_nomination(deps, &env),
        HandleMsg::NominateNewAdmin { address } => nominate_new_admin(deps, &env, address),
        HandleMsg::Receive {
            from, amount, msg, ..
        } => receive(deps, env, from, amount, msg),
        HandleMsg::UpdateFee { fee } => update_fee(deps, &env, fee),
        HandleMsg::UpdateTreasuryAddress { address } => {
            update_treasury_address(deps, &env, address)
        }
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
        QueryMsg::Txs {
            address,
            key,
            page,
            page_size,
        } => txs(deps, address, key, page, page_size),
    }
}

fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: HumanAddr,
    amount: Uint128,
    msg: Binary,
) -> StdResult<HandleResponse> {
    let msg: ReceiveMsg = from_binary(&msg)?;
    let response = match msg {
        ReceiveMsg::Cancel { position } => cancel(deps, &env, from, amount, position),
        ReceiveMsg::ConfirmAddress { position } => {
            confirm_address(deps, &env, from, amount, position)
        }
        ReceiveMsg::CreateReceiveRequest {
            address,
            send_amount,
            description,
            token,
        } => create_receive_request(
            deps,
            &env,
            from,
            amount,
            address,
            send_amount,
            description,
            token,
        ),
        ReceiveMsg::CreateSendRequest {
            address,
            send_amount,
            description,
            token,
        } => create_send_request(
            deps,
            &env,
            from,
            amount,
            address,
            send_amount,
            description,
            token,
        ),
        ReceiveMsg::SendPayment { position } => send_payment(deps, &env, from, amount, position),
    };
    pad_response(response)
}

fn confirm_address<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    from: HumanAddr,
    amount: Uint128,
    position: u32,
) -> StdResult<HandleResponse> {
    let config: Config = TypedStoreMut::attach(&mut deps.storage)
        .load(CONFIG_KEY)
        .unwrap();
    correct_amount_of_token(
        amount,
        Uint128(0),
        env.message.sender.clone(),
        config.sscrt.address,
    )?;

    let (mut from_tx, mut to_tx) = verify_txs_for_confirm_address(
        &mut deps.storage,
        &deps.api.canonical_address(&from)?,
        position,
    )?;

    // Update Txs
    from_tx.status = 1;
    to_tx.status = 1;
    update_tx(&mut deps.storage, &from_tx.from.clone(), from_tx)?;
    update_tx(&mut deps.storage, &to_tx.to.clone(), to_tx)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

fn cancel<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    from: HumanAddr,
    amount: Uint128,
    position: u32,
) -> StdResult<HandleResponse> {
    let config: Config = TypedStore::attach(&mut deps.storage)
        .load(CONFIG_KEY)
        .unwrap();
    correct_amount_of_token(
        amount,
        Uint128(0),
        env.message.sender.clone(),
        config.sscrt.address.clone(),
    )?;
    let (mut from_tx, mut to_tx) = verify_txs_for_cancel(
        &mut deps.storage,
        &deps.api.canonical_address(&from)?,
        position,
    )?;
    // Send refund to the creator
    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(snip20::transfer_msg(
        from_tx.creator.clone(),
        from_tx.fee,
        None,
        BLOCK_SIZE,
        config.sscrt.contract_hash,
        config.sscrt.address,
    )?);

    // Update Txs
    from_tx.status = 2;
    to_tx.status = 2;
    update_tx(&mut deps.storage, &from_tx.from.clone(), from_tx)?;
    update_tx(&mut deps.storage, &to_tx.to.clone(), to_tx)?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: None,
    })
}

fn send_payment<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    from: HumanAddr,
    amount: Uint128,
    position: u32,
) -> StdResult<HandleResponse> {
    let (mut from_tx, mut to_tx) = verify_txs(
        &mut deps.storage,
        &deps.api.canonical_address(&from)?,
        amount,
        position,
        1,
        env.message.sender.clone(),
    )?;
    from_tx.status = 3;
    to_tx.status = 3;
    update_tx(&mut deps.storage, &from_tx.from.clone(), from_tx.clone())?;
    update_tx(&mut deps.storage, &to_tx.to.clone(), to_tx)?;
    let config: Config = TypedStore::attach(&mut deps.storage)
        .load(CONFIG_KEY)
        .unwrap();
    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(snip20::transfer_msg(
        config.treasury_address,
        from_tx.fee,
        None,
        BLOCK_SIZE,
        config.sscrt.contract_hash,
        config.sscrt.address,
    )?);
    messages.push(snip20::transfer_msg(
        deps.api.human_address(&from_tx.to)?,
        from_tx.amount,
        None,
        BLOCK_SIZE,
        from_tx.token.contract_hash,
        env.message.sender.clone(),
    )?);

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: None,
    })
}

fn accept_new_admin_nomination<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {
    let mut config: Config = TypedStoreMut::attach(&mut deps.storage)
        .load(CONFIG_KEY)
        .unwrap();
    if config.new_admin_nomination.is_none() {
        return Err(StdError::Unauthorized { backtrace: None });
    }
    authorize(
        env.message.sender.clone(),
        config.new_admin_nomination.clone().unwrap(),
    )?;

    config.admin = config.new_admin_nomination.unwrap();
    config.new_admin_nomination = None;
    TypedStoreMut::attach(&mut deps.storage).store(CONFIG_KEY, &config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

fn correct_amount_of_token(
    amount_received: Uint128,
    amount_wanted: Uint128,
    token_received: HumanAddr,
    token_wanted: HumanAddr,
) -> StdResult<()> {
    if amount_received != amount_wanted {
        return Err(StdError::generic_err("Wrong amount received."));
    }
    if token_received != token_wanted {
        return Err(StdError::generic_err("Wrong token received."));
    }

    Ok(())
}

fn register_token<S: Storage>(
    storage: &mut S,
    contract_code_hash: String,
    token: SecretContract,
) -> StdResult<Option<CosmosMsg>> {
    let mut cosmos_msg: Option<CosmosMsg> = None;
    let mut registered_tokens_storage = RegisteredTokensStorage::from_storage(storage);
    let contract_hash = registered_tokens_storage.get_contract_hash(token.address.clone());
    if contract_hash.is_none() {
        registered_tokens_storage.set_contract_hash(token.address.clone(), &token.contract_hash);
        cosmos_msg = Some(snip20::register_receive_msg(
            contract_code_hash,
            None,
            BLOCK_SIZE,
            token.contract_hash,
            token.address,
        )?);
    }

    Ok(cosmos_msg)
}

fn create_receive_request<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    from: HumanAddr,
    amount: Uint128,
    address: HumanAddr,
    send_amount: Uint128,
    description: Option<String>,
    token: SecretContract,
) -> StdResult<HandleResponse> {
    let config: Config = TypedStore::attach(&mut deps.storage)
        .load(CONFIG_KEY)
        .unwrap();
    correct_amount_of_token(
        amount,
        config.fee,
        env.message.sender.clone(),
        config.sscrt.address,
    )?;

    let mut messages: Vec<CosmosMsg> = vec![];
    let register_token_msg: Option<CosmosMsg> = register_token(
        &mut deps.storage,
        env.contract_code_hash.clone(),
        token.clone(),
    )?;
    if register_token_msg.is_some() {
        messages.push(register_token_msg.unwrap())
    }
    store_txs(
        &mut deps.storage,
        config.fee,
        &deps.api.canonical_address(&address)?,
        &deps.api.canonical_address(&from)?,
        from,
        send_amount,
        token,
        description,
        1,
        &env.block,
    )?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

fn create_send_request<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    from: HumanAddr,
    amount: Uint128,
    address: HumanAddr,
    send_amount: Uint128,
    description: Option<String>,
    token: SecretContract,
) -> StdResult<HandleResponse> {
    let config: Config = TypedStore::attach(&mut deps.storage)
        .load(CONFIG_KEY)
        .unwrap();
    correct_amount_of_token(
        amount,
        config.fee,
        env.message.sender.clone(),
        config.sscrt.address,
    )?;

    let mut messages: Vec<CosmosMsg> = vec![];
    let register_token_msg: Option<CosmosMsg> = register_token(
        &mut deps.storage,
        env.contract_code_hash.clone(),
        token.clone(),
    )?;
    if register_token_msg.is_some() {
        messages.push(register_token_msg.unwrap())
    }
    store_txs(
        &mut deps.storage,
        config.fee,
        &deps.api.canonical_address(&from)?,
        &deps.api.canonical_address(&address)?,
        from,
        send_amount,
        token,
        description,
        0,
        &env.block,
    )?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

fn nominate_new_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    address: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut config: Config = TypedStoreMut::attach(&mut deps.storage)
        .load(CONFIG_KEY)
        .unwrap();
    authorize(env.message.sender.clone(), config.admin.clone())?;

    config.new_admin_nomination = Some(address);
    TypedStoreMut::attach(&mut deps.storage).store(CONFIG_KEY, &config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

fn pad_response(response: StdResult<HandleResponse>) -> StdResult<HandleResponse> {
    response.map(|mut response| {
        response.data = response.data.map(|mut data| {
            space_pad(BLOCK_SIZE, &mut data.0);
            data
        });
        response
    })
}

// Take a Vec<u8> and pad it up to a multiple of `block_size`, using spaces at the end.
fn space_pad(block_size: usize, message: &mut Vec<u8>) -> &mut Vec<u8> {
    let len = message.len();
    let surplus = len % block_size;
    if surplus == 0 {
        return message;
    }

    let missing = block_size - surplus;
    message.reserve(missing);
    message.extend(std::iter::repeat(b' ').take(missing));
    message
}

fn txs<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
    key: String,
    page: u32,
    page_size: u32,
) -> StdResult<Binary> {
    let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY).unwrap();

    // This is here so that the user can use their viewing key for shade for this
    snip20::balance_query(
        &deps.querier,
        address.clone(),
        key.to_string(),
        BLOCK_SIZE,
        config.shade.contract_hash,
        config.shade.address,
    )?;

    let address = deps.api.canonical_address(&address)?;
    let (txs, total) = get_txs(&deps.api, &deps.storage, &address, page, page_size)?;

    let result = QueryAnswer::Txs {
        txs,
        total: Some(total),
    };
    to_binary(&result)
}

fn update_fee<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    fee: Uint128,
) -> StdResult<HandleResponse> {
    let mut config: Config = TypedStoreMut::attach(&mut deps.storage)
        .load(CONFIG_KEY)
        .unwrap();
    authorize(env.message.sender.clone(), config.admin.clone())?;

    config.fee = fee;
    TypedStoreMut::attach(&mut deps.storage).store(CONFIG_KEY, &config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

fn update_treasury_address<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    address: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut config: Config = TypedStoreMut::attach(&mut deps.storage)
        .load(CONFIG_KEY)
        .unwrap();
    authorize(env.message.sender.clone(), config.admin.clone())?;

    config.treasury_address = address;
    TypedStoreMut::attach(&mut deps.storage).store(CONFIG_KEY, &config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::RegisteredTokensReadonlyStorage;
    use crate::transaction_history::{tx_at_position, Tx};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage};

    // === HELPERS ===
    fn init_helper() -> (
        StdResult<InitResponse>,
        Extern<MockStorage, MockApi, MockQuerier>,
    ) {
        let env = mock_env(mock_contract_initiator_address(), &[]);
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            fee: mock_fee(),
            shade: mock_shade(),
            sscrt: mock_sscrt(),
            treasury_address: mock_treasury_address(),
        };
        (init(&mut deps, env, msg), deps)
    }

    fn mock_fee() -> Uint128 {
        Uint128(1_000_000)
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
        HumanAddr::from("admin")
    }

    fn mock_sscrt() -> SecretContract {
        SecretContract {
            address: HumanAddr::from("mock-sscrt-address"),
            contract_hash: "mock-sscrt-contract-hash".to_string(),
        }
    }

    fn mock_shade() -> SecretContract {
        SecretContract {
            address: HumanAddr::from("mock-shade-address"),
            contract_hash: "mock-shade-contract-hash".to_string(),
        }
    }

    fn mock_user_address() -> HumanAddr {
        HumanAddr::from("gary")
    }

    // === INIT TEST ===
    #[test]
    fn test_init() {
        // * it registers receive for sscrt and shade
        let (init_result, deps) = init_helper();
        assert_eq!(
            init_result.unwrap().messages,
            vec![
                snip20::register_receive_msg(
                    mock_env(mock_contract_initiator_address(), &[]).contract_code_hash,
                    None,
                    BLOCK_SIZE,
                    mock_sscrt().contract_hash,
                    mock_sscrt().address,
                )
                .unwrap(),
                snip20::register_receive_msg(
                    mock_env(mock_contract_initiator_address(), &[]).contract_code_hash,
                    None,
                    BLOCK_SIZE,
                    mock_shade().contract_hash,
                    mock_shade().address,
                )
                .unwrap(),
            ],
        );

        // * it sets the correct Config
        let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY).unwrap();
        assert_eq!(
            config,
            Config {
                admin: mock_contract_initiator_address(),
                fee: mock_fee(),
                new_admin_nomination: None,
                shade: mock_shade(),
                sscrt: mock_sscrt(),
                treasury_address: mock_treasury_address(),
            }
        );

        // * it stores the token => contract_hash in storage
        let mut registered_tokens_storage =
            RegisteredTokensReadonlyStorage::from_storage(&deps.storage);
        assert_eq!(
            registered_tokens_storage
                .get_contract_hash(mock_sscrt().address)
                .unwrap(),
            mock_sscrt().contract_hash,
        );
        assert_eq!(
            registered_tokens_storage
                .get_contract_hash(mock_shade().address)
                .unwrap(),
            mock_shade().contract_hash,
        )
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
    fn test_accept_new_admin_nomination() {
        let (_init_result, mut deps) = init_helper();
        let env = mock_env(mock_user_address(), &[]);

        // when a new admin nomination does not exist
        // * it raises an unauthorized error
        let handle_msg = HandleMsg::AcceptNewAdminNomination {};
        let handle_result = handle(&mut deps, env.clone(), handle_msg.clone());
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::Unauthorized { backtrace: None }
        );

        // when a new admin nomination exists
        let new_admin_nomination_msg = HandleMsg::NominateNewAdmin {
            address: mock_user_address(),
        };
        let env = mock_env(mock_contract_initiator_address(), &[]);
        handle(&mut deps, env.clone(), new_admin_nomination_msg).unwrap();

        // = when accepting of new admin nomination is called by the wrong person
        // = * it raises an error
        let handle_result = handle(&mut deps, env.clone(), handle_msg.clone());
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::Unauthorized { backtrace: None }
        );

        // = when accepting of new admin nomination is called by the nominated person
        let env = mock_env(mock_user_address(), &[]);
        handle(&mut deps, env, handle_msg).unwrap();
        let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY).unwrap();
        // = * it updates the admin
        assert_eq!(config.admin, mock_user_address());
        // = * it sets the new admin nomination to None
        assert_eq!(config.new_admin_nomination, None);
    }

    #[test]
    fn test_cancel() {
        let (_init_result, mut deps) = init_helper();
        let env = mock_env(mock_user_address(), &[]);
        let description = Some("Mercy".to_string());

        // when pending address confirmation
        let send_amount: Uint128 = Uint128(555_555);
        let receive_msg = ReceiveMsg::CreateSendRequest {
            address: mock_contract_initiator_address(),
            send_amount: send_amount,
            description: description.clone(),
            token: mock_silk(),
        };
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: mock_fee(),
            msg: to_binary(&receive_msg).unwrap(),
        };
        let handle_result = handle(&mut deps, mock_env(mock_sscrt().address, &[]), handle_msg);
        handle_result.unwrap();
        // = when user sends in a positive amount
        let receive_msg = ReceiveMsg::Cancel { position: 1 };
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: Uint128(5),
            msg: to_binary(&receive_msg).unwrap(),
        };
        let handle_result = handle(&mut deps, env.clone(), handle_msg);
        // = * it raises an error
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "Wrong amount received.".to_string(),
                backtrace: None
            }
        );
        // = when user sends in a zero amount
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: Uint128(0),
            msg: to_binary(&receive_msg).unwrap(),
        };
        // == when user sends in a non-sscrt token
        let handle_result = handle(&mut deps, env.clone(), handle_msg.clone());
        // == * it raises an error
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "Wrong token received.".to_string(),
                backtrace: None
            }
        );
        // == when user sends in sscrt token
        let handle_result = handle(&mut deps, mock_env(mock_sscrt().address, &[]), handle_msg);
        // === when user tries to cancel a Tx that does not exist
        // === * it raises an error
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "AppendStorage access out of bounds".to_string(),
                backtrace: None
            }
        );
        // === when user tries to cancel Tx that exists
        // ==== when user tries to cancel Tx that is pending address confirmation
        let receive_msg = ReceiveMsg::Cancel { position: 0 };
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: Uint128(0),
            msg: to_binary(&receive_msg).unwrap(),
        };
        let handle_result_unwrapped = handle(
            &mut deps,
            mock_env(mock_sscrt().address, &[]),
            handle_msg.clone(),
        )
        .unwrap();
        let mut from_tx = tx_at_position(
            &mut deps.storage,
            &deps.api.canonical_address(&mock_user_address()).unwrap(),
            0,
        )
        .unwrap();
        let mut to_tx = tx_at_position(
            &mut deps.storage,
            &from_tx.to,
            from_tx.other_storage_position,
        )
        .unwrap();
        // ==== * it sends the fee back to the creator
        let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY).unwrap();
        assert_eq!(
            handle_result_unwrapped.messages,
            vec![snip20::transfer_msg(
                from_tx.creator.clone(),
                from_tx.fee,
                None,
                BLOCK_SIZE,
                config.sscrt.contract_hash,
                config.sscrt.address,
            )
            .unwrap()]
        );
        // ==== * it updates the Tx and counter Tx to cancdl
        assert_eq!(from_tx.status, 2);
        assert_eq!(to_tx.status, 2);

        // ==== when user tries to cancel Tx that is cancelled
        // ==== * it raises an error
        let handle_result = handle(
            &mut deps,
            mock_env(mock_sscrt().address, &[]),
            handle_msg.clone(),
        );
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "Tx already cancelled.".to_string(),
                backtrace: None
            }
        );
        // ==== when user tries to cancel Tx that is pending payment
        from_tx.status = 1;
        to_tx.status = 1;
        update_tx(&mut deps.storage, &from_tx.from.clone(), from_tx).unwrap();
        update_tx(&mut deps.storage, &to_tx.to.clone(), to_tx).unwrap();
        let receive_msg = ReceiveMsg::Cancel { position: 0 };
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: Uint128(0),
            msg: to_binary(&receive_msg).unwrap(),
        };
        let handle_result_unwrapped = handle(
            &mut deps,
            mock_env(mock_sscrt().address, &[]),
            handle_msg.clone(),
        )
        .unwrap();
        let mut from_tx = tx_at_position(
            &mut deps.storage,
            &deps.api.canonical_address(&mock_user_address()).unwrap(),
            0,
        )
        .unwrap();
        let mut to_tx = tx_at_position(
            &mut deps.storage,
            &from_tx.to,
            from_tx.other_storage_position,
        )
        .unwrap();
        // ==== * it sends the fee back to the creator
        let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY).unwrap();
        assert_eq!(
            handle_result_unwrapped.messages,
            vec![snip20::transfer_msg(
                from_tx.creator.clone(),
                from_tx.fee,
                None,
                BLOCK_SIZE,
                config.sscrt.contract_hash,
                config.sscrt.address,
            )
            .unwrap()]
        );
        // ==== * it updates the Tx and counter Tx to cancel
        assert_eq!(from_tx.status, 2);
        assert_eq!(to_tx.status, 2);
        // ==== when user tries to cancel Tx that is finalized
        from_tx.status = 3;
        to_tx.status = 3;
        update_tx(&mut deps.storage, &from_tx.from.clone(), from_tx).unwrap();
        update_tx(&mut deps.storage, &to_tx.to.clone(), to_tx).unwrap();
        // ==== * it raises an error
        let handle_result = handle(
            &mut deps,
            mock_env(mock_sscrt().address, &[]),
            handle_msg.clone(),
        );
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "Tx already finalized.".to_string(),
                backtrace: None
            }
        );
    }

    #[test]
    fn test_confirm_address() {
        let (_init_result, mut deps) = init_helper();
        let env = mock_env(mock_user_address(), &[]);
        let description = Some("Mercy".to_string());

        // when send receive exists
        let send_amount: Uint128 = Uint128(555_555);
        let receive_msg = ReceiveMsg::CreateSendRequest {
            address: mock_contract_initiator_address(),
            send_amount: send_amount,
            description: description.clone(),
            token: mock_silk(),
        };
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: mock_fee(),
            msg: to_binary(&receive_msg).unwrap(),
        };
        let handle_result = handle(&mut deps, mock_env(mock_sscrt().address, &[]), handle_msg);
        handle_result.unwrap();
        // = when user sends in a positive amount
        let receive_msg = ReceiveMsg::ConfirmAddress { position: 1 };
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: Uint128(5),
            msg: to_binary(&receive_msg).unwrap(),
        };
        let handle_result = handle(&mut deps, env.clone(), handle_msg);
        // = * it raises an error
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "Wrong amount received.".to_string(),
                backtrace: None
            }
        );
        // = when user sends in a zero amount
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: Uint128(0),
            msg: to_binary(&receive_msg).unwrap(),
        };
        // == when user sends in a non-sscrt token
        let handle_result = handle(&mut deps, env.clone(), handle_msg.clone());
        // == * it raises an error
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "Wrong token received.".to_string(),
                backtrace: None
            }
        );
        // == when user sends in sscrt token
        let handle_result = handle(&mut deps, mock_env(mock_sscrt().address, &[]), handle_msg);
        // === when user tries to confirm address of a Tx that does not exist
        // === * it raises an error
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "AppendStorage access out of bounds".to_string(),
                backtrace: None
            }
        );
        // === when user tries to confirm address of a Tx that exists
        // ==== when user tries to confirm Tx that is pending address confirmation
        // ==== * it updates the Tx and counter Tx to pending payment
        let receive_msg = ReceiveMsg::ConfirmAddress { position: 0 };
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: Uint128(0),
            msg: to_binary(&receive_msg).unwrap(),
        };
        let handle_result_unwrapped = handle(
            &mut deps,
            mock_env(mock_sscrt().address, &[]),
            handle_msg.clone(),
        )
        .unwrap();
        assert_eq!(handle_result_unwrapped.messages, vec![]);
        let from_tx = tx_at_position(
            &mut deps.storage,
            &deps.api.canonical_address(&mock_user_address()).unwrap(),
            0,
        )
        .unwrap();
        let to_tx = tx_at_position(
            &mut deps.storage,
            &from_tx.to,
            from_tx.other_storage_position,
        )
        .unwrap();
        assert_eq!(from_tx.status, 1);
        assert_eq!(to_tx.status, 1);

        // ==== when user tries to confirm Tx that is not pending address confirmation
        // ==== * it raises an error
        let handle_result = handle(
            &mut deps,
            mock_env(mock_sscrt().address, &[]),
            handle_msg.clone(),
        );
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "Tx not waiting for address confirmation.".to_string(),
                backtrace: None
            }
        );
    }

    #[test]
    fn test_create_receive_request() {
        let (_init_result, mut deps) = init_helper();
        let env = mock_env(mock_user_address(), &[]);
        let description = Some("Mercy".to_string());

        // when incorrect fee amount is sent in
        let receive_msg = ReceiveMsg::CreateReceiveRequest {
            address: mock_user_address(),
            send_amount: Uint128(555555),
            description: description.clone(),
            token: mock_silk(),
        };
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: Uint128(555),
            msg: to_binary(&receive_msg).unwrap(),
        };
        let handle_result = handle(&mut deps, env.clone(), handle_msg);
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "Wrong amount received.".to_string(),
                backtrace: None
            }
        );

        // when correct fee amount is sent in
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: mock_fee(),
            msg: to_binary(&receive_msg).unwrap(),
        };
        // = when incorrect fee token is sent in
        let handle_result = handle(&mut deps, env.clone(), handle_msg.clone());
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "Wrong token received.".to_string(),
                backtrace: None
            }
        );

        // = when correct fee token is sent in
        let handle_result = handle(&mut deps, mock_env(mock_sscrt().address, &[]), handle_msg);
        // == when sender sets the receiver to themselves
        // == * it raises an error
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "From and to addresses must be different.".to_string(),
                backtrace: None
            }
        );

        // == when sender sets the receiver to somebody else
        let send_amount: Uint128 = Uint128(555_555);
        let receive_msg = ReceiveMsg::CreateReceiveRequest {
            address: mock_contract_initiator_address(),
            send_amount: send_amount,
            description: description.clone(),
            token: mock_silk(),
        };
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: mock_fee(),
            msg: to_binary(&receive_msg).unwrap(),
        };
        let handle_result = handle(&mut deps, mock_env(mock_sscrt().address, &[]), handle_msg);
        let handle_result_unwrapped = handle_result.unwrap();
        assert_eq!(handle_result_unwrapped.messages, vec![]);
        // == * it creates the txs
        let from_tx = tx_at_position(
            &mut deps.storage,
            &deps.api.canonical_address(&mock_user_address()).unwrap(),
            0,
        )
        .unwrap();
        let to_tx = tx_at_position(
            &mut deps.storage,
            &from_tx.to,
            from_tx.other_storage_position,
        )
        .unwrap();
        assert_eq!(
            from_tx,
            Tx {
                position: 0,
                other_storage_position: 0,
                fee: mock_fee(),
                from: deps
                    .api
                    .canonical_address(&mock_contract_initiator_address())
                    .unwrap(),
                to: deps.api.canonical_address(&mock_user_address()).unwrap(),
                creator: mock_user_address(),
                amount: send_amount,
                token: mock_silk(),
                description: description.clone(),
                status: 1,
                block_time: env.block.time,
                block_height: env.block.height,
            }
        );
        assert_eq!(
            to_tx,
            Tx {
                position: 0,
                other_storage_position: 0,
                fee: mock_fee(),
                from: deps
                    .api
                    .canonical_address(&mock_contract_initiator_address())
                    .unwrap(),
                to: deps.api.canonical_address(&mock_user_address()).unwrap(),
                creator: mock_user_address(),
                amount: send_amount,
                token: mock_silk(),
                description: description,
                status: 1,
                block_time: env.block.time,
                block_height: env.block.height,
            }
        );
    }

    #[test]
    fn test_create_send_request() {
        let (_init_result, mut deps) = init_helper();
        let env = mock_env(mock_user_address(), &[]);
        let description = Some("Mercy".to_string());

        // when incorrect fee amount is sent in
        let receive_msg = ReceiveMsg::CreateSendRequest {
            address: mock_user_address(),
            send_amount: Uint128(555555),
            description: description.clone(),
            token: mock_silk(),
        };
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: Uint128(555),
            msg: to_binary(&receive_msg).unwrap(),
        };
        let handle_result = handle(&mut deps, env.clone(), handle_msg);
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "Wrong amount received.".to_string(),
                backtrace: None
            }
        );

        // when correct fee amount is sent in
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: mock_fee(),
            msg: to_binary(&receive_msg).unwrap(),
        };
        // = when incorrect fee token is sent in
        let handle_result = handle(&mut deps, env.clone(), handle_msg.clone());
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "Wrong token received.".to_string(),
                backtrace: None
            }
        );

        // = when correct fee token is sent in
        let handle_result = handle(&mut deps, mock_env(mock_sscrt().address, &[]), handle_msg);
        // == when sender sets the receiver to themselves
        // == * it raises an error
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::GenericErr {
                msg: "From and to addresses must be different.".to_string(),
                backtrace: None
            }
        );

        // == when sender sets the receiver to somebody else
        let send_amount: Uint128 = Uint128(555_555);
        let receive_msg = ReceiveMsg::CreateSendRequest {
            address: mock_contract_initiator_address(),
            send_amount: send_amount,
            description: description.clone(),
            token: mock_silk(),
        };
        let handle_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: mock_fee(),
            msg: to_binary(&receive_msg).unwrap(),
        };
        let handle_result = handle(&mut deps, mock_env(mock_sscrt().address, &[]), handle_msg);
        let handle_result_unwrapped = handle_result.unwrap();
        assert_eq!(handle_result_unwrapped.messages, vec![]);
        // == * it creates the txs
        let from_tx = tx_at_position(
            &mut deps.storage,
            &deps.api.canonical_address(&mock_user_address()).unwrap(),
            0,
        )
        .unwrap();
        let to_tx = tx_at_position(
            &mut deps.storage,
            &from_tx.to,
            from_tx.other_storage_position,
        )
        .unwrap();
        assert_eq!(
            from_tx,
            Tx {
                position: 0,
                other_storage_position: 0,
                fee: mock_fee(),
                from: deps.api.canonical_address(&mock_user_address()).unwrap(),
                to: deps
                    .api
                    .canonical_address(&mock_contract_initiator_address())
                    .unwrap(),
                creator: mock_user_address(),
                amount: send_amount,
                token: mock_silk(),
                description: description.clone(),
                status: 0,
                block_time: env.block.time,
                block_height: env.block.height,
            }
        );
        assert_eq!(
            to_tx,
            Tx {
                position: 0,
                other_storage_position: 0,
                fee: mock_fee(),
                from: deps.api.canonical_address(&mock_user_address()).unwrap(),
                to: deps
                    .api
                    .canonical_address(&mock_contract_initiator_address())
                    .unwrap(),
                creator: mock_user_address(),
                amount: send_amount,
                token: mock_silk(),
                description: description,
                status: 0,
                block_time: env.block.time,
                block_height: env.block.height,
            }
        );
    }

    #[test]
    fn test_nominate_new_admin() {
        let (_init_result, mut deps) = init_helper();
        let env = mock_env(mock_user_address(), &[]);

        // when called by non-admin
        // * it raises an unauthorized error
        let handle_msg = HandleMsg::NominateNewAdmin {
            address: mock_user_address(),
        };
        let handle_result = handle(&mut deps, env.clone(), handle_msg.clone());
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::Unauthorized { backtrace: None }
        );

        // when admin calls this
        let env = mock_env(mock_contract_initiator_address(), &[]);
        let handle_result = handle(&mut deps, env, handle_msg);
        handle_result.unwrap();
        let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY).unwrap();
        assert_eq!(config.new_admin_nomination, Some(mock_user_address()))
    }

    #[test]
    fn test_update_fee() {
        let (_init_result, mut deps) = init_helper();
        let env = mock_env(mock_user_address(), &[]);
        let new_fee = Uint128(555);

        // when called by non-admin
        // * it raises an unauthorized error
        let handle_msg = HandleMsg::UpdateFee { fee: new_fee };
        let handle_result = handle(&mut deps, env.clone(), handle_msg.clone());
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::Unauthorized { backtrace: None }
        );

        // when admin calls this
        let env = mock_env(mock_contract_initiator_address(), &[]);
        let handle_result = handle(&mut deps, env, handle_msg);
        handle_result.unwrap();
        let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY).unwrap();
        assert_eq!(config.fee, new_fee)
    }

    #[test]
    fn test_update_treasury_fee() {
        let (_init_result, mut deps) = init_helper();
        let env = mock_env(mock_user_address(), &[]);
        let new_treasury_address = mock_contract().address;

        // when called by non-admin
        // * it raises an unauthorized error
        let handle_msg = HandleMsg::UpdateTreasuryAddress {
            address: new_treasury_address.clone(),
        };
        let handle_result = handle(&mut deps, env.clone(), handle_msg.clone());
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::Unauthorized { backtrace: None }
        );

        // when admin calls this
        let env = mock_env(mock_contract_initiator_address(), &[]);
        let handle_result = handle(&mut deps, env, handle_msg);
        handle_result.unwrap();
        let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY).unwrap();
        assert_eq!(config.treasury_address, new_treasury_address)
    }
}

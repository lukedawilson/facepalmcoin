#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, Storage, Uint128};
use cosmwasm_storage::{Bucket, ReadonlyBucket, bucket, bucket_read};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{BalanceResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

const CONTRACT_NAME: &str = "crates.io:cw-facepalm-coin";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const PREFIX_BALANCE: &[u8] = b"balance";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let owner = info.sender.clone();
    let initial_balance = msg.initial_balance.clone();

    let state = State {
        burn_address: msg.burn_address.clone(),
        owner: owner.clone()
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    let owner_raw = deps.api.addr_canonicalize(owner.clone().as_str())?;
    balance(deps.storage).update(&owner_raw, |balance| -> StdResult<_> {
        Ok(balance.unwrap_or_default() + initial_balance)
    })?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("burn_address", msg.burn_address.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { receiver, amount } => transfer(deps, info.sender.clone(), receiver, amount),
        ExecuteMsg::Burn { amount } => burn(deps, info.sender.clone(), amount),
    }
}

pub fn transfer(deps: DepsMut, sender: Addr, receiver: Addr, amount: Uint128) -> Result<Response, ContractError> {
    let sender_raw = deps.api.addr_canonicalize(sender.as_str())?;
    let sender_balance = read_balance(deps.storage)
        .may_load(&sender_raw.as_slice())?
        .unwrap_or_default();
    if sender_balance < amount {
        return Err(ContractError::InsufficientBalance {});
    }

    balance(deps.storage).update(&sender_raw, |balance| -> StdResult<_> {
        Ok(balance.unwrap_or_default() - amount)
    })?;

    let receiver_raw = deps.api.addr_canonicalize(receiver.as_str())?;
    balance(deps.storage).update(&receiver_raw, |balance| -> StdResult<_> {
        Ok(balance.unwrap_or_default() + amount)
    })?;

    Ok(Response::new().add_attribute("method", "empty"))
}

pub fn burn(deps: DepsMut, sender: Addr, amount: Uint128) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage).ok().unwrap();
    transfer(deps, sender, state.burn_address, amount)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBalance { address } => to_binary(&query_balance(deps, address)?)
    }
}

fn query_balance(deps: Deps, address: Addr) -> StdResult<BalanceResponse> {
    let address_raw = deps.api.addr_canonicalize(address.as_str())?;
    let balance = read_balance(deps.storage)
        .may_load(address_raw.as_slice())?
        .unwrap_or_default();

    Ok(BalanceResponse { balance })
}

fn balance(storage: &mut dyn Storage) -> Bucket<Uint128> {
    bucket(storage, PREFIX_BALANCE)
}

fn read_balance(storage: &dyn Storage) -> ReadonlyBucket<Uint128> {
    bucket_read(storage, PREFIX_BALANCE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary};

    #[test]
    fn instantiate_transfer_burn() {
        // instantiate
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg { burn_address: Addr::unchecked("burnbabyburn"), initial_balance: Uint128::new(1000) };
        let info = mock_info("foo", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let balance_response = query(deps.as_ref(), mock_env(), QueryMsg::GetBalance { address: Addr::unchecked("foo") }).unwrap();
        let balance_value: BalanceResponse = from_binary(&balance_response).unwrap();
        assert_eq!(Uint128::new(1000), balance_value.balance);

        // transfer
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("foo", &[]),
            ExecuteMsg::Transfer { receiver: Addr::unchecked("bar"), amount: Uint128::new(10) },
        )
        .unwrap();
        println!("{:?}", ExecuteMsg::Transfer { receiver: Addr::unchecked("bar"), amount: Uint128::new(10) });

        let mut foo_balance_response = query(deps.as_ref(), mock_env(), QueryMsg::GetBalance { address: Addr::unchecked("foo") }).unwrap();
        let mut foo_balance_value: BalanceResponse = from_binary(&foo_balance_response).unwrap();
        assert_eq!(Uint128::new(990), foo_balance_value.balance);

        let mut bar_balance_response = query(deps.as_ref(), mock_env(), QueryMsg::GetBalance { address: Addr::unchecked("bar") }).unwrap();
        let mut bar_balance_value: BalanceResponse = from_binary(&bar_balance_response).unwrap();
        assert_eq!(Uint128::new(10), bar_balance_value.balance);

        // burn
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bar", &[]),
            ExecuteMsg::Burn { amount: Uint128::new(3) },
        )
        .unwrap();

        foo_balance_response = query(deps.as_ref(), mock_env(), QueryMsg::GetBalance { address: Addr::unchecked("foo") }).unwrap();
        foo_balance_value = from_binary(&foo_balance_response).unwrap();
        assert_eq!(Uint128::new(990), foo_balance_value.balance);

        bar_balance_response = query(deps.as_ref(), mock_env(), QueryMsg::GetBalance { address: Addr::unchecked("bar") }).unwrap();
        bar_balance_value = from_binary(&bar_balance_response).unwrap();
        assert_eq!(Uint128::new(7), bar_balance_value.balance);

        let burn_balance_response = query(deps.as_ref(), mock_env(), QueryMsg::GetBalance { address: Addr::unchecked("burnbabyburn") }).unwrap();
        let burn_balance_value: BalanceResponse = from_binary(&burn_balance_response).unwrap();
        assert_eq!(Uint128::new(3), burn_balance_value.balance);
    }
}

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

use cw_controllers::Admin;

use cw4_stake::state::{CONFIG, Config};
use cw4_stake::msg::StakedResponse;
use cw4_stake::contract::{execute_bond, execute_unbond, query_staked};
use cw4_stake::ContractError as CW4_ContractError;

use cw20::{Denom, Balance};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:astro_stake";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const SUPER_ADMIN: &Admin = &Admin::new("SUPER_ADMIN");
const ASTRO_ADDRESS: &str  = "inj16wx7ye3ce060tjvmmpu8lm0ak5xr7gm2e0qwcq"; //"ibc/EBD5A24C554198EBAF44979C5B4D2C2D312E6EBAB71962C92F735499C7575839";


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // let super_admin_addr = deps.api.addr_validate(&msg.admin)?;
    // SUPER_ADMIN.set(deps.branch(), Some(super_admin_addr))?;
    
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    let config = Config{
        denom: Denom::Native(ASTRO_ADDRESS.into()),
        tokens_per_weight: 1_000_000u128.into(),
        min_bond: 1u128.into(),
        unbonding_period: cw_utils::Duration::Time(86400)
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", msg.admin))
        
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, CW4_ContractError> {
    match msg {
        ExecuteMsg::OAstroStake {} => execute_bond(deps, env,Balance::from(info.funds), info.sender),
        ExecuteMsg::OAstroUnstake { amount } => execute_unbond(deps, env, info, amount.into())
    }
}

fn check_admin(deps: DepsMut, info: MessageInfo) -> Result<(), ContractError>{
    let super_admin = SUPER_ADMIN.assert_admin(deps.as_ref(), &info.sender);
    
    match super_admin {
        Ok(()) => Ok(()),
        Err(_not_admin) => return Err(ContractError::Unauthorized {  })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Staked {addr} => to_binary(&query_staked(deps, addr)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    fn get_instantiate_msg() -> InstantiateMsg{
        InstantiateMsg { admin: "inj16wx7ye3ce060tjvmmpu8lm0ak5xr7gm2e0qwcq".to_string() }
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = get_instantiate_msg();
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn bond() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = get_instantiate_msg();
        let info = mock_info("creator", &coins(5, ASTRO_ADDRESS));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();


        let info = mock_info("inj16wx7ye3ce060tjvmmpu8lm0ak5xr7gm2e0qwcp", &coins(5u128, ASTRO_ADDRESS));
        let msg = ExecuteMsg::OAstroStake {  };
        let _res = execute(deps.as_mut(), mock_env(), info, msg);
        // println!("{:#?}", _res);

       let res = query(deps.as_ref(), mock_env(), QueryMsg::Staked { addr: "inj16wx7ye3ce060tjvmmpu8lm0ak5xr7gm2e0qwcp".to_string() }).unwrap();
        let value: StakedResponse = from_binary(&res).unwrap();
        assert_eq!(5u128, value.stake.u128()); 
        
        
    }
}

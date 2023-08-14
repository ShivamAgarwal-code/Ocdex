#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{ Uint128, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{TokenAmountResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

use cw_controllers::Admin;
use cw20_base::contract::{execute_mint, query_balance};
use cw20_base::state::{TOKEN_INFO, TokenInfo, MinterData};

use crate::state::{POOLS_CONTAINER, PoolInfo, TMP_POOLS_CONTAINER};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:astro_stake";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const SUPER_ADMIN: &Admin = &Admin::new("SUPER_ADMIN");
const ASTRO_ADDRESS: &str  = "ibc/EBD5A24C554198EBAF44979C5B4D2C2D312E6EBAB71962C92F735499C7575839";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let super_admin_addr = deps.api.addr_validate(&msg.admin)?;
    SUPER_ADMIN.set(deps.branch(), Some(super_admin_addr))?;
    
    let token_info = TokenInfo{
        name: "octaneAstro".to_string(),
        symbol: "oAstro".to_string(),
        decimals: 6,
        total_supply: Uint128::zero(),
        mint: Some(MinterData { minter: _env.contract.address, cap: None })
    };

    TOKEN_INFO.save(deps.storage, &token_info)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    

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
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::XAstroStake {} => stake(deps, info, env),
        ExecuteMsg::AddPool { lp_token, guage, factory } => add_pool(deps, info, lp_token, guage, factory)

    }
}

fn add_pool(mut deps: DepsMut, info: MessageInfo, lp_token: String, guage: String, factory: String) -> Result<Response, ContractError>{
    check_admin(deps.branch(), info)?;

    let lp_token_addr = deps.api.addr_validate(&lp_token)?;
    let pools = POOLS_CONTAINER.load(deps.storage)?;

    let found = pools.items.iter().find(|item| item.lp_token == lp_token_addr);
    
    let _ = match found {
        Some(_) => Ok(()),
        None => TMP_POOLS_CONTAINER.save(deps.storage, lp_token_addr, &true)
    };
    

    Ok(Response::new())
}

fn stake(deps: DepsMut, info: MessageInfo, env: Env) -> Result<Response, ContractError> {
    let incoming_astro_amount = info
                        .funds
                        .iter()
                        .find(|c| c.denom == ASTRO_ADDRESS.to_string())
                        .map(|c| c.amount)
                        .unwrap_or_else(Uint128::zero);
                        
    println!("{:#}", incoming_astro_amount);
    
    if incoming_astro_amount > Uint128::from(0u128){
        let sub_info = MessageInfo {
            sender: env.contract.address.clone(),
            funds: vec![],
        };
        let _res= execute_mint(deps, env, sub_info, info.sender.to_string(), Uint128::from(incoming_astro_amount)).unwrap();
    }

    Ok(Response::new().add_attribute("method", "xastro staked"))
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
        QueryMsg::GetTokenAmount {addr} => to_binary(&query_token(deps, addr)?),
    }
}

fn query_token(deps: Deps, addr: String) -> StdResult<TokenAmountResponse> {
    let res = query_balance(deps, addr)?;
    Ok(TokenAmountResponse { amount: res.balance.u128() })    
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
    fn stake() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = get_instantiate_msg();
        let info = mock_info("creator", &coins(5, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();


        let info = mock_info("anyone", &coins(5, ASTRO_ADDRESS));
        let msg = ExecuteMsg::XAstroStake {  };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

       let res = query(deps.as_ref(), mock_env(), QueryMsg::GetTokenAmount { addr: "anyone".to_string() }).unwrap();
        let value: TokenAmountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.amount); 
        
        
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = get_instantiate_msg();
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        // let unauth_info = mock_info("anyone", &coins(2, "token"));
        // let msg = ExecuteMsg::Reset { count: 5 };
        // let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        // match res {
        //     Err(ContractError::Unauthorized {}) => {}
        //     _ => panic!("Must return unauthorized error"),
        // }

        // // only the original creator can reset the counter
        // let auth_info = mock_info("creator", &coins(2, "token"));
        // let msg = ExecuteMsg::Reset { count: 5 };
        // let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // // should now be 5
        // let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        // let value: CountResponse = from_binary(&res).unwrap();
        // assert_eq!(5, value.count);
    }
}

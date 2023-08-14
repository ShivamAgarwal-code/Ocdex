#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{ Uint128, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{TokenAmountResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

use cw_controllers::Admin;
use cw20_base::contract::{execute_mint, query_balance};
use cw20_base::state::{TOKEN_INFO, TokenInfo, MinterData};

use crate::state::{CONFIG, Config, RewardType};
use crate::state::{REWARD_MAP, REWARDS};

const MAX_REWARDS:u128 = 8;
const INJ_ADDRESS: &str = "inj16wx7ye3ce060tjvmmpu8lm0ak5xr7gm2e0qwcq";

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:octane_reward_pool";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const SUPER_ADMIN: &Admin = &Admin::new("SUPER_ADMIN");


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
        ExecuteMsg::Init { astro_token, astro_guage, octane_staker, octane_booster, lp_token, pool_id } => initialize(deps, info, astro_token, astro_guage, octane_staker, octane_booster, lp_token, pool_id),
        ExecuteMsg::GetReward{account, forward_to} => get_reward(deps, info, account, forward_to),
    }
}

fn initialize(mut deps: DepsMut, info: MessageInfo, astro_token: String, astro_guage: String, octane_staker:String, octane_booster: String, lp_token: String, pool_id: u128) -> Result<Response, ContractError>{
    // check_admin(deps.branch(), info)?;

    let config_to_store = Config{
        astro_token: deps.api.addr_validate(&astro_token)?,
        astro_guage: deps.api.addr_validate(&astro_guage)?,
        octane_staker: deps.api.addr_validate(&octane_staker)?,
        octane_booster: deps.api.addr_validate(&octane_booster)?,
        lp_token: deps.api.addr_validate(&lp_token)?,
        pool_id
    };

    CONFIG.save(deps.storage, &config_to_store)?;

    insert_reward_token(deps, INJ_ADDRESS.into())?;


    Ok(Response::new())
}

fn insert_reward_token(deps: DepsMut, reward_token: String) -> Result<Response, ContractError>{
    let reward_token_addr = deps.api.addr_validate(&reward_token)?;
    REWARDS.update(deps.storage, |mut rewards| -> Result<_, ContractError>{
        if rewards.items.len() < MAX_REWARDS as usize{
            rewards.items.push(RewardType{
                reward_token: reward_token_addr,
                reward_integral: 0,
                reward_remaining: 0
            });
        }
        Ok(rewards)
    })?;

    Ok(Response::new().add_attribute("added reward token", reward_token))

}

fn get_reward(deps: DepsMut, info: MessageInfo, account: String, forward_to: String) -> Result<Response, ContractError>{
    checkpoint(deps, info, account, forward_to);
    Ok(Response::new())
}

fn checkpoint(deps: DepsMut, info: MessageInfo, account: String, forward_to: String){
    update_and_claim_rewards(deps, info);
}

fn update_and_claim_rewards(deps: DepsMut, info: MessageInfo) {
    //TODO: Check if the pool is shutdown
    update_rewards_list(deps, info);
}

fn update_rewards_list(deps: DepsMut, info: MessageInfo){
    for i in 0..MAX_REWARDS {
        //TODO: Astroport hasn't implemented Gauges
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

    const ASTRO_ADDRESS: &str  = "astro";
    

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
    fn initialize() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = get_instantiate_msg();
        let info = mock_info("creator", &coins(5, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();


        let info = mock_info(&"inj16wx7ye3ce060tjvmmpu8lm0ak5xr7gm2e0qwcq".to_string(), &coins(5, ASTRO_ADDRESS));
        let msg = ExecuteMsg::Init { astro_token: ASTRO_ADDRESS.to_owned(), astro_guage: ASTRO_ADDRESS.to_owned(), octane_staker:ASTRO_ADDRESS.to_owned(), octane_booster:ASTRO_ADDRESS.to_owned(), lp_token:ASTRO_ADDRESS.to_owned(), pool_id: 1};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetTokenAmount { addr: "anyone".to_string() }).unwrap();
    //     let value: TokenAmountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.amount); 
        
        
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

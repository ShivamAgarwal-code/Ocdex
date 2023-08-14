use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardType{
    pub reward_token: Addr,
    pub reward_integral: u128,
    pub reward_remaining: u128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Rewards{
   pub items: Vec<RewardType>
}
pub const REWARDS: Item<Rewards> = Item::new("rewards");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config{
    pub astro_token: Addr,
    pub astro_guage: Addr,
    pub octane_staker: Addr,
    pub octane_booster: Addr,
    pub lp_token: Addr,
    pub pool_id: u128
}

pub const CONFIG: Item<Config> = Item::new("config"); 

pub const REWARD_MAP: Map<Addr, u128> = Map::new("reward_map");
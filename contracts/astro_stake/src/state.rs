use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolInfo {
    pub lp_token: Addr,
    pub guage: Addr,
    pub rewards: Addr,
    pub factory: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Pools{
    pub items: Vec<PoolInfo>
}

pub const POOLS_CONTAINER: Item<Pools> = Item::new("pools");

pub const TMP_POOLS_CONTAINER: Map<Addr, bool> = Map::new("tmp_pools");
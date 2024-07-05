use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub token_address: String,
    pub treasury_address: String,
    pub breeding_contract: String,
     pub asset_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    RewardPlayer { player: String, amount: Uint128 },
    CollectFee { amount: Uint128 },
    BreedNft { parent1: String, parent2: String },
    BattleOutcome { player: String, result: String },
    ManageGuild { guild: String, player: String, action: String },
    ChargeTransactionFee { amount: Uint128 },
    SellAsset { asset_id: String, amount: Uint128 },
    WithdrawTreasury { amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    GetPlayerRewards { player: String },
    #[returns(Uint128)]
    GetTreasuryBalance {},
    #[returns(Vec<String>)]
    GetGuildMembers { guild: String },
}

#[cw_serde]

pub struct BreedingMsg {
    pub parent1: String,
    pub parent2: String,
}
#[cw_serde]

pub struct AssetMsg {
    pub asset_id: String,
    pub amount: Uint128,
}
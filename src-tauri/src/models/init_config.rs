use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInitConfig {
    pub game_id: String,
    pub real_name: Option<String>,
    pub nationality: String,
    pub position: String, // "Top"/"Jug"/"Mid"/"Adc"/"Sup"
    pub age: u8,
    pub ability: u8,
    pub potential: u8,
    pub is_starter: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPoolPlayerInitConfig {
    pub game_id: String,
    pub real_name: String,
    pub position: String, // "Top"/"Jug"/"Mid"/"Adc"/"Sup"
    pub ability: u8,
    pub potential: u8,
    pub stability: u8,
    pub age: u8,
    pub tag: String, // "Genius"/"Normal"/"Ordinary"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInitConfig {
    pub name: String,
    pub short_name: String,
    pub initial_balance: i64, // 单位: 元
    pub players: Vec<PlayerInitConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionInitConfig {
    pub id: u64,
    pub name: String,
    pub short_name: String,
    pub teams: Vec<TeamInitConfig>,
    pub free_agents: Vec<PlayerInitConfig>,
    pub draft_pool: Vec<DraftPoolPlayerInitConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInitConfig {
    pub regions: Vec<RegionInitConfig>,
}

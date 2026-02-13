//! 选秀选手池初始数据
//! 包含四大赛区各50名新秀选手，共200人

use serde::{Deserialize, Serialize};

/// 选秀选手配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPlayerConfig {
    pub game_id: String,
    pub real_name: String,
    pub position: String, // "Top" | "Jug" | "Mid" | "Adc" | "Sup"
    pub ability: u8,
    pub potential: u8,
    pub stability: u8,
    pub age: u8,
    pub tag: String, // "Genius" | "Normal" | "Ordinary"
}

/// 获取赛区选秀选手池
/// region_id: 1=LPL, 2=LCK, 3=LEC, 4=LCS
pub fn get_draft_pool(region_id: u64) -> Vec<DraftPlayerConfig> {
    let json_str = match region_id {
        2 => include_str!("../../../draft_pool_lck.json"),
        3 => include_str!("../../../draft_pool_lec.json"),
        4 => include_str!("../../../draft_pool_lcs.json"),
        1 | _ => include_str!("../../../draft_pool_lpl.json"),
    };

    serde_json::from_str(json_str).unwrap_or_default()
}

/// 获取赛区对应的国籍
pub fn get_region_nationality(region_id: u64) -> &'static str {
    match region_id {
        1 => "CN", // LPL
        2 => "KR", // LCK
        3 => "EU", // LEC
        4 => "NA", // LCS
        _ => "CN",
    }
}

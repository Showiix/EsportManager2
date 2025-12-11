use serde::{Deserialize, Serialize};
use crate::models::{PlayerTag, Position};

/// 选秀球员 (选秀池)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPlayer {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub region_id: u64,
    /// 选秀排名 (1=状元, 2=榜眼, 3=探花...)
    pub draft_rank: u8,
    pub game_id: String,
    pub real_name: Option<String>,
    pub nationality: Option<String>,
    pub age: u8,
    pub ability: u8,
    pub potential: u8,
    pub tag: PlayerTag,
    pub position: Option<Position>,
    /// 是否已被选中
    pub is_picked: bool,
    /// 被哪支队伍选中
    pub picked_by_team_id: Option<u64>,
}

/// 选秀顺位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftOrder {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub region_id: u64,
    pub team_id: u64,
    /// 夏季赛排名
    pub summer_rank: u32,
    /// 选秀顺位 (抽签后)
    pub draft_position: u32,
    /// 抽签结果描述
    pub lottery_result: Option<String>,
}

/// 选秀结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftResult {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub region_id: u64,
    pub draft_player_id: u64,
    pub team_id: u64,
    /// 第几顺位
    pub pick_number: u32,
    /// 创建的选手ID
    pub player_id: Option<u64>,
}

/// 选秀抽签概率配置
/// 排名越靠后，获得高顺位的概率越高
pub fn draft_lottery_weights() -> Vec<(u32, Vec<f64>)> {
    // (排名, [获得1号签概率, 获得2号签概率, ...])
    // 排名1-14，每个排名都有一个概率分布
    vec![
        // 第14名 (最高获得状元签概率)
        (14, vec![0.25, 0.20, 0.15, 0.10, 0.08, 0.06, 0.05, 0.04, 0.03, 0.02, 0.01, 0.005, 0.003, 0.002]),
        // 第13名
        (13, vec![0.20, 0.22, 0.15, 0.10, 0.08, 0.07, 0.06, 0.05, 0.03, 0.02, 0.01, 0.007, 0.003]),
        // 第12名
        (12, vec![0.15, 0.18, 0.18, 0.12, 0.10, 0.08, 0.06, 0.05, 0.04, 0.02, 0.015, 0.005]),
        // 第11名
        (11, vec![0.12, 0.15, 0.16, 0.15, 0.12, 0.10, 0.08, 0.05, 0.04, 0.02, 0.01]),
        // 第10名
        (10, vec![0.10, 0.10, 0.14, 0.15, 0.15, 0.12, 0.10, 0.06, 0.04, 0.04]),
        // 第9名
        (9, vec![0.08, 0.08, 0.10, 0.12, 0.15, 0.15, 0.12, 0.10, 0.06, 0.04]),
        // 第8名
        (8, vec![0.05, 0.04, 0.06, 0.10, 0.12, 0.15, 0.16, 0.15, 0.10, 0.07]),
        // 第7名
        (7, vec![0.03, 0.02, 0.04, 0.08, 0.10, 0.13, 0.15, 0.18, 0.15, 0.12]),
        // 第6名
        (6, vec![0.015, 0.01, 0.015, 0.05, 0.08, 0.10, 0.14, 0.17, 0.20, 0.175]),
        // 第5名
        (5, vec![0.005, 0.005, 0.01, 0.02, 0.05, 0.07, 0.10, 0.15, 0.22, 0.27]),
        // 第1-4名通常不参与选秀抽签，保持原顺位
        (4, vec![0.0; 14]),
        (3, vec![0.0; 14]),
        (2, vec![0.0; 14]),
        (1, vec![0.0; 14]),
    ]
}

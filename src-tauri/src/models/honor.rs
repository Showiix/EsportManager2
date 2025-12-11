use serde::{Deserialize, Serialize};

/// 荣誉类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HonorType {
    // 战队荣誉
    /// 赛事冠军
    TeamChampion,
    /// 赛事亚军
    TeamRunnerUp,
    /// 赛事季军
    TeamThird,
    /// 赛事殿军
    TeamFourth,
    /// 常规赛第一
    RegularSeasonFirst,

    // 选手荣誉
    /// 赛事MVP（整个赛事表现最佳）
    TournamentMvp,
    /// 决赛MVP
    FinalsMvp,
    /// 常规赛MVP
    RegularSeasonMvp,
    /// 季后赛MVP
    PlayoffsMvp,

    // 选手冠军荣誉（跟随战队）
    /// 选手获得冠军（作为冠军队成员）
    PlayerChampion,
}

impl HonorType {
    pub fn name(&self) -> &'static str {
        match self {
            HonorType::TeamChampion => "冠军",
            HonorType::TeamRunnerUp => "亚军",
            HonorType::TeamThird => "季军",
            HonorType::TeamFourth => "殿军",
            HonorType::RegularSeasonFirst => "常规赛第一",
            HonorType::TournamentMvp => "赛事MVP",
            HonorType::FinalsMvp => "决赛MVP",
            HonorType::RegularSeasonMvp => "常规赛MVP",
            HonorType::PlayoffsMvp => "季后赛MVP",
            HonorType::PlayerChampion => "冠军成员",
        }
    }

    /// 是否是战队荣誉
    pub fn is_team_honor(&self) -> bool {
        matches!(
            self,
            HonorType::TeamChampion
                | HonorType::TeamRunnerUp
                | HonorType::TeamThird
                | HonorType::TeamFourth
                | HonorType::RegularSeasonFirst
        )
    }

    /// 是否是MVP类型荣誉
    pub fn is_mvp_honor(&self) -> bool {
        matches!(
            self,
            HonorType::TournamentMvp
                | HonorType::FinalsMvp
                | HonorType::RegularSeasonMvp
                | HonorType::PlayoffsMvp
        )
    }
}

/// MVP统计数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HonorStats {
    /// 总影响力分数
    pub total_impact: f64,
    /// 单场MVP次数
    pub mvp_count: u32,
    /// 参与局数
    pub games_played: u32,
    /// 胜场数
    pub wins: u32,
    /// 平均发挥值
    pub avg_performance: f64,
}

/// 荣誉记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Honor {
    pub id: u64,
    pub save_id: String,
    pub honor_type: HonorType,
    pub season_id: u64,
    pub tournament_id: u64,
    pub tournament_name: String,
    pub tournament_type: String,

    // 获得者（战队或选手）
    pub team_id: Option<u64>,
    pub team_name: Option<String>,
    pub player_id: Option<u64>,
    pub player_name: Option<String>,
    pub position: Option<String>,

    // MVP统计数据
    pub stats: Option<HonorStats>,

    pub created_at: String,
}

impl Honor {
    /// 创建战队荣誉
    pub fn new_team_honor(
        save_id: &str,
        honor_type: HonorType,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        team_id: u64,
        team_name: &str,
    ) -> Self {
        Self {
            id: 0,
            save_id: save_id.to_string(),
            honor_type,
            season_id,
            tournament_id,
            tournament_name: tournament_name.to_string(),
            tournament_type: tournament_type.to_string(),
            team_id: Some(team_id),
            team_name: Some(team_name.to_string()),
            player_id: None,
            player_name: None,
            position: None,
            stats: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 创建选手荣誉
    pub fn new_player_honor(
        save_id: &str,
        honor_type: HonorType,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        team_id: u64,
        team_name: &str,
        player_id: u64,
        player_name: &str,
        position: &str,
        stats: Option<HonorStats>,
    ) -> Self {
        Self {
            id: 0,
            save_id: save_id.to_string(),
            honor_type,
            season_id,
            tournament_id,
            tournament_name: tournament_name.to_string(),
            tournament_type: tournament_type.to_string(),
            team_id: Some(team_id),
            team_name: Some(team_name.to_string()),
            player_id: Some(player_id),
            player_name: Some(player_name.to_string()),
            position: Some(position.to_string()),
            stats,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// 荣誉殿堂数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HonorHallData {
    /// 所有冠军记录
    pub champions: Vec<Honor>,
    /// 所有MVP记录
    pub mvps: Vec<Honor>,
    /// 按赛事类型分组的冠军
    pub champions_by_type: std::collections::HashMap<String, Vec<Honor>>,
}

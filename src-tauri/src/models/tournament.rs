use serde::{Deserialize, Serialize};

/// 赛事类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TournamentType {
    SpringRegular,
    SpringPlayoffs,
    SummerRegular,
    SummerPlayoffs,
    Msi,
    MadridMasters,
    ClaudeIntercontinental,
    WorldChampionship,
    ShanghaiMasters,
    IcpIntercontinental,
    SuperIntercontinental,
}

impl TournamentType {
    pub fn name(&self) -> &'static str {
        match self {
            TournamentType::SpringRegular => "春季赛常规赛",
            TournamentType::SpringPlayoffs => "春季赛季后赛",
            TournamentType::SummerRegular => "夏季赛常规赛",
            TournamentType::SummerPlayoffs => "夏季赛季后赛",
            TournamentType::Msi => "MSI季中赛",
            TournamentType::MadridMasters => "马德里大师赛",
            TournamentType::ClaudeIntercontinental => "Claude洲际赛",
            TournamentType::WorldChampionship => "S世界赛",
            TournamentType::ShanghaiMasters => "上海大师赛",
            TournamentType::IcpIntercontinental => "ICP四赛区洲际对抗赛",
            TournamentType::SuperIntercontinental => "Super洲际年度邀请赛",
        }
    }

    /// 是否是赛区内赛事
    pub fn is_regional(&self) -> bool {
        matches!(
            self,
            TournamentType::SpringRegular
                | TournamentType::SpringPlayoffs
                | TournamentType::SummerRegular
                | TournamentType::SummerPlayoffs
        )
    }

    /// 是否是国际赛事
    pub fn is_international(&self) -> bool {
        !self.is_regional()
    }
}

/// 赛事状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TournamentStatus {
    Upcoming,
    InProgress,
    Completed,
}

/// 赛事
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub tournament_type: TournamentType,
    pub name: String,
    pub region_id: Option<u64>,
    pub status: TournamentStatus,
    pub current_stage: Option<String>,
    pub current_round: Option<u32>,
}

/// 积分配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointsConfig {
    pub tournament_type: TournamentType,
    pub position: String,
    pub points: u32,
}

/// 默认积分配置
pub fn default_points_config() -> Vec<PointsConfig> {
    vec![
        // 联赛季后赛积分
        PointsConfig { tournament_type: TournamentType::SpringPlayoffs, position: "CHAMPION".into(), points: 12 },
        PointsConfig { tournament_type: TournamentType::SpringPlayoffs, position: "RUNNER_UP".into(), points: 10 },
        PointsConfig { tournament_type: TournamentType::SpringPlayoffs, position: "THIRD".into(), points: 8 },
        PointsConfig { tournament_type: TournamentType::SpringPlayoffs, position: "FOURTH".into(), points: 6 },
        PointsConfig { tournament_type: TournamentType::SpringPlayoffs, position: "5TH_8TH".into(), points: 3 },

        // MSI积分
        PointsConfig { tournament_type: TournamentType::Msi, position: "CHAMPION".into(), points: 20 },
        PointsConfig { tournament_type: TournamentType::Msi, position: "RUNNER_UP".into(), points: 16 },
        PointsConfig { tournament_type: TournamentType::Msi, position: "THIRD".into(), points: 12 },
        PointsConfig { tournament_type: TournamentType::Msi, position: "FOURTH".into(), points: 8 },
        PointsConfig { tournament_type: TournamentType::Msi, position: "LOSERS_R2".into(), points: 6 },
        PointsConfig { tournament_type: TournamentType::Msi, position: "LOSERS_R1".into(), points: 4 },

        // 世界赛积分
        PointsConfig { tournament_type: TournamentType::WorldChampionship, position: "CHAMPION".into(), points: 20 },
        PointsConfig { tournament_type: TournamentType::WorldChampionship, position: "RUNNER_UP".into(), points: 16 },
        PointsConfig { tournament_type: TournamentType::WorldChampionship, position: "THIRD".into(), points: 12 },
        PointsConfig { tournament_type: TournamentType::WorldChampionship, position: "FOURTH".into(), points: 8 },
        PointsConfig { tournament_type: TournamentType::WorldChampionship, position: "KNOCKOUT_R1".into(), points: 6 },
        PointsConfig { tournament_type: TournamentType::WorldChampionship, position: "GROUP_STAGE".into(), points: 4 },
    ]
}

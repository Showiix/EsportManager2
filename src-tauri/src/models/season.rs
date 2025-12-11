use serde::{Deserialize, Serialize};

/// 赛季阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeasonPhase {
    /// 春季赛常规赛
    SpringRegular,
    /// 春季赛季后赛
    SpringPlayoffs,
    /// MSI季中赛
    Msi,
    /// 马德里大师赛
    MadridMasters,
    /// 夏季赛常规赛
    SummerRegular,
    /// 夏季赛季后赛
    SummerPlayoffs,
    /// Claude洲际赛
    ClaudeIntercontinental,
    /// S世界赛
    WorldChampionship,
    /// 上海大师赛
    ShanghaiMasters,
    /// ICP四赛区洲际对抗赛
    IcpIntercontinental,
    /// Super洲际年度邀请赛
    SuperIntercontinental,
    /// 转会期
    TransferWindow,
    /// 选秀
    Draft,
    /// 赛季结束
    SeasonEnd,
}

impl SeasonPhase {
    /// 获取阶段名称
    pub fn name(&self) -> &'static str {
        match self {
            SeasonPhase::SpringRegular => "春季赛常规赛",
            SeasonPhase::SpringPlayoffs => "春季赛季后赛",
            SeasonPhase::Msi => "MSI季中赛",
            SeasonPhase::MadridMasters => "马德里大师赛",
            SeasonPhase::SummerRegular => "夏季赛常规赛",
            SeasonPhase::SummerPlayoffs => "夏季赛季后赛",
            SeasonPhase::ClaudeIntercontinental => "Claude洲际赛",
            SeasonPhase::WorldChampionship => "S世界赛",
            SeasonPhase::ShanghaiMasters => "上海大师赛",
            SeasonPhase::IcpIntercontinental => "ICP四赛区洲际对抗赛",
            SeasonPhase::SuperIntercontinental => "Super洲际年度邀请赛",
            SeasonPhase::TransferWindow => "转会期",
            SeasonPhase::Draft => "选秀",
            SeasonPhase::SeasonEnd => "赛季结束",
        }
    }

    /// 获取下一个阶段
    pub fn next(&self) -> Option<SeasonPhase> {
        match self {
            SeasonPhase::SpringRegular => Some(SeasonPhase::SpringPlayoffs),
            SeasonPhase::SpringPlayoffs => Some(SeasonPhase::Msi),
            SeasonPhase::Msi => Some(SeasonPhase::MadridMasters),
            SeasonPhase::MadridMasters => Some(SeasonPhase::SummerRegular),
            SeasonPhase::SummerRegular => Some(SeasonPhase::SummerPlayoffs),
            SeasonPhase::SummerPlayoffs => Some(SeasonPhase::ClaudeIntercontinental),
            SeasonPhase::ClaudeIntercontinental => Some(SeasonPhase::WorldChampionship),
            SeasonPhase::WorldChampionship => Some(SeasonPhase::ShanghaiMasters),
            SeasonPhase::ShanghaiMasters => Some(SeasonPhase::IcpIntercontinental),
            SeasonPhase::IcpIntercontinental => Some(SeasonPhase::SuperIntercontinental),
            SeasonPhase::SuperIntercontinental => Some(SeasonPhase::TransferWindow),
            SeasonPhase::TransferWindow => Some(SeasonPhase::Draft),
            SeasonPhase::Draft => Some(SeasonPhase::SeasonEnd),
            SeasonPhase::SeasonEnd => None, // 需要新赛季
        }
    }

    /// 是否是选秀年份
    pub fn is_draft_year(season: u32) -> bool {
        season >= 2 && (season - 2) % 4 == 0
    }
}

/// 赛季数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    pub id: u64,
    pub save_id: String,
    pub season_number: u32,
    pub current_phase: SeasonPhase,
    pub phase_completed: bool,
}

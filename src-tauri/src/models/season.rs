use super::TournamentType;
use serde::{Deserialize, Serialize};

/// 赛季阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// 年度颁奖典礼
    AnnualAwards,
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
            SeasonPhase::AnnualAwards => "年度颁奖典礼",
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
            SeasonPhase::SuperIntercontinental => Some(SeasonPhase::AnnualAwards),
            SeasonPhase::AnnualAwards => Some(SeasonPhase::TransferWindow),
            SeasonPhase::TransferWindow => Some(SeasonPhase::Draft),
            SeasonPhase::Draft => Some(SeasonPhase::SeasonEnd),
            SeasonPhase::SeasonEnd => None, // 需要新赛季
        }
    }

    /// 是否是选秀年份（每年都有选秀）
    pub fn is_draft_year(_season: u32) -> bool {
        true
    }

    /// 转换为赛事类型枚举
    pub fn to_tournament_type(&self) -> Option<TournamentType> {
        match self {
            SeasonPhase::SpringRegular => Some(TournamentType::SpringRegular),
            SeasonPhase::SpringPlayoffs => Some(TournamentType::SpringPlayoffs),
            SeasonPhase::Msi => Some(TournamentType::Msi),
            SeasonPhase::MadridMasters => Some(TournamentType::MadridMasters),
            SeasonPhase::SummerRegular => Some(TournamentType::SummerRegular),
            SeasonPhase::SummerPlayoffs => Some(TournamentType::SummerPlayoffs),
            SeasonPhase::ClaudeIntercontinental => Some(TournamentType::ClaudeIntercontinental),
            SeasonPhase::WorldChampionship => Some(TournamentType::WorldChampionship),
            SeasonPhase::ShanghaiMasters => Some(TournamentType::ShanghaiMasters),
            SeasonPhase::IcpIntercontinental => Some(TournamentType::IcpIntercontinental),
            SeasonPhase::SuperIntercontinental => Some(TournamentType::SuperIntercontinental),
            _ => None,
        }
    }

    /// 赛事类型字符串（用于 DB 查询）
    pub fn tournament_type_str(&self) -> Option<&'static str> {
        match self {
            SeasonPhase::SpringRegular => Some("SpringRegular"),
            SeasonPhase::SpringPlayoffs => Some("SpringPlayoffs"),
            SeasonPhase::Msi => Some("Msi"),
            SeasonPhase::MadridMasters => Some("MadridMasters"),
            SeasonPhase::SummerRegular => Some("SummerRegular"),
            SeasonPhase::SummerPlayoffs => Some("SummerPlayoffs"),
            SeasonPhase::ClaudeIntercontinental => Some("ClaudeIntercontinental"),
            SeasonPhase::WorldChampionship => Some("WorldChampionship"),
            SeasonPhase::ShanghaiMasters => Some("ShanghaiMasters"),
            SeasonPhase::IcpIntercontinental => Some("IcpIntercontinental"),
            SeasonPhase::SuperIntercontinental => Some("SuperIntercontinental"),
            _ => None,
        }
    }

    /// 是否季后赛
    pub fn is_playoff(&self) -> bool {
        matches!(self, SeasonPhase::SpringPlayoffs | SeasonPhase::SummerPlayoffs)
    }

    /// 是否国际赛
    pub fn is_international(&self) -> bool {
        matches!(
            self,
            SeasonPhase::Msi
                | SeasonPhase::MadridMasters
                | SeasonPhase::ClaudeIntercontinental
                | SeasonPhase::WorldChampionship
                | SeasonPhase::ShanghaiMasters
                | SeasonPhase::IcpIntercontinental
                | SeasonPhase::SuperIntercontinental
        )
    }

    /// 是否无赛事阶段
    pub fn is_non_tournament(&self) -> bool {
        matches!(
            self,
            SeasonPhase::AnnualAwards
                | SeasonPhase::TransferWindow
                | SeasonPhase::Draft
                | SeasonPhase::SeasonEnd
        )
    }

    /// 显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            SeasonPhase::SpringRegular => "春季常规赛",
            SeasonPhase::SpringPlayoffs => "春季季后赛",
            SeasonPhase::Msi => "MSI季中赛",
            SeasonPhase::MadridMasters => "马德里大师赛",
            SeasonPhase::SummerRegular => "夏季常规赛",
            SeasonPhase::SummerPlayoffs => "夏季季后赛",
            SeasonPhase::ClaudeIntercontinental => "Claude洲际赛",
            SeasonPhase::WorldChampionship => "世界赛",
            SeasonPhase::ShanghaiMasters => "上海大师赛",
            SeasonPhase::IcpIntercontinental => "ICP洲际对抗赛",
            SeasonPhase::SuperIntercontinental => "Super洲际邀请赛",
            SeasonPhase::AnnualAwards => "年度颁奖典礼",
            SeasonPhase::TransferWindow => "转会期",
            SeasonPhase::Draft => "选秀大会",
            SeasonPhase::SeasonEnd => "赛季总结",
        }
    }

    /// 阶段顺序号
    pub fn order(&self) -> u8 {
        match self {
            SeasonPhase::SpringRegular => 0,
            SeasonPhase::SpringPlayoffs => 1,
            SeasonPhase::Msi => 2,
            SeasonPhase::MadridMasters => 3,
            SeasonPhase::SummerRegular => 4,
            SeasonPhase::SummerPlayoffs => 5,
            SeasonPhase::ClaudeIntercontinental => 6,
            SeasonPhase::WorldChampionship => 7,
            SeasonPhase::ShanghaiMasters => 8,
            SeasonPhase::IcpIntercontinental => 9,
            SeasonPhase::SuperIntercontinental => 10,
            SeasonPhase::AnnualAwards => 11,
            SeasonPhase::TransferWindow => 12,
            SeasonPhase::Draft => 13,
            SeasonPhase::SeasonEnd => 14,
        }
    }

    /// 是否在目标阶段之前
    pub fn is_before(&self, target: SeasonPhase) -> bool {
        self.order() < target.order()
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

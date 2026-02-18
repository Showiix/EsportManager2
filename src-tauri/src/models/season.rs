use super::TournamentType;
use serde::{Deserialize, Serialize};

/// 赛季阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeasonPhase {
    SpringRegular,
    SpringPlayoffs,
    Msi,
    MadridMasters,
    DouyuLadder,
    SummerRegular,
    SummerPlayoffs,
    ClaudeIntercontinental,
    WorldChampionship,
    DouyinLadder,
    ShanghaiMasters,
    IcpIntercontinental,
    SuperIntercontinental,
    HuyaLadder,
    AnnualAwards,
    TransferWindow,
    Draft,
    SeasonEnd,
}

impl SeasonPhase {
    pub fn name(&self) -> &'static str {
        match self {
            SeasonPhase::SpringRegular => "春季赛常规赛",
            SeasonPhase::SpringPlayoffs => "春季赛季后赛",
            SeasonPhase::Msi => "MSI季中赛",
            SeasonPhase::MadridMasters => "马德里大师赛",
            SeasonPhase::DouyuLadder => "斗鱼巅峰赛",
            SeasonPhase::SummerRegular => "夏季赛常规赛",
            SeasonPhase::SummerPlayoffs => "夏季赛季后赛",
            SeasonPhase::ClaudeIntercontinental => "Claude洲际赛",
            SeasonPhase::WorldChampionship => "S世界赛",
            SeasonPhase::DouyinLadder => "抖音巅峰赛",
            SeasonPhase::ShanghaiMasters => "上海大师赛",
            SeasonPhase::IcpIntercontinental => "ICP四赛区洲际对抗赛",
            SeasonPhase::SuperIntercontinental => "Super洲际年度邀请赛",
            SeasonPhase::HuyaLadder => "虎牙巅峰赛",
            SeasonPhase::AnnualAwards => "年度颁奖典礼",
            SeasonPhase::TransferWindow => "转会期",
            SeasonPhase::Draft => "选秀",
            SeasonPhase::SeasonEnd => "赛季结束",
        }
    }

    pub fn next(&self) -> Option<SeasonPhase> {
        match self {
            SeasonPhase::SpringRegular => Some(SeasonPhase::SpringPlayoffs),
            SeasonPhase::SpringPlayoffs => Some(SeasonPhase::Msi),
            SeasonPhase::Msi => Some(SeasonPhase::MadridMasters),
            SeasonPhase::MadridMasters => Some(SeasonPhase::DouyuLadder),
            SeasonPhase::DouyuLadder => Some(SeasonPhase::SummerRegular),
            SeasonPhase::SummerRegular => Some(SeasonPhase::SummerPlayoffs),
            SeasonPhase::SummerPlayoffs => Some(SeasonPhase::ClaudeIntercontinental),
            SeasonPhase::ClaudeIntercontinental => Some(SeasonPhase::WorldChampionship),
            SeasonPhase::WorldChampionship => Some(SeasonPhase::DouyinLadder),
            SeasonPhase::DouyinLadder => Some(SeasonPhase::ShanghaiMasters),
            SeasonPhase::ShanghaiMasters => Some(SeasonPhase::IcpIntercontinental),
            SeasonPhase::IcpIntercontinental => Some(SeasonPhase::SuperIntercontinental),
            SeasonPhase::SuperIntercontinental => Some(SeasonPhase::HuyaLadder),
            SeasonPhase::HuyaLadder => Some(SeasonPhase::AnnualAwards),
            SeasonPhase::AnnualAwards => Some(SeasonPhase::TransferWindow),
            SeasonPhase::TransferWindow => Some(SeasonPhase::Draft),
            SeasonPhase::Draft => Some(SeasonPhase::SeasonEnd),
            SeasonPhase::SeasonEnd => None,
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
        matches!(
            self,
            SeasonPhase::SpringPlayoffs | SeasonPhase::SummerPlayoffs
        )
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

    pub fn display_name(&self) -> &'static str {
        match self {
            SeasonPhase::SpringRegular => "春季常规赛",
            SeasonPhase::SpringPlayoffs => "春季季后赛",
            SeasonPhase::Msi => "MSI季中赛",
            SeasonPhase::MadridMasters => "马德里大师赛",
            SeasonPhase::DouyuLadder => "斗鱼巅峰赛",
            SeasonPhase::SummerRegular => "夏季常规赛",
            SeasonPhase::SummerPlayoffs => "夏季季后赛",
            SeasonPhase::ClaudeIntercontinental => "Claude洲际赛",
            SeasonPhase::WorldChampionship => "世界赛",
            SeasonPhase::DouyinLadder => "抖音巅峰赛",
            SeasonPhase::ShanghaiMasters => "上海大师赛",
            SeasonPhase::IcpIntercontinental => "ICP洲际对抗赛",
            SeasonPhase::SuperIntercontinental => "Super洲际邀请赛",
            SeasonPhase::HuyaLadder => "虎牙巅峰赛",
            SeasonPhase::AnnualAwards => "年度颁奖典礼",
            SeasonPhase::TransferWindow => "转会期",
            SeasonPhase::Draft => "选秀大会",
            SeasonPhase::SeasonEnd => "赛季总结",
        }
    }

    pub fn order(&self) -> u8 {
        match self {
            SeasonPhase::SpringRegular => 0,
            SeasonPhase::SpringPlayoffs => 1,
            SeasonPhase::Msi => 2,
            SeasonPhase::MadridMasters => 3,
            SeasonPhase::DouyuLadder => 4,
            SeasonPhase::SummerRegular => 5,
            SeasonPhase::SummerPlayoffs => 6,
            SeasonPhase::ClaudeIntercontinental => 7,
            SeasonPhase::WorldChampionship => 8,
            SeasonPhase::DouyinLadder => 9,
            SeasonPhase::ShanghaiMasters => 10,
            SeasonPhase::IcpIntercontinental => 11,
            SeasonPhase::SuperIntercontinental => 12,
            SeasonPhase::HuyaLadder => 13,
            SeasonPhase::AnnualAwards => 14,
            SeasonPhase::TransferWindow => 15,
            SeasonPhase::Draft => 16,
            SeasonPhase::SeasonEnd => 17,
        }
    }

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

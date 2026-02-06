use serde::{Deserialize, Serialize};
use crate::models::FinancialStatus;

/// 财务交易类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    /// 联盟分成
    LeagueShare,
    /// 排名奖金
    RankingBonus,
    /// 季后赛奖金
    PlayoffBonus,
    /// 国际赛奖金
    InternationalBonus,
    /// 周边收入
    Merchandise,
    /// 转会收入
    TransferIn,
    /// 选手薪资
    Salary,
    /// 转会支出
    TransferOut,
    /// 运营成本
    OperatingCost,
    /// 设施投资
    Facility,
    /// 违约金
    Penalty,
    /// 奢侈税
    LuxuryTax,
}

impl TransactionType {
    pub fn name(&self) -> &'static str {
        match self {
            TransactionType::LeagueShare => "联盟分成",
            TransactionType::RankingBonus => "排名奖金",
            TransactionType::PlayoffBonus => "季后赛奖金",
            TransactionType::InternationalBonus => "国际赛奖金",
            TransactionType::Merchandise => "周边收入",
            TransactionType::TransferIn => "转会收入",
            TransactionType::Salary => "选手薪资",
            TransactionType::TransferOut => "转会支出",
            TransactionType::OperatingCost => "运营成本",
            TransactionType::Facility => "设施投资",
            TransactionType::Penalty => "违约金",
            TransactionType::LuxuryTax => "奢侈税",
        }
    }

    pub fn is_income(&self) -> bool {
        matches!(
            self,
            TransactionType::LeagueShare
                | TransactionType::RankingBonus
                | TransactionType::PlayoffBonus
                | TransactionType::InternationalBonus
                | TransactionType::Merchandise
                | TransactionType::TransferIn
        )
    }
}

/// 财务交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialTransaction {
    pub id: u64,
    pub save_id: String,
    pub team_id: u64,
    pub season_id: u64,
    pub transaction_type: TransactionType,
    /// 金额 (正=收入, 负=支出)
    pub amount: i64,
    pub description: Option<String>,
    /// 关联选手ID
    pub related_player_id: Option<u64>,
    /// 关联赛事ID
    pub related_tournament_id: Option<u64>,
}

/// 球队赛季财务汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSeasonFinance {
    pub id: u64,
    pub team_id: u64,
    pub season_id: u64,
    /// 期初余额 (元)
    pub opening_balance: i64,
    /// 期末余额 (元)
    pub closing_balance: i64,
    /// 总收入
    pub total_income: u64,
    /// 总支出
    pub total_expense: u64,
    /// 财务状态
    pub financial_status: FinancialStatus,
    /// 薪资使用额
    pub salary_cap_used: u64,
}

/// 财务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialConfig {
    /// 联盟分成 (元/赛季)
    pub league_share: u64,
    /// 运营成本 (元/赛季)
    pub operating_cost: u64,
    /// 薪资上限 (元/赛季)
    pub salary_cap: u64,
    /// 单人薪资上限 (元/赛季)
    pub individual_salary_cap: u64,
    /// 奢侈税比例
    pub luxury_tax_rate: f64,
}

impl Default for FinancialConfig {
    fn default() -> Self {
        Self {
            league_share: 5_000_000,
            operating_cost: 1_000_000,
            salary_cap: 15_000_000,
            individual_salary_cap: 4_000_000,
            luxury_tax_rate: 1.5,
        }
    }
}

/// 排名奖金配置（单位：元）
pub fn ranking_bonus_config() -> Vec<(u32, u64)> {
    vec![
        (1, 2_000_000),   // 第1名 200万
        (2, 1_500_000),   // 第2名 150万
        (3, 1_200_000),   // 第3名 120万
        (4, 1_000_000),   // 第4名 100万
        (5, 800_000),     // 第5-6名 80万
        (6, 800_000),
        (7, 600_000),     // 第7-8名 60万
        (8, 600_000),
        (9, 400_000),     // 第9-10名 40万
        (10, 400_000),
        (11, 200_000),    // 第11-14名 20万
        (12, 200_000),
        (13, 200_000),
        (14, 200_000),
    ]
}

/// 季后赛奖金配置（单位：元）
pub fn playoff_bonus_config() -> Vec<(&'static str, u64)> {
    vec![
        ("CHAMPION", 3_000_000),    // 冠军 300万
        ("RUNNER_UP", 2_000_000),   // 亚军 200万
        ("THIRD", 1_500_000),       // 季军 150万
        ("FOURTH", 1_000_000),      // 殿军 100万
    ]
}

/// 国际赛奖金配置（单位：元）
pub fn international_bonus_config() -> Vec<(&'static str, &'static str, u64)> {
    vec![
        // MSI
        ("MSI", "CHAMPION", 4_000_000),         // 400万
        ("MSI", "RUNNER_UP", 2_500_000),         // 250万
        ("MSI", "SEMI_FINAL", 1_500_000),        // 150万
        ("MSI", "QUARTER_FINAL", 800_000),       // 80万
        ("MSI", "GROUP_STAGE", 500_000),          // 50万
        // 世界赛
        ("WORLDS", "CHAMPION", 5_000_000),       // 500万
        ("WORLDS", "RUNNER_UP", 3_500_000),       // 350万
        ("WORLDS", "SEMI_FINAL", 2_000_000),     // 200万
        ("WORLDS", "QUARTER_FINAL", 1_200_000),  // 120万
        ("WORLDS", "GROUP_STAGE", 800_000),       // 80万
    ]
}

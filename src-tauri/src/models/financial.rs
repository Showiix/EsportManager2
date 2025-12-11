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
    /// 期初余额 (万元)
    pub opening_balance: i64,
    /// 期末余额 (万元)
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
    /// 联盟分成 (万元/赛季)
    pub league_share: u64,
    /// 运营成本 (万元/赛季)
    pub operating_cost: u64,
    /// 薪资上限 (万元/赛季)
    pub salary_cap: u64,
    /// 单人薪资上限 (万元/赛季)
    pub individual_salary_cap: u64,
    /// 奢侈税比例
    pub luxury_tax_rate: f64,
}

impl Default for FinancialConfig {
    fn default() -> Self {
        Self {
            league_share: 500,
            operating_cost: 100,
            salary_cap: 1500,
            individual_salary_cap: 400,
            luxury_tax_rate: 1.5,
        }
    }
}

/// 排名奖金配置
pub fn ranking_bonus_config() -> Vec<(u32, u64)> {
    vec![
        (1, 200),   // 第1名
        (2, 150),   // 第2名
        (3, 120),   // 第3名
        (4, 100),   // 第4名
        (5, 80),    // 第5-6名
        (6, 80),
        (7, 60),    // 第7-8名
        (8, 60),
        (9, 40),    // 第9-10名
        (10, 40),
        (11, 20),   // 第11-14名
        (12, 20),
        (13, 20),
        (14, 20),
    ]
}

/// 季后赛奖金配置
pub fn playoff_bonus_config() -> Vec<(&'static str, u64)> {
    vec![
        ("CHAMPION", 300),
        ("RUNNER_UP", 200),
        ("THIRD", 150),
        ("FOURTH", 100),
    ]
}

/// 国际赛奖金配置
pub fn international_bonus_config() -> Vec<(&'static str, &'static str, u64)> {
    vec![
        // MSI
        ("MSI", "CHAMPION", 400),
        ("MSI", "RUNNER_UP", 250),
        ("MSI", "SEMI_FINAL", 150),
        ("MSI", "QUARTER_FINAL", 80),
        ("MSI", "GROUP_STAGE", 50),
        // 世界赛
        ("WORLDS", "CHAMPION", 500),
        ("WORLDS", "RUNNER_UP", 350),
        ("WORLDS", "SEMI_FINAL", 200),
        ("WORLDS", "QUARTER_FINAL", 120),
        ("WORLDS", "GROUP_STAGE", 80),
    ]
}

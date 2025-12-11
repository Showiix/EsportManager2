use serde::{Deserialize, Serialize};

/// 赛区代码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegionCode {
    LPL,  // 中国赛区
    LCK,  // 韩国赛区
    LEC,  // 欧洲赛区
    LCS,  // 北美赛区
}

impl RegionCode {
    pub fn name(&self) -> &'static str {
        match self {
            RegionCode::LPL => "中国赛区",
            RegionCode::LCK => "韩国赛区",
            RegionCode::LEC => "欧洲赛区",
            RegionCode::LCS => "北美赛区",
        }
    }
}

/// 赛区
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: u64,
    pub code: RegionCode,
    pub name: String,
    pub full_name: String,
}

/// 战队
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: u64,
    pub region_id: u64,
    pub name: String,
    pub short_name: Option<String>,
    pub power_rating: f64,
    pub total_matches: u32,
    pub wins: u32,
    pub win_rate: f64,
    pub annual_points: u32,
    pub cross_year_points: u32,
    pub balance: i64,
}

impl Team {
    /// 计算胜率
    pub fn calculate_win_rate(&self) -> f64 {
        if self.total_matches == 0 {
            0.0
        } else {
            (self.wins as f64 / self.total_matches as f64) * 100.0
        }
    }
}

/// 财务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FinancialStatus {
    /// 富裕 (>1000万)
    Wealthy,
    /// 健康 (500-1000万)
    Healthy,
    /// 紧张 (100-500万)
    Tight,
    /// 赤字 (0-100万)
    Deficit,
    /// 破产 (<0)
    Bankrupt,
}

impl FinancialStatus {
    pub fn from_balance(balance: i64) -> Self {
        match balance {
            b if b > 10_000_000 => FinancialStatus::Wealthy,
            b if b > 5_000_000 => FinancialStatus::Healthy,
            b if b > 1_000_000 => FinancialStatus::Tight,
            b if b >= 0 => FinancialStatus::Deficit,
            _ => FinancialStatus::Bankrupt,
        }
    }
}

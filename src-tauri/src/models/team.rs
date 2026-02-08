use serde::{Deserialize, Serialize};

// RegionCode 已在 player.rs 中定义，此处直接引用
pub use super::player::RegionCode;

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
    pub brand_value: f64,
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

    pub fn can_buy(&self) -> bool {
        matches!(self, FinancialStatus::Wealthy | FinancialStatus::Healthy | FinancialStatus::Tight)
    }

    pub fn must_sell(&self) -> bool {
        matches!(self, FinancialStatus::Deficit | FinancialStatus::Bankrupt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Team ====================

    #[test]
    fn test_calculate_win_rate() {
        let team = Team {
            id: 1, region_id: 1, name: "T".into(), short_name: None,
            power_rating: 70.0, total_matches: 20, wins: 15,
            win_rate: 0.0, annual_points: 0, cross_year_points: 0,
            balance: 0, brand_value: 50.0,
        };
        let rate = team.calculate_win_rate();
        assert!((rate - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_win_rate_zero_matches() {
        let team = Team {
            id: 1, region_id: 1, name: "T".into(), short_name: None,
            power_rating: 70.0, total_matches: 0, wins: 0,
            win_rate: 0.0, annual_points: 0, cross_year_points: 0,
            balance: 0, brand_value: 50.0,
        };
        assert_eq!(team.calculate_win_rate(), 0.0);
    }

    // ==================== FinancialStatus ====================

    #[test]
    fn test_financial_status_from_balance() {
        assert_eq!(FinancialStatus::from_balance(20_000_000), FinancialStatus::Wealthy);
        assert_eq!(FinancialStatus::from_balance(8_000_000), FinancialStatus::Healthy);
        assert_eq!(FinancialStatus::from_balance(3_000_000), FinancialStatus::Tight);
        assert_eq!(FinancialStatus::from_balance(500_000), FinancialStatus::Deficit);
        assert_eq!(FinancialStatus::from_balance(-1_000_000), FinancialStatus::Bankrupt);
    }

    #[test]
    fn test_financial_status_can_buy() {
        assert!(FinancialStatus::Wealthy.can_buy());
        assert!(FinancialStatus::Healthy.can_buy());
        assert!(FinancialStatus::Tight.can_buy());
        assert!(!FinancialStatus::Deficit.can_buy());
        assert!(!FinancialStatus::Bankrupt.can_buy());
    }

    #[test]
    fn test_financial_status_must_sell() {
        assert!(!FinancialStatus::Wealthy.must_sell());
        assert!(!FinancialStatus::Tight.must_sell());
        assert!(FinancialStatus::Deficit.must_sell());
        assert!(FinancialStatus::Bankrupt.must_sell());
    }

    #[test]
    fn test_financial_status_boundary_values() {
        // Exact boundary: 10_000_000 → Healthy (not Wealthy, since > not >=)
        assert_eq!(FinancialStatus::from_balance(10_000_000), FinancialStatus::Healthy);
        assert_eq!(FinancialStatus::from_balance(10_000_001), FinancialStatus::Wealthy);
        // Exact boundary: 0 → Deficit
        assert_eq!(FinancialStatus::from_balance(0), FinancialStatus::Deficit);
        assert_eq!(FinancialStatus::from_balance(-1), FinancialStatus::Bankrupt);
    }
}

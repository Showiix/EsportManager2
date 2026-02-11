use crate::models::{
    FinancialStatus, FinancialTransaction, Team, TeamSeasonFinance, TournamentType, TransactionType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 财务系统引擎 - 管理队伍财务
pub struct FinancialEngine {
    /// 财务配置
    config: FinancialConfig,
}

/// 财务配置
#[derive(Debug, Clone)]
pub struct FinancialConfig {
    /// 基础运营成本 (每赛季)
    pub base_operating_cost: u64,
    /// 比赛奖金配置
    pub prize_pools: HashMap<TournamentType, PrizePool>,
    /// 联赛分成 (每赛季)
    pub league_revenue_share: u64,
    /// 赞助收入系数 (基于队伍评级)
    pub sponsorship_coefficient: f64,
}

/// 奖金池配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrizePool {
    pub total: u64,
    pub distribution: HashMap<String, f64>, // position -> percentage
}

impl Default for FinancialConfig {
    fn default() -> Self {
        let mut prize_pools = HashMap::new();

        // MSI奖金池 (3000万元)
        let mut msi_dist = HashMap::new();
        msi_dist.insert("CHAMPION".to_string(), 0.40); // 1200万
        msi_dist.insert("RUNNER_UP".to_string(), 0.25); // 750万
        msi_dist.insert("THIRD".to_string(), 0.15); // 450万
        msi_dist.insert("FOURTH".to_string(), 0.08); // 240万
        msi_dist.insert("LOSERS_R2".to_string(), 0.06); // 180万
        msi_dist.insert("LOSERS_R1".to_string(), 0.03); // 90万
        prize_pools.insert(
            TournamentType::Msi,
            PrizePool {
                total: 30000000, // 3000万元
                distribution: msi_dist,
            },
        );

        // 世界赛奖金池 (6000万元)
        let mut worlds_dist = HashMap::new();
        worlds_dist.insert("CHAMPION".to_string(), 0.35); // 2100万
        worlds_dist.insert("RUNNER_UP".to_string(), 0.20); // 1200万
        worlds_dist.insert("THIRD".to_string(), 0.10); // 600万
        worlds_dist.insert("FOURTH".to_string(), 0.10); // 600万
        worlds_dist.insert("QUARTER_FINAL".to_string(), 0.05); // 300万
        worlds_dist.insert("GROUP_STAGE".to_string(), 0.025); // 150万
        worlds_dist.insert("PLAY_IN".to_string(), 0.0125); // 75万
        prize_pools.insert(
            TournamentType::WorldChampionship,
            PrizePool {
                total: 60000000, // 6000万元
                distribution: worlds_dist,
            },
        );

        // 马德里大师赛奖金池 (1500万元)
        let mut madrid_dist = HashMap::new();
        madrid_dist.insert("CHAMPION".to_string(), 0.40); // 600万
        madrid_dist.insert("RUNNER_UP".to_string(), 0.20); // 300万
        madrid_dist.insert("THIRD".to_string(), 0.12); // 180万
        madrid_dist.insert("FOURTH".to_string(), 0.10); // 150万
        madrid_dist.insert("SEMI_LOSER".to_string(), 0.08); // 120万
        madrid_dist.insert("R1_LOSER".to_string(), 0.05); // 75万
        prize_pools.insert(
            TournamentType::MadridMasters,
            PrizePool {
                total: 15000000, // 1500万元
                distribution: madrid_dist,
            },
        );

        // Claude洲际赛奖金池 (1500万元)
        let mut claude_dist = HashMap::new();
        claude_dist.insert("CHAMPION".to_string(), 0.40); // 600万
        claude_dist.insert("RUNNER_UP".to_string(), 0.20); // 300万
        claude_dist.insert("THIRD".to_string(), 0.12); // 180万
        claude_dist.insert("FOURTH".to_string(), 0.10); // 150万
        claude_dist.insert("SEMI_LOSER".to_string(), 0.08); // 120万
        claude_dist.insert("R1_LOSER".to_string(), 0.05); // 75万
        prize_pools.insert(
            TournamentType::ClaudeIntercontinental,
            PrizePool {
                total: 15000000, // 1500万元
                distribution: claude_dist,
            },
        );

        // 上海大师赛奖金池 (2000万元)
        let mut shanghai_dist = HashMap::new();
        shanghai_dist.insert("CHAMPION".to_string(), 0.35); // 700万
        shanghai_dist.insert("RUNNER_UP".to_string(), 0.20); // 400万
        shanghai_dist.insert("THIRD".to_string(), 0.12); // 240万
        shanghai_dist.insert("FOURTH".to_string(), 0.10); // 200万
        shanghai_dist.insert("LOSERS_R2".to_string(), 0.08); // 160万
        shanghai_dist.insert("LOSERS_R1".to_string(), 0.05); // 100万
        prize_pools.insert(
            TournamentType::ShanghaiMasters,
            PrizePool {
                total: 20000000, // 2000万元
                distribution: shanghai_dist,
            },
        );

        // Super洲际赛奖金池 (8000万元 - 年度最高奖金)
        let mut super_dist = HashMap::new();
        super_dist.insert("CHAMPION".to_string(), 0.35); // 2800万
        super_dist.insert("RUNNER_UP".to_string(), 0.20); // 1600万
        super_dist.insert("THIRD".to_string(), 0.12); // 960万
        super_dist.insert("FOURTH".to_string(), 0.10); // 800万
        super_dist.insert("QUARTER_FINAL".to_string(), 0.05); // 400万
        super_dist.insert("ROUND_OF_16".to_string(), 0.025); // 200万
        prize_pools.insert(
            TournamentType::SuperIntercontinental,
            PrizePool {
                total: 80000000, // 8000万元
                distribution: super_dist,
            },
        );

        // ICP洲际对抗赛奖金池 (2000万元)
        let mut icp_dist = HashMap::new();
        icp_dist.insert("CHAMPION".to_string(), 0.35); // 700万
        icp_dist.insert("RUNNER_UP".to_string(), 0.20); // 400万
        icp_dist.insert("THIRD".to_string(), 0.12); // 240万
        icp_dist.insert("FOURTH".to_string(), 0.10); // 200万
        icp_dist.insert("QUARTER_FINAL".to_string(), 0.05); // 100万
        icp_dist.insert("GROUP_STAGE".to_string(), 0.03); // 60万
        prize_pools.insert(
            TournamentType::IcpIntercontinental,
            PrizePool {
                total: 20000000, // 2000万元
                distribution: icp_dist,
            },
        );

        // 春季季后赛奖金池 (1000万元)
        let mut spring_dist = HashMap::new();
        spring_dist.insert("CHAMPION".to_string(), 0.30);
        spring_dist.insert("RUNNER_UP".to_string(), 0.20);
        spring_dist.insert("THIRD".to_string(), 0.15);
        spring_dist.insert("FOURTH".to_string(), 0.12);
        spring_dist.insert("5TH_8TH".to_string(), 0.04);
        prize_pools.insert(
            TournamentType::SpringPlayoffs,
            PrizePool {
                total: 10000000, // 1000万元
                distribution: spring_dist,
            },
        );

        // 夏季季后赛奖金池 (1000万元)
        let mut summer_dist = HashMap::new();
        summer_dist.insert("CHAMPION".to_string(), 0.30);
        summer_dist.insert("RUNNER_UP".to_string(), 0.20);
        summer_dist.insert("THIRD".to_string(), 0.15);
        summer_dist.insert("FOURTH".to_string(), 0.12);
        summer_dist.insert("5TH_8TH".to_string(), 0.04);
        prize_pools.insert(
            TournamentType::SummerPlayoffs,
            PrizePool {
                total: 10000000, // 1000万元
                distribution: summer_dist,
            },
        );

        Self {
            base_operating_cost: 3000000, // 每赛季300万基础运营成本（单位：元）
            prize_pools,
            league_revenue_share: 1500000, // 每赛季150万联赛分成（单位：元）
            sponsorship_coefficient: 2.0,
        }
    }
}

impl Default for FinancialEngine {
    fn default() -> Self {
        Self {
            config: FinancialConfig::default(),
        }
    }
}

impl FinancialEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: FinancialConfig) -> Self {
        Self { config }
    }

    /// 计算赛事奖金
    pub fn calculate_prize_money(&self, tournament_type: TournamentType, position: &str) -> u64 {
        if let Some(pool) = self.config.prize_pools.get(&tournament_type) {
            if let Some(&percentage) = pool.distribution.get(position) {
                return (pool.total as f64 * percentage) as u64;
            }
        }
        0
    }

    /// 计算赛季总薪资支出
    pub fn calculate_salary_expense(players: &[(u64, u64)]) -> u64 {
        players.iter().map(|(_, salary)| salary).sum()
    }

    /// 计算赞助收入（品牌价值驱动）
    pub fn calculate_sponsorship(&self, team: &Team) -> u64 {
        // 品牌基础：brand_value 0-100 映射到 50万-500万
        let brand_factor = (team.brand_value / 100.0).clamp(0.1, 1.0);
        let base = 500_000.0 + brand_factor * 4_500_000.0; // 50万~500万

        // 战力系数
        let power_factor = match team.power_rating as u32 {
            70..=100 => 1.4,
            65..=69 => 1.2,
            60..=64 => 1.0,
            55..=59 => 0.85,
            50..=54 => 0.7,
            _ => 0.5,
        };

        // 胜率加成
        let win_bonus = if team.win_rate > 0.7 {
            1.3
        } else if team.win_rate > 0.5 {
            1.1
        } else {
            1.0
        };

        (base * power_factor * win_bonus) as u64
    }

    /// 计算联赛分成（赛区+排名差异化）
    pub fn calculate_league_share(&self, region_code: &str, rank: Option<u32>) -> u64 {
        let base = match region_code {
            "CN" => 2_000_000, // LPL 200万
            "KR" => 1_750_000, // LCK 175万
            "EU" => 1_500_000, // LEC 150万
            "NA" => 1_250_000, // LCS 125万
            _ => 1_500_000,
        };
        let rank_bonus = match rank {
            Some(1..=3) => 500_000,  // 前三 +50万
            Some(4..=7) => 250_000,  // 中上 +25万
            Some(8..=10) => 100_000, // 中游 +10万
            _ => 0,                  // 下游或无排名
        };
        (base + rank_bonus) as u64
    }

    /// 计算运营成本（阵容挂钩）
    pub fn calculate_operating_cost(&self, total_salary: u64) -> u64 {
        let base = 1_500_000_u64; // 150万基础
        let salary_overhead = (total_salary as f64 * 0.15) as u64; // 薪资的15%作为附加运营成本
        base + salary_overhead
    }

    /// 计算弱队补贴金
    pub fn calculate_weak_team_subsidy(
        &self,
        team_balance: i64,
        league_avg_balance: i64,
        rank: Option<u32>,
        total_teams: u32,
    ) -> u64 {
        let mut subsidy: u64 = 0;

        // 1. 余额差距补贴：低于联盟平均余额时触发
        let gap = league_avg_balance - team_balance;
        if gap > 2_000_000 {
            // 差距超过200万才触发
            let balance_subsidy = (gap as f64 * 0.30) as u64; // 差距的30%
            subsidy += balance_subsidy.min(15_000_000); // 上限1500万
        }

        // 2. 排名补贴：赛区倒数4名额外补贴
        if let Some(r) = rank {
            let bottom_threshold = total_teams.saturating_sub(3); // 倒数4名
            if r >= bottom_threshold {
                subsidy += if r >= total_teams {
                    3_000_000
                }
                // 末位 +300万
                else if r >= total_teams - 1 {
                    2_500_000
                }
                // 倒数第2 +250万
                else {
                    2_000_000
                }; // 倒数第3-4 +200万

                // 3. 选秀奖金：倒数4名额外500万用于选秀竞拍
                subsidy += 5_000_000; // +500万选秀奖金
            }
        }

        subsidy
    }

    /// 确定财务状态
    fn determine_financial_status(balance: i64) -> FinancialStatus {
        FinancialStatus::from_balance(balance)
    }

    /// 生成赛季财务报告
    pub fn generate_season_report(
        &self,
        team: &Team,
        season_id: u64,
        salary_expense: u64,
        prize_money: u64,
        transfer_income: i64, // 转会净收入 (可为负)
        region_code: &str,
        rank: Option<u32>,
    ) -> TeamSeasonFinance {
        let sponsorship = self.calculate_sponsorship(team);
        let league_share = self.calculate_league_share(region_code, rank);
        let operating_cost = self.calculate_operating_cost(salary_expense);

        let total_income = sponsorship
            + league_share
            + prize_money
            + if transfer_income > 0 {
                transfer_income as u64
            } else {
                0
            };
        let total_expense = salary_expense
            + operating_cost
            + if transfer_income < 0 {
                (-transfer_income) as u64
            } else {
                0
            };

        let net_change = total_income as i64 - total_expense as i64;
        let closing_balance = team.balance + net_change;

        TeamSeasonFinance {
            id: 0,
            team_id: team.id,
            season_id,
            opening_balance: team.balance,
            closing_balance,
            total_income,
            total_expense,
            financial_status: Self::determine_financial_status(closing_balance),
            salary_cap_used: salary_expense,
        }
    }

    /// 记录财务交易
    pub fn record_transaction(
        &self,
        save_id: &str,
        season_id: u64,
        team_id: u64,
        transaction_type: TransactionType,
        amount: i64,
        description: &str,
        related_player_id: Option<u64>,
        related_tournament_id: Option<u64>,
    ) -> FinancialTransaction {
        FinancialTransaction {
            id: 0,
            save_id: save_id.to_string(),
            team_id,
            season_id,
            transaction_type,
            amount,
            description: Some(description.to_string()),
            related_player_id,
            related_tournament_id,
        }
    }

    /// 检查队伍是否有足够资金
    pub fn can_afford(&self, team: &Team, amount: i64) -> bool {
        team.balance >= amount
    }

    /// 检查队伍是否处于财务危机
    pub fn is_in_financial_crisis(&self, team: &Team) -> bool {
        // 余额低于基础运营成本的50%视为财务危机
        team.balance < (self.config.base_operating_cost as i64 / 2)
    }

    /// 计算建议的转会预算
    pub fn suggest_transfer_budget(&self, team: &Team) -> i64 {
        if self.is_in_financial_crisis(team) {
            0
        } else if team.balance > 10_000_000 {
            // 富裕队伍：余额的75%
            (team.balance as f64 * 0.75) as i64
        } else if team.balance > 5_000_000 {
            // 健康队伍：余额的70%
            (team.balance as f64 * 0.70) as i64
        } else {
            // 紧张队伍：余额的65%
            (team.balance as f64 * 0.65) as i64
        }
    }

    /// 计算最大可承受薪资
    pub fn max_affordable_salary(&self, team: &Team, current_salary_total: u64) -> u64 {
        let projected_income =
            self.calculate_sponsorship(team) + self.calculate_league_share("CN", None);

        // 薪资支出不应超过预计收入的60%
        let max_salary_budget = (projected_income as f64 * 0.6) as u64;

        if max_salary_budget > current_salary_total {
            max_salary_budget - current_salary_total
        } else {
            0
        }
    }

    /// 分配比赛奖金
    pub fn distribute_prize_money(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_type: TournamentType,
        results: &[(u64, String)], // (team_id, position)
    ) -> Vec<FinancialTransaction> {
        // 根据比赛类型选择合适的交易类型
        let transaction_type = if tournament_type.is_regional() {
            TransactionType::PlayoffBonus
        } else {
            TransactionType::InternationalBonus
        };

        results
            .iter()
            .filter_map(|(team_id, position)| {
                let prize = self.calculate_prize_money(tournament_type, position);
                if prize > 0 {
                    Some(self.record_transaction(
                        save_id,
                        season_id,
                        *team_id,
                        transaction_type,
                        prize as i64,
                        &format!("{:?} - {}", tournament_type, position),
                        None,
                        None,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// 财务状态摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialStatusSummary {
    pub team_id: u64,
    pub balance: i64,
    pub is_crisis: bool,
    pub transfer_budget: i64,
    pub max_new_salary: u64,
    pub projected_season_profit: i64,
}

impl FinancialEngine {
    /// 获取队伍财务状态
    pub fn get_financial_status(
        &self,
        team: &Team,
        current_salary_total: u64,
    ) -> FinancialStatusSummary {
        let projected_income =
            self.calculate_sponsorship(team) + self.calculate_league_share("CN", None);
        let projected_expense =
            current_salary_total + self.calculate_operating_cost(current_salary_total);
        let projected_profit = projected_income as i64 - projected_expense as i64;

        FinancialStatusSummary {
            team_id: team.id,
            balance: team.balance,
            is_crisis: self.is_in_financial_crisis(team),
            transfer_budget: self.suggest_transfer_budget(team),
            max_new_salary: self.max_affordable_salary(team, current_salary_total),
            projected_season_profit: projected_profit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_team(id: u64, balance: i64, power: f64, win_rate: f64) -> Team {
        Team {
            id,
            region_id: 1,
            name: format!("Team {}", id),
            short_name: Some(format!("T{}", id)),
            power_rating: power,
            total_matches: 100,
            wins: (100.0 * win_rate) as u32,
            win_rate,
            annual_points: 50,
            cross_year_points: 100,
            balance,
            brand_value: ((power - 50.0) * 2.0).clamp(0.0, 100.0),
        }
    }

    #[test]
    fn test_prize_calculation() {
        let engine = FinancialEngine::new();

        let msi_champion = engine.calculate_prize_money(TournamentType::Msi, "CHAMPION");
        assert_eq!(msi_champion, 12_000_000); // 3000万 * 0.40 = 1200万元

        let worlds_champion =
            engine.calculate_prize_money(TournamentType::WorldChampionship, "CHAMPION");
        assert_eq!(worlds_champion, 21_000_000); // 6000万 * 0.35 = 2100万元

        let super_champion =
            engine.calculate_prize_money(TournamentType::SuperIntercontinental, "CHAMPION");
        assert_eq!(super_champion, 28_000_000); // 8000万 * 0.35 = 2800万元
    }

    #[test]
    fn test_sponsorship() {
        let engine = FinancialEngine::new();

        let strong_team = create_test_team(1, 10_000_000, 92.0, 0.75);
        let weak_team = create_test_team(2, 5_000_000, 68.0, 0.40);

        let strong_sponsor = engine.calculate_sponsorship(&strong_team);
        let weak_sponsor = engine.calculate_sponsorship(&weak_team);

        assert!(strong_sponsor > weak_sponsor);
    }

    #[test]
    fn test_financial_crisis() {
        let engine = FinancialEngine::new();

        let rich_team = create_test_team(1, 10_000_000, 75.0, 0.5);
        let poor_team = create_test_team(2, 1_000_000, 75.0, 0.5);

        assert!(!engine.is_in_financial_crisis(&rich_team));
        assert!(engine.is_in_financial_crisis(&poor_team));
    }

    #[test]
    fn test_transfer_budget() {
        let engine = FinancialEngine::new();

        let team = create_test_team(1, 10_000_000, 75.0, 0.5);
        let budget = engine.suggest_transfer_budget(&team);

        assert_eq!(budget, 7_000_000); // 10_000_000 * 0.70
    }
}

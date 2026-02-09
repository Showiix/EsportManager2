use crate::models::{AnnualPointsDetail, GlobalRanking, TournamentType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 年度积分引擎 - 计算和管理各队伍的年度积分
pub struct PointsCalculationEngine {
    /// 积分配置
    config: PointsConfigMap,
}

type PointsConfigMap = HashMap<(TournamentType, String), u32>;

impl Default for PointsCalculationEngine {
    fn default() -> Self {
        let mut config = HashMap::new();

        // 联赛季后赛积分
        config.insert((TournamentType::SpringPlayoffs, "CHAMPION".into()), 12);
        config.insert((TournamentType::SpringPlayoffs, "RUNNER_UP".into()), 10);
        config.insert((TournamentType::SpringPlayoffs, "THIRD".into()), 8);
        config.insert((TournamentType::SpringPlayoffs, "FOURTH".into()), 6);
        config.insert((TournamentType::SpringPlayoffs, "5TH_8TH".into()), 3);

        config.insert((TournamentType::SummerPlayoffs, "CHAMPION".into()), 12);
        config.insert((TournamentType::SummerPlayoffs, "RUNNER_UP".into()), 10);
        config.insert((TournamentType::SummerPlayoffs, "THIRD".into()), 8);
        config.insert((TournamentType::SummerPlayoffs, "FOURTH".into()), 6);
        config.insert((TournamentType::SummerPlayoffs, "5TH_8TH".into()), 3);

        // MSI积分
        config.insert((TournamentType::Msi, "CHAMPION".into()), 20);
        config.insert((TournamentType::Msi, "RUNNER_UP".into()), 16);
        config.insert((TournamentType::Msi, "THIRD".into()), 12);
        config.insert((TournamentType::Msi, "FOURTH".into()), 8);
        config.insert((TournamentType::Msi, "LOSERS_R2".into()), 6);
        config.insert((TournamentType::Msi, "LOSERS_R1".into()), 4);

        // 马德里大师赛积分
        config.insert((TournamentType::MadridMasters, "CHAMPION".into()), 20);
        config.insert((TournamentType::MadridMasters, "RUNNER_UP".into()), 16);
        config.insert((TournamentType::MadridMasters, "THIRD".into()), 12);
        config.insert((TournamentType::MadridMasters, "FOURTH".into()), 8);
        config.insert((TournamentType::MadridMasters, "SEMI_LOSER".into()), 6);
        config.insert((TournamentType::MadridMasters, "QUARTER_LOSER".into()), 4);
        config.insert((TournamentType::MadridMasters, "R1_LOSER".into()), 2);

        // Claude洲际赛积分
        config.insert((TournamentType::ClaudeIntercontinental, "CHAMPION".into()), 20);
        config.insert((TournamentType::ClaudeIntercontinental, "RUNNER_UP".into()), 16);
        config.insert((TournamentType::ClaudeIntercontinental, "THIRD".into()), 12);
        config.insert((TournamentType::ClaudeIntercontinental, "FOURTH".into()), 8);
        config.insert((TournamentType::ClaudeIntercontinental, "SEMI_LOSER".into()), 6);
        config.insert((TournamentType::ClaudeIntercontinental, "QUARTER_LOSER".into()), 4);
        config.insert((TournamentType::ClaudeIntercontinental, "R1_LOSER".into()), 2);

        // 世界赛积分
        config.insert((TournamentType::WorldChampionship, "CHAMPION".into()), 20);
        config.insert((TournamentType::WorldChampionship, "RUNNER_UP".into()), 16);
        config.insert((TournamentType::WorldChampionship, "THIRD".into()), 12);
        config.insert((TournamentType::WorldChampionship, "FOURTH".into()), 8);
        config.insert((TournamentType::WorldChampionship, "QUARTER_FINAL".into()), 6);
        config.insert((TournamentType::WorldChampionship, "GROUP_STAGE".into()), 4);

        // 上海大师赛积分
        config.insert((TournamentType::ShanghaiMasters, "CHAMPION".into()), 20);
        config.insert((TournamentType::ShanghaiMasters, "RUNNER_UP".into()), 16);
        config.insert((TournamentType::ShanghaiMasters, "THIRD".into()), 12);
        config.insert((TournamentType::ShanghaiMasters, "FOURTH".into()), 8);
        config.insert((TournamentType::ShanghaiMasters, "LOSERS_R2".into()), 6);
        config.insert((TournamentType::ShanghaiMasters, "LOSERS_R1".into()), 4);

        // ICP四赛区洲际对抗赛积分
        // 最强赛区参赛队伍12分，未参赛6分
        // 第二赛区参赛8分，未参赛4分
        // 第三赛区参赛6分，未参赛3分
        // 第四赛区参赛4分，未参赛2分
        config.insert((TournamentType::IcpIntercontinental, "FIRST_PARTICIPANT".into()), 12);
        config.insert((TournamentType::IcpIntercontinental, "FIRST_NON_PARTICIPANT".into()), 6);
        config.insert((TournamentType::IcpIntercontinental, "SECOND_PARTICIPANT".into()), 8);
        config.insert((TournamentType::IcpIntercontinental, "SECOND_NON_PARTICIPANT".into()), 4);
        config.insert((TournamentType::IcpIntercontinental, "THIRD_PARTICIPANT".into()), 6);
        config.insert((TournamentType::IcpIntercontinental, "THIRD_NON_PARTICIPANT".into()), 3);
        config.insert((TournamentType::IcpIntercontinental, "FOURTH_PARTICIPANT".into()), 4);
        config.insert((TournamentType::IcpIntercontinental, "FOURTH_NON_PARTICIPANT".into()), 2);

        // Super洲际年度邀请赛 - 不获得年度积分！
        // Super赛是年度积分的终点，参加Super赛就是年度积分的奖励
        // 因此Super赛不颁发任何积分

        Self { config }
    }
}

impl PointsCalculationEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取指定赛事和名次的积分
    pub fn get_points(&self, tournament_type: TournamentType, position: &str) -> u32 {
        self.config
            .get(&(tournament_type, position.to_string()))
            .copied()
            .unwrap_or(0)
    }

    /// 计算单场赛事积分
    pub fn calculate_event_points(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_type: TournamentType,
        results: &[(u64, String)], // (team_id, position)
    ) -> Vec<AnnualPointsDetail> {
        results
            .iter()
            .map(|(team_id, position)| {
                let points = self.get_points(tournament_type, position);
                AnnualPointsDetail {
                    id: 0, // 由数据库生成
                    save_id: save_id.to_string(),
                    season_id,
                    team_id: *team_id,
                    tournament_id,
                    tournament_name: None, // 由仓库层填充
                    tournament_type: None, // 由仓库层填充
                    points,
                    final_rank: self.position_to_rank(position),
                }
            })
            .collect()
    }

    /// 位置转排名数字
    fn position_to_rank(&self, position: &str) -> Option<u32> {
        match position {
            "CHAMPION" => Some(1),
            "RUNNER_UP" => Some(2),
            "THIRD" => Some(3),
            "FOURTH" => Some(4),
            "QUARTER_FINAL" | "5TH_8TH" => Some(5),
            // ICP积分位置
            "FIRST_PARTICIPANT" | "FIRST_NON_PARTICIPANT" => Some(1),
            "SECOND_PARTICIPANT" | "SECOND_NON_PARTICIPANT" => Some(2),
            "THIRD_PARTICIPANT" | "THIRD_NON_PARTICIPANT" => Some(3),
            "FOURTH_PARTICIPANT" | "FOURTH_NON_PARTICIPANT" => Some(4),
            _ => None,
        }
    }

    /// 更新年度积分排名
    pub fn calculate_annual_rankings(
        &self,
        save_id: &str,
        season_id: u64,
        team_points: &HashMap<u64, u32>, // team_id -> total_points
    ) -> Vec<GlobalRanking> {
        let mut rankings: Vec<_> = team_points
            .iter()
            .map(|(&team_id, &points)| (team_id, points))
            .collect();

        // 按积分降序排序
        rankings.sort_by(|a, b| b.1.cmp(&a.1));

        rankings
            .into_iter()
            .enumerate()
            .map(|(idx, (team_id, total_points))| GlobalRanking {
                id: 0,
                save_id: save_id.to_string(),
                season_id,
                team_id,
                global_rank: (idx + 1) as u32,
                total_points,
            })
            .collect()
    }

    /// 获取全球Top16排名 (用于洲际超级杯赛资格)
    pub fn get_global_top16<'a>(&self, rankings: &'a [GlobalRanking]) -> Vec<&'a GlobalRanking> {
        rankings.iter().take(16).collect()
    }

    /// 分配洲际超级杯赛参赛资格
    pub fn assign_super_cup_groups(
        &self,
        top16: &[&GlobalRanking],
    ) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
        // 传奇组：1-4名
        let legendary: Vec<u64> = top16.iter().take(4).map(|r| r.team_id).collect();

        // 挑战者组：5-8名
        let challenger: Vec<u64> = top16.iter().skip(4).take(4).map(|r| r.team_id).collect();

        // Fighter组：9-16名
        let fighter: Vec<u64> = top16.iter().skip(8).take(8).map(|r| r.team_id).collect();

        (legendary, challenger, fighter)
    }
}

/// 积分更新事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointsUpdateEvent {
    pub team_id: u64,
    pub tournament_type: TournamentType,
    pub tournament_name: String,
    pub position: String,
    pub points_earned: u32,
    pub total_points: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_points() {
        let engine = PointsCalculationEngine::new();

        assert_eq!(engine.get_points(TournamentType::Msi, "CHAMPION"), 20);
        assert_eq!(engine.get_points(TournamentType::SpringPlayoffs, "RUNNER_UP"), 10);
        assert_eq!(engine.get_points(TournamentType::WorldChampionship, "THIRD"), 12);
    }

    #[test]
    fn test_rankings() {
        let engine = PointsCalculationEngine::new();

        let mut team_points = HashMap::new();
        team_points.insert(1, 100);
        team_points.insert(2, 80);
        team_points.insert(3, 120);

        let rankings = engine.calculate_annual_rankings("save1", 1, &team_points);

        assert_eq!(rankings[0].team_id, 3); // 120分
        assert_eq!(rankings[1].team_id, 1); // 100分
        assert_eq!(rankings[2].team_id, 2); // 80分
    }
}

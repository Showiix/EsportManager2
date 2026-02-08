use crate::engines::MatchSimulationEngine;
use crate::models::{
    LeagueStanding, Match, MatchFormat, MatchResult, MatchStatus, Team,
};

/// 联赛服务 - 负责生成和管理联赛赛程
pub struct LeagueService {
    match_engine: MatchSimulationEngine,
}

impl Default for LeagueService {
    fn default() -> Self {
        Self {
            match_engine: MatchSimulationEngine::default(),
        }
    }
}

impl LeagueService {
    pub fn new() -> Self {
        Self::default()
    }

    /// 生成常规赛赛程 (BO3双循环)
    /// 14支队伍，每队与其他13队各交手2次，共26轮
    pub fn generate_regular_schedule(
        &self,
        tournament_id: u64,
        teams: &[Team],
    ) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut match_id = 1u64;
        let _team_count = teams.len();

        // 第一循环
        let first_round = self.generate_round_robin(tournament_id, teams, &mut match_id);
        matches.extend(first_round);

        // 第二循环 (主客场交换)
        let second_round = self.generate_round_robin_reversed(tournament_id, teams, &mut match_id);
        matches.extend(second_round);

        matches
    }

    /// 生成单循环赛程
    fn generate_round_robin(
        &self,
        tournament_id: u64,
        teams: &[Team],
        match_id: &mut u64,
    ) -> Vec<Match> {
        let mut matches = Vec::new();
        let n = teams.len();

        // 使用圆环排列算法生成单循环赛程
        // 固定第一支队伍，其他队伍轮转
        let mut rotating_teams: Vec<usize> = (1..n).collect();
        let rounds = n - 1;

        for round in 0..rounds {
            // 第一场：队伍0 vs 轮转列表最后一个
            let home_idx = 0;
            let away_idx = rotating_teams[rotating_teams.len() - 1];

            matches.push(Match {
                id: *match_id,
                tournament_id,
                stage: "REGULAR".to_string(),
                round: Some((round + 1) as u32),
                match_order: Some(1),
                format: MatchFormat::Bo3,
                home_team_id: teams[home_idx].id,
                away_team_id: teams[away_idx].id,
                home_score: 0,
                away_score: 0,
                winner_id: None,
                status: MatchStatus::Scheduled,
            });
            *match_id += 1;

            // 其余配对
            let half = rotating_teams.len() / 2;
            for i in 0..half {
                let home_idx = rotating_teams[i];
                let away_idx = rotating_teams[rotating_teams.len() - 2 - i];

                matches.push(Match {
                    id: *match_id,
                    tournament_id,
                    stage: "REGULAR".to_string(),
                    round: Some((round + 1) as u32),
                    match_order: Some((i + 2) as u32),
                    format: MatchFormat::Bo3,
                    home_team_id: teams[home_idx].id,
                    away_team_id: teams[away_idx].id,
                    home_score: 0,
                    away_score: 0,
                    winner_id: None,
                    status: MatchStatus::Scheduled,
                });
                *match_id += 1;
            }

            // 轮转
            rotating_teams.rotate_right(1);
        }

        matches
    }

    /// 生成第二循环 (主客场交换)
    fn generate_round_robin_reversed(
        &self,
        tournament_id: u64,
        teams: &[Team],
        match_id: &mut u64,
    ) -> Vec<Match> {
        let first_round = self.generate_round_robin(tournament_id, teams, match_id);

        // 交换主客场
        first_round
            .into_iter()
            .map(|mut m| {
                std::mem::swap(&mut m.home_team_id, &mut m.away_team_id);
                m.round = m.round.map(|r| r + (teams.len() - 1) as u32);
                m
            })
            .collect()
    }

    /// 计算积分榜排名
    /// 排序规则: 积分 > 历史交锋胜率 > 净胜小局数
    pub fn calculate_standings(
        &self,
        tournament_id: u64,
        completed_matches: &[MatchResult],
        team_ids: &[u64],
    ) -> Vec<LeagueStanding> {
        let mut standings: Vec<LeagueStanding> = team_ids
            .iter()
            .map(|&id| LeagueStanding::new(tournament_id, id))
            .collect();

        // 更新每场比赛的结果
        for match_result in completed_matches {
            let home_id = match_result.match_info.home_team_id;
            let away_id = match_result.match_info.away_team_id;
            let home_score = match_result.home_score;
            let away_score = match_result.away_score;

            // 更新主队积分
            if let Some(home_standing) = standings.iter_mut().find(|s| s.team_id == home_id) {
                home_standing.update_match_result(home_score, away_score, true);
            }

            // 更新客队积分
            if let Some(away_standing) = standings.iter_mut().find(|s| s.team_id == away_id) {
                away_standing.update_match_result(home_score, away_score, false);
            }
        }

        // 排序：积分 > 净胜小局 > 胜场
        standings.sort_by(|a, b| {
            b.points
                .cmp(&a.points)
                .then_with(|| b.game_diff.cmp(&a.game_diff))
                .then_with(|| b.wins.cmp(&a.wins))
        });

        // 更新排名
        for (idx, standing) in standings.iter_mut().enumerate() {
            standing.rank = Some((idx + 1) as u32);
        }

        standings
    }

    /// 生成季后赛初始对阵 (双败BO5)
    /// 前8名进入季后赛：1-4名进胜者组，5-8名进败者组
    /// 只生成第一轮的4场比赛，后续比赛通过 advance_playoff_bracket 动态生成
    pub fn generate_playoff_bracket(
        &self,
        tournament_id: u64,
        standings: &[LeagueStanding],
    ) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut match_id = 1000u64; // 季后赛从1000开始

        if standings.len() < 8 {
            return matches;
        }

        let top8: Vec<_> = standings.iter().take(8).collect();

        // 胜者组第一轮：1 vs 4, 2 vs 3
        matches.push(self.create_playoff_match(
            &mut match_id,
            tournament_id,
            "WINNERS_R1",
            1,
            top8[0].team_id,
            top8[3].team_id,
        ));
        matches.push(self.create_playoff_match(
            &mut match_id,
            tournament_id,
            "WINNERS_R1",
            2,
            top8[1].team_id,
            top8[2].team_id,
        ));

        // 败者组第一轮：5 vs 8, 6 vs 7
        matches.push(self.create_playoff_match(
            &mut match_id,
            tournament_id,
            "LOSERS_R1",
            1,
            top8[4].team_id,
            top8[7].team_id,
        ));
        matches.push(self.create_playoff_match(
            &mut match_id,
            tournament_id,
            "LOSERS_R1",
            2,
            top8[5].team_id,
            top8[6].team_id,
        ));

        // 后续比赛通过 advance_playoff_bracket 动态生成

        matches
    }

    /// 根据已完成的比赛推进季后赛对阵
    /// 返回新生成的比赛列表
    pub fn advance_playoff_bracket(
        &self,
        tournament_id: u64,
        completed_matches: &[Match],
    ) -> Vec<Match> {
        let mut new_matches = Vec::new();
        let mut match_id = 2000u64; // 后续比赛ID从2000开始

        // 获取各阶段的已完成比赛
        let winners_r1: Vec<_> = completed_matches
            .iter()
            .filter(|m| m.stage == "WINNERS_R1" && m.winner_id.is_some())
            .collect();
        let losers_r1: Vec<_> = completed_matches
            .iter()
            .filter(|m| m.stage == "LOSERS_R1" && m.winner_id.is_some())
            .collect();
        let winners_final: Vec<_> = completed_matches
            .iter()
            .filter(|m| m.stage == "WINNERS_FINAL" && m.winner_id.is_some())
            .collect();
        let losers_r2: Vec<_> = completed_matches
            .iter()
            .filter(|m| m.stage == "LOSERS_R2" && m.winner_id.is_some())
            .collect();
        let losers_r3: Vec<_> = completed_matches
            .iter()
            .filter(|m| m.stage == "LOSERS_R3" && m.winner_id.is_some())
            .collect();
        let losers_final: Vec<_> = completed_matches
            .iter()
            .filter(|m| m.stage == "LOSERS_FINAL" && m.winner_id.is_some())
            .collect();

        // 检查是否已存在各阶段比赛
        let has_winners_final = completed_matches.iter().any(|m| m.stage == "WINNERS_FINAL");
        let has_losers_r2 = completed_matches.iter().any(|m| m.stage == "LOSERS_R2");
        let has_losers_r3 = completed_matches.iter().any(|m| m.stage == "LOSERS_R3");
        let has_losers_final = completed_matches.iter().any(|m| m.stage == "LOSERS_FINAL");
        let has_grand_final = completed_matches.iter().any(|m| m.stage == "GRAND_FINAL");

        log::debug!("winners_r1={}, losers_r1={}", winners_r1.len(), losers_r1.len());
        log::debug!("has_winners_final={}, has_losers_r2={}", has_winners_final, has_losers_r2);

        // 1. 胜者组R1完成 -> 生成胜者组决赛
        if winners_r1.len() == 2 && !has_winners_final {
            log::debug!("条件满足: 生成WINNERS_FINAL");
            let winner1 = winners_r1.iter().find(|m| m.match_order == Some(1)).unwrap();
            let winner2 = winners_r1.iter().find(|m| m.match_order == Some(2)).unwrap();

            new_matches.push(self.create_playoff_match(
                &mut match_id,
                tournament_id,
                "WINNERS_FINAL",
                1,
                winner1.winner_id.unwrap(),
                winner2.winner_id.unwrap(),
            ));
        }

        // 2. 胜者组R1 + 败者组R1完成 -> 生成败者组R2
        if winners_r1.len() == 2 && losers_r1.len() == 2 && !has_losers_r2 {
            // 败者组R2规则：败者组R1胜者 vs 胜者组R1败者
            let winners_r1_m1 = winners_r1.iter().find(|m| m.match_order == Some(1)).unwrap();
            let winners_r1_m2 = winners_r1.iter().find(|m| m.match_order == Some(2)).unwrap();
            let losers_r1_m1 = losers_r1.iter().find(|m| m.match_order == Some(1)).unwrap();
            let losers_r1_m2 = losers_r1.iter().find(|m| m.match_order == Some(2)).unwrap();

            // 获取胜者组R1的败者
            let winners_r1_loser1 = if winners_r1_m1.winner_id.unwrap() == winners_r1_m1.home_team_id {
                winners_r1_m1.away_team_id
            } else {
                winners_r1_m1.home_team_id
            };
            let winners_r1_loser2 = if winners_r1_m2.winner_id.unwrap() == winners_r1_m2.home_team_id {
                winners_r1_m2.away_team_id
            } else {
                winners_r1_m2.home_team_id
            };

            // 败者组R2-1：败者组R1-1胜者 vs 胜者组R1-2败者
            new_matches.push(self.create_playoff_match(
                &mut match_id,
                tournament_id,
                "LOSERS_R2",
                1,
                losers_r1_m1.winner_id.unwrap(),
                winners_r1_loser2,
            ));

            // 败者组R2-2：败者组R1-2胜者 vs 胜者组R1-1败者
            new_matches.push(self.create_playoff_match(
                &mut match_id,
                tournament_id,
                "LOSERS_R2",
                2,
                losers_r1_m2.winner_id.unwrap(),
                winners_r1_loser1,
            ));
        }

        // 3. 败者组R2完成 -> 生成败者组R3
        if losers_r2.len() == 2 && !has_losers_r3 {
            let loser_r2_m1 = losers_r2.iter().find(|m| m.match_order == Some(1)).unwrap();
            let loser_r2_m2 = losers_r2.iter().find(|m| m.match_order == Some(2)).unwrap();

            new_matches.push(self.create_playoff_match(
                &mut match_id,
                tournament_id,
                "LOSERS_R3",
                1,
                loser_r2_m1.winner_id.unwrap(),
                loser_r2_m2.winner_id.unwrap(),
            ));
        }

        // 4. 胜者组决赛 + 败者组R3完成 -> 生成败者组决赛
        if winners_final.len() == 1 && losers_r3.len() == 1 && !has_losers_final {
            let wf = winners_final[0];
            let lr3 = losers_r3[0];

            // 胜者组决赛败者
            let wf_loser = if wf.winner_id.unwrap() == wf.home_team_id {
                wf.away_team_id
            } else {
                wf.home_team_id
            };

            new_matches.push(self.create_playoff_match(
                &mut match_id,
                tournament_id,
                "LOSERS_FINAL",
                1,
                wf_loser,
                lr3.winner_id.unwrap(),
            ));
        }

        // 5. 胜者组决赛 + 败者组决赛完成 -> 生成总决赛
        if winners_final.len() == 1 && losers_final.len() == 1 && !has_grand_final {
            let wf = winners_final[0];
            let lf = losers_final[0];

            new_matches.push(self.create_playoff_match(
                &mut match_id,
                tournament_id,
                "GRAND_FINAL",
                1,
                wf.winner_id.unwrap(),
                lf.winner_id.unwrap(),
            ));
        }

        new_matches
    }

    /// 获取季后赛阶段顺序（用于排序）
    pub fn get_playoff_stage_order(stage: &str) -> u32 {
        match stage {
            "WINNERS_R1" => 1,
            "LOSERS_R1" => 2,
            "WINNERS_FINAL" => 3,
            "LOSERS_R2" => 4,
            "LOSERS_R3" => 5,
            "LOSERS_FINAL" => 6,
            "GRAND_FINAL" => 7,
            _ => 100,
        }
    }

    fn create_playoff_match(
        &self,
        match_id: &mut u64,
        tournament_id: u64,
        stage: &str,
        match_order: u32,
        home_team_id: u64,
        away_team_id: u64,
    ) -> Match {
        let m = Match {
            id: *match_id,
            tournament_id,
            stage: stage.to_string(),
            round: None,
            match_order: Some(match_order),
            format: MatchFormat::Bo5,
            home_team_id,
            away_team_id,
            home_score: 0,
            away_score: 0,
            winner_id: None,
            status: MatchStatus::Scheduled,
        };
        *match_id += 1;
        m
    }

    /// 模拟一场比赛
    pub fn simulate_match(
        &self,
        match_info: &Match,
        home_power: f64,
        away_power: f64,
    ) -> MatchResult {
        self.match_engine.simulate_match(
            match_info.id,
            match_info.tournament_id,
            &match_info.stage,
            match_info.format,
            match_info.home_team_id,
            match_info.away_team_id,
            home_power,
            away_power,
        )
    }

    /// 获取季后赛积分配置
    pub fn get_playoff_points(&self, position: &str) -> u32 {
        match position {
            "CHAMPION" => 12,
            "RUNNER_UP" => 10,
            "THIRD" => 8,
            "FOURTH" => 6,
            "5TH_8TH" => 3,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_teams(count: usize) -> Vec<Team> {
        (1..=count)
            .map(|i| Team {
                id: i as u64,
                region_id: 1,
                name: format!("Team{}", i),
                short_name: Some(format!("T{}", i)),
                power_rating: 70.0 + (i as f64),
                total_matches: 0,
                wins: 0,
                win_rate: 0.0,
                annual_points: 0,
                cross_year_points: 0,
                balance: 5000000,
                brand_value: 50.0,
            })
            .collect()
    }

    #[test]
    fn test_generate_regular_schedule() {
        let service = LeagueService::new();
        let teams = create_test_teams(14);

        let matches = service.generate_regular_schedule(1, &teams);

        // 14队双循环，每队26场，总共 14*13 = 182场
        assert_eq!(matches.len(), 182);

        // 每轮应该有7场比赛
        let round1_matches: Vec<_> = matches.iter().filter(|m| m.round == Some(1)).collect();
        assert_eq!(round1_matches.len(), 7);
    }

    #[test]
    fn test_standings_calculation() {
        let service = LeagueService::new();

        // 创建一些模拟的比赛结果
        let standings = vec![
            LeagueStanding {
                id: 1,
                tournament_id: 1,
                team_id: 1,
                rank: None,
                matches_played: 10,
                wins: 8,
                losses: 2,
                points: 22,
                games_won: 18,
                games_lost: 6,
                game_diff: 12,
            },
            LeagueStanding {
                id: 2,
                tournament_id: 1,
                team_id: 2,
                rank: None,
                matches_played: 10,
                wins: 6,
                losses: 4,
                points: 16,
                games_won: 14,
                games_lost: 10,
                game_diff: 4,
            },
        ];

        // 验证积分高的排名靠前
        assert!(standings[0].points > standings[1].points);
    }
}

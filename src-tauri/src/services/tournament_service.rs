use crate::engines::MatchSimulationEngine;
use crate::models::{Match, MatchFormat, MatchResult, MatchStatus, Team, TournamentType};
use rand::seq::SliceRandom;

/// 国际赛事服务 - 负责生成和管理各类杯赛
pub struct TournamentService {
    match_engine: MatchSimulationEngine,
}

impl Default for TournamentService {
    fn default() -> Self {
        Self {
            match_engine: MatchSimulationEngine::default(),
        }
    }
}

impl TournamentService {
    pub fn new() -> Self {
        Self::default()
    }

    // ==================== MSI/上海大师赛 (12队双败制) ====================

    /// 生成MSI对阵 (12队双败制)
    /// 参赛队伍：4赛区春季赛冠/亚/季军
    /// - 冠军进传奇组
    /// - 亚军进挑战者组
    /// - 季军进资格赛组
    pub fn generate_msi_bracket(
        &self,
        tournament_id: u64,
        legendary_teams: &[Team],    // 4支冠军
        challenger_teams: &[Team],   // 4支亚军
        qualifier_teams: &[Team],    // 4支季军
    ) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut match_id = 1u64;

        // === 预选赛 ===

        // 资格赛组：4队两两BO5单淘汰
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "QUALIFIER_R1",
            1,
            qualifier_teams[0].id,
            qualifier_teams[3].id,
            MatchFormat::Bo5,
        ));
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "QUALIFIER_R1",
            2,
            qualifier_teams[1].id,
            qualifier_teams[2].id,
            MatchFormat::Bo5,
        ));

        // 挑战者组：4队PK
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "CHALLENGER_R1",
            1,
            challenger_teams[0].id,
            challenger_teams[3].id,
            MatchFormat::Bo5,
        ));
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "CHALLENGER_R1",
            2,
            challenger_teams[1].id,
            challenger_teams[2].id,
            MatchFormat::Bo5,
        ));

        // === 败者组 ===

        // 败者组R1：资格赛胜者 vs 挑战者败者
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "LOSERS_R1",
            1,
            0, 0, // 待定
            MatchFormat::Bo5,
        ));
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "LOSERS_R1",
            2,
            0, 0,
            MatchFormat::Bo5,
        ));

        // 败者组R2：挑战者胜者 vs 败者组R1胜者
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "LOSERS_R2",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "LOSERS_R2",
            2,
            0, 0,
            MatchFormat::Bo5,
        ));

        // === 胜者组 ===

        // 胜者组R1：4传奇组对决
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "WINNERS_R1",
            1,
            legendary_teams[0].id,
            legendary_teams[3].id,
            MatchFormat::Bo5,
        ));
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "WINNERS_R1",
            2,
            legendary_teams[1].id,
            legendary_teams[2].id,
            MatchFormat::Bo5,
        ));

        // 败者组R3：败者组R2胜者 vs 胜者组R1败者
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "LOSERS_R3",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "LOSERS_R3",
            2,
            0, 0,
            MatchFormat::Bo5,
        ));

        // 胜者组决赛
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "WINNERS_FINAL",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        // 败者组R4：败者组R3胜者对决
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "LOSERS_R4",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        // 败者组决赛：胜者组决赛败者 vs 败者组R4胜者
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "LOSERS_FINAL",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        // 总决赛
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "GRAND_FINAL",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        matches
    }

    // ==================== 马德里大师赛/Claude洲际赛 (32队分组+淘汰) ====================

    /// 生成32队赛事对阵 (8组×4队小组赛 + 16队淘汰赛)
    pub fn generate_masters_bracket(
        &self,
        tournament_id: u64,
        teams: &[Team], // 32队
    ) -> Vec<Match> {
        if teams.len() != 32 {
            return Vec::new();
        }

        let mut matches = Vec::new();
        let mut match_id = 1u64;

        // 分组：8组×4队
        let groups = self.draw_groups(teams, 8);

        // 小组赛：组内BO3单循环
        for (group_idx, group) in groups.iter().enumerate() {
            let group_name = (b'A' + group_idx as u8) as char;

            // 组内单循环：每队打3场
            for i in 0..4 {
                for j in (i + 1)..4 {
                    matches.push(self.create_match(
                        &mut match_id,
                        tournament_id,
                        &format!("GROUP_{}", group_name),
                        ((i * 3 + j) + 1) as u32,
                        group[i].id,
                        group[j].id,
                        MatchFormat::Bo3,
                    ));
                }
            }
        }

        // 淘汰赛：东西半区各8队
        // 东半区淘汰赛第一轮 (4场)
        for i in 0..4 {
            matches.push(self.create_match(
                &mut match_id,
                tournament_id,
                "EAST_R1",
                (i + 1) as u32,
                0, 0, // 待定：小组赛结果后填充
                MatchFormat::Bo5,
            ));
        }

        // 西半区淘汰赛第一轮 (4场)
        for i in 0..4 {
            matches.push(self.create_match(
                &mut match_id,
                tournament_id,
                "WEST_R1",
                (i + 1) as u32,
                0, 0,
                MatchFormat::Bo5,
            ));
        }

        // 东半区半决赛 (2场)
        for i in 0..2 {
            matches.push(self.create_match(
                &mut match_id,
                tournament_id,
                "EAST_SEMI",
                (i + 1) as u32,
                0, 0,
                MatchFormat::Bo5,
            ));
        }

        // 西半区半决赛 (2场)
        for i in 0..2 {
            matches.push(self.create_match(
                &mut match_id,
                tournament_id,
                "WEST_SEMI",
                (i + 1) as u32,
                0, 0,
                MatchFormat::Bo5,
            ));
        }

        // 东半区决赛
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "EAST_FINAL",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        // 西半区决赛
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "WEST_FINAL",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        // 季军赛
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "THIRD_PLACE",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        // 总决赛
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "GRAND_FINAL",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        matches
    }

    // ==================== 世界赛 (瑞士轮+淘汰赛) ====================

    /// 生成世界赛对阵
    /// - 小组赛：8队BO1瑞士轮，决出4队晋级
    /// - 淘汰赛：8队BO5单淘汰 + 季军赛
    pub fn generate_worlds_bracket(
        &self,
        tournament_id: u64,
        direct_teams: &[Team],    // 4支冠军直接进淘汰赛
        group_teams: &[Team],     // 8支亚/季军进小组赛
    ) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut match_id = 1u64;

        // 瑞士轮小组赛：最多5轮
        // 第1轮：4场 (8队)
        for i in 0..4 {
            matches.push(self.create_match(
                &mut match_id,
                tournament_id,
                "SWISS_R1",
                (i + 1) as u32,
                group_teams[i * 2].id,
                group_teams[i * 2 + 1].id,
                MatchFormat::Bo1,
            ));
        }

        // 后续轮次待生成（根据结果动态配对）
        // SWISS_R2, SWISS_R3, SWISS_R4, SWISS_R5

        // 淘汰赛
        // 八强赛 (4场)
        for i in 0..4 {
            matches.push(self.create_match(
                &mut match_id,
                tournament_id,
                "QUARTER_FINAL",
                (i + 1) as u32,
                if i < 4 { direct_teams.get(i).map_or(0, |t| t.id) } else { 0 },
                0, // 小组赛晋级者
                MatchFormat::Bo5,
            ));
        }

        // 半决赛 (2场)
        for i in 0..2 {
            matches.push(self.create_match(
                &mut match_id,
                tournament_id,
                "SEMI_FINAL",
                (i + 1) as u32,
                0, 0,
                MatchFormat::Bo5,
            ));
        }

        // 季军赛
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "THIRD_PLACE",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        // 总决赛
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "GRAND_FINAL",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        matches
    }

    // ==================== Super洲际年度邀请赛 (16队四阶段制) ====================

    /// 生成Super洲际赛对阵
    pub fn generate_super_bracket(
        &self,
        tournament_id: u64,
        legendary_teams: &[Team],    // 前4名 (传奇组)
        challenger_teams: &[Team],   // 5-8名 (挑战者组)
        fighter_teams: &[Team],      // 9-16名 (Fighter组)
    ) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut match_id = 1u64;

        // === 第一阶段：Fighter组预选赛 ===
        // 8队分2组，BO3单循环

        // A组 (4队)
        for i in 0..4 {
            for j in (i + 1)..4 {
                if i < fighter_teams.len() && j < fighter_teams.len() {
                    matches.push(self.create_match(
                        &mut match_id,
                        tournament_id,
                        "FIGHTER_GROUP_A",
                        ((i * 3 + j) + 1) as u32,
                        fighter_teams[i].id,
                        fighter_teams[j].id,
                        MatchFormat::Bo3,
                    ));
                }
            }
        }

        // B组 (4队)
        for i in 0..4 {
            for j in (i + 1)..4 {
                let ai = i + 4;
                let aj = j + 4;
                if ai < fighter_teams.len() && aj < fighter_teams.len() {
                    matches.push(self.create_match(
                        &mut match_id,
                        tournament_id,
                        "FIGHTER_GROUP_B",
                        ((i * 3 + j) + 1) as u32,
                        fighter_teams[ai].id,
                        fighter_teams[aj].id,
                        MatchFormat::Bo3,
                    ));
                }
            }
        }

        // === 第二阶段：挑战者组定位赛与晋级赛 ===

        // 定位赛：5 vs 8, 6 vs 7
        if challenger_teams.len() >= 4 {
            matches.push(self.create_match(
                &mut match_id,
                tournament_id,
                "CHALLENGER_QUALIFIER",
                1,
                challenger_teams[0].id,
                challenger_teams[3].id,
                MatchFormat::Bo5,
            ));
            matches.push(self.create_match(
                &mut match_id,
                tournament_id,
                "CHALLENGER_QUALIFIER",
                2,
                challenger_teams[1].id,
                challenger_teams[2].id,
                MatchFormat::Bo5,
            ));
        }

        // 晋级赛：Fighter胜者 vs 挑战者败者
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "PROMOTION",
            1,
            0, 0, // 待定
            MatchFormat::Bo5,
        ));
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "PROMOTION",
            2,
            0, 0,
            MatchFormat::Bo5,
        ));

        // === 第三阶段：冠军赛预备战 ===

        // 胜者组：定位赛胜者对决
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "PREP_WINNERS",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        // 败者组：晋级赛胜者对决
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "PREP_LOSERS",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        // 败者组决赛
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "PREP_LOSERS_FINAL",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        // === 第四阶段：终极冠军赛 ===

        // 首轮：传奇组4名 vs 第三阶段胜者组胜者
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "FINALS_R1",
            1,
            legendary_teams.get(3).map_or(0, |t| t.id),
            0, // 第三阶段胜者组胜者
            MatchFormat::Bo5,
        ));
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "FINALS_R1",
            2,
            legendary_teams.get(2).map_or(0, |t| t.id),
            0, // 第三阶段败者组决赛胜者
            MatchFormat::Bo5,
        ));

        // 次轮
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "FINALS_R2",
            1,
            legendary_teams.get(0).map_or(0, |t| t.id),
            0, // 首轮胜者1
            MatchFormat::Bo5,
        ));
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "FINALS_R2",
            2,
            legendary_teams.get(1).map_or(0, |t| t.id),
            0, // 首轮胜者2
            MatchFormat::Bo5,
        ));

        // 季军加赛
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "THIRD_PLACE",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        // 总决赛
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "GRAND_FINAL",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        matches
    }

    // ==================== ICP四赛区洲际对抗赛 ====================

    /// 生成ICP对阵
    pub fn generate_icp_bracket(
        &self,
        tournament_id: u64,
        region_teams: Vec<Vec<Team>>, // 4个赛区各4队
    ) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut match_id = 1u64;

        // 种子分组：A组(1号种子)、B组(2号种子)、C组(3号种子)、D组(4号种子)
        for seed in 0..4 {
            let group_name = (b'A' + seed as u8) as char;

            // 收集该种子的所有队伍
            let seed_teams: Vec<_> = region_teams
                .iter()
                .filter_map(|r| r.get(seed))
                .collect();

            // 组内BO3单循环
            for i in 0..seed_teams.len() {
                for j in (i + 1)..seed_teams.len() {
                    matches.push(self.create_match(
                        &mut match_id,
                        tournament_id,
                        &format!("ICP_GROUP_{}", group_name),
                        ((i * 3 + j) + 1) as u32,
                        seed_teams[i].id,
                        seed_teams[j].id,
                        MatchFormat::Bo3,
                    ));
                }
            }
        }

        // 赛区对抗决赛阶段（根据徽章数量确定对阵）
        // 半决赛
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "ICP_SEMI",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        // 决赛
        matches.push(self.create_match(
            &mut match_id,
            tournament_id,
            "ICP_FINAL",
            1,
            0, 0,
            MatchFormat::Bo5,
        ));

        matches
    }

    // ==================== 辅助方法 ====================

    fn create_match(
        &self,
        match_id: &mut u64,
        tournament_id: u64,
        stage: &str,
        match_order: u32,
        home_team_id: u64,
        away_team_id: u64,
        format: MatchFormat,
    ) -> Match {
        let m = Match {
            id: *match_id,
            tournament_id,
            stage: stage.to_string(),
            round: None,
            match_order: Some(match_order),
            format,
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

    /// 抽签分组
    fn draw_groups(&self, teams: &[Team], num_groups: usize) -> Vec<Vec<Team>> {
        let mut teams_copy: Vec<_> = teams.to_vec();
        let mut rng = rand::thread_rng();
        teams_copy.shuffle(&mut rng);

        let teams_per_group = teams.len() / num_groups;
        teams_copy
            .chunks(teams_per_group)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// 模拟比赛
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

    /// 获取赛事积分配置
    pub fn get_tournament_points(&self, tournament_type: TournamentType, position: &str) -> u32 {
        match tournament_type {
            TournamentType::Msi | TournamentType::ShanghaiMasters => {
                match position {
                    "CHAMPION" => 20,
                    "RUNNER_UP" => 16,
                    "THIRD" => 12,
                    "FOURTH" => 8,
                    "LOSERS_R2" => 6,
                    "LOSERS_R1" => 4,
                    _ => 0,
                }
            }
            TournamentType::MadridMasters | TournamentType::ClaudeIntercontinental => {
                match position {
                    "CHAMPION" => 20,
                    "RUNNER_UP" => 16,
                    "THIRD" => 12,
                    "FOURTH" => 8,
                    "EAST_FINAL_LOSER" | "WEST_FINAL_LOSER" => 6,
                    "SEMI_LOSER" => 4,
                    "R1_LOSER" => 2,
                    _ => 0,
                }
            }
            TournamentType::WorldChampionship => {
                match position {
                    "CHAMPION" => 20,
                    "RUNNER_UP" => 16,
                    "THIRD" => 12,
                    "FOURTH" => 8,
                    "QUARTER_FINAL" => 6,
                    "GROUP_STAGE" => 4,
                    _ => 0,
                }
            }
            TournamentType::SuperIntercontinental => {
                match position {
                    "CHAMPION" => 35,
                    "RUNNER_UP" => 30,
                    "THIRD" => 25,
                    "FOURTH" => 20,
                    "PREP_LOSER" => 8,
                    "PROMOTION_LOSER" => 5,
                    "FIGHTER_OUT" => 2,
                    _ => 0,
                }
            }
            _ => 0,
        }
    }
}

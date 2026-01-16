use serde::{Deserialize, Serialize};

/// 联赛积分榜
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeagueStanding {
    pub id: u64,
    pub tournament_id: u64,
    pub team_id: u64,
    /// 排名
    pub rank: Option<u32>,
    /// 已比场次
    pub matches_played: u32,
    /// 胜场 (大比分)
    pub wins: u32,
    /// 负场
    pub losses: u32,
    /// 积分
    pub points: u32,
    /// 小局胜场
    pub games_won: u32,
    /// 小局负场
    pub games_lost: u32,
    /// 净胜小局
    pub game_diff: i32,
}

impl LeagueStanding {
    pub fn new(tournament_id: u64, team_id: u64) -> Self {
        Self {
            id: 0,
            tournament_id,
            team_id,
            rank: None,
            matches_played: 0,
            wins: 0,
            losses: 0,
            points: 0,
            games_won: 0,
            games_lost: 0,
            game_diff: 0,
        }
    }

    /// 更新比赛结果
    /// home_score: 主队小局得分, away_score: 客队小局得分
    /// is_home: 该队是否为主队
    pub fn update_match_result(&mut self, home_score: u8, away_score: u8, is_home: bool) {
        self.matches_played += 1;

        let (own_score, opp_score) = if is_home {
            (home_score, away_score)
        } else {
            (away_score, home_score)
        };

        self.games_won += own_score as u32;
        self.games_lost += opp_score as u32;
        self.game_diff = self.games_won as i32 - self.games_lost as i32;

        // 积分规则：2:0胜3分、2:1胜2分、1:2负1分、0:2负0分
        if own_score > opp_score {
            self.wins += 1;
            if opp_score == 0 {
                self.points += 3; // 2:0胜
            } else {
                self.points += 2; // 2:1胜
            }
        } else {
            self.losses += 1;
            if own_score == 1 {
                self.points += 1; // 1:2负
            }
            // 0:2负 = 0分
        }
    }
}

/// 年度积分明细
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnualPointsDetail {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub team_id: u64,
    pub tournament_id: u64,
    /// 赛事名称
    #[serde(default)]
    pub tournament_name: Option<String>,
    /// 赛事类型
    #[serde(default)]
    pub tournament_type: Option<String>,
    /// 获得积分
    pub points: u32,
    /// 赛事最终排名
    pub final_rank: Option<u32>,
}

/// 全球积分排名
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalRanking {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub team_id: u64,
    /// 全球排名
    pub global_rank: u32,
    /// 总积分
    pub total_points: u32,
}

/// 历史交锋统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadToHeadStats {
    pub id: u64,
    pub save_id: String,
    pub team1_id: u64,
    pub team2_id: u64,
    /// 总交手次数
    pub total_matches: u32,
    /// team1胜场
    pub team1_wins: u32,
    /// team2胜场
    pub team2_wins: u32,
    /// team1小局胜
    pub team1_games: u32,
    /// team2小局胜
    pub team2_games: u32,
    /// 最近一次交手ID
    pub last_match_id: Option<u64>,
}

impl HeadToHeadStats {
    pub fn new(save_id: String, team1_id: u64, team2_id: u64) -> Self {
        // 确保 team1_id < team2_id，避免重复记录
        let (t1, t2) = if team1_id < team2_id {
            (team1_id, team2_id)
        } else {
            (team2_id, team1_id)
        };

        Self {
            id: 0,
            save_id,
            team1_id: t1,
            team2_id: t2,
            total_matches: 0,
            team1_wins: 0,
            team2_wins: 0,
            team1_games: 0,
            team2_games: 0,
            last_match_id: None,
        }
    }

    /// 更新交锋记录
    pub fn update(
        &mut self,
        winner_id: u64,
        team1_score: u8,
        team2_score: u8,
        match_id: u64,
    ) {
        self.total_matches += 1;
        self.team1_games += team1_score as u32;
        self.team2_games += team2_score as u32;

        if winner_id == self.team1_id {
            self.team1_wins += 1;
        } else {
            self.team2_wins += 1;
        }

        self.last_match_id = Some(match_id);
    }

    /// 获取team1的胜率
    pub fn team1_win_rate(&self) -> f64 {
        if self.total_matches == 0 {
            0.5
        } else {
            self.team1_wins as f64 / self.total_matches as f64
        }
    }
}

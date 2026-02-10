use serde::{Deserialize, Serialize};

/// 选手赛季统计数据
/// 用于数据中心记录选手在每个赛季的表现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSeasonStatistics {
    pub id: Option<i64>,
    pub save_id: String,
    pub player_id: i64,
    pub player_name: String,
    pub season_id: i64,
    pub team_id: Option<i64>,
    pub region_id: Option<String>,
    pub position: String,

    // 比赛统计
    pub matches_played: i32,
    pub games_played: i32,

    // 影响力统计
    pub total_impact: f64,
    pub avg_impact: f64,

    // 发挥统计
    pub avg_performance: f64,
    pub best_performance: f64,
    pub worst_performance: f64,

    // 稳定性评分
    pub consistency_score: f64,

    // 冠军加成
    pub international_titles: i32,
    pub regional_titles: i32,
    pub champion_bonus: f64,

    // 年度Top得分
    pub yearly_top_score: f64,

    // 统治力得分
    pub dominance_score: f64,
}

impl PlayerSeasonStatistics {
    /// 创建新的选手赛季统计
    pub fn new(
        save_id: String,
        player_id: i64,
        player_name: String,
        season_id: i64,
        team_id: Option<i64>,
        region_id: Option<String>,
        position: String,
    ) -> Self {
        Self {
            id: None,
            save_id,
            player_id,
            player_name,
            season_id,
            team_id,
            region_id,
            position,
            matches_played: 0,
            games_played: 0,
            total_impact: 0.0,
            avg_impact: 0.0,
            avg_performance: 0.0,
            best_performance: 0.0,
            worst_performance: 100.0,
            consistency_score: 100.0,
            international_titles: 0,
            regional_titles: 0,
            champion_bonus: 0.0,
            yearly_top_score: 0.0,
            dominance_score: 0.0,
        }
    }

    /// 五维归一化年度Top评分
    pub fn calculate_yearly_top_score(
        avg_impact: f64,
        avg_performance: f64,
        consistency_score: f64,
        games_played: i32,
        champion_bonus: f64,
    ) -> f64 {
        let impact_norm = ((avg_impact + 10.0) * 3.33).clamp(0.0, 100.0);
        let perf_norm = ((avg_performance - 50.0) * 2.0).clamp(0.0, 100.0);
        let stability_norm = consistency_score.clamp(0.0, 100.0);
        let appearance_norm = (games_played as f64 * 0.83).clamp(0.0, 100.0);
        let honor_norm = (champion_bonus * 6.67).clamp(0.0, 100.0);

        impact_norm * 0.45
            + perf_norm * 0.20
            + stability_norm * 0.15
            + appearance_norm * 0.10
            + honor_norm * 0.10
    }

    /// 统治力评分（独立公式）
    pub fn calculate_dominance_score(
        best_performance: f64,
        avg_impact: f64,
        avg_performance: f64,
    ) -> f64 {
        let peak_norm = ((best_performance - 60.0) * 2.5).clamp(0.0, 100.0);
        let impact_norm = ((avg_impact + 5.0) * 5.0).clamp(0.0, 100.0);
        let perf_norm = ((avg_performance - 50.0) * 2.0).clamp(0.0, 100.0);

        peak_norm * 0.35 + impact_norm * 0.45 + perf_norm * 0.20
    }

    /// 记录一局比赛的表现
    pub fn record_game(&mut self, impact_score: f64, actual_ability: f64) {
        self.games_played += 1;
        self.total_impact += impact_score;
        self.avg_impact = self.total_impact / self.games_played as f64;

        // 更新发挥统计
        let new_avg = ((self.avg_performance * (self.games_played - 1) as f64) + actual_ability)
            / self.games_played as f64;
        self.avg_performance = new_avg;

        if actual_ability > self.best_performance {
            self.best_performance = actual_ability;
        }
        if actual_ability < self.worst_performance {
            self.worst_performance = actual_ability;
        }

        // 计算稳定性评分
        self.consistency_score =
            (100.0 - (self.best_performance - self.worst_performance) * 2.0).max(0.0);

        // 更新年度Top得分（五维归一化）
        self.yearly_top_score = Self::calculate_yearly_top_score(
            self.avg_impact,
            self.avg_performance,
            self.consistency_score,
            self.games_played,
            self.champion_bonus,
        );

        // 更新统治力得分
        self.dominance_score = Self::calculate_dominance_score(
            self.best_performance,
            self.avg_impact,
            self.avg_performance,
        );
    }

    /// 记录比赛结束（增加比赛场数）
    pub fn record_match_completed(&mut self) {
        self.matches_played += 1;
    }

    /// 记录冠军荣誉
    pub fn record_championship(&mut self, is_international: bool) {
        if is_international {
            self.international_titles += 1;
        } else {
            self.regional_titles += 1;
        }
        // 国际赛冠军+3, 赛区冠军+1
        let bonus = if is_international { 3.0 } else { 1.0 };
        self.champion_bonus += bonus;
        self.recalculate_score();
    }

    /// placement: "RUNNER_UP" | "THIRD"
    pub fn record_placement(&mut self, placement: &str, is_international: bool) {
        let bonus = match (placement, is_international) {
            ("RUNNER_UP", true) => 2.0,
            ("RUNNER_UP", false) => 0.5,
            ("THIRD", true) => 1.0,
            ("THIRD", false) => 0.25,
            _ => 0.0,
        };
        self.champion_bonus += bonus;
        self.recalculate_score();
    }

    fn recalculate_score(&mut self) {
        self.yearly_top_score = Self::calculate_yearly_top_score(
            self.avg_impact,
            self.avg_performance,
            self.consistency_score,
            self.games_played,
            self.champion_bonus,
        );
    }
}

/// 记录选手表现的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPerformanceRequest {
    pub player_id: i64,
    pub player_name: String,
    pub team_id: i64,
    pub position: String,
    pub impact_score: f64,
    pub actual_ability: f64,
    pub season_id: i64,
    pub region_id: Option<String>,
}

/// 记录冠军荣誉的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordChampionshipRequest {
    pub team_id: i64,
    pub is_international: bool,
    pub season_id: i64,
}

/// 选手排行榜项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRankingItem {
    pub player_id: i64,
    pub player_name: String,
    pub team_id: Option<i64>,
    pub position: String,
    pub region_id: Option<String>,
    pub games_played: i32,
    pub avg_impact: f64,
    pub avg_performance: f64,
    pub consistency_score: f64,
    pub champion_bonus: f64,
    pub yearly_top_score: f64,
    pub dominance_score: f64,
}

impl From<PlayerSeasonStatistics> for PlayerRankingItem {
    fn from(stats: PlayerSeasonStatistics) -> Self {
        Self {
            player_id: stats.player_id,
            player_name: stats.player_name,
            team_id: stats.team_id,
            position: stats.position,
            region_id: stats.region_id,
            games_played: stats.games_played,
            avg_impact: stats.avg_impact,
            avg_performance: stats.avg_performance,
            consistency_score: stats.consistency_score,
            champion_bonus: stats.champion_bonus,
            yearly_top_score: stats.yearly_top_score,
            dominance_score: stats.dominance_score,
        }
    }
}

use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LadderPlayer {
    pub player_id: i64,
    pub player_name: String,
    pub game_id: String,
    pub position: String,
    pub team_name: Option<String>,
    pub rating: i32,
}

#[derive(Debug, Clone)]
pub struct LadderTeam {
    pub players: Vec<LadderPlayer>,
}

#[derive(Debug, Clone)]
pub struct LadderMatch {
    pub blue_team: LadderTeam,
    pub red_team: LadderTeam,
    pub blue_avg_rating: i32,
    pub red_avg_rating: i32,
}

pub struct LadderMatchmaker;

impl LadderMatchmaker {
    /// 按位置配对选手，确保每支队伍包含 Top/Jug/Mid/Adc/Sup 各一名。
    /// 同位置内按 rating 排序后相邻配对，随机分蓝红方。
    pub fn create_round_matches(
        players: Vec<LadderPlayer>,
        bye_players: &mut Vec<LadderPlayer>,
    ) -> Vec<LadderMatch> {
        let mut rng = rand::thread_rng();

        // 1. 按位置分池
        let mut players_by_position: HashMap<String, Vec<LadderPlayer>> = HashMap::new();
        for player in players {
            players_by_position
                .entry(player.position.clone())
                .or_insert_with(Vec::new)
                .push(player);
        }

        let positions = vec!["Top", "Jug", "Mid", "Adc", "Sup"];

        // 2. 每个位置内按 rating 排序，相邻配对，分蓝红队列
        let mut blue_by_pos: HashMap<String, Vec<LadderPlayer>> = HashMap::new();
        let mut red_by_pos: HashMap<String, Vec<LadderPlayer>> = HashMap::new();

        for position in &positions {
            let pos_key = position.to_string();
            if let Some(pool) = players_by_position.get_mut(&pos_key) {
                pool.shuffle(&mut rng);
                pool.sort_by_key(|p| -(p.rating as i32));

                let blue_list = blue_by_pos.entry(pos_key.clone()).or_insert_with(Vec::new);
                let red_list = red_by_pos.entry(pos_key.clone()).or_insert_with(Vec::new);

                while pool.len() >= 2 {
                    let p1 = pool.remove(0);
                    let mut p2_idx = 0;

                    if let Some(ref t1) = p1.team_name {
                        for (i, candidate) in pool.iter().enumerate() {
                            if candidate.team_name.as_ref() != Some(t1) {
                                p2_idx = i;
                                break;
                            }
                        }
                    }

                    let p2 = pool.remove(p2_idx);

                    if rng.gen_bool(0.5) {
                        blue_list.push(p1);
                        red_list.push(p2);
                    } else {
                        blue_list.push(p2);
                        red_list.push(p1);
                    }
                }

                // 奇数个选手，最后一个轮空
                if !pool.is_empty() {
                    bye_players.push(pool.remove(0));
                }
            }
        }

        // 3. 计算可组成的完整队伍数（取各位置最小配对数）
        let match_count = positions
            .iter()
            .map(|pos| blue_by_pos.get(*pos).map_or(0, |v| v.len()))
            .min()
            .unwrap_or(0);

        // 4. 逐场组队：每场从每个位置各取一名蓝方/红方选手
        let mut matches = Vec::with_capacity(match_count);
        for _ in 0..match_count {
            let mut blue_players = Vec::with_capacity(5);
            let mut red_players = Vec::with_capacity(5);

            for position in &positions {
                let pos_key = position.to_string();
                if let Some(blue_list) = blue_by_pos.get_mut(&pos_key) {
                    if !blue_list.is_empty() {
                        blue_players.push(blue_list.remove(0));
                    }
                }
                if let Some(red_list) = red_by_pos.get_mut(&pos_key) {
                    if !red_list.is_empty() {
                        red_players.push(red_list.remove(0));
                    }
                }
            }

            if blue_players.len() == 5 && red_players.len() == 5 {
                let blue_avg_rating = blue_players.iter().map(|p| p.rating).sum::<i32>() / 5;
                let red_avg_rating = red_players.iter().map(|p| p.rating).sum::<i32>() / 5;

                matches.push(LadderMatch {
                    blue_team: LadderTeam {
                        players: blue_players,
                    },
                    red_team: LadderTeam {
                        players: red_players,
                    },
                    blue_avg_rating,
                    red_avg_rating,
                });
            }
        }

        // 5. 剩余未配对的选手也加入轮空
        for (_pos, remaining) in blue_by_pos.drain() {
            bye_players.extend(remaining);
        }
        for (_pos, remaining) in red_by_pos.drain() {
            bye_players.extend(remaining);
        }

        matches
    }
}

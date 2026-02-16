//! 英雄/BP系统 Tauri 命令

use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::bp_engine::{hard_counter_pairs, soft_counter_pairs, CompType};
use crate::engines::champion::CHAMPIONS;
use crate::engines::meta_engine::MetaType;
use crate::get_pool;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use tauri::State;

// ── 响应结构体 ──────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChampionInfo {
    pub id: u8,
    pub name_cn: String,
    pub name_en: String,
    pub position: String,
    pub archetype: String,
    pub archetype_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChampionStatInfo {
    pub champion_id: u8,
    pub name_cn: String,
    pub name_en: String,
    pub position: String,
    pub pick_count: u32,
    pub win_count: u32,
    pub ban_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DraftResultInfo {
    pub match_id: i64,
    pub game_number: i64,
    pub bans_json: String,
    pub home_picks_json: String,
    pub away_picks_json: String,
    pub home_comp: Option<String>,
    pub away_comp: Option<String>,
    pub draft_narrative_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompStatInfo {
    pub comp_type: String,
    pub pick_count: u32,
    pub win_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompMatchupInfo {
    pub comp_type: String,
    pub display_name: String,
    pub core_archetypes: Vec<String>,
    pub difficulty_bonus: f64,
    pub hard_counters: Vec<String>,
    pub hard_countered_by: Vec<String>,
    pub soft_counters: Vec<String>,
    pub soft_countered_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaCompEffect {
    pub meta_type: String,
    pub meta_name: String,
    pub favored_comps: Vec<String>,
    pub unfavored_comps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerMasteryInfo {
    pub champion_id: u8,
    pub name_cn: String,
    pub position: String,
    pub mastery_tier: String,
    pub games_played: i32,
    pub games_won: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerChampionUsageItem {
    pub player_id: u64,
    pub player_name: String,
    pub team_name: String,
    pub position: String,
    pub champion_id: u8,
    pub champion_name: String,
    pub games: u32,
    pub wins: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamCompUsageItem {
    pub team_id: u64,
    pub team_name: String,
    pub comp_type: String,
    pub comp_name: String,
    pub games: u32,
    pub wins: u32,
}

// ── 用于反序列化 DB 中 JSON 的辅助结构 ──────────────────

#[derive(Debug, Deserialize)]
struct StoredBan {
    champion_id: u8,
}

#[derive(Debug, Deserialize)]
struct StoredPick {
    champion_id: u8,
}

#[derive(Debug, Deserialize)]
struct StoredPickFull {
    player_id: u64,
    champion_id: u8,
}

fn comp_type_name(comp_type: CompType) -> String {
    format!("{:?}", comp_type)
}

fn comp_display_name(comp_type: CompType) -> &'static str {
    match comp_type {
        CompType::Rush => "速推",
        CompType::PickOff => "抓单",
        CompType::AllIn => "莽夫",
        CompType::MidJungle => "中野联动",
        CompType::TopJungle => "上野联动",
        CompType::Protect => "保C",
        CompType::Fortress => "铁桶阵",
        CompType::UtilityComp => "功能流",
        CompType::Stall => "龟缩",
        CompType::BotLane => "下路统治",
        CompType::Teamfight => "团战",
        CompType::Dive => "开团",
        CompType::Skirmish => "小规模团战",
        CompType::DualCarry => "双C",
        CompType::Flex => "全能",
        CompType::Splitpush => "分推",
        CompType::SideLane => "4-1分带",
        CompType::Control => "运营",
        CompType::TripleThreat => "三线施压",
        CompType::LateGame => "后期发育",
    }
}

// ── 命令 ────────────────────────────────────────────────

/// 获取全部50个英雄列表
#[tauri::command]
pub fn get_champion_list() -> CommandResult<Vec<ChampionInfo>> {
    let champions: Vec<ChampionInfo> = CHAMPIONS
        .iter()
        .map(|c| ChampionInfo {
            id: c.id,
            name_cn: c.name_cn.to_string(),
            name_en: c.name_en.to_string(),
            position: format!("{:?}", c.position),
            archetype: c.archetype.id().to_string(),
            archetype_name: c.archetype.display_name().to_string(),
        })
        .collect();

    CommandResult::ok(champions)
}

/// 获取英雄使用/胜率/ban率统计
#[tauri::command]
pub async fn get_champion_stats(
    state: State<'_, AppState>,
    save_id: String,
    season_id: Option<i64>,
) -> Result<CommandResult<Vec<ChampionStatInfo>>, String> {
    let pool = get_pool!(state);

    let mut sql = String::from(
        "SELECT d.bans_json, d.home_picks_json, d.away_picks_json,
                m.home_team_id, g.winner_team_id
         FROM game_draft_results d
         JOIN match_games g ON g.save_id = d.save_id
              AND g.match_id = d.match_id AND g.game_number = d.game_number
         JOIN matches m ON m.save_id = d.save_id AND m.id = d.match_id",
    );

    let filter_season = season_id.filter(|&s| s > 0);
    if filter_season.is_some() {
        sql.push_str(" JOIN tournaments t ON m.tournament_id = t.id AND t.save_id = d.save_id");
    }
    sql.push_str(" WHERE d.save_id = ?");
    if filter_season.is_some() {
        sql.push_str(" AND t.season_id = ?");
    }

    let mut query = sqlx::query(&sql).bind(&save_id);
    if let Some(sid) = filter_season {
        query = query.bind(sid);
    }
    let rows = query.fetch_all(&pool).await.map_err(|e| e.to_string())?;

    // champion_id → (pick_count, win_count, ban_count)
    let mut stats: HashMap<u8, (u32, u32, u32)> = HashMap::new();

    for row in &rows {
        let bans_json: String = row.get("bans_json");
        let home_picks_json: String = row.get("home_picks_json");
        let away_picks_json: String = row.get("away_picks_json");
        let home_team_id: i64 = row.get("home_team_id");
        let winner_team_id: i64 = row.get("winner_team_id");

        // ban 统计
        if let Ok(bans) = serde_json::from_str::<Vec<StoredBan>>(&bans_json) {
            for ban in &bans {
                stats.entry(ban.champion_id).or_default().2 += 1;
            }
        }

        // home picks
        if let Ok(picks) = serde_json::from_str::<Vec<StoredPick>>(&home_picks_json) {
            let home_won = home_team_id == winner_team_id;
            for pick in &picks {
                let entry = stats.entry(pick.champion_id).or_default();
                entry.0 += 1;
                if home_won {
                    entry.1 += 1;
                }
            }
        }

        // away picks
        if let Ok(picks) = serde_json::from_str::<Vec<StoredPick>>(&away_picks_json) {
            let away_won = home_team_id != winner_team_id;
            for pick in &picks {
                let entry = stats.entry(pick.champion_id).or_default();
                entry.0 += 1;
                if away_won {
                    entry.1 += 1;
                }
            }
        }
    }

    // 只返回有数据的英雄，按 pick_count 降序
    let mut result: Vec<ChampionStatInfo> = stats
        .into_iter()
        .filter_map(|(cid, (pick_count, win_count, ban_count))| {
            let champ = CHAMPIONS.iter().find(|c| c.id == cid)?;
            Some(ChampionStatInfo {
                champion_id: cid,
                name_cn: champ.name_cn.to_string(),
                name_en: champ.name_en.to_string(),
                position: format!("{:?}", champ.position),
                pick_count,
                win_count,
                ban_count,
            })
        })
        .collect();

    result.sort_by(|a, b| b.pick_count.cmp(&a.pick_count));

    Ok(CommandResult::ok(result))
}

#[tauri::command]
pub async fn get_draft_result(
    state: State<'_, AppState>,
    save_id: String,
    match_id: i64,
    game_number: i64,
) -> Result<CommandResult<Option<DraftResultInfo>>, String> {
    let pool = get_pool!(state);

    let row = sqlx::query(
        "SELECT match_id, game_number, bans_json, home_picks_json, away_picks_json,
                home_comp, away_comp, draft_narrative_json
         FROM game_draft_results
         WHERE save_id = ? AND match_id = ? AND game_number = ?",
    )
    .bind(&save_id)
    .bind(match_id)
    .bind(game_number)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let info = row.map(|r| DraftResultInfo {
        match_id: r.get("match_id"),
        game_number: r.get("game_number"),
        bans_json: r.get("bans_json"),
        home_picks_json: r.get("home_picks_json"),
        away_picks_json: r.get("away_picks_json"),
        home_comp: r.get("home_comp"),
        away_comp: r.get("away_comp"),
        draft_narrative_json: r.get("draft_narrative_json"),
    });

    Ok(CommandResult::ok(info))
}

#[tauri::command]
pub async fn get_player_champion_mastery(
    state: State<'_, AppState>,
    save_id: String,
    player_id: i64,
) -> Result<CommandResult<Vec<PlayerMasteryInfo>>, String> {
    let pool = get_pool!(state);

    let rows = sqlx::query(
        "SELECT champion_id, mastery_tier, games_played, games_won
         FROM player_champion_mastery
         WHERE save_id = ? AND player_id = ?
         ORDER BY CASE mastery_tier WHEN 'SS' THEN 0 WHEN 'S' THEN 1 WHEN 'A' THEN 2 ELSE 3 END,
                  games_played DESC",
    )
    .bind(&save_id)
    .bind(player_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let result: Vec<PlayerMasteryInfo> = rows
        .iter()
        .filter_map(|r| {
            let cid: u8 = r.get::<i64, _>("champion_id") as u8;
            let champ = CHAMPIONS.iter().find(|c| c.id == cid)?;
            Some(PlayerMasteryInfo {
                champion_id: cid,
                name_cn: champ.name_cn.to_string(),
                position: format!("{:?}", champ.position),
                mastery_tier: r.get("mastery_tier"),
                games_played: r.get::<i64, _>("games_played") as i32,
                games_won: r.get::<i64, _>("games_won") as i32,
            })
        })
        .collect();

    Ok(CommandResult::ok(result))
}

/// 获取体系使用/胜率统计
#[tauri::command]
pub async fn get_comp_stats(
    state: State<'_, AppState>,
    save_id: String,
    season_id: Option<i64>,
) -> Result<CommandResult<Vec<CompStatInfo>>, String> {
    let pool = get_pool!(state);

    let mut sql = String::from(
        "SELECT d.home_comp, d.away_comp, m.home_team_id, g.winner_team_id
         FROM game_draft_results d
         JOIN match_games g ON g.save_id = d.save_id
              AND g.match_id = d.match_id AND g.game_number = d.game_number
         JOIN matches m ON m.save_id = d.save_id AND m.id = d.match_id",
    );

    let filter_season = season_id.filter(|&s| s > 0);
    if filter_season.is_some() {
        sql.push_str(" JOIN tournaments t ON m.tournament_id = t.id AND t.save_id = d.save_id");
    }
    sql.push_str(" WHERE d.save_id = ?");
    if filter_season.is_some() {
        sql.push_str(" AND t.season_id = ?");
    }

    let mut query = sqlx::query(&sql).bind(&save_id);
    if let Some(sid) = filter_season {
        query = query.bind(sid);
    }
    let rows = query.fetch_all(&pool).await.map_err(|e| e.to_string())?;

    // comp_type → (pick_count, win_count)
    let mut stats: HashMap<String, (u32, u32)> = HashMap::new();

    for row in &rows {
        let home_comp: Option<String> = row.get("home_comp");
        let away_comp: Option<String> = row.get("away_comp");
        let home_team_id: i64 = row.get("home_team_id");
        let winner_team_id: i64 = row.get("winner_team_id");

        let home_won = home_team_id == winner_team_id;

        if let Some(comp) = home_comp {
            let entry = stats.entry(comp).or_default();
            entry.0 += 1;
            if home_won {
                entry.1 += 1;
            }
        }

        if let Some(comp) = away_comp {
            let entry = stats.entry(comp).or_default();
            entry.0 += 1;
            if !home_won {
                entry.1 += 1;
            }
        }
    }

    let mut result: Vec<CompStatInfo> = stats
        .into_iter()
        .map(|(comp_type, (pick_count, win_count))| CompStatInfo {
            comp_type,
            pick_count,
            win_count,
        })
        .collect();

    result.sort_by(|a, b| b.pick_count.cmp(&a.pick_count));

    Ok(CommandResult::ok(result))
}

#[tauri::command]
pub fn get_comp_matchups() -> CommandResult<Vec<CompMatchupInfo>> {
    let result: Vec<CompMatchupInfo> = CompType::all()
        .iter()
        .map(|comp| {
            let comp_type = *comp;

            let hard_counters = hard_counter_pairs()
                .iter()
                .filter_map(|(attacker, victim)| {
                    (*attacker == comp_type).then(|| comp_type_name(*victim))
                })
                .collect();

            let hard_countered_by = hard_counter_pairs()
                .iter()
                .filter_map(|(attacker, victim)| {
                    (*victim == comp_type).then(|| comp_type_name(*attacker))
                })
                .collect();

            let soft_counters = soft_counter_pairs()
                .iter()
                .filter_map(|(attacker, victim)| {
                    (*attacker == comp_type).then(|| comp_type_name(*victim))
                })
                .collect();

            let soft_countered_by = soft_counter_pairs()
                .iter()
                .filter_map(|(attacker, victim)| {
                    (*victim == comp_type).then(|| comp_type_name(*attacker))
                })
                .collect();

            CompMatchupInfo {
                comp_type: comp_type_name(comp_type),
                display_name: comp_display_name(comp_type).to_string(),
                core_archetypes: comp_type
                    .core_archetypes()
                    .iter()
                    .map(|archetype| format!("{:?}", archetype))
                    .collect(),
                difficulty_bonus: comp_type.difficulty_bonus(),
                hard_counters,
                hard_countered_by,
                soft_counters,
                soft_countered_by,
            }
        })
        .collect();

    CommandResult::ok(result)
}

#[tauri::command]
pub fn get_meta_comp_effects() -> CommandResult<Vec<MetaCompEffect>> {
    let result: Vec<MetaCompEffect> = MetaType::all()
        .iter()
        .map(|meta| {
            let meta_type = *meta;
            let favored_comps = CompType::all()
                .iter()
                .filter_map(|comp| comp.is_meta_favored(meta_type).then(|| comp_type_name(*comp)))
                .collect();

            let unfavored_comps = CompType::all()
                .iter()
                .filter_map(|comp| (!comp.is_meta_favored(meta_type)).then(|| comp_type_name(*comp)))
                .collect();

            MetaCompEffect {
                meta_type: meta_type.id().to_string(),
                meta_name: meta_type.display_name().to_string(),
                favored_comps,
                unfavored_comps,
            }
        })
        .collect();

    CommandResult::ok(result)
}

#[tauri::command]
pub async fn get_player_champion_usage(
    state: State<'_, AppState>,
    save_id: String,
    season_id: Option<i64>,
) -> Result<CommandResult<Vec<PlayerChampionUsageItem>>, String> {
    let pool = get_pool!(state);

    let mut sql = String::from(
        "SELECT d.home_picks_json, d.away_picks_json,
                m.home_team_id, m.away_team_id, g.winner_team_id
         FROM game_draft_results d
         JOIN match_games g ON g.save_id = d.save_id
              AND g.match_id = d.match_id AND g.game_number = d.game_number
         JOIN matches m ON m.save_id = d.save_id AND m.id = d.match_id",
    );

    let filter_season = season_id.filter(|&s| s > 0);
    if filter_season.is_some() {
        sql.push_str(" JOIN tournaments t ON m.tournament_id = t.id AND t.save_id = d.save_id");
    }
    sql.push_str(" WHERE d.save_id = ?");
    if filter_season.is_some() {
        sql.push_str(" AND t.season_id = ?");
    }

    let mut query = sqlx::query(&sql).bind(&save_id);
    if let Some(sid) = filter_season {
        query = query.bind(sid);
    }
    let rows = query.fetch_all(&pool).await.map_err(|e| e.to_string())?;

    let mut usage: HashMap<(u64, u8), (u32, u32)> = HashMap::new();

    for row in &rows {
        let home_picks_json: String = row.get("home_picks_json");
        let away_picks_json: String = row.get("away_picks_json");
        let home_team_id: i64 = row.get("home_team_id");
        let winner_team_id: i64 = row.get("winner_team_id");

        if let Ok(picks) = serde_json::from_str::<Vec<StoredPickFull>>(&home_picks_json) {
            let won = home_team_id == winner_team_id;
            for pick in &picks {
                let entry = usage.entry((pick.player_id, pick.champion_id)).or_default();
                entry.0 += 1;
                if won { entry.1 += 1; }
            }
        }

        if let Ok(picks) = serde_json::from_str::<Vec<StoredPickFull>>(&away_picks_json) {
            let won = home_team_id != winner_team_id;
            for pick in &picks {
                let entry = usage.entry((pick.player_id, pick.champion_id)).or_default();
                entry.0 += 1;
                if won { entry.1 += 1; }
            }
        }
    }

    let player_ids: Vec<i64> = usage.keys().map(|(pid, _)| *pid as i64).collect::<std::collections::HashSet<_>>().into_iter().collect();
    let mut player_info: HashMap<u64, (String, String, String)> = HashMap::new();

    if !player_ids.is_empty() {
        let placeholders: String = player_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let q = format!(
            "SELECT p.id, p.game_id, p.position, COALESCE(t.name, '') as team_name
             FROM players p LEFT JOIN teams t ON p.team_id = t.id
             WHERE p.save_id = ? AND p.id IN ({})", placeholders
        );
        let mut qb = sqlx::query(&q).bind(&save_id);
        for pid in &player_ids {
            qb = qb.bind(*pid);
        }
        if let Ok(prows) = qb.fetch_all(&pool).await {
            for r in &prows {
                let id: i64 = r.get("id");
                let game_id: String = r.get("game_id");
                let position: String = r.get::<Option<String>, _>("position").unwrap_or_default();
                let team_name: String = r.get("team_name");
                player_info.insert(id as u64, (game_id, position, team_name));
            }
        }
    }

    let mut result: Vec<PlayerChampionUsageItem> = usage
        .into_iter()
        .filter_map(|((pid, cid), (games, wins))| {
            let champ = CHAMPIONS.iter().find(|c| c.id == cid)?;
            let (player_name, position, team_name) = player_info.get(&pid).cloned()
                .unwrap_or_else(|| (format!("Player#{}", pid), String::new(), String::new()));
            Some(PlayerChampionUsageItem {
                player_id: pid,
                player_name,
                team_name,
                position,
                champion_id: cid,
                champion_name: champ.name_cn.to_string(),
                games,
                wins,
            })
        })
        .collect();

    result.sort_by(|a, b| b.games.cmp(&a.games));

    Ok(CommandResult::ok(result))
}

#[tauri::command]
pub async fn get_team_comp_usage(
    state: State<'_, AppState>,
    save_id: String,
    season_id: Option<i64>,
) -> Result<CommandResult<Vec<TeamCompUsageItem>>, String> {
    let pool = get_pool!(state);

    let mut sql = String::from(
        "SELECT d.home_comp, d.away_comp, m.home_team_id, m.away_team_id, g.winner_team_id
         FROM game_draft_results d
         JOIN match_games g ON g.save_id = d.save_id
              AND g.match_id = d.match_id AND g.game_number = d.game_number
         JOIN matches m ON m.save_id = d.save_id AND m.id = d.match_id",
    );

    let filter_season = season_id.filter(|&s| s > 0);
    if filter_season.is_some() {
        sql.push_str(" JOIN tournaments t ON m.tournament_id = t.id AND t.save_id = d.save_id");
    }
    sql.push_str(" WHERE d.save_id = ?");
    if filter_season.is_some() {
        sql.push_str(" AND t.season_id = ?");
    }

    let mut query = sqlx::query(&sql).bind(&save_id);
    if let Some(sid) = filter_season {
        query = query.bind(sid);
    }
    let rows = query.fetch_all(&pool).await.map_err(|e| e.to_string())?;

    let mut usage: HashMap<(i64, String), (u32, u32)> = HashMap::new();

    for row in &rows {
        let home_comp: Option<String> = row.get("home_comp");
        let away_comp: Option<String> = row.get("away_comp");
        let home_team_id: i64 = row.get("home_team_id");
        let away_team_id: i64 = row.get("away_team_id");
        let winner_team_id: i64 = row.get("winner_team_id");

        if let Some(comp) = home_comp {
            let entry = usage.entry((home_team_id, comp)).or_default();
            entry.0 += 1;
            if home_team_id == winner_team_id { entry.1 += 1; }
        }
        if let Some(comp) = away_comp {
            let entry = usage.entry((away_team_id, comp)).or_default();
            entry.0 += 1;
            if away_team_id == winner_team_id { entry.1 += 1; }
        }
    }

    let team_ids: Vec<i64> = usage.keys().map(|(tid, _)| *tid).collect::<std::collections::HashSet<_>>().into_iter().collect();
    let mut team_names: HashMap<i64, String> = HashMap::new();

    if !team_ids.is_empty() {
        let placeholders: String = team_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let q = format!("SELECT id, name FROM teams WHERE id IN ({})", placeholders);
        let mut qb = sqlx::query(&q);
        for tid in &team_ids {
            qb = qb.bind(*tid);
        }
        if let Ok(trows) = qb.fetch_all(&pool).await {
            for r in &trows {
                let id: i64 = r.get("id");
                let name: String = r.get("name");
                team_names.insert(id, name);
            }
        }
    }

    let mut result: Vec<TeamCompUsageItem> = usage
        .into_iter()
        .map(|((tid, comp_type), (games, wins))| {
            let team_name = team_names.get(&tid).cloned().unwrap_or_else(|| format!("Team#{}", tid));
            let comp_name = comp_display_name_str(&comp_type).to_string();
            TeamCompUsageItem {
                team_id: tid as u64,
                team_name,
                comp_type,
                comp_name,
                games,
                wins,
            }
        })
        .collect();

    result.sort_by(|a, b| b.games.cmp(&a.games));

    Ok(CommandResult::ok(result))
}

fn comp_display_name_str(s: &str) -> &'static str {
    match s {
        "Rush" => "速推", "PickOff" => "抓单", "AllIn" => "莽夫",
        "MidJungle" => "中野联动", "TopJungle" => "上野联动",
        "Protect" => "保C", "Fortress" => "铁桶阵", "UtilityComp" => "功能流",
        "Stall" => "龟缩", "BotLane" => "下路统治",
        "Teamfight" => "团战", "Dive" => "开团", "Skirmish" => "小规模团战",
        "DualCarry" => "双C", "Flex" => "全能",
        "Splitpush" => "分推", "SideLane" => "4-1分带", "Control" => "运营",
        "TripleThreat" => "三线施压", "LateGame" => "后期发育",
        _ => "未知",
    }
}

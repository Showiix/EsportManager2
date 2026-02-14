//! 英雄/BP系统 Tauri 命令

use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::champion::CHAMPIONS;
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompStatInfo {
    pub comp_type: String,
    pub pick_count: u32,
    pub win_count: u32,
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

// ── 用于反序列化 DB 中 JSON 的辅助结构 ──────────────────

#[derive(Debug, Deserialize)]
struct StoredBan {
    champion_id: u8,
}

#[derive(Debug, Deserialize)]
struct StoredPick {
    champion_id: u8,
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
) -> Result<CommandResult<Vec<ChampionStatInfo>>, String> {
    let pool = get_pool!(state);

    let rows = sqlx::query(
        "SELECT d.bans_json, d.home_picks_json, d.away_picks_json,
                m.home_team_id, g.winner_team_id
         FROM game_draft_results d
         JOIN match_games g ON g.save_id = d.save_id
              AND g.match_id = d.match_id AND g.game_number = d.game_number
         JOIN matches m ON m.save_id = d.save_id AND m.id = d.match_id
         WHERE d.save_id = ?",
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

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
                home_comp, away_comp
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
) -> Result<CommandResult<Vec<CompStatInfo>>, String> {
    let pool = get_pool!(state);

    let rows = sqlx::query(
        "SELECT d.home_comp, d.away_comp, m.home_team_id, g.winner_team_id
         FROM game_draft_results d
         JOIN match_games g ON g.save_id = d.save_id
              AND g.match_id = d.match_id AND g.game_number = d.game_number
         JOIN matches m ON m.save_id = d.save_id AND m.id = d.match_id
         WHERE d.save_id = ?",
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

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

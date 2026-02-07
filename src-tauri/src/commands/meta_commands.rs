//! Meta 版本系统 Tauri 命令

use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::meta_engine::{MetaType, get_meta_weights};
use crate::models::meta::{MetaInfo, MetaWeightsInfo, MetaHistoryEntry, MetaTypeInfo};
use sqlx::Row;
use tauri::State;
use crate::get_pool;

/// 获取当前赛季的 Meta 版本信息
#[tauri::command]
pub async fn get_current_meta(
    state: State<'_, AppState>,
    save_id: String,
) -> Result<CommandResult<MetaInfo>, String> {
    let pool = get_pool!(state);

    // 获取当前赛季
    let current_season: i64 = sqlx::query_scalar(
        "SELECT current_season FROM saves WHERE id = ?"
    )
    .bind(&save_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or(1);

    // 查询当前赛季 meta
    let row = sqlx::query(
        "SELECT meta_type, meta_name, weight_top, weight_jug, weight_mid, weight_adc, weight_sup FROM meta_versions WHERE save_id = ? AND season_id = ?"
    )
    .bind(&save_id)
    .bind(current_season)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let info = match row {
        Some(r) => {
            let meta_type_str: String = r.get("meta_type");
            let description = MetaType::from_id(&meta_type_str)
                .map(|m| m.description().to_string())
                .unwrap_or_default();

            MetaInfo {
                season_id: current_season,
                meta_type: meta_type_str,
                meta_name: r.get("meta_name"),
                description,
                weights: MetaWeightsInfo {
                    top: r.get("weight_top"),
                    jug: r.get("weight_jug"),
                    mid: r.get("weight_mid"),
                    adc: r.get("weight_adc"),
                    sup: r.get("weight_sup"),
                },
            }
        }
        None => {
            // 没有记录时返回均衡版本
            MetaInfo {
                season_id: current_season,
                meta_type: "Balanced".to_string(),
                meta_name: "均衡版本".to_string(),
                description: MetaType::Balanced.description().to_string(),
                weights: MetaWeightsInfo {
                    top: 1.0, jug: 1.0, mid: 1.0, adc: 1.0, sup: 1.0,
                },
            }
        }
    };

    Ok(CommandResult::ok(info))
}

/// 获取 Meta 历史版本列表
#[tauri::command]
pub async fn get_meta_history(
    state: State<'_, AppState>,
    save_id: String,
) -> Result<CommandResult<Vec<MetaHistoryEntry>>, String> {
    let pool = get_pool!(state);

    let rows = sqlx::query(
        "SELECT season_id, meta_type, meta_name, weight_top, weight_jug, weight_mid, weight_adc, weight_sup FROM meta_versions WHERE save_id = ? ORDER BY season_id ASC"
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let history: Vec<MetaHistoryEntry> = rows.iter().map(|r| MetaHistoryEntry {
        season_id: r.get("season_id"),
        meta_type: r.get("meta_type"),
        meta_name: r.get("meta_name"),
        weight_top: r.get("weight_top"),
        weight_jug: r.get("weight_jug"),
        weight_mid: r.get("weight_mid"),
        weight_adc: r.get("weight_adc"),
        weight_sup: r.get("weight_sup"),
    }).collect();

    Ok(CommandResult::ok(history))
}

/// 获取全部 20 种 Meta 类型配置
#[tauri::command]
pub fn get_all_meta_types() -> CommandResult<Vec<MetaTypeInfo>> {
    let types: Vec<MetaTypeInfo> = MetaType::all().iter().map(|m| {
        let w = get_meta_weights(*m);
        MetaTypeInfo {
            id: m.id().to_string(),
            name: m.display_name().to_string(),
            description: m.description().to_string(),
            weights: MetaWeightsInfo {
                top: w.top,
                jug: w.jug,
                mid: w.mid,
                adc: w.adc,
                sup: w.sup,
            },
        }
    }).collect();

    CommandResult::ok(types)
}

/// 获取指定 Meta 类型的详细信息
#[tauri::command]
pub fn get_meta_detail(meta_type: String) -> CommandResult<MetaTypeInfo> {
    match MetaType::from_id(&meta_type) {
        Some(m) => {
            let w = get_meta_weights(m);
            CommandResult::ok(MetaTypeInfo {
                id: m.id().to_string(),
                name: m.display_name().to_string(),
                description: m.description().to_string(),
                weights: MetaWeightsInfo {
                    top: w.top,
                    jug: w.jug,
                    mid: w.mid,
                    adc: w.adc,
                    sup: w.sup,
                },
            })
        }
        None => CommandResult::err(format!("未知的 Meta 类型: {}", meta_type)),
    }
}

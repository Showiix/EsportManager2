//! Meta 数据模型

use serde::{Deserialize, Serialize};

/// 当前赛季 Meta 信息（前端展示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaInfo {
    pub season_id: i64,
    pub meta_type: String,
    pub meta_name: String,
    pub description: String,
    pub weights: MetaWeightsInfo,
}

/// Meta 权重信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaWeightsInfo {
    pub top: f64,
    pub jug: f64,
    pub mid: f64,
    pub adc: f64,
    pub sup: f64,
}

/// Meta 历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaHistoryEntry {
    pub season_id: i64,
    pub meta_type: String,
    pub meta_name: String,
    pub weight_top: f64,
    pub weight_jug: f64,
    pub weight_mid: f64,
    pub weight_adc: f64,
    pub weight_sup: f64,
}

/// Meta 类型信息（所有 20 种）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaTypeInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub weights: MetaWeightsInfo,
}

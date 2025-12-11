use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::SeasonPhase;

/// 存档数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Save {
    pub id: String,
    pub name: String,
    pub current_season: u32,
    pub current_phase: SeasonPhase,
    pub phase_completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Save {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            current_season: 1,
            current_phase: SeasonPhase::SpringRegular,
            phase_completed: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// 推进到下一阶段
    pub fn advance_phase(&mut self) -> Option<SeasonPhase> {
        if let Some(next_phase) = self.current_phase.next() {
            self.current_phase = next_phase;
            self.phase_completed = false;
            self.updated_at = Utc::now();
            Some(next_phase)
        } else {
            // 赛季结束，进入新赛季
            None
        }
    }

    /// 开始新赛季
    pub fn start_new_season(&mut self) {
        self.current_season += 1;
        self.current_phase = SeasonPhase::SpringRegular;
        self.phase_completed = false;
        self.updated_at = Utc::now();
    }
}

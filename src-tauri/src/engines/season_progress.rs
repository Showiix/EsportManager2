use crate::models::SeasonPhase;
use serde::{Deserialize, Serialize};

/// 游戏操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameAction {
    /// 生成春季赛赛程
    GenerateSpringSchedule,
    /// 模拟春季赛比赛
    SimulateSpringMatch,
    /// 生成季后赛对阵
    GeneratePlayoffs,
    /// 模拟季后赛比赛
    SimulatePlayoffMatch,
    /// 生成MSI对阵
    GenerateMSI,
    /// 模拟MSI比赛
    SimulateMSIMatch,
    /// 生成马德里大师赛对阵
    GenerateMadrid,
    /// 模拟马德里大师赛比赛
    SimulateMadridMatch,
    /// 生成夏季赛赛程
    GenerateSummerSchedule,
    /// 模拟夏季赛比赛
    SimulateSummerMatch,
    /// 生成Claude洲际赛对阵
    GenerateClaude,
    /// 模拟Claude洲际赛比赛
    SimulateClaudeMatch,
    /// 生成世界赛对阵
    GenerateWorlds,
    /// 模拟世界赛比赛
    SimulateWorldsMatch,
    /// 生成上海大师赛对阵
    GenerateShanghai,
    /// 模拟上海大师赛比赛
    SimulateShanghaiMatch,
    /// 生成ICP对阵
    GenerateICP,
    /// 模拟ICP比赛
    SimulateICPMatch,
    /// 生成Super洲际赛对阵
    GenerateSuper,
    /// 模拟Super洲际赛比赛
    SimulateSuperMatch,
    /// 开始转会期
    StartTransferWindow,
    /// 执行转会
    ProcessTransfer,
    /// 结束转会期
    EndTransferWindow,
    /// 开始选秀
    StartDraft,
    /// 执行选秀
    ProcessDraft,
    /// 结束赛季
    EndSeason,
    /// 开始新赛季
    StartNewSeason,
}

/// 时间推进引擎 - 管理赛季阶段和可执行操作
pub struct SeasonProgressEngine {
    current_season: u32,
    current_phase: SeasonPhase,
    phase_completed: bool,
}

impl SeasonProgressEngine {
    pub fn new(current_season: u32, current_phase: SeasonPhase) -> Self {
        Self {
            current_season,
            current_phase,
            phase_completed: false,
        }
    }

    /// 获取当前赛季
    pub fn current_season(&self) -> u32 {
        self.current_season
    }

    /// 获取当前阶段
    pub fn current_phase(&self) -> SeasonPhase {
        self.current_phase
    }

    /// 当前阶段是否已完成
    pub fn is_phase_completed(&self) -> bool {
        self.phase_completed
    }

    /// 标记当前阶段完成
    pub fn mark_phase_completed(&mut self) {
        self.phase_completed = true;
    }

    /// 检查是否可以进入下一阶段
    pub fn can_advance(&self) -> bool {
        self.phase_completed
    }

    /// 推进到下一阶段
    pub fn advance_phase(&mut self) -> Option<SeasonPhase> {
        if !self.phase_completed {
            return None;
        }

        if let Some(next_phase) = self.current_phase.next() {
            self.current_phase = next_phase;
            self.phase_completed = false;
            Some(next_phase)
        } else {
            // 赛季结束
            None
        }
    }

    /// 开始新赛季
    pub fn start_new_season(&mut self) {
        self.current_season += 1;
        self.current_phase = SeasonPhase::SpringRegular;
        self.phase_completed = false;
    }

    /// 获取当前阶段可执行的操作
    pub fn get_available_actions(&self) -> Vec<GameAction> {
        match self.current_phase {
            SeasonPhase::SpringRegular => {
                if self.phase_completed {
                    vec![]
                } else {
                    vec![GameAction::GenerateSpringSchedule, GameAction::SimulateSpringMatch]
                }
            }
            SeasonPhase::SpringPlayoffs => {
                if self.phase_completed {
                    vec![]
                } else {
                    vec![GameAction::GeneratePlayoffs, GameAction::SimulatePlayoffMatch]
                }
            }
            SeasonPhase::Msi => {
                if self.phase_completed {
                    vec![]
                } else {
                    vec![GameAction::GenerateMSI, GameAction::SimulateMSIMatch]
                }
            }
            SeasonPhase::MadridMasters => {
                if self.phase_completed {
                    vec![]
                } else {
                    vec![GameAction::GenerateMadrid, GameAction::SimulateMadridMatch]
                }
            }
            SeasonPhase::SummerRegular => {
                if self.phase_completed {
                    vec![]
                } else {
                    vec![GameAction::GenerateSummerSchedule, GameAction::SimulateSummerMatch]
                }
            }
            SeasonPhase::SummerPlayoffs => {
                if self.phase_completed {
                    vec![]
                } else {
                    vec![GameAction::GeneratePlayoffs, GameAction::SimulatePlayoffMatch]
                }
            }
            SeasonPhase::ClaudeIntercontinental => {
                if self.phase_completed {
                    vec![]
                } else {
                    vec![GameAction::GenerateClaude, GameAction::SimulateClaudeMatch]
                }
            }
            SeasonPhase::WorldChampionship => {
                if self.phase_completed {
                    vec![]
                } else {
                    vec![GameAction::GenerateWorlds, GameAction::SimulateWorldsMatch]
                }
            }
            SeasonPhase::ShanghaiMasters => {
                if self.phase_completed {
                    vec![]
                } else {
                    vec![GameAction::GenerateShanghai, GameAction::SimulateShanghaiMatch]
                }
            }
            SeasonPhase::IcpIntercontinental => {
                if self.phase_completed {
                    vec![]
                } else {
                    vec![GameAction::GenerateICP, GameAction::SimulateICPMatch]
                }
            }
            SeasonPhase::SuperIntercontinental => {
                if self.phase_completed {
                    vec![]
                } else {
                    vec![GameAction::GenerateSuper, GameAction::SimulateSuperMatch]
                }
            }
            SeasonPhase::TransferWindow => {
                if self.phase_completed {
                    vec![]
                } else {
                    vec![
                        GameAction::StartTransferWindow,
                        GameAction::ProcessTransfer,
                        GameAction::EndTransferWindow,
                    ]
                }
            }
            SeasonPhase::Draft => {
                // 检查是否是选秀年
                if SeasonPhase::is_draft_year(self.current_season) {
                    if self.phase_completed {
                        vec![]
                    } else {
                        vec![GameAction::StartDraft, GameAction::ProcessDraft]
                    }
                } else {
                    // 不是选秀年，自动跳过
                    vec![]
                }
            }
            SeasonPhase::SeasonEnd => {
                vec![GameAction::EndSeason, GameAction::StartNewSeason]
            }
        }
    }

    /// 验证操作是否在当前阶段允许
    pub fn validate_action(&self, action: &GameAction) -> bool {
        self.get_available_actions().contains(action)
    }

    /// 获取赛季进度 (已完成阶段数 / 总阶段数)
    pub fn get_progress(&self) -> (u32, u32) {
        let total = 14u32; // 总共14个阶段
        let completed = match self.current_phase {
            SeasonPhase::SpringRegular => 0,
            SeasonPhase::SpringPlayoffs => 1,
            SeasonPhase::Msi => 2,
            SeasonPhase::MadridMasters => 3,
            SeasonPhase::SummerRegular => 4,
            SeasonPhase::SummerPlayoffs => 5,
            SeasonPhase::ClaudeIntercontinental => 6,
            SeasonPhase::WorldChampionship => 7,
            SeasonPhase::ShanghaiMasters => 8,
            SeasonPhase::IcpIntercontinental => 9,
            SeasonPhase::SuperIntercontinental => 10,
            SeasonPhase::TransferWindow => 11,
            SeasonPhase::Draft => 12,
            SeasonPhase::SeasonEnd => 13,
        };

        let completed = if self.phase_completed {
            completed + 1
        } else {
            completed
        };

        (completed.min(total), total)
    }

    /// 获取赛季显示名称
    pub fn season_display_name(&self) -> String {
        format!("S{}", self.current_season)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_progression() {
        let mut engine = SeasonProgressEngine::new(1, SeasonPhase::SpringRegular);

        assert_eq!(engine.current_phase(), SeasonPhase::SpringRegular);
        assert!(!engine.can_advance());

        engine.mark_phase_completed();
        assert!(engine.can_advance());

        let next = engine.advance_phase();
        assert_eq!(next, Some(SeasonPhase::SpringPlayoffs));
        assert_eq!(engine.current_phase(), SeasonPhase::SpringPlayoffs);
    }

    #[test]
    fn test_draft_year() {
        assert!(!SeasonPhase::is_draft_year(1));
        assert!(SeasonPhase::is_draft_year(2));
        assert!(!SeasonPhase::is_draft_year(3));
        assert!(!SeasonPhase::is_draft_year(4));
        assert!(!SeasonPhase::is_draft_year(5));
        assert!(SeasonPhase::is_draft_year(6));
    }

    #[test]
    fn test_progress() {
        let engine = SeasonProgressEngine::new(1, SeasonPhase::Msi);
        let (completed, total) = engine.get_progress();
        assert_eq!(completed, 2);
        assert_eq!(total, 14);
    }
}

mod contract;
mod growth;
mod retirement;
mod season_end;

#[cfg(test)]
mod tests;

/// 事件引擎 - 处理赛季结束时的各种事件
pub struct EventEngine {
    config: EventEngineConfig,
}

/// 事件引擎配置
#[derive(Debug, Clone)]
pub struct EventEngineConfig {
    /// 退役年龄阈值
    pub retirement_age: u8,
    /// 低能力退役阈值
    pub low_ability_threshold: u8,
    /// 年龄+能力组合退役的年龄阈值
    pub age_ability_age_threshold: u8,
    /// 年龄+能力组合退役的能力阈值
    pub age_ability_ability_threshold: u8,
    /// 衰退开始年龄
    pub decline_start_age: u8,
    /// 成长停止年龄
    pub growth_stop_age: u8,
    /// 新秀最小年龄
    pub rookie_min_age: u8,
    /// 新秀最大年龄
    pub rookie_max_age: u8,
    /// 合同续约概率基础值
    pub contract_renewal_base_probability: f64,
}

impl Default for EventEngineConfig {
    fn default() -> Self {
        Self {
            retirement_age: 36,
            low_ability_threshold: 50,
            age_ability_age_threshold: 30,
            age_ability_ability_threshold: 60,
            decline_start_age: 30,
            growth_stop_age: 28,
            rookie_min_age: 17,
            rookie_max_age: 19,
            contract_renewal_base_probability: 0.7,
        }
    }
}

impl Default for EventEngine {
    fn default() -> Self {
        Self {
            config: EventEngineConfig::default(),
        }
    }
}

impl EventEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: EventEngineConfig) -> Self {
        Self { config }
    }
}

// Re-export public types from season_end module
pub use season_end::SeasonEndProcessResult;

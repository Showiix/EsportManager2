use rand::Rng;
use rand_distr::{Distribution, Normal};

/// 选手发挥值计算引擎
#[allow(dead_code)]
pub struct PlayerPerformanceEngine {
    /// 基础能力值
    base_ability: f64,
    /// 年龄
    age: u8,
    /// 稳定性
    stability: f64,
    /// 状态加成
    form_bonus: f64,
}

impl PlayerPerformanceEngine {
    pub fn new(base_ability: u8, age: u8, stability: u8, form_bonus: i8) -> Self {
        Self {
            base_ability: base_ability as f64,
            age,
            stability: stability as f64,
            form_bonus: form_bonus as f64,
        }
    }

    /// 根据年龄计算稳定性
    pub fn calculate_stability_from_age(age: u8) -> u8 {
        match age {
            16..=24 => 60 + (age.saturating_sub(16)) * 2,  // 60-76
            25..=29 => 75 + (age.saturating_sub(25)) * 2,  // 75-85
            30..=36 => 85 + (age.saturating_sub(30)),      // 85-91
            _ => 70,
        }
    }

    /// 根据年龄生成随机状态加成
    pub fn generate_form_bonus(age: u8) -> i8 {
        let mut rng = rand::thread_rng();
        let (min, max) = match age {
            16..=24 => (-3, 8),   // 年轻选手波动大
            25..=29 => (-2, 3),   // 巅峰期稳定
            _ => (0, 2),          // 老将稳定但上限低
        };
        rng.gen_range(min..=max)
    }

    /// 计算标准差 σ = (100 - 稳定性) / 10
    fn calculate_std_dev(&self) -> f64 {
        (100.0 - self.stability) / 10.0
    }

    /// 基于Box-Muller变换生成高斯随机数
    fn gaussian_random(&self, mean: f64, std_dev: f64) -> f64 {
        let normal = Normal::new(mean, std_dev).unwrap();
        let mut rng = rand::thread_rng();
        normal.sample(&mut rng)
    }

    /// 计算单局比赛的实际发挥值
    pub fn calculate_performance(&self) -> f64 {
        let std_dev = self.calculate_std_dev();

        // 生成高斯噪声
        let noise = self.gaussian_random(0.0, std_dev);

        // 计算原始发挥值
        let raw_performance = self.base_ability + self.form_bonus + noise;

        // 钳位处理
        let lower_bound = (self.base_ability - 15.0).max(0.0);
        let upper_bound = (self.base_ability + 10.0).min(100.0);

        raw_performance.clamp(lower_bound, upper_bound)
    }

    /// 计算贡献值 (发挥值 / 5)
    pub fn calculate_contribution(&self) -> f64 {
        self.calculate_performance() / 5.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stability_calculation() {
        assert_eq!(PlayerPerformanceEngine::calculate_stability_from_age(18), 64);
        assert_eq!(PlayerPerformanceEngine::calculate_stability_from_age(24), 76);
        assert_eq!(PlayerPerformanceEngine::calculate_stability_from_age(27), 79);
        assert_eq!(PlayerPerformanceEngine::calculate_stability_from_age(30), 85);
    }

    #[test]
    fn test_performance_range() {
        let engine = PlayerPerformanceEngine::new(80, 25, 80, 0);

        // 运行多次测试，确保结果在合理范围内
        for _ in 0..100 {
            let performance = engine.calculate_performance();
            assert!(performance >= 65.0 && performance <= 90.0);
        }
    }
}

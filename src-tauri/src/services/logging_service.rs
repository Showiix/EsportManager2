//! 日志服务模块
//!
//! 提供统一的日志系统初始化和配置功能。
//! 支持控制台输出、文件滚动、JSON 格式日志。

use std::path::PathBuf;
use std::sync::Once;
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

static INIT: Once = Once::new();

/// 日志配置
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// 日志目录
    pub log_dir: PathBuf,
    /// 最大日志级别
    pub max_level: Level,
    /// 是否启用控制台输出
    pub enable_console: bool,
    /// 是否启用文件输出
    pub enable_file: bool,
    /// 是否使用 JSON 格式
    pub enable_json: bool,
    /// 应用名称（用于日志目标过滤）
    pub app_name: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("logs"),
            max_level: Level::INFO,
            enable_console: true,
            enable_file: true,
            enable_json: true,
            app_name: "esport_manager_2".to_string(),
        }
    }
}

impl LoggingConfig {
    /// 开发模式配置
    pub fn development() -> Self {
        Self {
            max_level: Level::DEBUG,
            enable_console: true,
            enable_file: true,
            enable_json: false, // 开发时不用 JSON，更易读
            ..Default::default()
        }
    }

    /// 生产模式配置
    pub fn production() -> Self {
        Self {
            max_level: Level::INFO,
            enable_console: false,
            enable_file: true,
            enable_json: true,
            ..Default::default()
        }
    }

    /// 设置日志目录
    pub fn with_log_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.log_dir = dir.into();
        self
    }
}

/// 初始化日志系统
///
/// # 参数
/// * `config` - 日志配置
///
/// # 返回
/// * `Ok(())` - 初始化成功
/// * `Err` - 初始化失败
pub fn init_logging(config: LoggingConfig) -> anyhow::Result<()> {
    let mut initialized = false;

    INIT.call_once(|| {
        if let Err(e) = init_logging_inner(&config) {
            eprintln!("日志系统初始化失败: {}", e);
        } else {
            initialized = true;
        }
    });

    if initialized {
        tracing::info!("日志系统初始化完成");
        tracing::info!(log_dir = ?config.log_dir, level = ?config.max_level, "日志配置");
    }

    Ok(())
}

fn init_logging_inner(config: &LoggingConfig) -> anyhow::Result<()> {
    // 确保日志目录存在
    if config.enable_file {
        std::fs::create_dir_all(&config.log_dir)?;
    }

    // 环境变量过滤器
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new(format!(
                "{}={},sqlx=warn,hyper=warn,reqwest=warn",
                config.app_name,
                config.max_level
            ))
        });

    // 构建订阅器
    let subscriber = tracing_subscriber::registry().with(env_filter);

    if config.enable_console && config.enable_file {
        // 控制台 + 文件
        let console_layer = create_console_layer();
        let file_layers = create_file_layers(config)?;

        subscriber
            .with(console_layer)
            .with(file_layers)
            .init();
    } else if config.enable_console {
        // 仅控制台
        let console_layer = create_console_layer();
        subscriber.with(console_layer).init();
    } else if config.enable_file {
        // 仅文件
        let file_layers = create_file_layers(config)?;
        subscriber
            .with(file_layers)
            .init();
    } else {
        // 无输出
        subscriber.init();
    }

    Ok(())
}

/// 创建控制台日志层
fn create_console_layer<S>() -> impl tracing_subscriber::Layer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_level(true)
        .with_ansi(true)
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
}

/// 创建文件日志层（返回组合后的单一层）
fn create_file_layers<S>(
    config: &LoggingConfig,
) -> anyhow::Result<Box<dyn Layer<S> + Send + Sync + 'static>>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a> + Send + Sync,
{
    // 应用日志 (按天滚动)
    let app_file = RollingFileAppender::new(
        Rotation::DAILY,
        &config.log_dir,
        "app.log",
    );

    let file_layer = if config.enable_json {
        fmt::layer()
            .with_writer(app_file)
            .with_target(true)
            .with_ansi(false)
            .json()
            .boxed()
    } else {
        fmt::layer()
            .with_writer(app_file)
            .with_target(true)
            .with_ansi(false)
            .boxed()
    };

    Ok(file_layer)
}

/// 获取日志目录路径
pub fn get_log_dir(app_data_dir: Option<&PathBuf>) -> PathBuf {
    if let Some(dir) = app_data_dir {
        dir.join("logs")
    } else {
        PathBuf::from("logs")
    }
}

/// 清理旧日志文件
///
/// # 参数
/// * `log_dir` - 日志目录
/// * `max_age_days` - 最大保留天数
pub fn cleanup_old_logs(log_dir: &PathBuf, max_age_days: u64) -> anyhow::Result<u32> {
    use std::time::{Duration, SystemTime};

    let max_age = Duration::from_secs(max_age_days * 24 * 60 * 60);
    let now = SystemTime::now();
    let mut deleted_count = 0;

    if !log_dir.exists() {
        return Ok(0);
    }

    for entry in std::fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(age) = now.duration_since(modified) {
                        if age > max_age {
                            if std::fs::remove_file(&path).is_ok() {
                                deleted_count += 1;
                                tracing::info!(file = ?path, "已删除过期日志文件");
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(deleted_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.max_level, Level::INFO);
        assert!(config.enable_console);
        assert!(config.enable_file);
    }

    #[test]
    fn test_development_config() {
        let config = LoggingConfig::development();
        assert_eq!(config.max_level, Level::DEBUG);
        assert!(!config.enable_json);
    }

    #[test]
    fn test_production_config() {
        let config = LoggingConfig::production();
        assert_eq!(config.max_level, Level::INFO);
        assert!(!config.enable_console);
        assert!(config.enable_json);
    }
}

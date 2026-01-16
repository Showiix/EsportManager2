//! LLM AI 服务模块
//!
//! 提供与大语言模型 (OpenAI/Claude) 的集成，用于生成智能转会策略。
//! 支持多种 LLM 提供商，可配置 API Key 和模型。

use crate::models::{
    AITransferStrategy, BudgetAllocation, GMPersonality, Player, PlayerStatus,
    SellCandidate, Team, TeamGMProfile, TransferTarget,
    PlayerTransferStrategy, PreferredTeam, TeamPreferenceReason,
    LLMPlayerStrategyResponse, OfferEvaluation, TeamOffer,
};
use crate::models::player_status::DepartureReason;
use crate::engines::transfer::FreeAgentInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ==================== 配置 ====================

/// LLM 提供商类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LLMProvider {
    OpenAI,
    Claude,
    DeepSeek,
    Qwen,      // 通义千问
    Moonshot,  // 月之暗面/Kimi
    Zhipu,     // 智谱 GLM
}

impl Default for LLMProvider {
    fn default() -> Self {
        LLMProvider::OpenAI
    }
}

/// LLM 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// API 提供商
    pub provider: LLMProvider,
    /// API Key
    pub api_key: String,
    /// 模型名称
    pub model: String,
    /// API Base URL (可选，用于代理)
    pub base_url: Option<String>,
    /// 最大 Token 数
    pub max_tokens: u32,
    /// 温度 (0.0-1.0)
    pub temperature: f32,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: LLMProvider::OpenAI,
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
            base_url: None,
            max_tokens: 8000,  // 增加到 8000 避免截断
            temperature: 0.7,
        }
    }
}

impl LLMConfig {
    /// 创建 OpenAI 配置
    pub fn openai(api_key: String) -> Self {
        Self {
            provider: LLMProvider::OpenAI,
            api_key,
            model: "gpt-4o-mini".to_string(),
            base_url: None,
            max_tokens: 8000,
            temperature: 0.7,
        }
    }

    /// 创建 Claude 配置
    pub fn claude(api_key: String) -> Self {
        Self {
            provider: LLMProvider::Claude,
            api_key,
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: None,
            max_tokens: 8000,
            temperature: 0.7,
        }
    }

    /// 获取 API URL
    pub fn get_api_url(&self) -> String {
        match &self.base_url {
            Some(url) => url.clone(),
            None => match self.provider {
                LLMProvider::OpenAI => "https://api.openai.com/v1/chat/completions".to_string(),
                LLMProvider::Claude => "https://api.anthropic.com/v1/messages".to_string(),
                LLMProvider::DeepSeek => "https://api.deepseek.com/chat/completions".to_string(),
                LLMProvider::Qwen => "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".to_string(),
                LLMProvider::Moonshot => "https://api.moonshot.cn/v1/chat/completions".to_string(),
                LLMProvider::Zhipu => "https://open.bigmodel.cn/api/paas/v4/chat/completions".to_string(),
            },
        }
    }
}

// ==================== 请求/响应结构 ====================

/// OpenAI API 请求格式
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
    temperature: f32,
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// 响应格式配置
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ResponseFormat {
    /// 简单 JSON 模式 (兼容大多数提供商)
    Simple {
        #[serde(rename = "type")]
        format_type: String,
    },
    /// Structured Outputs 模式 (OpenAI gpt-4o 等支持)
    JsonSchema {
        #[serde(rename = "type")]
        format_type: String,
        json_schema: JsonSchemaWrapper,
    },
}

/// JSON Schema 包装器
#[derive(Debug, Serialize)]
struct JsonSchemaWrapper {
    name: String,
    strict: bool,
    schema: serde_json::Value,
}

/// OpenAI API 响应格式
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageContent,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessageContent {
    content: String,
}

/// Claude API 请求格式
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// Claude API 响应格式
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

// ==================== LLM 生成的策略结构 ====================

/// LLM 返回的策略 JSON 结构
#[derive(Debug, Deserialize)]
struct LLMStrategyResponse {
    /// 整体策略名称
    #[serde(default)]
    overall_strategy: String,
    /// 策略描述
    #[serde(default)]
    strategy_description: String,
    /// 决策理由
    #[serde(default)]
    reasoning: String,
    /// 优先补强位置
    #[serde(default)]
    priority_positions: Vec<String>,
    /// 目标签约选手
    #[serde(default)]
    targets: Vec<LLMTransferTarget>,
    /// 愿意出售的选手
    #[serde(default)]
    willing_to_sell: Vec<LLMSellCandidate>,
    /// 预算分配
    #[serde(default)]
    budget_allocation: LLMBudgetAllocation,
    /// AI分析步骤详情
    #[serde(default)]
    analysis_steps: Vec<LLMTeamAnalysisStep>,
}

#[derive(Debug, Deserialize)]
struct LLMTransferTarget {
    #[serde(default)]
    player_id: u64,
    #[serde(default)]
    max_offer: f64,
    #[serde(default)]
    priority: u8,
    #[serde(default)]
    reasoning: String,
}

#[derive(Debug, Deserialize)]
struct LLMSellCandidate {
    #[serde(default)]
    player_id: u64,
    #[serde(default)]
    min_price: f64,
    #[serde(default)]
    urgency: u8,
    #[serde(default)]
    reasoning: String,
}

#[derive(Debug, Deserialize, Default)]
struct LLMBudgetAllocation {
    #[serde(default)]
    transfer_spend: f64,
    #[serde(default)]
    salary_spend: f64,
    #[serde(default)]
    reserve: f64,
}

/// LLM 返回的分析步骤（战队策略）
#[derive(Debug, Deserialize)]
struct LLMTeamAnalysisStep {
    #[serde(default)]
    step_name: String,
    #[serde(default)]
    data_used: String,
    #[serde(default)]
    threshold: String,
    #[serde(default)]
    result: String,
    #[serde(default)]
    impact: String,
}

/// LLM 返回的续约评估响应
#[derive(Debug, Deserialize)]
struct LLMRenewalResponse {
    /// 球队是否想续约
    team_wants_renewal: bool,
    /// 球队不想续约的原因
    team_rejection_reason: Option<String>,
    /// 报价薪资（万/年）
    #[serde(default)]
    offered_salary: u64,
    /// 报价年限
    #[serde(default = "default_contract_years")]
    offered_years: u8,
    /// 选手是否接受
    player_accepts: bool,
    /// 选手拒绝原因
    player_rejection_reason: Option<String>,
    /// 球队分析步骤
    #[serde(default)]
    team_analysis: Vec<LLMRenewalAnalysisStep>,
    /// 选手分析步骤
    #[serde(default)]
    player_analysis: Vec<LLMRenewalAnalysisStep>,
    /// 总结
    summary: String,
}

fn default_contract_years() -> u8 {
    2
}

/// LLM 返回的续约分析步骤
#[derive(Debug, Deserialize)]
struct LLMRenewalAnalysisStep {
    step_name: String,
    data_used: String,
    result: String,
    impact: String,
}

// ==================== 数据上下文 ====================

/// 传递给 LLM 的球队上下文
#[derive(Debug, Serialize)]
struct TeamContext {
    team_name: String,
    balance: i64,
    roster: Vec<PlayerSummary>,
    avg_ability: f64,
    avg_age: f64,
    position_needs: HashMap<String, u32>,
}

/// 传递给 LLM 的选手摘要
#[derive(Debug, Serialize)]
struct PlayerSummary {
    id: u64,
    name: String,
    position: String,
    age: u8,
    ability: u8,
    potential: u8,
    salary: u64,
    market_value: u64,
    is_starter: bool,
}

/// 传递给 LLM 的自由球员摘要
#[derive(Debug, Serialize)]
struct FreeAgentSummary {
    id: u64,
    name: String,
    position: String,
    age: u8,
    ability: u8,
    potential: u8,
    market_value: u64,
    expected_salary: u64,
}

/// 传递给 LLM 的 GM 配置摘要
#[derive(Debug, Serialize)]
struct GMProfileSummary {
    personality: String,
    personality_description: String,
    risk_tolerance: u8,
    budget_ratio: f64,
    preferred_age_min: u8,
    preferred_age_max: u8,
    min_ability_threshold: u8,
    price_premium_max: f64,
    position_priorities: HashMap<String, u8>,
}

// ==================== JSON 清理辅助函数 ====================

/// 清理 LLM 返回的 JSON 字符串，修复常见格式问题
fn clean_json_response(text: &str) -> String {
    let mut result = text.to_string();

    // 0. 先提取 JSON 部分（去掉 markdown 代码块标记）
    // 这必须在截断检测之前，否则 ```json...``` 会被误判为截断
    if let Some(start) = result.find('{') {
        if let Some(end) = result.rfind('}') {
            result = result[start..=end].to_string();
        }
    }

    // 1. 检测截断（响应不以 } 结尾通常意味着被截断）
    let trimmed = result.trim();
    if !trimmed.ends_with('}') && !trimmed.ends_with(']') {
        log::warn!("检测到响应可能被截断！末尾字符: {:?}", trimmed.chars().last());
        // 尝试补全 JSON
        result = try_complete_truncated_json(&result);
    }

    // 2. 统一换行符为 \n
    result = result.replace("\r\n", "\n").replace("\r", "\n");

    // 3. 移除所有控制字符（只保留换行符用于JSON格式化，其他控制字符全部移除）
    // 控制字符 \u0000-\u001F 在 JSON 字符串内部是非法的
    result = result.chars()
        .filter(|c| *c == '\n' || (*c >= '\u{0020}' && *c != '\u{007F}'))
        .collect();

    // 4. 替换中文引号为英文引号
    result = result.replace('"', "\"").replace('"', "\"");
    result = result.replace('\u{2018}', "'").replace('\u{2019}', "'");

    // 5. 修复尾随逗号
    result = fix_trailing_commas(&result);

    // 6. 修复缺少逗号的问题
    result = fix_missing_commas(&result);

    // 7. 先尝试标准 JSON 解析
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&result) {
        if let Ok(cleaned) = serde_json::to_string(&value) {
            return cleaned;
        }
    }

    // 8. 尝试 json5（可以处理尾随逗号、无引号的 key 等）
    if let Ok(value) = json5::from_str::<serde_json::Value>(&result) {
        if let Ok(cleaned) = serde_json::to_string(&value) {
            log::debug!("JSON5 解析成功");
            return cleaned;
        }
    }

    // 9. 最后尝试迭代修复
    let mut attempts = 0;
    while attempts < 5 {
        match serde_json::from_str::<serde_json::Value>(&result) {
            Ok(value) => {
                if let Ok(cleaned) = serde_json::to_string(&value) {
                    return cleaned;
                }
                return result;
            }
            Err(e) => {
                let err_msg = e.to_string();
                if let Some(fixed) = try_fix_at_error_position(&result, &err_msg) {
                    result = fixed;
                    attempts += 1;
                } else {
                    break;
                }
            }
        }
    }

    // 10. 修复后再试一次 json5
    if let Ok(value) = json5::from_str::<serde_json::Value>(&result) {
        if let Ok(cleaned) = serde_json::to_string(&value) {
            return cleaned;
        }
    }

    log::error!("JSON 清理失败，内容: {}", &result.chars().take(500).collect::<String>());
    result
}

/// 修复尾随逗号
fn fix_trailing_commas(text: &str) -> String {
    let mut result = text.to_string();

    // 简单替换
    loop {
        let before = result.clone();
        result = result.replace(",}", "}");
        result = result.replace(",]", "]");
        result = result.replace(", }", "}");
        result = result.replace(", ]", "]");
        result = result.replace(",\n}", "\n}");
        result = result.replace(",\n]", "\n]");
        result = result.replace(",  }", "}");
        result = result.replace(",  ]", "]");

        // 处理多个空白字符
        let re = regex::Regex::new(r",\s*\}").unwrap();
        result = re.replace_all(&result, "}").to_string();
        let re = regex::Regex::new(r",\s*\]").unwrap();
        result = re.replace_all(&result, "]").to_string();

        if result == before {
            break;
        }
    }

    result
}

/// 尝试在错误位置修复 JSON
fn try_fix_at_error_position(json: &str, error_msg: &str) -> Option<String> {
    // 解析错误位置: "expected `,` or `}` at line X column Y" 或 "trailing comma at line X column Y"
    let re = regex::Regex::new(r"at line (\d+) column (\d+)").ok()?;
    let caps = re.captures(error_msg)?;

    let line: usize = caps.get(1)?.as_str().parse().ok()?;
    let col: usize = caps.get(2)?.as_str().parse().ok()?;

    // 找到错误位置（使用字符而非字节）
    let lines: Vec<&str> = json.lines().collect();
    if line == 0 || line > lines.len() || col == 0 {
        return None;
    }

    // 计算字符偏移量
    let mut char_offset = 0;
    for (i, l) in lines.iter().enumerate() {
        if i == line - 1 {
            // 在目标行，加上列偏移（注意列是1-based）
            let line_chars: Vec<char> = l.chars().collect();
            let actual_col = col.saturating_sub(1).min(line_chars.len());
            char_offset += actual_col;
            break;
        }
        char_offset += l.chars().count() + 1; // +1 for newline
    }

    // 转换为字符数组
    let chars: Vec<char> = json.chars().collect();
    if char_offset >= chars.len() {
        return None;
    }

    // 处理 "trailing comma" 错误 - 需要删除多余的逗号
    if error_msg.contains("trailing comma") {
        // 从错误位置向前找逗号并删除
        let mut comma_pos = char_offset;
        // 向前搜索逗号
        while comma_pos > 0 {
            comma_pos -= 1;
            if chars[comma_pos] == ',' {
                // 找到逗号，删除它
                let mut fixed: String = chars[..comma_pos].iter().collect();
                fixed.extend(chars[comma_pos + 1..].iter());
                log::debug!("删除字符位置 {} 处的尾随逗号", comma_pos);
                return Some(fixed);
            }
            // 跳过空白字符继续向前搜索
            if !chars[comma_pos].is_whitespace() && chars[comma_pos] != ']' && chars[comma_pos] != '}' {
                break;
            }
        }
        return None;
    }

    // 向前找到合适的插入点（跳过空白）
    let mut insert_pos = char_offset;
    while insert_pos > 0 && chars.get(insert_pos - 1).map(|c| c.is_whitespace()).unwrap_or(false) {
        insert_pos -= 1;
    }

    // 检查前一个非空白字符
    if insert_pos > 0 {
        let prev_char = chars[insert_pos - 1];

        // 检查是否是值结束字符
        let should_insert_comma = prev_char == '"'
            || prev_char == '}'
            || prev_char == ']'
            || prev_char.is_ascii_digit()
            || prev_char == 'e'  // true/false/null 结尾
            || prev_char == 'l'; // null 结尾

        if should_insert_comma {
            // 重建字符串，在 insert_pos 位置插入逗号
            let mut fixed: String = chars[..insert_pos].iter().collect();
            fixed.push(',');
            fixed.extend(chars[insert_pos..].iter());
            log::debug!("在字符位置 {} 插入逗号 (前一字符: '{}')", insert_pos, prev_char);
            return Some(fixed);
        }
    }

    None
}

/// 修复缺少逗号的 JSON 问题
fn fix_missing_commas(text: &str) -> String {
    use regex::Regex;

    let mut result = text.to_string();

    // 修复: "string"\n"key" 或 "string"\n    "key"
    // 匹配: 引号结尾 + 换行 + 可选空白 + 引号开头
    if let Ok(re) = Regex::new(r#""\s*\n(\s*)"([^"]*)"(\s*):"#) {
        result = re.replace_all(&result, "\",\n$1\"$2\"$3:").to_string();
    }

    // 修复: number\n"key"
    // 匹配: 数字 + 换行 + 可选空白 + 引号开头
    if let Ok(re) = Regex::new(r#"(\d)\s*\n(\s*)"([^"]*)"(\s*):"#) {
        result = re.replace_all(&result, "$1,\n$2\"$3\"$4:").to_string();
    }

    // 修复: true/false/null\n"key"
    if let Ok(re) = Regex::new(r#"(true|false|null)\s*\n(\s*)"([^"]*)"(\s*):"#) {
        result = re.replace_all(&result, "$1,\n$2\"$3\"$4:").to_string();
    }

    // 修复: }\n"key"
    if let Ok(re) = Regex::new(r#"\}\s*\n(\s*)"([^"]*)"(\s*):"#) {
        result = re.replace_all(&result, "},\n$1\"$2\"$3:").to_string();
    }

    // 修复: ]\n"key"
    if let Ok(re) = Regex::new(r#"\]\s*\n(\s*)"([^"]*)"(\s*):"#) {
        result = re.replace_all(&result, "],\n$1\"$2\"$3:").to_string();
    }

    // 修复: }\n] (对象结束后数组结束)
    if let Ok(re) = Regex::new(r#"\}\s*\n(\s*)\]"#) {
        result = re.replace_all(&result, "}\n$1]").to_string();
    }

    // 修复: "string"\n}
    if let Ok(re) = Regex::new(r#""\s*\n(\s*)\}"#) {
        result = re.replace_all(&result, "\"\n$1}").to_string();
    }

    result
}

/// 尝试补全被截断的 JSON
fn try_complete_truncated_json(text: &str) -> String {
    let mut result = text.to_string();

    // 统计未闭合的括号
    let mut brace_count = 0;  // {}
    let mut bracket_count = 0; // []
    let mut in_string = false;
    let mut escape_next = false;

    for c in result.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match c {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => brace_count += 1,
            '}' if !in_string => brace_count -= 1,
            '[' if !in_string => bracket_count += 1,
            ']' if !in_string => bracket_count -= 1,
            _ => {}
        }
    }

    // 如果在字符串中被截断，先关闭字符串
    if in_string {
        result.push('"');
        log::debug!("补全未闭合的字符串");
    }

    // 添加缺失的闭合括号
    for _ in 0..bracket_count {
        result.push(']');
        log::debug!("补全 ]");
    }
    for _ in 0..brace_count {
        result.push('}');
        log::debug!("补全 }}");
    }

    result
}

// ==================== LLM 服务 ====================

/// LLM 转会策略服务
pub struct LLMTransferService {
    config: LLMConfig,
    client: reqwest::Client,
}

impl LLMTransferService {
    /// 创建新的 LLM 服务
    pub fn new(config: LLMConfig) -> Self {
        // 创建带超时的 HTTP 客户端
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))  // 60秒超时
            .connect_timeout(std::time::Duration::from_secs(10))  // 10秒连接超时
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            config,
            client,
        }
    }

    /// 检查 API Key 是否已配置
    pub fn is_configured(&self) -> bool {
        !self.config.api_key.is_empty()
    }

    /// 检查模型是否支持 Structured Outputs (JSON Schema)
    fn supports_structured_outputs(&self) -> bool {
        // OpenAI gpt-4o 系列和 gpt-4o-mini 支持 Structured Outputs
        // DeepSeek 也支持 json_schema 模式
        let model = self.config.model.to_lowercase();
        model.contains("gpt-4o") ||
        model.contains("gpt-4-turbo") ||
        model.contains("deepseek")
    }

    /// 获取响应格式配置
    fn get_response_format(&self) -> ResponseFormat {
        if self.supports_structured_outputs() {
            // 使用 Structured Outputs 强制格式
            ResponseFormat::JsonSchema {
                format_type: "json_schema".to_string(),
                json_schema: JsonSchemaWrapper {
                    name: "transfer_strategy".to_string(),
                    strict: true,
                    schema: self.get_team_strategy_schema(),
                },
            }
        } else {
            // 回退到简单 JSON 模式
            ResponseFormat::Simple {
                format_type: "json_object".to_string(),
            }
        }
    }

    /// 获取球队策略的 JSON Schema
    fn get_team_strategy_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "overall_strategy": { "type": "string", "description": "策略名称" },
                "strategy_description": { "type": "string", "description": "策略描述" },
                "reasoning": { "type": "string", "description": "决策理由" },
                "priority_positions": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "优先补强位置"
                },
                "targets": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "player_id": { "type": "integer" },
                            "max_offer": { "type": "number" },
                            "priority": { "type": "integer" },
                            "reasoning": { "type": "string" }
                        },
                        "required": ["player_id", "max_offer", "priority", "reasoning"],
                        "additionalProperties": false
                    }
                },
                "willing_to_sell": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "player_id": { "type": "integer" },
                            "min_price": { "type": "number" },
                            "urgency": { "type": "integer" },
                            "reasoning": { "type": "string" }
                        },
                        "required": ["player_id", "min_price", "urgency", "reasoning"],
                        "additionalProperties": false
                    }
                },
                "budget_allocation": {
                    "type": "object",
                    "properties": {
                        "transfer_spend": { "type": "number" },
                        "salary_spend": { "type": "number" },
                        "reserve": { "type": "number" }
                    },
                    "required": ["transfer_spend", "salary_spend", "reserve"],
                    "additionalProperties": false
                },
                "analysis_steps": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "step_name": { "type": "string" },
                            "data_used": { "type": "string" },
                            "threshold": { "type": "string" },
                            "result": { "type": "string" },
                            "impact": { "type": "string" }
                        },
                        "required": ["step_name", "data_used", "result", "impact"],
                        "additionalProperties": false
                    }
                }
            },
            "required": ["overall_strategy", "strategy_description", "reasoning", "priority_positions", "targets", "willing_to_sell", "budget_allocation", "analysis_steps"],
            "additionalProperties": false
        })
    }

    /// 生成转会策略
    pub async fn generate_strategy(
        &self,
        team: &Team,
        roster: &[Player],
        gm_profile: &TeamGMProfile,
        free_agents: &[FreeAgentInfo],
        all_players_by_team: &HashMap<u64, Vec<Player>>,
        save_id: &str,
        season_id: u64,
        team_honors: Option<&TeamHonorInfo>,
        roster_honors: Option<&[RosterPlayerHonorSummary]>,
        roster_performance: Option<&[PlayerPerformanceSummary]>,
    ) -> Result<AITransferStrategy, String> {
        log::info!("开始为球队 {} (ID: {}) 生成 LLM 策略", team.name, team.id);

        if !self.is_configured() {
            return Err("LLM API Key 未配置".to_string());
        }

        // 1. 构建上下文数据
        log::debug!("构建球队上下文数据...");
        let team_context = self.build_team_context(team, roster);
        let gm_summary = self.build_gm_summary(gm_profile);
        let fa_summaries = self.build_free_agent_summaries(free_agents, 20); // 只取前20个
        let roster_summaries: Vec<PlayerSummary> = roster
            .iter()
            .filter(|p| p.status == PlayerStatus::Active)
            .map(|p| self.build_player_summary(p))
            .collect();

        // 构建其他队伍可挖角选手列表（按能力值排序，取前30名）
        let mut other_teams_players: Vec<&Player> = all_players_by_team
            .iter()
            .filter(|(tid, _)| **tid != team.id)  // 排除自己队伍
            .flat_map(|(_, players)| players.iter())
            .filter(|p| p.status == PlayerStatus::Active && p.ability >= 75)  // 只看能力75+的
            .collect();
        other_teams_players.sort_by(|a, b| b.ability.cmp(&a.ability));  // 按能力降序
        let other_teams_summaries: Vec<PlayerSummary> = other_teams_players
            .into_iter()
            .take(30)  // 取前30个
            .map(|p| self.build_player_summary(p))
            .collect();

        // 2. 构建 Prompt
        log::debug!("构建 Prompt...");
        let system_prompt = self.build_system_prompt();
        let user_prompt = self.build_user_prompt(
            &team_context,
            &gm_summary,
            &roster_summaries,
            &fa_summaries,
            &other_teams_summaries,
            team_honors,
            roster_honors,
            roster_performance,
        );

        // 3. 调用 LLM API (DeepSeek/Qwen/Moonshot/Zhipu 兼容 OpenAI 格式)
        log::info!("调用 LLM API ({:?})...", self.config.provider);
        let response_text = match self.config.provider {
            LLMProvider::Claude => self.call_claude(&system_prompt, &user_prompt).await?,
            _ => self.call_openai_compatible(&system_prompt, &user_prompt).await?,
        };
        log::info!("LLM API 返回成功，响应长度: {} 字符", response_text.len());

        // 4. 解析响应
        log::info!("解析 LLM 响应...");
        let cleaned_response = clean_json_response(&response_text);
        let llm_strategy: LLMStrategyResponse = match serde_json::from_str(&cleaned_response) {
            Ok(s) => {
                log::info!("LLM 响应解析成功");
                s
            }
            Err(e) => {
                log::error!("解析 LLM 响应失败: {}", e);
                log::error!("原始响应: {}", response_text);
                return Err(format!("解析 LLM 响应失败: {}", e));
            }
        };

        // 5. 转换为 AITransferStrategy
        let strategy = self.convert_to_strategy(
            llm_strategy,
            team,
            roster,
            free_agents,
            all_players_by_team,
            gm_profile,
            save_id,
            season_id,
        );

        Ok(strategy)
    }

    /// 构建 System Prompt
    fn build_system_prompt(&self) -> String {
        r#"你是一个电竞经理游戏的AI转会分析师。你需要根据球队状态和GM人格，制定合理的转会策略。

**重要：你必须只输出一个有效的JSON对象，不要输出任何其他内容（包括markdown代码块、解释、注释等）。**

## 一、阵容规则
- 每支球队需要5-10名选手
- 5个位置：TOP(上单)、JUG(打野)、MID(中单)、ADC(射手)、SUP(辅助)
- 每个位置至少需要1名选手，建议每位置2人（首发+替补）

## 二、选手属性系统

### 核心属性
| 属性 | 范围 | 说明 |
|------|------|------|
| ability | 0-100 | 基础能力值，决定选手实力上限 |
| potential | 0-100 | 潜力值，决定成长空间（ability最高可成长到potential） |
| stability | 0-100 | 稳定性，影响比赛发挥波动 |
| condition | -10~+10 | 状态加成，受年龄、动能、压力等影响 |
| age | 16-36 | 年龄，影响多个属性和身价 |
| satisfaction | 0-100 | 满意度，影响转会意愿 |
| loyalty | 0-100 | 忠诚度，影响续约和被挖走难度 |

### 能力值分档参考
- 95-100：世界顶级（S级明星）
- 90-94：世界级（国际赛主力）
- 85-89：顶尖（赛区顶级首发）
- 80-84：优秀（强队首发）
- 75-79：合格首发（中游队伍）
- 70-74：替补级
- <70：青训/边缘选手

### 年龄对选手的影响
| 年龄段 | 特点 | 稳定性 | 状态范围 |
|--------|------|--------|----------|
| 17-19岁 | 超新星，波动大，成长快 | 60-70 | -5~+8 |
| 20-22岁 | 年轻潜力股 | 70-76 | -5~+8 |
| 23-25岁 | 黄金年龄，巅峰期 | 75-80 | -3~+3 |
| 26-27岁 | 巅峰末期 | 80-85 | -3~+3 |
| 28-29岁 | 开始下滑 | 85-88 | 0~+2 |
| 30+岁 | 老将，稳定但上限低 | 88-91 | 0~+2 |

## 三、比赛模拟系统

### 选手实际发挥计算
```
实际发挥 = ability + condition + 高斯噪声(σ = (100-stability)/10)
发挥范围：[ability-15, ability+10]
```

### 队伍战力
- 队伍战力 = 5名首发选手实际发挥的平均值
- 比赛胜负基于正态分布随机模拟

### Condition（状态）影响因素
1. **状态周期波动**：年轻选手波动±6，老将波动±2
2. **动能(momentum)**：连胜+1，连败-1，范围-5~+5
3. **信心因子**：上场发挥好→下场更自信
4. **大赛压力**：MSI/Worlds -1.5，决赛-1.0，决胜局-0.5

## 四、身价计算系统

### 基础身价 = ability × 基础系数
| 能力值 | 基础系数 | 身价范围 |
|--------|----------|----------|
| 95-100 | 50万/点 | 4750-5000万 |
| 90-94 | 35万/点 | 3150-3290万 |
| 85-89 | 20万/点 | 1700-1780万 |
| 80-84 | 12万/点 | 960-1008万 |
| 75-79 | 7万/点 | 525-553万 |
| 70-74 | 4万/点 | 280-296万 |

### 完整身价 = 基础身价 × 年龄系数 × 位置系数 × 赛区系数 × 荣誉系数

**年龄系数**：17-19岁×1.5，20-22岁×1.3，23-25岁×1.0，26-27岁×0.85，28-29岁×0.7，30+×0.5

**位置系数**：Mid×1.2，Adc×1.15，Jug×1.1，Top×1.0，Sup×0.9

**赛区系数**：LPL×1.3，LCK×1.2，LEC×1.0，LCS×0.9

**荣誉系数**：1.0~3.0，根据冠军和MVP数量

## 五、转会窗口7轮流程

| 轮次 | 名称 | 内容 |
|------|------|------|
| 第0轮 | 赛季结算 | 计算选手满意度，评估赛季表现 |
| 第1轮 | 合同与退役 | 处理到期合同和退役选手 |
| 第2轮 | 意愿处理 | 处理转会申请、续约谈判 |
| 第3轮 | 自由球员签约 | 签约自由市场的选手 |
| 第4轮 | 重建清洗 | 重建型球队清理高薪老将 |
| 第5轮 | 财政清洗 | 财务困难球队被迫出售选手 |
| 第6轮 | 强队补强 | 实力球队引进明星选手 |
| 第7轮 | 收尾 | 处理剩余交易，确保阵容完整 |

## 六、满意度与忠诚度系统

### 满意度影响因素
- 战队成绩（冠军+20，降级-30）
- 薪资水平（高于市场+10，低于-15）
- 出场时间（替补-20，首发+5）
- 队内地位（核心+10）

### 忠诚度类型
| 类型 | 忠诚度 | 特点 |
|------|--------|------|
| 忠心耿耿 | >90 | 几乎不会离队 |
| 忠诚 | 70-90 | 愿意降薪续约 |
| 中立 | 40-70 | 正常市场行为 |
| 机会主义 | 20-40 | 容易被高薪挖走 |
| 雇佣兵 | <20 | 只看钱，随时可走 |

### 转会意愿
- 满意度<40 且 忠诚度<50 → 想离队
- 满意度<30 → 强烈想离队，可能公开申请转会

## 七、荣誉系统

### 赛事荣誉
- 战队：冠军、亚军、季军、殿军、常规赛第一
- 选手：赛事MVP、决赛MVP、常规赛MVP、季后赛FMVP

### 年度颁奖
- 年度MVP：基于 yearly_top_score = avg_impact×0.7 + champion_bonus×0.3
- 年度Top20：按yearly_top_score排名
- 年度最佳位置：各位置的最佳选手
- 年度最佳新秀：22岁以下表现最好的选手

### 荣誉对身价影响
- 国际赛冠军（MSI/Worlds）：身价+50%
- 赛区冠军：身价+20%
- MVP：身价+10-30%

## 八、GM人格类型

1. **争冠型 (Championship)**
   - 追求顶级选手（85+能力）
   - 愿意高价签人（溢价30%）
   - 偏好黄金年龄（23-27岁）

2. **青训型 (YouthDevelopment)**
   - 培养年轻选手（<22岁）
   - 看重潜力（potential>ability+10）
   - 低预算运营

3. **稳健型 (Balanced)**
   - 平衡发展，控制成本
   - 追求性价比
   - 不冒险高价签人

4. **投机型 (Speculator)**
   - 买低卖高，赚取差价
   - 寻找被低估球员
   - 短期合同为主

5. **重建型 (Rebuilding)**
   - 清洗高薪老将
   - 为年轻人让路
   - 积累选秀资产

## 九、特性系统（影响比赛表现）

| 特性 | 效果 |
|------|------|
| Clutch（大赛型） | 季后赛/国际赛 condition +3 |
| SlowStarter（慢热型） | 第1局-2，第3+局+2 |
| Consistent（稳定型） | stability +10 |
| Explosive（爆发型） | stability -15，上限+5 |
| ComebackKing（逆风王） | 落后时 +3 |
| Tilter（顺风浪） | 落后时 -3 |

## 输出要求

请严格按照以下JSON格式返回策略，**必须包含analysis_steps字段**，展示你的分析思考过程：

```json
{
  "overall_strategy": "策略名称",
  "strategy_description": "策略描述",
  "reasoning": "决策理由（结合GM人格、球队状况、市场分析）",
  "analysis_steps": [
    {
      "step_name": "阵容评估",
      "data_used": "平均能力85.2, 平均年龄24.1, 阵容人数8人",
      "threshold": "争冠型目标能力阈值: 85",
      "result": "当前阵容竞争力符合争冠要求",
      "impact": "维持现有核心，针对性补强"
    },
    {
      "step_name": "位置分析",
      "data_used": "TOP位置选手1人(能力82), MID位置选手2人(能力90,85)",
      "threshold": "每位置建议2人，首发能力应达85+",
      "result": "TOP位置需要补强，缺乏替补深度",
      "impact": "优先级: TOP > SUP > ADC"
    },
    {
      "step_name": "财务评估",
      "data_used": "账户余额3500万, 薪资支出1200万/年",
      "threshold": "争冠型预算比例80%",
      "result": "可用转会预算2800万",
      "impact": "有充足资金引进1-2名顶级选手"
    },
    {
      "step_name": "荣誉与表现分析",
      "data_used": "球队荣誉: 赛区冠军x2, 核心选手表现: 平均88分(优秀)",
      "threshold": "争冠球队应有冠军经历",
      "result": "球队有争冠底蕴，核心表现稳定",
      "impact": "保留核心阵容，补充位置短板"
    },
    {
      "step_name": "最终决策",
      "data_used": "综合阵容、位置、财务、荣誉分析",
      "threshold": "",
      "result": "制定针对性补强策略，重点引进TOP位置选手",
      "impact": "目标: 自由市场签约1名TOP + 考虑挖角其他队伍ADC"
    }
  ],
  "priority_positions": ["需要补强的位置"],
  "targets": [
    {
      "player_id": 选手ID,
      "max_offer": 最高出价(万元),
      "priority": 优先级(1-10),
      "reasoning": "签约理由"
    }
  ],
  "willing_to_sell": [
    {
      "player_id": 选手ID,
      "min_price": 最低接受价(万元),
      "urgency": 紧迫度(1-10),
      "reasoning": "出售理由"
    }
  ],
  "budget_allocation": {
    "transfer_spend": 转会费预算(万元),
    "salary_spend": 薪资预算(万元/年),
    "reserve": 预留资金(万元)
  }
}
```

### analysis_steps 必填说明
analysis_steps 是你分析球队时的思考过程，**必须按顺序包含以下分析步骤**：

1. **阵容评估**
   - 分析球队平均能力、年龄结构、阵容深度
   - 判断当前竞争力水平

2. **位置分析**
   - 分析各位置选手配置情况
   - 找出需要补强的位置

3. **财务评估**
   - 分析球队财务状况和可用预算
   - 判断转会窗口可操作空间

4. **荣誉与表现分析**（如有数据）
   - 分析球队荣誉记录和核心选手表现
   - 评估球队发展潜力

5. **最终决策**
   - 综合以上所有因素
   - 给出具体的转会策略和目标

每一步需要包含：
- step_name: 分析步骤名称
- data_used: 你分析时使用的具体数据
- threshold: 判断时使用的阈值或规则（基于GM人格）
- result: 这一步的分析结论
- impact: 这一步对最终决策的影响

注意：
1. targets（目标签约选手）可以从【自由球员市场】或【其他球队】中选择，但绝对不能选择自己队伍的现有选手！
2. willing_to_sell（愿意出售）只能选择自己队伍的现有选手
3. player_id 必须是提供的选手列表中的真实ID
4. 价格单位是万元
5. 策略要符合GM人格特点
6. 考虑选手年龄、潜力、身价性价比
7. 考虑球队财务状况和阵容需求
8. **analysis_steps 必须包含至少5个分析步骤**

**再次强调：直接输出JSON对象，第一个字符必须是 { ，最后一个字符必须是 } ，不要包含任何其他内容！**"#.to_string()
    }

    /// 构建 User Prompt
    fn build_user_prompt(
        &self,
        team: &TeamContext,
        gm: &GMProfileSummary,
        roster: &[PlayerSummary],
        free_agents: &[FreeAgentSummary],
        other_teams_players: &[PlayerSummary],
        team_honors: Option<&TeamHonorInfo>,
        roster_honors: Option<&[RosterPlayerHonorSummary]>,
        roster_performance: Option<&[PlayerPerformanceSummary]>,
    ) -> String {
        let roster_json = serde_json::to_string_pretty(roster).unwrap_or_default();
        let fa_json = serde_json::to_string_pretty(free_agents).unwrap_or_default();
        let other_teams_json = serde_json::to_string_pretty(other_teams_players).unwrap_or_default();
        let position_needs_json = serde_json::to_string_pretty(&team.position_needs).unwrap_or_default();

        // 构建荣誉信息字符串
        let honors_section = if let Some(honors) = team_honors {
            let mut parts = Vec::new();

            if honors.worlds_championships > 0 {
                parts.push(format!("世界赛冠军 x{}", honors.worlds_championships));
            }
            if honors.msi_championships > 0 {
                parts.push(format!("MSI冠军 x{}", honors.msi_championships));
            }
            if honors.regional_championships > 0 {
                parts.push(format!("赛区冠军 x{}", honors.regional_championships));
            }

            let honor_str = if parts.is_empty() {
                "暂无重大荣誉".to_string()
            } else {
                parts.join("、")
            };

            let defending_str = if honors.is_defending_champion {
                " (卫冕冠军)"
            } else {
                ""
            };

            let results_str = if !honors.recent_results.is_empty() {
                format!("\n最近成绩: {}", honors.recent_results.join("、"))
            } else {
                String::new()
            };

            let star_str = if honors.has_star_players {
                format!("\n明星选手: {}名（能力90+）", honors.star_player_count)
            } else {
                String::new()
            };

            format!(
                r#"
### 球队荣誉
荣誉记录: {}{}{}{}
"#,
                honor_str, defending_str, results_str, star_str
            )
        } else {
            String::new()
        };

        // 构建选手荣誉摘要
        let roster_honors_section = if let Some(honors) = roster_honors {
            let honor_lines: Vec<String> = honors.iter()
                .filter(|h| h.championship_count > 0 || h.mvp_count > 0)
                .map(|h| {
                    format!(
                        "- {} ({}): {}",
                        h.player_name,
                        h.position,
                        h.honor_summary
                    )
                })
                .collect();

            if honor_lines.is_empty() {
                String::new()
            } else {
                format!(
                    r#"
### 核心选手荣誉
{}
"#,
                    honor_lines.join("\n")
                )
            }
        } else {
            String::new()
        };

        // 构建选手赛季表现摘要
        let performance_section = if let Some(performance) = roster_performance {
            let perf_lines: Vec<String> = performance.iter()
                .filter(|p| p.games_played > 0)
                .map(|p| {
                    let ability_diff_str = if p.ability_diff > 0.0 {
                        format!("+{:.1}", p.ability_diff)
                    } else {
                        format!("{:.1}", p.ability_diff)
                    };
                    format!(
                        "- {} ({}): 出场{}场, 平均表现{:.1}, 稳定性{:.0}, 等级:{}, 与能力值差异{}",
                        p.player_name,
                        p.position,
                        p.games_played,
                        p.avg_performance,
                        p.consistency_score,
                        p.performance_tier,
                        ability_diff_str
                    )
                })
                .collect();

            if perf_lines.is_empty() {
                String::new()
            } else {
                format!(
                    r#"
### 本赛季选手表现
{}
"#,
                    perf_lines.join("\n")
                )
            }
        } else {
            String::new()
        };

        format!(
            r#"## 球队状态

球队名称: {}
账户余额: {}万元
平均能力: {:.1}
平均年龄: {:.1}
{}{}{}
### GM人格配置
- 人格类型: {} ({})
- 风险偏好: {}/100
- 预算比例: {:.0}%
- 偏好年龄: {}-{}岁
- 能力阈值: {}+
- 溢价容忍: {:.0}%
- 位置优先级: {:?}

### 当前阵容（不能作为签约目标）
{}

### 位置需求评估 (0-100, 越高越需要)
{}

### 自由球员市场 (Top 20)
{}

### 其他球队可挖角选手 (Top 30)
{}

请根据以上信息，为这支球队制定转会策略。注意：targets只能从【自由球员市场】和【其他球队可挖角选手】中选择！"#,
            team.team_name,
            team.balance / 10000,
            team.avg_ability,
            team.avg_age,
            honors_section,
            roster_honors_section,
            performance_section,
            gm.personality,
            gm.personality_description,
            gm.risk_tolerance,
            gm.budget_ratio * 100.0,
            gm.preferred_age_min,
            gm.preferred_age_max,
            gm.min_ability_threshold,
            gm.price_premium_max * 100.0,
            gm.position_priorities,
            roster_json,
            position_needs_json,
            fa_json,
            other_teams_json
        )
    }

    /// 调用 OpenAI 兼容 API (支持 OpenAI, DeepSeek, Qwen, Moonshot, Zhipu)
    /// 带有自动重试机制，最多重试3次
    async fn call_openai_compatible(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        let api_url = self.config.get_api_url();
        log::info!("发送请求到: {} (模型: {})", api_url, self.config.model);

        // 使用简单 JSON 模式（大多数 LLM 都支持）
        let response_format = Some(ResponseFormat::Simple {
            format_type: "json_object".to_string(),
        });

        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            response_format, // 使用简单 JSON 模式
        };

        log::debug!("请求 Token 数估算: system={}, user={}", system_prompt.len() / 4, user_prompt.len() / 4);

        // 重试逻辑：最多重试3次，每次等待时间递增
        let max_retries = 3;
        let mut last_error = String::new();

        for attempt in 0..max_retries {
            if attempt > 0 {
                // 指数退避：1秒, 2秒, 4秒
                let wait_secs = 1u64 << attempt;
                log::warn!("第 {} 次重试，等待 {} 秒...", attempt, wait_secs);
                tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
            }

            let start = std::time::Instant::now();
            let response = self
                .client
                .post(&api_url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let elapsed = start.elapsed();
                    log::info!("LLM API 响应耗时: {:.2}秒, 状态码: {}", elapsed.as_secs_f64(), resp.status());

                    if !resp.status().is_success() {
                        let status = resp.status();
                        let error_text = resp.text().await.unwrap_or_default();

                        // 如果是 429 (Rate Limit) 或 5xx 错误，重试
                        if status.as_u16() == 429 || status.is_server_error() {
                            last_error = format!("LLM API 错误 {}: {}", status, error_text);
                            log::warn!("{}", last_error);
                            continue;
                        }

                        log::error!("LLM API 错误 {}: {}", status, error_text);
                        return Err(format!("LLM API 错误 {}: {}", status, error_text));
                    }

                    let api_response: OpenAIResponse = resp
                        .json()
                        .await
                        .map_err(|e| {
                            log::error!("解析 LLM 响应失败: {}", e);
                            format!("解析 LLM 响应失败: {}", e)
                        })?;

                    return api_response
                        .choices
                        .first()
                        .map(|c| c.message.content.clone())
                        .ok_or_else(|| "LLM 返回空响应".to_string());
                }
                Err(e) => {
                    last_error = format!("LLM API 请求失败: {}", e);
                    log::warn!("请求失败 (尝试 {}/{}): {}", attempt + 1, max_retries, e);
                    // 网络错误，继续重试
                    continue;
                }
            }
        }

        log::error!("LLM API 请求在 {} 次重试后仍然失败: {}", max_retries, last_error);
        Err(last_error)
    }

    /// 调用 Claude API
    /// 带有自动重试机制，最多重试3次
    async fn call_claude(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        let request = ClaudeRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            system: system_prompt.to_string(),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            }],
        };

        // 重试逻辑：最多重试3次，每次等待时间递增
        let max_retries = 3;
        let mut last_error = String::new();

        for attempt in 0..max_retries {
            if attempt > 0 {
                let wait_secs = 1u64 << attempt;
                log::warn!("Claude API 第 {} 次重试，等待 {} 秒...", attempt, wait_secs);
                tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
            }

            let response = self
                .client
                .post(&self.config.get_api_url())
                .header("x-api-key", &self.config.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        let status = resp.status();
                        let error_text = resp.text().await.unwrap_or_default();

                        // 如果是 429 (Rate Limit) 或 5xx 错误，重试
                        if status.as_u16() == 429 || status.is_server_error() {
                            last_error = format!("Claude API 错误 {}: {}", status, error_text);
                            log::warn!("{}", last_error);
                            continue;
                        }

                        return Err(format!("Claude API 错误 {}: {}", status, error_text));
                    }

                    let api_response: ClaudeResponse = resp
                        .json()
                        .await
                        .map_err(|e| format!("解析 Claude 响应失败: {}", e))?;

                    return api_response
                        .content
                        .first()
                        .map(|c| c.text.clone())
                        .ok_or_else(|| "Claude 返回空响应".to_string());
                }
                Err(e) => {
                    last_error = format!("Claude API 请求失败: {}", e);
                    log::warn!("Claude 请求失败 (尝试 {}/{}): {}", attempt + 1, max_retries, e);
                    continue;
                }
            }
        }

        log::error!("Claude API 请求在 {} 次重试后仍然失败: {}", max_retries, last_error);
        Err(last_error)
    }

    /// 构建球队上下文
    fn build_team_context(&self, team: &Team, roster: &[Player]) -> TeamContext {
        let active_roster: Vec<_> = roster
            .iter()
            .filter(|p| p.status == PlayerStatus::Active)
            .collect();

        let (avg_ability, avg_age) = if !active_roster.is_empty() {
            let total_ability: u32 = active_roster.iter().map(|p| p.ability as u32).sum();
            let total_age: u32 = active_roster.iter().map(|p| p.age as u32).sum();
            (
                total_ability as f64 / active_roster.len() as f64,
                total_age as f64 / active_roster.len() as f64,
            )
        } else {
            (0.0, 0.0)
        };

        let mut position_needs = HashMap::new();
        for pos in &["TOP", "JUG", "MID", "ADC", "SUP"] {
            let count = active_roster
                .iter()
                .filter(|p| {
                    p.position
                        .map(|pp| format!("{:?}", pp).to_uppercase() == *pos)
                        .unwrap_or(false)
                })
                .count();
            let need = match count {
                0 => 100,
                1 => 60,
                2 => 30,
                _ => 10,
            };
            position_needs.insert(pos.to_string(), need);
        }

        TeamContext {
            team_name: team.name.clone(),
            balance: team.balance,
            roster: active_roster
                .iter()
                .map(|p| self.build_player_summary(p))
                .collect(),
            avg_ability,
            avg_age,
            position_needs,
        }
    }

    /// 构建 GM 配置摘要
    fn build_gm_summary(&self, gm: &TeamGMProfile) -> GMProfileSummary {
        GMProfileSummary {
            personality: gm.personality.name().to_string(),
            personality_description: gm.personality.description().to_string(),
            risk_tolerance: gm.risk_tolerance,
            budget_ratio: gm.budget_ratio,
            preferred_age_min: gm.preferred_age_min,
            preferred_age_max: gm.preferred_age_max,
            min_ability_threshold: gm.min_ability_threshold,
            price_premium_max: gm.price_premium_max,
            position_priorities: gm.position_priorities.clone(),
        }
    }

    /// 构建选手摘要
    fn build_player_summary(&self, player: &Player) -> PlayerSummary {
        PlayerSummary {
            id: player.id,
            name: player.game_id.clone(),
            position: player
                .position
                .map(|p| format!("{:?}", p).to_uppercase())
                .unwrap_or_else(|| "UNKNOWN".to_string()),
            age: player.age,
            ability: player.ability,
            potential: player.potential,
            salary: player.salary / 10000,  // 转换为万
            market_value: player.calculate_market_value() / 10000,  // 转换为万
            is_starter: player.is_starter,
        }
    }

    /// 构建自由球员摘要列表
    fn build_free_agent_summaries(&self, free_agents: &[FreeAgentInfo], limit: usize) -> Vec<FreeAgentSummary> {
        let mut fas: Vec<_> = free_agents.to_vec();
        fas.sort_by(|a, b| b.player.ability.cmp(&a.player.ability));

        fas.into_iter()
            .take(limit)
            .map(|fa| FreeAgentSummary {
                id: fa.player.id,
                name: fa.player.game_id.clone(),
                position: fa
                    .player
                    .position
                    .map(|p| format!("{:?}", p).to_uppercase())
                    .unwrap_or_else(|| "UNKNOWN".to_string()),
                age: fa.player.age,
                ability: fa.player.ability,
                potential: fa.player.potential,
                market_value: fa.market_value / 10000,  // 转换为万
                expected_salary: fa.expected_salary / 10000,  // 转换为万
            })
            .collect()
    }

    /// 将 LLM 响应转换为 AITransferStrategy
    fn convert_to_strategy(
        &self,
        llm: LLMStrategyResponse,
        team: &Team,
        roster: &[Player],
        free_agents: &[FreeAgentInfo],
        all_players_by_team: &HashMap<u64, Vec<Player>>,
        gm_profile: &TeamGMProfile,
        save_id: &str,
        season_id: u64,
    ) -> AITransferStrategy {
        // 构建选手查找表
        let mut player_map: HashMap<u64, &Player> = HashMap::new();

        // 构建自己队伍的选手ID集合（用于过滤targets）
        let roster_ids: std::collections::HashSet<u64> = roster.iter().map(|p| p.id).collect();

        for p in roster {
            player_map.insert(p.id, p);
        }
        for fa in free_agents {
            player_map.insert(fa.player.id, &fa.player);
        }
        // 添加其他队伍的选手到查找表
        for (tid, players) in all_players_by_team {
            if *tid != team.id {  // 排除自己队伍
                for p in players {
                    player_map.insert(p.id, p);
                }
            }
        }

        // 转换目标列表（过滤掉自己队伍的选手）
        // 归一化价格值 - LLM 可能返回元而非万
        // 如果值超过 50000（5亿），认为 LLM 返回的是元，需要转换为万
        let normalize_price = |value: f64| -> u64 {
            if value > 50000.0 {
                log::warn!("LLM 返回的价格值 {} 过大，可能是元单位，自动转换为万", value);
                (value / 10000.0) as u64
            } else {
                value as u64
            }
        };

        let targets: Vec<TransferTarget> = llm
            .targets
            .into_iter()
            .filter_map(|t| {
                // 过滤掉自己队伍的选手
                if roster_ids.contains(&t.player_id) {
                    log::warn!("LLM 错误地将自己队伍的选手 {} 作为目标，已过滤", t.player_id);
                    return None;
                }
                let player = player_map.get(&t.player_id)?;
                Some(TransferTarget {
                    player_id: t.player_id,
                    player_name: player.game_id.clone(),
                    position: player
                        .position
                        .map(|p| format!("{:?}", p).to_uppercase())
                        .unwrap_or_default(),
                    ability: player.ability,
                    potential: player.potential,
                    age: player.age,
                    current_team_id: player.team_id,
                    current_team_name: None,
                    market_value: player.calculate_market_value(),
                    max_offer: normalize_price(t.max_offer),
                    priority: t.priority,
                    reasoning: t.reasoning,
                })
            })
            .collect();

        // 转换出售列表
        let willing_to_sell: Vec<SellCandidate> = llm
            .willing_to_sell
            .into_iter()
            .filter_map(|s| {
                let player = player_map.get(&s.player_id)?;
                Some(SellCandidate {
                    player_id: s.player_id,
                    player_name: player.game_id.clone(),
                    position: player
                        .position
                        .map(|p| format!("{:?}", p).to_uppercase())
                        .unwrap_or_default(),
                    ability: player.ability,
                    age: player.age,
                    salary: player.salary,
                    market_value: player.calculate_market_value(),
                    min_price: normalize_price(s.min_price),
                    urgency: s.urgency,
                    reasoning: s.reasoning,
                })
            })
            .collect();

        // 归一化预算值
        let transfer_spend = normalize_price(llm.budget_allocation.transfer_spend);
        let salary_spend = normalize_price(llm.budget_allocation.salary_spend);
        let reserve = normalize_price(llm.budget_allocation.reserve);
        let total_budget = transfer_spend + salary_spend + reserve;

        AITransferStrategy {
            id: 0,
            team_id: team.id,
            team_name: team.name.clone(),
            save_id: save_id.to_string(),
            season_id,
            overall_strategy: llm.overall_strategy,
            strategy_description: llm.strategy_description,
            targets,
            willing_to_sell,
            priority_positions: llm.priority_positions,
            budget_allocation: BudgetAllocation {
                total_budget,
                transfer_spend,
                salary_spend,
                reserve,
            },
            reasoning: llm.reasoning,
            analysis_steps: llm.analysis_steps.into_iter()
                .map(|step| crate::models::TeamAnalysisStep {
                    step_name: step.step_name,
                    data_used: step.data_used,
                    threshold: step.threshold,
                    result: step.result,
                    impact: step.impact,
                })
                .collect(),
            is_mock: false, // 这是真实 AI 生成的
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    // ==================== 选手策略生成 ====================

    /// 生成选手转会策略
    pub async fn generate_player_strategy(
        &self,
        player: &Player,
        current_team: &Team,
        team_roster: &[Player],
        available_teams: &[TeamInfo],
        save_id: &str,
        season_id: u64,
        honors: Option<&PlayerHonorInfo>,
        performance: Option<&PlayerPerformanceInfo>,
        current_team_rank: Option<(u32, u32)>, // (global_rank, annual_points)
    ) -> Result<PlayerTransferStrategy, String> {
        log::info!("开始为选手 {} (ID: {}) 生成 LLM 策略", player.game_id, player.id);

        if !self.is_configured() {
            return Err("LLM API Key 未配置".to_string());
        }

        // 1. 构建 Prompt
        let system_prompt = self.build_player_system_prompt();
        let user_prompt = self.build_player_user_prompt(player, current_team, team_roster, available_teams, season_id, honors, performance, current_team_rank);

        // 2. 调用 LLM API
        log::info!("调用 LLM API ({:?}) 生成选手策略...", self.config.provider);
        let response_text = match self.config.provider {
            LLMProvider::Claude => self.call_claude(&system_prompt, &user_prompt).await?,
            _ => self.call_openai_compatible(&system_prompt, &user_prompt).await?,
        };
        log::info!("LLM API 返回成功，响应长度: {} 字符", response_text.len());

        // 3. 解析响应
        let cleaned_response = clean_json_response(&response_text);
        let llm_response: LLMPlayerStrategyResponse = serde_json::from_str(&cleaned_response)
            .map_err(|e| format!("解析 LLM 响应失败: {}\n原始响应: {}", e, response_text))?;

        // 4. 构建分析数据快照（即使是 LLM 也需要展示分析依据）
        let analysis_data = self.build_analysis_data_snapshot(player, current_team, team_roster);

        // 5. 转换为 PlayerTransferStrategy
        let mut strategy = self.convert_to_player_strategy(
            llm_response,
            player,
            available_teams,
            save_id,
            season_id,
        );

        // 6. 附加分析数据
        strategy.analysis_data = Some(analysis_data);

        Ok(strategy)
    }

    /// 构建分析数据快照
    fn build_analysis_data_snapshot(
        &self,
        player: &Player,
        current_team: &Team,
        team_roster: &[Player],
    ) -> crate::models::AnalysisDataSnapshot {
        use crate::models::AnalysisDataSnapshot;

        // 计算球队平均能力
        let active_roster: Vec<_> = team_roster
            .iter()
            .filter(|p| p.status == PlayerStatus::Active)
            .collect();
        let team_avg_ability = if !active_roster.is_empty() {
            active_roster.iter().map(|p| p.ability as f64).sum::<f64>() / active_roster.len() as f64
        } else {
            0.0
        };

        // 计算忠诚度类型和离队阈值
        let (loyalty_type, departure_threshold) = match player.loyalty {
            90..=100 => ("忠心耿耿", 20u8),
            70..=89 => ("忠诚", 35u8),
            50..=69 => ("中立", 50u8),
            30..=49 => ("机会主义", 60u8),
            _ => ("雇佣兵", 70u8),
        };

        AnalysisDataSnapshot {
            player_name: player.game_id.clone(),
            position: player
                .position
                .map(|p| format!("{:?}", p).to_uppercase())
                .unwrap_or_default(),
            age: player.age,
            ability: player.ability,
            potential: player.potential,
            satisfaction: player.satisfaction,
            loyalty: player.loyalty,
            is_starter: player.is_starter,
            current_salary: player.salary / 10000, // 转换为万
            contract_end_season: player.contract_end_season,
            team_name: current_team.name.clone(),
            team_avg_ability,
            loyalty_type: loyalty_type.to_string(),
            departure_threshold,
        }
    }

    /// 构建选手策略的 System Prompt
    fn build_player_system_prompt(&self) -> String {
        r#"你是一个电竞选手的AI决策助手。你需要根据选手的当前状态、满意度、忠诚度、荣誉记录、本赛季表现等属性，
**只分析选手是否想要离开当前球队**。你不需要考虑选手想去哪个球队。

## 选手心理因素

### 满意度影响（参考，不是唯一决定因素！）
- 满意度 >= 90：非常满意，大概率留队
- 满意度 80-89：满意，除非有明显不满才会考虑离队
- 满意度 60-79：**可被挖角**，会考虑市场机会
- 满意度 40-59：**倾向离队**，积极寻找新机会
- 满意度 < 40：**强烈想离队**

### 忠诚度类型（参考，不是唯一决定因素！）
| 类型 | 忠诚度 | 转会倾向 |
|------|--------|----------|
| 忠心耿耿 (Devoted) | >90 | **几乎不会离队** |
| 忠诚 (Loyal) | 70-90 | 较少主动离队 |
| 中立 (Neutral) | 40-70 | **正常市场行为** |
| 机会主义 (Opportunist) | 20-40 | 容易被外界吸引 |
| 雇佣兵 (Mercenary) | <20 | 随时可走 |

### 想离队的常见情况（重要！约30%的选手会考虑离队）
满足以下条件之一**通常会想离队**：
1. **替补选手**：不是首发，想获得更多出场机会
2. **满意度一般**：满意度 < 75，对现状不太满意
3. **追求冠军**：球队排名靠后，选手想去更强的队
4. **薪资不满**：能力强但薪资偏低
5. **年轻有野心**：年龄 < 25，潜力高，想挑战自己

### 想留队的情况（约70%的选手会选择留队）
1. **极度满意**：满意度 >= 85
2. **忠心耿耿**：忠诚度 >= 90
3. **老将稳定**：年龄 >= 30 且 满意度 >= 70

**重要判断原则**：
- 电竞转会市场比较活跃，**约25%-35%的选手会考虑离队**
- **替补选手大概率想离队**（约60%的替补会想离队）
- 满意度 60-75 的选手应该有较高概率想离队
- 只有满意度 >= 85 或 忠诚度 >= 90 的选手才会坚定留队

### 荣誉系统对心态的影响
| 荣誉情况 | 心理影响 | 转会倾向 |
|----------|----------|----------|
| 多冠选手(5+冠军) | 功成名就，情怀深厚 | 倾向留队 |
| 有冠军经历(2-4冠) | 渴望更多荣誉 | 可能追求新挑战 |
| 单冠选手 | 尝过冠军滋味 | 渴望再次夺冠 |
| 无冠军 | 渴望证明自己 | 更容易被机会吸引 |

### 本赛季表现对心态的影响
| 表现等级 | 判定标准 | 心理影响 |
|----------|----------|----------|
| 顶级表现 | 平均表现 >= 90 | 信心爆棚，期望更高待遇 |
| 优秀表现 | 平均表现 80-89 | 证明了价值 |
| 合格表现 | 平均表现 70-79 | 正常发挥 |
| 一般表现 | 平均表现 60-69 | 可能面临竞争压力 |
| 表现欠佳 | 平均表现 < 60 | 可能想换环境 |

### 离队原因（可多选）
- LACK_OF_PLAYTIME：缺乏出场时间（替补想当首发）
- TEAM_PERFORMANCE：战队成绩差
- SALARY_DISPUTE：薪资不满
- SEEKING_CHAMPIONSHIP：追求冠军
- SEEKING_OPPORTUNITY：寻求新机会
- PERSONAL_REASONS：个人原因

## 输出要求

请严格按照以下JSON格式返回。**注意：不需要填写 preferred_teams，留空数组即可。**

### 示例1：选手想留队
```json
{
  "wants_to_leave": false,
  "decision_confidence": 90,
  "departure_reasons": [],
  "leave_reasoning": "选手对当前球队满意，满意度和忠诚度都很高，没有离开的理由。",
  "analysis_steps": [
    {
      "step_name": "满意度分析",
      "data_used": "满意度: 88",
      "threshold": "满意度>=80 满意",
      "result": "选手对球队满意",
      "impact": "留队倾向"
    },
    {
      "step_name": "忠诚度分析",
      "data_used": "忠诚度: 75 (忠诚型)",
      "threshold": "忠诚度>=70 较少主动离队",
      "result": "对球队有感情",
      "impact": "留队倾向"
    },
    {
      "step_name": "最终决策",
      "data_used": "综合满意度88、忠诚度75",
      "threshold": "",
      "result": "选手满意现状，决定留队",
      "impact": "最终决定: 不想离队"
    }
  ],
  "preferred_teams": [],
  "expected_salary": 150,
  "expected_min_salary": 120,
  "expected_years": 2,
  "requires_starter": true,
  "team_preference_reasoning": "对当前球队满意，暂无转会意向。"
}
```

### 示例2：替补选手想离队
```json
{
  "wants_to_leave": true,
  "decision_confidence": 85,
  "departure_reasons": ["LACK_OF_PLAYTIME"],
  "leave_reasoning": "作为替补长期得不到出场机会，希望寻找能给予首发位置的球队。",
  "analysis_steps": [
    {
      "step_name": "出场机会分析",
      "data_used": "替补身份，同位置有竞争",
      "threshold": "替补选手普遍想获得首发机会",
      "result": "缺乏出场机会是主要不满",
      "impact": "离队倾向"
    },
    {
      "step_name": "满意度分析",
      "data_used": "满意度: 55",
      "threshold": "满意度<60 有明显不满",
      "result": "对替补身份不满",
      "impact": "离队倾向"
    },
    {
      "step_name": "最终决策",
      "data_used": "综合替补身份、满意度55",
      "threshold": "",
      "result": "决定寻求转会，目标是获得首发机会",
      "impact": "最终决定: 想离队"
    }
  ],
  "preferred_teams": [],
  "expected_salary": 180,
  "expected_min_salary": 140,
  "expected_years": 2,
  "requires_starter": true,
  "team_preference_reasoning": "希望获得首发机会。"
}
```

## 重要决策原则

### 想离队的常见情况（约25%-35%的选手会考虑）
1. **追求首发**：替补想获得首发机会
2. **不满现状**：满意度 < 60
3. **球队成绩差**：球队排名靠后
4. **薪资不满**：当前薪资低于市场价值

### 想留队的情况（约65%-75%的选手会选择）
1. **满意**：满意度 >= 70 且 是首发
2. **忠诚**：忠诚度 >= 70
3. **老将情怀**：30岁以上

### 判断原则
- 替补选手大概率想离队寻找首发机会
- 满意度 >= 70 且 是首发的选手大概率留队
- 忠诚度 >= 70 的选手不会轻易离队
- 只有满意度很低（<50）或有明确不满的选手才会想离队

注意：
1. 价格单位是万元/年
2. **preferred_teams 留空数组即可，不需要填写**
3. 只输出JSON，不要有其他内容"#.to_string()
    }

    /// 构建选手策略的 User Prompt
    fn build_player_user_prompt(
        &self,
        player: &Player,
        team: &Team,
        roster: &[Player],
        available_teams: &[TeamInfo],
        season_id: u64,
        honors: Option<&PlayerHonorInfo>,
        performance: Option<&PlayerPerformanceInfo>,
        current_team_rank: Option<(u32, u32)>, // (global_rank, annual_points)
    ) -> String {
        // 计算同位置竞争情况
        let position_str = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        let same_position_players: Vec<_> = roster.iter()
            .filter(|p| {
                p.id != player.id &&
                p.status == PlayerStatus::Active &&
                p.position.map(|pos| format!("{:?}", pos).to_uppercase()) == Some(position_str.clone())
            })
            .collect();

        let competition_info = if same_position_players.is_empty() {
            "无竞争（唯一该位置选手）".to_string()
        } else {
            same_position_players.iter()
                .map(|p| format!("- {} (能力{}, {})", p.game_id, p.ability, if p.is_starter { "首发" } else { "替补" }))
                .collect::<Vec<_>>()
                .join("\n")
        };

        // 计算球队平均能力
        let active_roster: Vec<_> = roster.iter()
            .filter(|p| p.status == PlayerStatus::Active)
            .collect();
        let avg_ability = if !active_roster.is_empty() {
            active_roster.iter().map(|p| p.ability as f64).sum::<f64>() / active_roster.len() as f64
        } else {
            0.0
        };

        // 不再需要可选球队列表，选手只需决定是否离队
        let _ = available_teams; // 标记为已使用，避免编译警告

        // 忠诚度类型
        let loyalty_type = if player.loyalty > 90 {
            "忠心耿耿"
        } else if player.loyalty > 70 {
            "忠诚"
        } else if player.loyalty > 40 {
            "中立"
        } else if player.loyalty > 20 {
            "机会主义"
        } else {
            "雇佣兵"
        };

        // 计算合同剩余年数
        let contract_years_remaining = player.contract_end_season
            .map(|end| end.saturating_sub(season_id as u32))
            .unwrap_or(0);

        // 格式化荣誉信息
        let honors_info = if let Some(h) = honors {
            let mut honor_parts = Vec::new();

            if h.worlds_championships > 0 {
                honor_parts.push(format!("世界赛冠军 x{}", h.worlds_championships));
            }
            if h.msi_championships > 0 {
                honor_parts.push(format!("MSI冠军 x{}", h.msi_championships));
            }
            if h.regional_championships > 0 {
                honor_parts.push(format!("赛区冠军 x{}", h.regional_championships));
            }
            if h.tournament_mvps > 0 {
                honor_parts.push(format!("赛事MVP x{}", h.tournament_mvps));
            }
            if h.finals_mvps > 0 {
                honor_parts.push(format!("决赛MVP x{}", h.finals_mvps));
            }
            if h.yearly_mvps > 0 {
                honor_parts.push(format!("年度MVP x{}", h.yearly_mvps));
            }

            if honor_parts.is_empty() {
                "暂无重大荣誉".to_string()
            } else {
                honor_parts.join("、")
            }
        } else {
            "暂无荣誉数据".to_string()
        };

        // 荣誉对选手心态的影响说明
        let honors_impact = if let Some(h) = honors {
            let total_titles = h.worlds_championships + h.msi_championships + h.regional_championships;
            if total_titles >= 5 {
                "（多冠选手，对当前球队有深厚感情，不轻易离队）"
            } else if total_titles >= 2 {
                "（有夺冠经历，可能追求更多荣誉）"
            } else if total_titles == 1 {
                "（已尝过冠军滋味，渴望再次夺冠）"
            } else {
                "（尚无冠军，可能渴望夺冠机会）"
            }
        } else {
            ""
        };

        // 格式化赛季表现数据
        let performance_section = if let Some(perf) = performance {
            if perf.games_played > 0 {
                let ability_diff_str = if perf.ability_diff > 0.0 {
                    format!("+{:.1}", perf.ability_diff)
                } else {
                    format!("{:.1}", perf.ability_diff)
                };
                format!(
                    r#"
### 本赛季表现（数据中心）
出场次数: {}场
平均表现: {:.1} ({})
最佳表现: {:.1}
稳定性: {:.0}/100
与能力值差异: {} (正数=超常发挥，负数=表现不佳)
"#,
                    perf.games_played,
                    perf.avg_performance,
                    perf.performance_tier,
                    perf.best_performance,
                    perf.consistency_score,
                    ability_diff_str
                )
            } else {
                "\n### 本赛季表现\n暂无出场数据\n".to_string()
            }
        } else {
            String::new()
        };

        // 格式化当前球队排名信息
        let current_team_rank_str = if let Some((rank, points)) = current_team_rank {
            if rank > 0 {
                format!("全球排名: #{} (年度积分: {}分)", rank, points)
            } else {
                format!("全球排名: 未排名 (年度积分: {}分)", points)
            }
        } else {
            "全球排名: 未知".to_string()
        };

        format!(
            r#"## 选手信息

游戏ID: {}
年龄: {}岁
位置: {}
能力值: {}
潜力值: {}
当前薪资: {}万/年
身价: {}万
合同剩余: {}年

### 荣誉记录 {}
{}
{}
### 心理状态
满意度: {}/100
忠诚度: {}/100
忠诚类型: {}

### 当前战队
战队名称: {}
{}
是否首发: {}
球队平均能力: {:.1}

### 同位置竞争
{}

请根据以上信息，分析这名选手是否想要离开当前球队。
**注意**：你只需要判断选手是否想离队，不需要考虑去哪个球队。"#,
            player.game_id,
            player.age,
            position_str,
            player.ability,
            player.potential,
            player.salary / 10000,
            player.calculate_market_value() / 10000,
            contract_years_remaining,
            honors_impact,
            honors_info,
            performance_section,
            player.satisfaction,
            player.loyalty,
            loyalty_type,
            team.name,
            current_team_rank_str,
            if player.is_starter { "是" } else { "否" },
            avg_ability,
            competition_info
        )
    }

    /// 将 LLM 响应转换为 PlayerTransferStrategy
    fn convert_to_player_strategy(
        &self,
        llm: LLMPlayerStrategyResponse,
        player: &Player,
        available_teams: &[TeamInfo],
        save_id: &str,
        season_id: u64,
    ) -> PlayerTransferStrategy {
        use crate::models::AnalysisStep;

        // 不再需要处理 preferred_teams，因为选手策略只决定是否离队
        let _ = available_teams;

        // 转换离队原因
        let departure_reasons: Vec<DepartureReason> = llm.departure_reasons.iter()
            .filter_map(|s| DepartureReason::from_str(s))
            .collect();

        // preferred_teams 现在总是空的，选手不再选择目标球队
        let preferred_teams: Vec<PreferredTeam> = vec![];

        // 转换 LLM 分析步骤
        let analysis_steps: Vec<AnalysisStep> = llm.analysis_steps.into_iter()
            .map(|step| AnalysisStep {
                step_name: step.step_name,
                data_used: step.data_used,
                threshold: step.threshold,
                result: step.result,
                impact: step.impact,
            })
            .collect();

        log::info!("LLM 返回了 {} 个分析步骤", analysis_steps.len());

        // ========== 安全检查：覆盖不合理的 LLM 决定 ==========
        // 只在极端情况下覆盖，目标是让约25-35%的选手想离队
        let mut final_wants_to_leave = llm.wants_to_leave;
        let mut override_reason = String::new();

        if llm.wants_to_leave {
            // 只在非常极端的情况下强制留队（条件大幅收紧）
            let should_stay =
                // 情况1: 极度满意 - 满意度非常高
                (player.satisfaction >= 90) ||
                // 情况2: 极度忠诚且满意 - 忠诚度极高且不太不满
                (player.loyalty >= 92 && player.satisfaction >= 70) ||
                // 情况3: 老将且非常满意 - 30岁以上的老将且很满意
                (player.age >= 30 && player.satisfaction >= 80 && player.loyalty >= 75);

            if should_stay {
                final_wants_to_leave = false;
                override_reason = format!(
                    "安全检查覆盖: 满意度{}, 忠诚度{}, 年龄{}, 首发={}, 强制留队",
                    player.satisfaction, player.loyalty, player.age, player.is_starter
                );
            }
        }

        if !override_reason.is_empty() {
            log::warn!(
                "选手 {} LLM决定被覆盖: {} -> {} ({})",
                player.game_id,
                llm.wants_to_leave,
                final_wants_to_leave,
                override_reason
            );
        }

        // 如果被覆盖为留队，清空离队相关数据
        let (final_departure_reasons, final_leave_reasoning, final_preferred_teams) = if !final_wants_to_leave && llm.wants_to_leave {
            (
                vec![],
                override_reason.clone(),
                vec![],
            )
        } else {
            (departure_reasons, llm.leave_reasoning, preferred_teams)
        };

        PlayerTransferStrategy {
            id: 0,
            player_id: player.id,
            save_id: save_id.to_string(),
            season_id,
            wants_to_leave: final_wants_to_leave,
            decision_confidence: llm.decision_confidence.min(100),
            departure_reasons: final_departure_reasons,
            leave_reasoning: final_leave_reasoning,
            preferred_teams: final_preferred_teams,
            team_preference_reasoning: llm.team_preference_reasoning,
            expected_salary: llm.expected_salary as u64,
            expected_min_salary: llm.expected_min_salary as u64,
            expected_years: llm.expected_years,
            requires_starter: llm.requires_starter,
            analysis_data: None,      // 会在调用方填充
            analysis_steps,           // LLM 返回的分析步骤
            is_mock: false,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 评估选手对邀约的反应
    pub async fn evaluate_offer(
        &self,
        player: &Player,
        strategy: &PlayerTransferStrategy,
        offer: &TeamOffer,
    ) -> Result<OfferEvaluation, String> {
        if !self.is_configured() {
            // 无LLM时使用规则评估
            return Ok(self.rule_based_offer_evaluation(player, strategy, offer));
        }

        let system_prompt = r#"你是一个电竞选手的决策助手。根据选手的期望条件和收到的邀约，判断选手是否会接受。

输出JSON格式：
{
  "accept": true/false,
  "confidence": 1-100,
  "reasoning": "详细理由...",
  "salary_score": 1-100,
  "team_score": 1-100,
  "role_score": 1-100,
  "overall_score": 1-100
}

注意：
1. 只输出JSON，不要其他内容"#;

        let user_prompt = format!(
            r#"## 选手期望
- 期望薪资: {}万/年
- 最低接受: {}万/年
- 期望年限: {}年
- 要求首发: {}
- 偏好球队: {:?}

## 收到邀约
- 球队: {}
- 年薪: {}万/年
- 年限: {}年
- 首发保证: {}
- 转会费: {}万

请判断选手是否会接受这个邀约。"#,
            strategy.expected_salary,
            strategy.expected_min_salary,
            strategy.expected_years,
            if strategy.requires_starter { "是" } else { "否" },
            strategy.preferred_teams.iter().map(|t| &t.team_name).collect::<Vec<_>>(),
            offer.team_name,
            offer.salary_offer,
            offer.contract_years,
            if offer.starter_guarantee { "是" } else { "否" },
            offer.transfer_fee
        );

        let response_text = match self.config.provider {
            LLMProvider::Claude => self.call_claude(system_prompt, &user_prompt).await?,
            _ => self.call_openai_compatible(system_prompt, &user_prompt).await?,
        };

        let cleaned_response = clean_json_response(&response_text);
        serde_json::from_str(&cleaned_response)
            .map_err(|e| format!("解析邀约评估响应失败: {}", e))
    }

    // ==================== 续约评估 ====================

    /// 评估续约决策
    ///
    /// 对于不想离队的选手，评估：
    /// 1. 球队是否想续约
    /// 2. 续约条件（薪资、年限）
    /// 3. 选手是否接受
    pub async fn evaluate_renewal(
        &self,
        player: &Player,
        team: &Team,
        player_strategy: &PlayerTransferStrategy,
        team_strategy: Option<&AITransferStrategy>,
        player_honors: Option<&PlayerHonorInfo>,
        player_performance: Option<&PlayerPerformanceInfo>,
    ) -> Result<crate::models::RenewalDecision, String> {
        use crate::models::{RenewalDecision, RenewalAnalysisStep};

        log::info!("评估选手 {} 与 {} 的续约", player.game_id, team.name);

        if !self.is_configured() {
            return Err("LLM API Key 未配置".to_string());
        }

        let system_prompt = self.build_renewal_system_prompt();
        let user_prompt = self.build_renewal_user_prompt(
            player,
            team,
            player_strategy,
            team_strategy,
            player_honors,
            player_performance,
        );

        // 调用 LLM
        let response_text = match self.config.provider {
            LLMProvider::Claude => self.call_claude(&system_prompt, &user_prompt).await?,
            _ => self.call_openai_compatible(&system_prompt, &user_prompt).await?,
        };

        // 解析响应
        let cleaned_response = clean_json_response(&response_text);
        let llm_response: LLMRenewalResponse = serde_json::from_str(&cleaned_response)
            .map_err(|e| format!("解析续约评估响应失败: {}\n原始响应: {}", e, response_text))?;

        // 转换为 RenewalDecision
        let decision = RenewalDecision {
            player_id: player.id,
            player_name: player.game_id.clone(),
            team_id: team.id,
            team_name: team.name.clone(),
            team_wants_renewal: llm_response.team_wants_renewal,
            team_rejection_reason: llm_response.team_rejection_reason,
            offered_salary: llm_response.offered_salary,
            offered_years: llm_response.offered_years,
            player_accepts: llm_response.player_accepts,
            player_rejection_reason: llm_response.player_rejection_reason,
            renewal_successful: llm_response.team_wants_renewal && llm_response.player_accepts,
            final_salary: if llm_response.team_wants_renewal && llm_response.player_accepts {
                Some(llm_response.offered_salary)
            } else {
                None
            },
            final_years: if llm_response.team_wants_renewal && llm_response.player_accepts {
                Some(llm_response.offered_years)
            } else {
                None
            },
            team_analysis: llm_response.team_analysis.into_iter().map(|s| RenewalAnalysisStep {
                step_name: s.step_name,
                data_used: s.data_used,
                result: s.result,
                impact: s.impact,
            }).collect(),
            player_analysis: llm_response.player_analysis.into_iter().map(|s| RenewalAnalysisStep {
                step_name: s.step_name,
                data_used: s.data_used,
                result: s.result,
                impact: s.impact,
            }).collect(),
            summary: llm_response.summary,
        };

        log::info!(
            "续约评估完成: {} - {} -> 球队想续约: {}, 选手接受: {}, 成功: {}",
            player.game_id, team.name,
            decision.team_wants_renewal, decision.player_accepts, decision.renewal_successful
        );

        Ok(decision)
    }

    /// 构建续约评估 System Prompt
    fn build_renewal_system_prompt(&self) -> String {
        r#"你是一个专业的电竞转会市场续约评估系统。你需要评估球队是否想与选手续约，以及选手是否接受续约。

## 你的任务
1. 从球队角度分析是否值得续约这名选手
2. 如果球队想续约，确定合理的薪资和合同年限
3. 从选手角度分析是否接受球队的续约条件

## 球队决定【不续约】的情况（重要！这些情况球队应该放弃选手）
- 选手能力低于70：能力不足以胜任职业赛场
- 选手年龄 >= 30 且能力 < 85：老将且能力不够顶尖
- 选手年龄 >= 28 且能力 < 80：接近职业末期且表现平庸
- 赛季表现评级为"表现欠佳"或"一般表现"：表现不佳不值得续约
- 球队余额 < 500万 且选手期望薪资 > 100万：财务紧张无法负担
- 满意度 < 40：选手已经非常不满，留下来也会有问题
- 该位置有更年轻或更强的替补：可以培养新人

## 球队决定【续约】的情况
- 选手能力 >= 85：核心选手，必须留住
- 选手能力 >= 75 且年龄 <= 26：有潜力的年轻选手
- 选手表现优异（优秀或顶级表现）：用表现证明了价值
- 选手忠诚度 >= 80：忠心耿耿的选手值得培养

## 选手决定【拒绝续约】的情况
- 球队报价薪资 < 期望薪资的70%：薪资差距太大
- 球队世界排名很低且选手能力 >= 85：顶级选手想去更好的球队
- 满意度 < 50 且有其他更好选择：对球队不满想换环境

## 选手决定【接受续约】的情况
- 报价薪资 >= 期望薪资的85%：薪资基本满意
- 忠诚度 >= 70 且报价薪资 >= 期望薪资的70%：忠诚选手愿意接受略低薪资
- 满意度 >= 60：对球队满意愿意留下

## 续约条件计算参考
- 基础薪资 = 选手当前薪资
- 能力 >= 90：薪资 +30%
- 能力 >= 85：薪资 +20%
- 能力 >= 80：薪资 +10%
- 能力 < 75：薪资 -10%
- 年龄 >= 30：薪资 -20%
- 年龄 >= 28：薪资 -10%
- 表现优异：薪资 +15%

## 输出格式
请返回 JSON 格式，包含完整的分析步骤：
```json
{
  "team_wants_renewal": true或false,
  "team_rejection_reason": "球队不想续约的原因（如果不想续约，必须给出具体原因）",
  "offered_salary": 续约薪资数字（单位：万/年，如125表示125万/年）,
  "offered_years": 合同年限数字(1-3),
  "player_accepts": true或false,
  "player_rejection_reason": "选手拒绝原因（如果拒绝，必须给出具体原因）",
  "team_analysis": [
    {"step_name": "步骤名", "data_used": "使用的数据", "result": "分析结果", "impact": "正面/负面/中性影响"}
  ],
  "player_analysis": [
    {"step_name": "步骤名", "data_used": "使用的数据", "result": "分析结果", "impact": "正面/负面/中性影响"}
  ],
  "summary": "综合总结续约情况（一句话）"
}
```

重要提示：
1. 只输出 JSON，不要有其他内容
2. 请认真评估，不是所有选手都应该被续约！约有10-30%的选手应该被球队放弃或主动拒绝续约
3. offered_salary 必须是数字，单位是万，例如125表示125万/年"#.to_string()
    }

    /// 构建续约评估 User Prompt
    fn build_renewal_user_prompt(
        &self,
        player: &Player,
        team: &Team,
        player_strategy: &PlayerTransferStrategy,
        team_strategy: Option<&AITransferStrategy>,
        player_honors: Option<&PlayerHonorInfo>,
        player_performance: Option<&PlayerPerformanceInfo>,
    ) -> String {
        let position = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_else(|| "未知".to_string());

        // 构建荣誉信息
        let honors_str = if let Some(h) = player_honors {
            format!(
                "世界冠军: {}次, MSI冠军: {}次, 赛区冠军: {}次, 年度MVP: {}次",
                h.worlds_championships, h.msi_championships, h.regional_championships, h.yearly_mvps
            )
        } else {
            "暂无重大荣誉".to_string()
        };

        // 构建表现信息
        let performance_str = if let Some(p) = player_performance {
            format!(
                "出场: {}场, 平均表现: {:.1}, 最佳表现: {:.1}, 稳定性: {:.0}, 表现等级: {}",
                p.games_played, p.avg_performance, p.best_performance, p.consistency_score, p.performance_tier
            )
        } else {
            "暂无赛季数据".to_string()
        };

        // 球队策略信息
        let team_strategy_str = if let Some(ts) = team_strategy {
            format!(
                "球队策略: {}, 转会预算: {}万, 目标选手数: {}",
                ts.overall_strategy, ts.budget_allocation.transfer_spend, ts.targets.len()
            )
        } else {
            "球队策略未知".to_string()
        };

        format!(
            r#"## 选手信息
- 姓名: {}
- 位置: {}
- 年龄: {}岁
- 能力: {}
- 潜力: {}
- 当前薪资: {}万/年
- 合同结束赛季: {}

## 选手心理状态
- 满意度: {}/100
- 忠诚度: {}/100
- 期望薪资: {}万/年
- 期望合同年限: {}年

## 选手荣誉
{}

## 本赛季表现
{}

## 球队信息
- 球队: {}
- 余额: {}万
- {}

请根据以上信息，评估这次续约谈判。"#,
            player.game_id,
            position,
            player.age,
            player.ability,
            player.potential,
            player.salary / 10000,  // 转换为万
            player.contract_end_season.map(|s| format!("第{}赛季", s)).unwrap_or_else(|| "无合同".to_string()),
            player.satisfaction,
            player.loyalty,
            player_strategy.expected_salary,
            player_strategy.expected_years,
            honors_str,
            performance_str,
            team.name,
            team.balance / 10000,  // 转换为万
            team_strategy_str
        )
    }

    // ==================== 谈判系统 LLM 方法 ====================

    /// 生成球队报价决策
    ///
    /// 球队决定是否向某个选手发出报价，以及报价的具体条件
    pub async fn generate_offer_decision(
        &self,
        team: &Team,
        team_strategy: &AITransferStrategy,
        target_player: &Player,
        player_strategy: &PlayerTransferStrategy,
        current_round: u8,
        competing_teams: &[TeamInfo],
        team_honors: Option<&TeamHonorInfo>,
        player_honors: Option<&PlayerHonorInfo>,
        player_performance: Option<&PlayerPerformanceInfo>,
    ) -> Result<crate::models::negotiation::LLMOfferDecision, String> {
        use crate::models::negotiation::LLMOfferDecision;

        if !self.is_configured() {
            return Err("LLM API Key 未配置".to_string());
        }

        let system_prompt = self.build_offer_decision_system_prompt();
        let user_prompt = self.build_offer_decision_user_prompt(
            team,
            team_strategy,
            target_player,
            player_strategy,
            current_round,
            competing_teams,
            team_honors,
            player_honors,
            player_performance,
        );

        log::info!("调用 LLM 生成球队 {} 对选手 {} 的报价决策", team.name, target_player.game_id);

        let response_text = match self.config.provider {
            LLMProvider::Claude => self.call_claude(&system_prompt, &user_prompt).await?,
            _ => self.call_openai_compatible(&system_prompt, &user_prompt).await?,
        };

        let cleaned_response = clean_json_response(&response_text);
        serde_json::from_str(&cleaned_response)
            .map_err(|e| format!("解析报价决策响应失败: {}\\n原始响应: {}", e, response_text))
    }

    /// 构建报价决策的 System Prompt
    fn build_offer_decision_system_prompt(&self) -> String {
        r#"你是一个电竞俱乐部的转会决策系统。你需要帮助球队 GM 决定是否向目标选手发出报价，以及报价的具体条件。

## 决策因素

### 1. 球队需求
- 该位置的紧迫程度（100=急需，0=充足）
- 预算剩余情况
- 阵容完整度

### 2. 选手价值
- 能力值和潜力
- 年龄和巅峰期
- 赛季表现数据
- 荣誉记录
- 市场身价

### 3. 竞争态势
- 有多少球队在竞争
- 选手的偏好球队是谁
- 我们是否在选手的偏好列表中

### 4. 报价策略
- 选手的期望薪资和最低接受薪资
- 是否需要保证首发
- 合同年限偏好

## 报价原则

1. **溢价上限**：一般不超过选手期望薪资的 120%
2. **竞争加价**：如果竞争激烈，可适当提高
3. **偏好加成**：如果选手明确偏好我们，可以少出一些
4. **紧迫程度**：位置越急需，出价越激进

## 输出要求

请严格按照以下 JSON 格式返回，**必须包含 analysis_steps**：

```json
{
  "should_offer": true,
  "salary_offer": 200,
  "contract_years": 2,
  "guarantee_starter": true,
  "signing_bonus": 50,
  "reasoning": "详细的决策理由...",
  "analysis_steps": [
    {
      "step_name": "需求分析",
      "data_used": "位置需求: 85, 当前该位置人数: 1",
      "threshold": "需求>60为高需求",
      "result": "该位置急需补强",
      "impact": "提高报价意愿 +30%"
    },
    {
      "step_name": "选手价值评估",
      "data_used": "能力88, 潜力92, 年龄23, 身价1500万",
      "threshold": "目标能力阈值: 85",
      "result": "选手能力达标，年龄处于上升期",
      "impact": "值得投入较高预算"
    },
    {
      "step_name": "竞争分析",
      "data_used": "竞争球队: 3支, 选手偏好: 我们排第2",
      "threshold": "",
      "result": "竞争激烈但有机会",
      "impact": "需要有竞争力的报价"
    },
    {
      "step_name": "预算评估",
      "data_used": "剩余预算: 3000万, 已花费: 500万",
      "threshold": "单笔最高预算: 预算的40%",
      "result": "预算充足",
      "impact": "可以出具竞争性报价"
    },
    {
      "step_name": "最终报价决策",
      "data_used": "选手期望: 180万/年, 最低: 150万/年",
      "threshold": "",
      "result": "决定报价 200万/年，2年合同，保证首发",
      "impact": "略高于期望以增加成功率"
    }
  ]
}
```

注意：
1. salary_offer 和 signing_bonus 单位是万元
2. should_offer 为 false 时，其他字段可以为默认值
3. analysis_steps 必须包含至少 5 个步骤
4. 只输出 JSON，不要其他内容"#.to_string()
    }

    /// 构建报价决策的 User Prompt
    fn build_offer_decision_user_prompt(
        &self,
        team: &Team,
        team_strategy: &AITransferStrategy,
        target_player: &Player,
        player_strategy: &PlayerTransferStrategy,
        current_round: u8,
        competing_teams: &[TeamInfo],
        team_honors: Option<&TeamHonorInfo>,
        player_honors: Option<&PlayerHonorInfo>,
        player_performance: Option<&PlayerPerformanceInfo>,
    ) -> String {
        let position = target_player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        // 检查选手是否在我们的目标列表中
        let is_target = team_strategy.is_target(target_player.id);
        let max_offer = team_strategy.get_max_offer(target_player.id);

        // 检查我们是否在选手的偏好列表中
        let our_priority = player_strategy.get_team_priority(team.id);

        // 格式化竞争球队信息
        let competing_info = if competing_teams.is_empty() {
            "暂无其他球队竞争".to_string()
        } else {
            competing_teams.iter()
                .map(|t| format!("- {} (排名#{}, 实力{:.1})", t.name, t.global_rank, t.avg_ability))
                .collect::<Vec<_>>()
                .join("\n")
        };

        // 格式化荣誉信息
        let team_honors_str = if let Some(h) = team_honors {
            format!(
                "世界赛冠军x{}, MSI冠军x{}, 赛区冠军x{}",
                h.worlds_championships, h.msi_championships, h.regional_championships
            )
        } else {
            "暂无荣誉数据".to_string()
        };

        let player_honors_str = if let Some(h) = player_honors {
            format!(
                "世界赛冠军x{}, MSI冠军x{}, 赛区冠军x{}, MVP x{}",
                h.worlds_championships, h.msi_championships, h.regional_championships,
                h.tournament_mvps + h.finals_mvps + h.yearly_mvps
            )
        } else {
            "暂无荣誉数据".to_string()
        };

        // 格式化表现信息
        let performance_str = if let Some(p) = player_performance {
            format!(
                "出场{}场, 平均表现{:.1}, 稳定性{:.0}, 与能力差异{:+.1}",
                p.games_played, p.avg_performance, p.consistency_score, p.ability_diff
            )
        } else {
            "暂无赛季表现数据".to_string()
        };

        format!(
            r#"## 当前轮次
第 {} 轮报价

## 我方球队信息
- 球队: {}
- 荣誉: {}
- 剩余预算: {}万
- 薪资预算: {}万/年
- {}位置需求: {}

## 目标选手信息
- 姓名: {}
- 位置: {}
- 年龄: {}岁
- 能力: {}
- 潜力: {}
- 身价: {}万
- 荣誉: {}
- 本赛季表现: {}

## 选手转会意愿
- 想离队: {}
- 期望薪资: {}万/年
- 最低接受: {}万/年
- 期望年限: {}年
- 要求首发: {}
- 偏好球队数量: {}
- 我方在偏好中的排名: {}

## 我方策略
- 是否在目标列表: {}
- 策略建议最高出价: {}万

## 竞争态势
{}

## 市场阶段
当前处于第{}轮，后续还有谈判机会。

请决定是否发出报价，以及具体报价条件。"#,
            current_round,
            team.name,
            team_honors_str,
            team_strategy.budget_allocation.transfer_spend,
            team_strategy.budget_allocation.salary_spend,
            position,
            team_strategy.get_position_priority(&position).map(|p| format!("第{}优先", p + 1)).unwrap_or_else(|| "非优先".to_string()),
            target_player.game_id,
            position,
            target_player.age,
            target_player.ability,
            target_player.potential,
            target_player.calculate_market_value() / 10000,
            player_honors_str,
            performance_str,
            if player_strategy.wants_to_leave { "是" } else { "否" },
            player_strategy.expected_salary,
            player_strategy.expected_min_salary,
            player_strategy.expected_years,
            if player_strategy.requires_starter { "是" } else { "否" },
            player_strategy.preferred_teams.len(),
            our_priority.map(|p| format!("第{}", p)).unwrap_or_else(|| "不在列表中".to_string()),
            if is_target { "是" } else { "否" },
            max_offer.map(|o| format!("{}", o)).unwrap_or_else(|| "未设定".to_string()),
            competing_info,
            current_round
        )
    }

    /// 生成选手对报价的回应
    ///
    /// 选手收到报价后决定接受、拒绝、还价或继续等待
    pub async fn generate_player_response(
        &self,
        player: &Player,
        player_strategy: &PlayerTransferStrategy,
        offer: &crate::models::negotiation::Offer,
        other_offers: &[crate::models::negotiation::Offer],
        current_round: u8,
        max_rounds: u8,
        player_honors: Option<&PlayerHonorInfo>,
        player_performance: Option<&PlayerPerformanceInfo>,
    ) -> Result<crate::models::negotiation::LLMResponseDecision, String> {
        use crate::models::negotiation::LLMResponseDecision;

        if !self.is_configured() {
            return Err("LLM API Key 未配置".to_string());
        }

        let system_prompt = self.build_player_response_system_prompt();
        let user_prompt = self.build_player_response_user_prompt(
            player,
            player_strategy,
            offer,
            other_offers,
            current_round,
            max_rounds,
            player_honors,
            player_performance,
        );

        log::info!("调用 LLM 生成选手 {} 对 {} 报价的回应", player.game_id, offer.from_team_name);

        let response_text = match self.config.provider {
            LLMProvider::Claude => self.call_claude(&system_prompt, &user_prompt).await?,
            _ => self.call_openai_compatible(&system_prompt, &user_prompt).await?,
        };

        let cleaned_response = clean_json_response(&response_text);
        serde_json::from_str(&cleaned_response)
            .map_err(|e| format!("解析选手回应响应失败: {}\\n原始响应: {}", e, response_text))
    }

    /// 构建选手回应的 System Prompt
    fn build_player_response_system_prompt(&self) -> String {
        r#"你是一个电竞选手的决策系统。你需要帮助选手决定如何回应收到的报价。

## 回应类型

1. **ACCEPT** - 接受报价
   - 报价满足或超过期望
   - 球队在偏好列表中
   - 条件符合要求（首发等）

2. **REJECT** - 拒绝报价
   - 报价远低于最低接受线
   - 球队完全不在考虑范围
   - 条件完全不满足

3. **COUNTER** - 还价
   - 报价接近但未达预期
   - 可以通过调整达成一致
   - 需要提出具体的还价条件

4. **WAIT** - 继续等待
   - 还有更好的机会
   - 其他心仪球队尚未出价
   - 时间还充裕

## 决策因素

### 1. 报价匹配度
- 薪资 vs 期望薪资
- 薪资 vs 最低接受薪资
- 合同年限 vs 期望年限
- 首发保证 vs 首发要求

### 2. 球队吸引力
- 是否在偏好列表中
- 在列表中的排名
- 球队实力和争冠前景

### 3. 时间压力
- 当前轮次 vs 最大轮次
- 是否还有其他报价
- 后续是否可能有更好报价

### 4. 个人特质
- 忠诚度影响谈判策略
- 高忠诚选手更愿意接受
- 雇佣兵型更倾向高价

## 输出要求

请严格按照以下 JSON 格式返回，**必须包含 analysis_steps**：

```json
{
  "response_type": "COUNTER",
  "counter_salary": 220,
  "counter_years": 2,
  "counter_starter": true,
  "reasoning": "详细的决策理由...",
  "analysis_steps": [
    {
      "step_name": "报价分析",
      "data_used": "报价: 180万/年, 期望: 200万/年, 最低: 160万/年",
      "threshold": "接受阈值: 期望的90%",
      "result": "报价在可接受范围内但未达期望",
      "impact": "可以考虑还价"
    },
    {
      "step_name": "球队评估",
      "data_used": "报价球队: EDG, 我的偏好排名: 第2",
      "threshold": "偏好前3为优选球队",
      "result": "球队在优选范围内",
      "impact": "增加接受意愿 +20%"
    },
    {
      "step_name": "竞争分析",
      "data_used": "还有2个报价待处理, 最高230万",
      "threshold": "",
      "result": "有更好的选择",
      "impact": "可以继续观望或还价"
    },
    {
      "step_name": "时间评估",
      "data_used": "当前第2轮, 最多5轮",
      "threshold": "第4轮后应尽快决定",
      "result": "时间充裕",
      "impact": "不需要着急接受"
    },
    {
      "step_name": "最终决策",
      "data_used": "综合报价、球队、竞争、时间分析",
      "threshold": "",
      "result": "决定还价至220万/年",
      "impact": "提高报价同时保持吸引力"
    }
  ]
}
```

注意：
1. response_type 必须是 ACCEPT/REJECT/COUNTER/WAIT 之一
2. counter_* 字段只在 response_type=COUNTER 时需要
3. analysis_steps 必须包含至少 5 个步骤
4. 只输出 JSON，不要其他内容"#.to_string()
    }

    /// 构建选手回应的 User Prompt
    fn build_player_response_user_prompt(
        &self,
        player: &Player,
        player_strategy: &PlayerTransferStrategy,
        offer: &crate::models::negotiation::Offer,
        other_offers: &[crate::models::negotiation::Offer],
        current_round: u8,
        max_rounds: u8,
        player_honors: Option<&PlayerHonorInfo>,
        player_performance: Option<&PlayerPerformanceInfo>,
    ) -> String {
        let position = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        // 忠诚度类型
        let loyalty_type = if player.loyalty > 90 {
            "忠心耿耿"
        } else if player.loyalty > 70 {
            "忠诚"
        } else if player.loyalty > 40 {
            "中立"
        } else if player.loyalty > 20 {
            "机会主义"
        } else {
            "雇佣兵"
        };

        // 格式化偏好球队
        let preferred_teams_str = player_strategy.preferred_teams.iter()
            .map(|t| format!("{}. {} ({})", t.priority, t.team_name, t.reason.display_name()))
            .collect::<Vec<_>>()
            .join("\n");

        // 检查当前报价球队在偏好列表中的位置
        let offer_team_priority = player_strategy.get_team_priority(offer.from_team_id);

        // 格式化其他报价
        let other_offers_str = if other_offers.is_empty() {
            "暂无其他报价".to_string()
        } else {
            other_offers.iter()
                .map(|o| format!(
                    "- {} (薪资{}万/年, {}年, 首发{})",
                    o.from_team_name,
                    o.salary_offer,
                    o.contract_years,
                    if o.guarantee_starter { "保证" } else { "不保证" }
                ))
                .collect::<Vec<_>>()
                .join("\n")
        };

        // 格式化荣誉
        let honors_str = if let Some(h) = player_honors {
            let total = h.worlds_championships + h.msi_championships + h.regional_championships;
            if total > 0 {
                format!(
                    "世界赛冠军x{}, MSI冠军x{}, 赛区冠军x{}, MVP x{}",
                    h.worlds_championships, h.msi_championships, h.regional_championships,
                    h.tournament_mvps + h.finals_mvps + h.yearly_mvps
                )
            } else {
                "尚无重大荣誉".to_string()
            }
        } else {
            "暂无荣誉数据".to_string()
        };

        // 格式化表现
        let performance_str = if let Some(p) = player_performance {
            format!(
                "出场{}场, 平均表现{:.1} ({}), 稳定性{:.0}",
                p.games_played, p.avg_performance, p.performance_tier, p.consistency_score
            )
        } else {
            "暂无赛季表现数据".to_string()
        };

        format!(
            r#"## 选手信息
- 姓名: {}
- 位置: {}
- 年龄: {}岁
- 能力: {}
- 忠诚度: {} ({})
- 荣誉: {}
- 本赛季表现: {}

## 转会意愿
- 期望薪资: {}万/年
- 最低接受: {}万/年
- 期望年限: {}年
- 要求首发: {}

## 偏好球队列表
{}

## 当前收到的报价（待回应）
- 球队: {}
- 薪资: {}万/年
- 年限: {}年
- 首发保证: {}
- 签字费: {}万
- 报价轮次: 第{}轮
- 该球队在我偏好中的排名: {}

## 其他已收到的报价
{}

## 时间状态
- 当前轮次: 第{}轮
- 最大轮次: {}轮
- 剩余轮次: {}轮

请决定如何回应这个报价。"#,
            player.game_id,
            position,
            player.age,
            player.ability,
            player.loyalty,
            loyalty_type,
            honors_str,
            performance_str,
            player_strategy.expected_salary,
            player_strategy.expected_min_salary,
            player_strategy.expected_years,
            if player_strategy.requires_starter { "是" } else { "否" },
            if preferred_teams_str.is_empty() { "无特定偏好" } else { &preferred_teams_str },
            offer.from_team_name,
            offer.salary_offer,
            offer.contract_years,
            if offer.guarantee_starter { "是" } else { "否" },
            offer.signing_bonus,
            offer.round,
            offer_team_priority.map(|p| format!("第{}", p)).unwrap_or_else(|| "不在列表中".to_string()),
            other_offers_str,
            current_round,
            max_rounds,
            max_rounds.saturating_sub(current_round)
        )
    }

    /// 基于规则的邀约评估（无LLM时的后备方案）
    fn rule_based_offer_evaluation(
        &self,
        player: &Player,
        strategy: &PlayerTransferStrategy,
        offer: &TeamOffer,
    ) -> OfferEvaluation {
        let mut accept = true;
        let mut reasons = Vec::new();

        // 检查薪资
        let min_salary = strategy.get_min_salary_for_team(offer.team_id);
        let salary_score = if offer.salary_offer >= strategy.expected_salary {
            100
        } else if offer.salary_offer >= min_salary {
            ((offer.salary_offer as f64 / strategy.expected_salary as f64) * 100.0) as u8
        } else {
            accept = false;
            reasons.push(format!("薪资{}万低于最低要求{}万", offer.salary_offer, min_salary));
            30
        };

        // 检查首发要求
        let role_score = if strategy.requires_starter && !offer.starter_guarantee {
            accept = false;
            reasons.push("要求首发但邀约未保证".to_string());
            20
        } else if offer.starter_guarantee {
            100
        } else {
            60
        };

        // 检查球队偏好
        let team_score = if let Some(priority) = strategy.get_team_priority(offer.team_id) {
            100 - (priority.saturating_sub(1) * 15).min(60)
        } else {
            // 不在偏好列表中，但可能仍会考虑
            40
        };

        // 检查合同年限
        if offer.contract_years < strategy.expected_years {
            reasons.push(format!("合同{}年短于期望{}年", offer.contract_years, strategy.expected_years));
        }

        let overall_score = (salary_score as u32 * 4 + team_score as u32 * 3 + role_score as u32 * 3) / 10;

        // 综合评估
        if overall_score < 50 {
            accept = false;
        }

        let reasoning = if accept {
            format!("邀约符合期望：薪资{}万/年，{}",
                offer.salary_offer,
                if offer.starter_guarantee { "保证首发" } else { "替补位置" })
        } else {
            format!("拒绝原因：{}", reasons.join("；"))
        };

        OfferEvaluation {
            accept,
            confidence: 80,
            reasoning,
            salary_score,
            team_score,
            role_score,
            overall_score: overall_score as u8,
        }
    }

    /// 评估多个报价并选择最佳的一个
    ///
    /// 当选手收到多个球队的报价时，调用 LLM 综合评估所有报价，
    /// 必须选择其中一个接受（不能全部拒绝）
    pub async fn evaluate_multiple_offers(
        &self,
        player: &Player,
        strategy: &PlayerTransferStrategy,
        offers: &[TeamOffer],
        player_honors: Option<&PlayerHonorInfo>,
        player_performance: Option<&PlayerPerformanceInfo>,
    ) -> Result<MultipleOffersDecision, String> {
        if offers.is_empty() {
            return Ok(MultipleOffersDecision {
                chosen_offer_id: None,
                reasoning: "没有收到任何报价".to_string(),
                evaluations: vec![],
            });
        }

        // 只有一个报价，必须接受（新规则：不能拒绝）
        if offers.len() == 1 {
            let reasoning = format!(
                "只收到 {} 的报价，{}万/年，{}年合同。虽然{}，但作为唯一选择，决定接受。",
                offers[0].team_name,
                offers[0].salary_offer,
                offers[0].contract_years,
                if offers[0].salary_offer >= strategy.expected_min_salary {
                    "条件可以接受"
                } else {
                    "薪资低于期望"
                }
            );
            return Ok(MultipleOffersDecision {
                chosen_offer_id: Some(offers[0].id), // 必须接受
                reasoning,
                evaluations: vec![(offers[0].id, OfferEvaluation {
                    accept: true,
                    confidence: 70,
                    reasoning: "唯一报价，接受".to_string(),
                    salary_score: 0,
                    team_score: 0,
                    role_score: 0,
                    overall_score: 70,
                })],
            });
        }

        if !self.is_configured() {
            // 无LLM时使用规则评估
            return Ok(self.rule_based_multiple_offers_evaluation(player, strategy, offers));
        }

        // 格式化荣誉信息
        let honors_str = if let Some(h) = player_honors {
            let mut parts = Vec::new();
            if h.worlds_championships > 0 { parts.push(format!("世界赛冠军x{}", h.worlds_championships)); }
            if h.msi_championships > 0 { parts.push(format!("MSI冠军x{}", h.msi_championships)); }
            if h.regional_championships > 0 { parts.push(format!("赛区冠军x{}", h.regional_championships)); }
            let mvps = h.tournament_mvps + h.finals_mvps + h.yearly_mvps;
            if mvps > 0 { parts.push(format!("MVP x{}", mvps)); }
            if parts.is_empty() { "暂无荣誉".to_string() } else { parts.join(", ") }
        } else {
            "暂无荣誉数据".to_string()
        };

        // 格式化表现信息
        let performance_str = if let Some(p) = player_performance {
            format!(
                "本赛季出场{}场，平均表现{:.1}分（{}），稳定性{:.0}",
                p.games_played, p.avg_performance, p.performance_tier, p.consistency_score
            )
        } else {
            "暂无赛季表现数据".to_string()
        };

        let system_prompt = r#"你是一个电竞选手的决策助手。选手收到了多份邀约，你需要帮他评估所有邀约并**必须选择其中一个**。

## 重要规则（必须严格遵守）
1. **你必须从收到的报价中选择一个接受！不能全部拒绝！**
2. **chosen_team_id 必须填写你最终选择的球队ID！**
3. **reasoning 和 chosen_team_id 必须一致！！！**
   - 如果 reasoning 中说"A队是最佳选择"，那 chosen_team_id 就必须是 A队的ID
   - 绝对不允许 reasoning 说选A，chosen_team_id 却填B

## 输出JSON格式
{
  "chosen_team_id": 123,
  "reasoning": "分析过程...最终选择 XXX 队（ID:123）",
  "evaluations": [
    {
      "team_id": 123,
      "team_name": "球队名",
      "overall_score": 1-100,
      "pros": ["优点1", "优点2"],
      "cons": ["缺点1", "缺点2"]
    }
  ]
}

## 决策原则（按优先级排序）
1. **偏好球队优先**：如果报价来自偏好列表中的球队，优先考虑
2. **薪资水平**：满足最低接受薪资的报价更好
3. **首发保证**：如果要求首发，优先选择能保证首发的报价
4. **合同年限**：与期望年限接近的报价更好
5. **球队实力**：考虑球队的争冠能力和发展前景

## 荣誉和表现对决策的影响
- 有冠军经历的选手倾向选择有争冠能力的球队
- 表现优秀的选手期望得到更好的待遇
- MVP获得者希望获得核心地位和首发保证

## 重要提醒
- reasoning 最后必须明确写出选择了哪个球队
- chosen_team_id 必须和 reasoning 中的结论完全一致
- 例如：reasoning 写"综合考虑，选择 T1"，chosen_team_id 就必须是 T1 的 team_id

注意：chosen_team_id **不能为 null**！只输出JSON，不要其他内容"#;

        // 构建所有报价的描述
        let offers_desc: Vec<String> = offers.iter().enumerate().map(|(i, o)| {
            let in_preferred = strategy.preferred_teams.iter()
                .position(|t| t.team_id == o.team_id)
                .map(|p| format!("（偏好第{}位）", p + 1))
                .unwrap_or_default();
            format!(
                "{}. {} [team_id={}] {}\n   - 年薪: {}万/年\n   - 合同: {}年\n   - 首发保证: {}\n   - 转会费: {}万",
                i + 1, o.team_name, o.team_id, in_preferred,
                o.salary_offer, o.contract_years,
                if o.starter_guarantee { "是" } else { "否" },
                o.transfer_fee
            )
        }).collect();

        let user_prompt = format!(
            r#"## 选手信息
- 姓名: {}
- 能力值: {}
- 年龄: {}
- 荣誉: {}
- 本赛季表现: {}

## 选手期望
- 期望薪资: {}万/年
- 最低接受: {}万/年
- 期望年限: {}年
- 要求首发: {}
- 偏好球队: {}

## 收到的所有邀约（共{}个）
{}

## 任务
请综合评估所有邀约，**必须选择其中一个最好的**！
即使条件不够理想，也要选择相对最优的报价。"#,
            player.game_id, player.ability, player.age,
            honors_str, performance_str,
            strategy.expected_salary,
            strategy.expected_min_salary,
            strategy.expected_years,
            if strategy.requires_starter { "是" } else { "否" },
            if strategy.preferred_teams.is_empty() {
                "无特定偏好".to_string()
            } else {
                strategy.preferred_teams.iter()
                    .map(|t| format!("{}({})", t.team_name, t.priority))
                    .collect::<Vec<_>>()
                    .join(", ")
            },
            offers.len(),
            offers_desc.join("\n\n")
        );

        let response_text = match self.config.provider {
            LLMProvider::Claude => self.call_claude(system_prompt, &user_prompt).await?,
            _ => self.call_openai_compatible(system_prompt, &user_prompt).await?,
        };

        let cleaned_response = clean_json_response(&response_text);
        let llm_response: LLMMultipleOffersResponse = serde_json::from_str(&cleaned_response)
            .map_err(|e| format!("解析多报价评估响应失败: {}\n原始响应: {}", e, cleaned_response))?;

        // 转换评估结果，并找出评分最高的球队
        let mut best_scoring_team: Option<(u64, u8)> = None; // (team_id, score)
        let evaluations: Vec<(u64, OfferEvaluation)> = llm_response.evaluations.iter().filter_map(|eval| {
            offers.iter().find(|o| o.team_id == eval.team_id).map(|o| {
                // 记录评分最高的球队
                if best_scoring_team.map(|(_, score)| eval.overall_score > score).unwrap_or(true) {
                    best_scoring_team = Some((eval.team_id, eval.overall_score));
                }

                (o.id, OfferEvaluation {
                    accept: llm_response.chosen_team_id == Some(eval.team_id),
                    confidence: 80,
                    reasoning: format!("优点: {}; 缺点: {}",
                        eval.pros.join("、"),
                        eval.cons.join("、")),
                    salary_score: 0, // LLM 返回的是综合评分
                    team_score: 0,
                    role_score: 0,
                    overall_score: eval.overall_score,
                })
            })
        }).collect();

        // 验证 LLM 返回的选择是否有效
        // 注意：LLM 可能返回 team_id，我们需要检查它是否在报价列表中
        let chosen_team_id = llm_response.chosen_team_id.and_then(|team_id| {
            // 检查这个 team_id 是否在报价列表中
            if offers.iter().any(|o| o.team_id == team_id) {
                log::info!("LLM 选择的 team_id {} 有效", team_id);
                Some(team_id)
            } else {
                log::warn!("LLM 返回的 chosen_team_id {} 不在报价列表中", team_id);
                None
            }
        });

        // 最终选择：优先使用 LLM 的选择，只有在无效时才使用后备
        let final_chosen_team_id = if chosen_team_id.is_some() {
            // LLM 返回了有效的选择，直接使用，不要覆盖！
            chosen_team_id
        } else {
            // LLM 选择无效，使用评分最高的球队作为后备
            if let Some((team_id, score)) = best_scoring_team {
                log::warn!("LLM 选择无效，使用评分最高的球队: team_id={}, score={}", team_id, score);
                Some(team_id)
            } else {
                // 最后的后备：选择第一个报价
                offers.first().map(|o| {
                    log::warn!("无有效评分，选择第一个报价: {}", o.team_name);
                    o.team_id
                })
            }
        };

        // 找到选中的 offer_id
        let chosen_offer_id = final_chosen_team_id.and_then(|team_id| {
            offers.iter().find(|o| o.team_id == team_id).map(|o| o.id)
        });

        // 只有在 LLM 选择被覆盖时才添加修正说明
        let final_reasoning = if chosen_team_id.is_none() && final_chosen_team_id.is_some() {
            if let Some(team_id) = final_chosen_team_id {
                let team_name = offers.iter()
                    .find(|o| o.team_id == team_id)
                    .map(|o| o.team_name.clone())
                    .unwrap_or_else(|| format!("ID:{}", team_id));
                format!("{}\n\n[系统修正] LLM选择无效，根据评估结果选择: {}", llm_response.reasoning, team_name)
            } else {
                llm_response.reasoning
            }
        } else {
            // LLM 选择有效，直接使用原始 reasoning
            llm_response.reasoning
        };

        Ok(MultipleOffersDecision {
            chosen_offer_id,
            reasoning: final_reasoning,
            evaluations,
        })
    }

    /// 基于规则的多报价评估（无LLM时的后备方案）
    fn rule_based_multiple_offers_evaluation(
        &self,
        player: &Player,
        strategy: &PlayerTransferStrategy,
        offers: &[TeamOffer],
    ) -> MultipleOffersDecision {
        let mut evaluations: Vec<(u64, OfferEvaluation)> = Vec::new();
        let mut best_offer: Option<(u64, u8)> = None; // (offer_id, score)

        for offer in offers {
            let eval = self.rule_based_offer_evaluation(player, strategy, offer);

            if eval.accept {
                // 如果这个报价可以接受，检查是否是最好的
                if best_offer.map(|(_, score)| eval.overall_score > score).unwrap_or(true) {
                    best_offer = Some((offer.id, eval.overall_score));
                }
            }

            evaluations.push((offer.id, eval));
        }

        let chosen_offer_id = best_offer.map(|(id, _)| id);

        let reasoning = if let Some(id) = chosen_offer_id {
            let chosen = offers.iter().find(|o| o.id == id).unwrap();
            format!("选择 {} 的报价：{}万/年，{}年合同",
                chosen.team_name, chosen.salary_offer, chosen.contract_years)
        } else {
            "所有报价均不满足期望条件".to_string()
        };

        MultipleOffersDecision {
            chosen_offer_id,
            reasoning,
            evaluations,
        }
    }

    // ==================== 自由市场评估 ====================

    /// 评估挖角市场（85+能力值选手）
    ///
    /// 球队评估所有可挖角的85+能力值选手，决定是否发起挖人报价
    pub async fn evaluate_poaching_market(
        &self,
        team: &Team,
        team_strategy: &AITransferStrategy,
        poachable_players: &[&Player],  // 所有85+且有合同的选手
        player_strategies: &HashMap<u64, PlayerTransferStrategy>,
        transferred_player_ids: &std::collections::HashSet<u64>,  // 已完成转会的
        already_offered_ids: &std::collections::HashSet<u64>,      // 已报价的
        current_round: u8,
        player_honors: &HashMap<u64, PlayerHonorInfo>,
        player_performances: &HashMap<u64, PlayerPerformanceInfo>,
    ) -> Result<crate::models::negotiation::LLMMarketEvaluation, String> {
        use crate::models::negotiation::{LLMMarketEvaluation, LLMPlayerEvaluation, LLMOfferDetails};

        if !self.is_configured() {
            return Err("LLM API Key 未配置".to_string());
        }

        // 过滤出可挖角的选手（未转会且未报价过的，且不是自己队的）
        let available_players: Vec<&Player> = poachable_players.iter()
            .filter(|p| {
                !transferred_player_ids.contains(&p.id) &&
                !already_offered_ids.contains(&p.id) &&
                p.team_id != Some(team.id)  // 不能挖自己队的选手
            })
            .copied()
            .collect();

        if available_players.is_empty() {
            return Ok(LLMMarketEvaluation {
                player_evaluations: vec![],
                chosen_player_id: None,
                offer_details: None,
                overall_reasoning: "没有可挖角的目标".to_string(),
            });
        }

        let system_prompt = self.build_poaching_market_system_prompt();
        let user_prompt = self.build_poaching_market_user_prompt(
            team,
            team_strategy,
            &available_players,
            player_strategies,
            current_round,
            player_honors,
            player_performances,
        );

        log::info!("调用 LLM 评估挖角市场，球队 {} 评估 {} 名85+选手", team.name, available_players.len());

        let response_text = match self.config.provider {
            LLMProvider::Claude => self.call_claude(&system_prompt, &user_prompt).await?,
            _ => self.call_openai_compatible(&system_prompt, &user_prompt).await?,
        };

        let cleaned_response = clean_json_response(&response_text);
        serde_json::from_str(&cleaned_response)
            .map_err(|e| format!("解析挖角市场评估响应失败: {}\n原始响应: {}", e, response_text))
    }

    /// 构建挖角市场评估的 System Prompt
    fn build_poaching_market_system_prompt(&self) -> String {
        r#"你是一个电竞俱乐部的 GM 助手。你需要帮助球队评估挖角市场上的所有85+能力值选手，并决定是否发起挖人报价。

## 任务说明

你将看到市场上所有能力值85+的有合同选手（来自其他球队）。你需要：
1. **逐一评估每位选手**，考虑他们是否值得挖角
2. **给出适配度评分**（0-100）
3. **决定是否考虑报价**
4. **如果决定报价，选择最优目标并制定报价方案**

## 评估因素

### 1. 位置需求
- 球队在该位置的补强需求（查看策略中的 targets）
- 该位置现有人员的能力水平

### 2. 选手价值
- 能力值和潜力（85+都是顶级选手）
- 年龄和巅峰期
- 赛季表现数据
- 荣誉记录

### 3. 成本因素（重要！挖角需要支付转会费）
- **转会费**：约等于选手市场价值（身价）
- **年薪**：与原合同一致（不需要额外涨薪）
- 球队的剩余预算是否足够
- 总成本 = 转会费 + 年薪总额

### 4. 竞争态势
- 选手的离队意愿（是否想离开当前球队）
- 选手的偏好球队是谁
- 我们是否在选手的偏好列表中

### 5. 挖角难度
- 选手的忠诚度（高忠诚度难挖）
- 选手对当前球队的满意度
- 原球队是否愿意放人

## 输出要求

请严格按照以下 JSON 格式返回：

```json
{
  "player_evaluations": [
    {
      "player_id": 123,
      "fit_score": 75,
      "consider_offer": true,
      "evaluation": "详细的评估理由...",
      "rejection_reason": null
    },
    {
      "player_id": 456,
      "fit_score": 30,
      "consider_offer": false,
      "evaluation": "评估说明...",
      "rejection_reason": "成本过高，预算不足"
    }
  ],
  "chosen_player_id": 123,
  "offer_details": {
    "salary_offer": 200,
    "contract_years": 2,
    "guarantee_starter": true,
    "signing_bonus": 0,
    "reasoning": "决定挖角该选手的理由..."
  },
  "overall_reasoning": "整体挖角市场评估思路和最终决策理由..."
}
```

注意：
1. **必须评估所有提供的85+选手**，每个选手都要有评估结果
2. `chosen_player_id` 和 `offer_details` 可以为 null（如果决定不报价）
3. `salary_offer` 和 `signing_bonus` 单位是万元
4. 只输出 JSON，不要其他内容
5. **考虑转会费成本**：挖角比自由市场贵很多！"#.to_string()
    }

    /// 构建挖角市场评估的 User Prompt
    fn build_poaching_market_user_prompt(
        &self,
        team: &Team,
        team_strategy: &AITransferStrategy,
        available_players: &[&Player],
        player_strategies: &HashMap<u64, PlayerTransferStrategy>,
        current_round: u8,
        player_honors: &HashMap<u64, PlayerHonorInfo>,
        player_performances: &HashMap<u64, PlayerPerformanceInfo>,
    ) -> String {
        // 格式化球队策略中的目标位置需求
        let position_needs: Vec<String> = team_strategy.targets.iter()
            .take(5)
            .map(|t| {
                format!("- {} (ID:{}): 优先级{}, 最高出价{}万",
                    t.player_name, t.player_id, t.priority, t.max_offer)
            })
            .collect();

        let position_needs_str = if position_needs.is_empty() {
            "无特定目标".to_string()
        } else {
            position_needs.join("\n")
        };

        // 格式化所有可挖角选手信息
        let players_desc: Vec<String> = available_players.iter().map(|player| {
            let position = player.position
                .map(|p| format!("{:?}", p).to_uppercase())
                .unwrap_or_else(|| "UNKNOWN".to_string());

            let market_value = player.calculate_market_value() / 10000;  // 身价（万）
            let transfer_fee = market_value;  // 转会费约等于身价

            // 获取选手策略
            let strategy_info = player_strategies.get(&player.id)
                .map(|s| {
                    let wants_leave = if s.wants_to_leave { "想离队" } else { "不想离队" };
                    let preferred = if s.preferred_teams.is_empty() {
                        "无偏好".to_string()
                    } else {
                        s.preferred_teams.iter()
                            .take(3)
                            .map(|t| t.team_name.clone())
                            .collect::<Vec<_>>()
                            .join("、")
                    };
                    let our_priority = s.get_team_priority(team.id)
                        .map(|p| format!("我们在偏好第{}位", p))
                        .unwrap_or_else(|| "我们不在偏好列表中".to_string());
                    format!("{}, 期望{}万/年, 偏好: {} ({})",
                        wants_leave, s.expected_salary, preferred, our_priority)
                })
                .unwrap_or_else(|| "无策略数据".to_string());

            // 获取荣誉信息
            let honors_str = player_honors.get(&player.id)
                .map(|h| {
                    let mut parts = Vec::new();
                    if h.worlds_championships > 0 { parts.push(format!("世界冠军x{}", h.worlds_championships)); }
                    if h.msi_championships > 0 { parts.push(format!("MSI冠军x{}", h.msi_championships)); }
                    if h.regional_championships > 0 { parts.push(format!("赛区冠军x{}", h.regional_championships)); }
                    let mvps = h.tournament_mvps + h.finals_mvps + h.yearly_mvps;
                    if mvps > 0 { parts.push(format!("MVP x{}", mvps)); }
                    if parts.is_empty() { "无荣誉".to_string() } else { parts.join(", ") }
                })
                .unwrap_or_else(|| "无荣誉数据".to_string());

            // 获取表现信息
            let perf_str = player_performances.get(&player.id)
                .map(|p| format!("{}场, 平均{:.1}分({})",
                    p.games_played, p.avg_performance, p.performance_tier))
                .unwrap_or_else(|| "无表现数据".to_string());

            format!(
                "【选手 ID:{}】{}
  当前球队: {} | 位置: {} | 能力: {} | 潜力: {} | 年龄: {}
  身价: {}万 | 预估转会费: {}万 | 当前薪资: {}万/年
  转会意愿: {}
  荣誉: {}
  表现: {}",
                player.id, player.game_id,
                player.team_id.map(|t| format!("ID:{}", t)).unwrap_or("无".to_string()),
                position, player.ability, player.potential, player.age,
                market_value,
                transfer_fee,
                player.salary / 10000,
                strategy_info,
                honors_str,
                perf_str
            )
        }).collect();

        format!(
            r#"## 球队信息
- 球队名称: {}
- 剩余预算: {}万
- 当前轮次: 第{}轮（挖角阶段）

## 球队转会策略目标
{}

## 策略分析
{}

## 可挖角选手市场（85+能力值，共{}人）

{}

## 任务
请逐一评估以上所有选手，给出适配度评分和评估理由。
如果有合适的目标，请选择最优的一位并给出挖人报价方案。

**重要提示**：
1. 挖角需要支付转会费（约等于选手身价）
2. 总成本 = 转会费 + 年薪×合同年限
3. 确保预算足够支付转会费
4. 优先考虑想离队的选手（挖角难度较低）"#,
            team.name,
            team.balance / 10000,
            current_round,
            position_needs_str,
            team_strategy.reasoning,
            available_players.len(),
            players_desc.join("\n\n")
        )
    }

    /// 评估所有自由市场选手并决定是否报价
    ///
    /// 球队会评估市场上的所有可用选手，生成详细的思考过程，
    /// 然后决定是否向某一位选手发出报价
    pub async fn evaluate_free_market(
        &self,
        team: &Team,
        team_strategy: &AITransferStrategy,
        free_agents: &[FreeAgentInfo],
        player_strategies: &HashMap<u64, PlayerTransferStrategy>,
        signed_player_ids: &std::collections::HashSet<u64>,
        already_offered_ids: &std::collections::HashSet<u64>,
        current_round: u8,
        player_honors: &HashMap<u64, PlayerHonorInfo>,
        player_performances: &HashMap<u64, PlayerPerformanceInfo>,
    ) -> Result<crate::models::negotiation::LLMMarketEvaluation, String> {
        use crate::models::negotiation::{LLMMarketEvaluation, LLMPlayerEvaluation, LLMOfferDetails};

        if !self.is_configured() {
            return Err("LLM API Key 未配置".to_string());
        }

        // 过滤出可用的自由球员（未签约且未报价过的）
        let available_agents: Vec<&FreeAgentInfo> = free_agents.iter()
            .filter(|fa| {
                !signed_player_ids.contains(&fa.player.id) &&
                !already_offered_ids.contains(&fa.player.id)
            })
            .collect();

        if available_agents.is_empty() {
            return Ok(LLMMarketEvaluation {
                player_evaluations: vec![],
                chosen_player_id: None,
                offer_details: None,
                overall_reasoning: "市场上没有可追逐的目标".to_string(),
            });
        }

        let system_prompt = self.build_free_market_system_prompt();
        let user_prompt = self.build_free_market_user_prompt(
            team,
            team_strategy,
            &available_agents,
            player_strategies,
            current_round,
            player_honors,
            player_performances,
        );

        log::info!("调用 LLM 评估自由市场，球队 {} 评估 {} 名选手", team.name, available_agents.len());

        let response_text = match self.config.provider {
            LLMProvider::Claude => self.call_claude(&system_prompt, &user_prompt).await?,
            _ => self.call_openai_compatible(&system_prompt, &user_prompt).await?,
        };

        let cleaned_response = clean_json_response(&response_text);
        serde_json::from_str(&cleaned_response)
            .map_err(|e| format!("解析自由市场评估响应失败: {}\n原始响应: {}", e, response_text))
    }

    /// 构建自由市场评估的 System Prompt
    fn build_free_market_system_prompt(&self) -> String {
        r#"你是一个电竞俱乐部的 GM 助手。你需要帮助球队评估自由市场上的所有可用选手，并决定是否向某位选手发出报价。

## 任务说明

你将看到自由市场上所有可追逐的选手信息。你需要：
1. **逐一评估每位选手**，考虑他们是否适合球队
2. **给出适配度评分**（0-100）
3. **决定是否考虑报价**
4. **如果决定报价，选择最优目标并制定报价方案**

## 评估因素

### 1. 位置需求
- 球队在该位置的补强需求（查看策略中的 targets）
- 该位置现有人员的能力水平

### 2. 选手价值
- 能力值和潜力
- 年龄和发展空间
- 赛季表现数据
- 荣誉记录

### 3. 价格因素
- 选手的期望薪资
- 球队的剩余预算
- 是否物有所值

### 4. 竞争态势
- 选手的偏好球队是谁
- 我们是否在选手的偏好列表中

## 输出要求

请严格按照以下 JSON 格式返回：

```json
{
  "player_evaluations": [
    {
      "player_id": 123,
      "fit_score": 75,
      "consider_offer": true,
      "evaluation": "详细的评估理由...",
      "rejection_reason": null
    },
    {
      "player_id": 456,
      "fit_score": 30,
      "consider_offer": false,
      "evaluation": "评估说明...",
      "rejection_reason": "位置不急需，且薪资要求过高"
    }
  ],
  "chosen_player_id": 123,
  "offer_details": {
    "salary_offer": 200,
    "contract_years": 2,
    "guarantee_starter": true,
    "signing_bonus": 0,
    "reasoning": "决定向该选手报价的理由..."
  },
  "overall_reasoning": "整体市场评估思路和最终决策理由..."
}
```

注意：
1. **必须评估所有提供的选手**，每个选手都要有评估结果
2. `chosen_player_id` 和 `offer_details` 可以为 null（如果决定不报价）
3. `salary_offer` 和 `signing_bonus` 单位是万元
4. 只输出 JSON，不要其他内容"#.to_string()
    }

    /// 构建自由市场评估的 User Prompt
    fn build_free_market_user_prompt(
        &self,
        team: &Team,
        team_strategy: &AITransferStrategy,
        available_agents: &[&FreeAgentInfo],
        player_strategies: &HashMap<u64, PlayerTransferStrategy>,
        current_round: u8,
        player_honors: &HashMap<u64, PlayerHonorInfo>,
        player_performances: &HashMap<u64, PlayerPerformanceInfo>,
    ) -> String {
        // 格式化球队策略中的目标位置需求
        let position_needs: Vec<String> = team_strategy.targets.iter()
            .take(5)
            .map(|t| {
                format!("- {} (ID:{}): 优先级{}, 最高出价{}万",
                    t.player_name, t.player_id, t.priority, t.max_offer)
            })
            .collect();

        let position_needs_str = if position_needs.is_empty() {
            "无特定目标".to_string()
        } else {
            position_needs.join("\n")
        };

        // 格式化所有可用选手信息
        let players_desc: Vec<String> = available_agents.iter().map(|fa| {
            let player = &fa.player;
            let position = player.position
                .map(|p| format!("{:?}", p).to_uppercase())
                .unwrap_or_else(|| "UNKNOWN".to_string());

            // 获取选手策略
            let strategy_info = player_strategies.get(&player.id)
                .map(|s| {
                    let preferred = if s.preferred_teams.is_empty() {
                        "无偏好".to_string()
                    } else {
                        s.preferred_teams.iter()
                            .take(3)
                            .map(|t| t.team_name.clone())
                            .collect::<Vec<_>>()
                            .join("、")
                    };
                    let our_priority = s.get_team_priority(team.id)
                        .map(|p| format!("我们在偏好第{}位", p))
                        .unwrap_or_else(|| "我们不在偏好列表中".to_string());
                    format!("期望{}万/年, 最低{}万, {}年合同, 偏好: {} ({})",
                        s.expected_salary, s.expected_min_salary, s.expected_years,
                        preferred, our_priority)
                })
                .unwrap_or_else(|| "无策略数据".to_string());

            // 获取荣誉信息
            let honors_str = player_honors.get(&player.id)
                .map(|h| {
                    let mut parts = Vec::new();
                    if h.worlds_championships > 0 { parts.push(format!("世界冠军x{}", h.worlds_championships)); }
                    if h.msi_championships > 0 { parts.push(format!("MSI冠军x{}", h.msi_championships)); }
                    if h.regional_championships > 0 { parts.push(format!("赛区冠军x{}", h.regional_championships)); }
                    let mvps = h.tournament_mvps + h.finals_mvps + h.yearly_mvps;
                    if mvps > 0 { parts.push(format!("MVP x{}", mvps)); }
                    if parts.is_empty() { "无荣誉".to_string() } else { parts.join(", ") }
                })
                .unwrap_or_else(|| "无荣誉数据".to_string());

            // 获取表现信息
            let perf_str = player_performances.get(&player.id)
                .map(|p| format!("{}场, 平均{:.1}分({})",
                    p.games_played, p.avg_performance, p.performance_tier))
                .unwrap_or_else(|| "无表现数据".to_string());

            format!(
                "【选手 ID:{}】{}\n  位置: {} | 能力: {} | 潜力: {} | 年龄: {}\n  市场价: {}万 | {}\n  荣誉: {}\n  表现: {}",
                player.id, player.game_id,
                position, player.ability, player.potential, player.age,
                fa.market_value / 10000,
                strategy_info,
                honors_str,
                perf_str
            )
        }).collect();

        format!(
            r#"## 球队信息
- 球队名称: {}
- 剩余预算: {}万
- 当前轮次: 第{}轮

## 球队转会策略目标
{}

## 策略分析
{}

## 自由市场可用选手（共{}人）

{}

## 任务
请逐一评估以上所有选手，给出适配度评分和评估理由。
如果有合适的目标，请选择最优的一位并给出报价方案。"#,
            team.name,
            team.balance / 10000,
            current_round,
            position_needs_str,
            team_strategy.reasoning,
            available_agents.len(),
            players_desc.join("\n\n")
        )
    }
}

/// 多报价评估决策结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleOffersDecision {
    /// 选中的报价ID（如果接受其中一个）
    pub chosen_offer_id: Option<u64>,
    /// 决策理由
    pub reasoning: String,
    /// 每个报价的评估结果
    pub evaluations: Vec<(u64, OfferEvaluation)>,
}

/// LLM 多报价评估响应
#[derive(Debug, Clone, Deserialize)]
struct LLMMultipleOffersResponse {
    chosen_team_id: Option<u64>,
    reasoning: String,
    evaluations: Vec<LLMOfferEval>,
}

#[derive(Debug, Clone, Deserialize)]
struct LLMOfferEval {
    team_id: u64,
    team_name: String,
    overall_score: u8,
    pros: Vec<String>,
    cons: Vec<String>,
}

/// 球队信息（用于选手策略生成）
#[derive(Debug, Clone, Serialize)]
pub struct TeamInfo {
    pub id: u64,
    pub name: String,
    pub region_name: String,
    pub avg_ability: f64,
    pub balance: i64,
    pub position_need: u32, // 对选手位置的需求度 (0-100)
    pub annual_points: u32, // 年度积分
    pub global_rank: u32,   // 全球排名 (1=最强, 0=未排名)
}

/// 选手荣誉信息（用于 LLM 策略生成）
#[derive(Debug, Clone, Default, Serialize)]
pub struct PlayerHonorInfo {
    /// 世界赛冠军数
    pub worlds_championships: u32,
    /// MSI 冠军数
    pub msi_championships: u32,
    /// 赛区冠军数
    pub regional_championships: u32,
    /// 赛事 MVP 数量
    pub tournament_mvps: u32,
    /// 决赛 MVP 数量
    pub finals_mvps: u32,
    /// 年度 MVP 数量
    pub yearly_mvps: u32,
    /// 最近获得的重要荣誉（用于描述）
    pub recent_honors: Vec<String>,
}

/// 选手赛季表现信息（用于 LLM 策略生成）
#[derive(Debug, Clone, Default, Serialize)]
pub struct PlayerPerformanceInfo {
    /// 本赛季出场次数
    pub games_played: u32,
    /// 平均影响力 (0-100)
    pub avg_impact: f64,
    /// 平均表现评分 (0-100)
    pub avg_performance: f64,
    /// 最佳表现
    pub best_performance: f64,
    /// 稳定性评分 (0-100)
    pub consistency_score: f64,
    /// 表现等级描述
    pub performance_tier: String,
    /// 与能力值的差异（正数表示超常发挥）
    pub ability_diff: f64,
}

/// 球队荣誉信息（用于 LLM 策略生成）
#[derive(Debug, Clone, Default, Serialize)]
pub struct TeamHonorInfo {
    /// 世界赛冠军数
    pub worlds_championships: u32,
    /// MSI 冠军数
    pub msi_championships: u32,
    /// 赛区冠军数（春季+夏季）
    pub regional_championships: u32,
    /// 是否为卫冕冠军
    pub is_defending_champion: bool,
    /// 最近赛季成绩（如"夏季赛第3名"）
    pub recent_results: Vec<String>,
    /// 球队核心选手获得的荣誉数量
    pub roster_total_mvps: u32,
    /// 是否有明星选手（能力90+）
    pub has_star_players: bool,
    /// 明星选手数量
    pub star_player_count: u32,
}

/// 选手荣誉摘要（用于球队策略生成）
#[derive(Debug, Clone, Serialize)]
pub struct RosterPlayerHonorSummary {
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub ability: u8,
    /// 是否为核心（首发）
    pub is_core: bool,
    /// 冠军数量
    pub championship_count: u32,
    /// MVP数量
    pub mvp_count: u32,
    /// 荣誉描述
    pub honor_summary: String,
}

/// 选手赛季表现摘要（用于球队策略生成）
#[derive(Debug, Clone, Serialize)]
pub struct PlayerPerformanceSummary {
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    /// 本赛季出场次数
    pub games_played: u32,
    /// 平均影响力（0-100）
    pub avg_impact: f64,
    /// 平均表现评分（0-100）
    pub avg_performance: f64,
    /// 最佳表现
    pub best_performance: f64,
    /// 稳定性评分（0-100，越高越稳定）
    pub consistency_score: f64,
    /// 表现等级描述
    pub performance_tier: String,
    /// 与能力值的差异（正数表示超常发挥）
    pub ability_diff: f64,
}

// ==================== 全局实例管理 ====================

use std::sync::RwLock;
use std::fs;
use std::path::PathBuf;
use std::env;
use once_cell::sync::Lazy;

/// 全局 LLM 配置
static LLM_CONFIG: Lazy<RwLock<Option<LLMConfig>>> = Lazy::new(|| {
    // 启动时尝试从文件加载配置
    RwLock::new(load_config_from_file())
});

/// 获取配置文件路径
fn get_config_file_path() -> PathBuf {
    // 使用 HOME 环境变量获取用户目录
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE")) // Windows 兼容
        .unwrap_or_else(|_| ".".to_string());

    let config_dir = PathBuf::from(home).join(".esport-manager");

    // 确保目录存在
    if !config_dir.exists() {
        let _ = fs::create_dir_all(&config_dir);
    }

    config_dir.join("llm_config.json")
}

/// 从文件加载配置
fn load_config_from_file() -> Option<LLMConfig> {
    let path = get_config_file_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<LLMConfig>(&content) {
                    Ok(mut config) => {
                        // 确保 max_tokens 至少为 8000，避免响应被截断
                        if config.max_tokens < 8000 {
                            log::info!("升级 max_tokens: {} -> 8000", config.max_tokens);
                            config.max_tokens = 8000;
                        }
                        log::info!("LLM 配置已从文件加载: {:?}", path);
                        Some(config)
                    }
                    Err(e) => {
                        log::warn!("解析 LLM 配置文件失败: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                log::warn!("读取 LLM 配置文件失败: {}", e);
                None
            }
        }
    } else {
        None
    }
}

/// 保存配置到文件
fn save_config_to_file(config: &LLMConfig) -> Result<(), String> {
    let path = get_config_file_path();
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    log::info!("LLM 配置已保存到: {:?}", path);
    Ok(())
}

/// 设置 LLM 配置（同时保存到文件）
pub fn set_llm_config(config: LLMConfig) {
    // 保存到文件
    if let Err(e) = save_config_to_file(&config) {
        log::error!("保存 LLM 配置失败: {}", e);
    }

    // 更新内存
    let mut guard = LLM_CONFIG.write().unwrap();
    *guard = Some(config);
}

/// 获取 LLM 配置
pub fn get_llm_config() -> Option<LLMConfig> {
    let guard = LLM_CONFIG.read().unwrap();
    guard.clone()
}

/// 检查 LLM 是否已配置
pub fn is_llm_configured() -> bool {
    let guard = LLM_CONFIG.read().unwrap();
    guard.as_ref().map(|c| !c.api_key.is_empty()).unwrap_or(false)
}

/// 创建 LLM 服务实例
pub fn create_llm_service() -> Option<LLMTransferService> {
    get_llm_config().map(LLMTransferService::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_default() {
        let config = LLMConfig::default();
        assert_eq!(config.provider, LLMProvider::OpenAI);
        assert_eq!(config.model, "gpt-4o-mini");
    }

    #[test]
    fn test_api_url() {
        let openai = LLMConfig::openai("test".to_string());
        assert!(openai.get_api_url().contains("openai.com"));

        let claude = LLMConfig::claude("test".to_string());
        assert!(claude.get_api_url().contains("anthropic.com"));
    }
}

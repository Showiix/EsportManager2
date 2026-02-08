# 转会系统

## 概述

转会系统管理选手的合同、转会和球员交易。采用8轮制的自动转会决策，由AI性格驱动。R7 完成后需手动确认关闭转会窗口。

## 转会窗口

转会窗口在每个赛季结束后开放：

```
赛季结束 → 转会窗口开放 → 8轮转会 → 确认关闭 → 选秀 → 新赛季开始
```

## 8轮转会流程

| 轮次 | 名称 | 说明 |
|------|------|------|
| 1 | 赛季结算 | 年龄/能力更新 |
| 2 | 双向评估 | 球队和选手互相评估 |
| 3 | 续约谈判 | 合同到期选手续约 |
| 4 | 自由球员竞标 | 对自由球员进行竞标 |
| 5 | 合同选手转会 | 在役选手转会 |
| 6 | 财务调整 | 处理财务问题 |
| 7 | 收尾补救 | 阵容不足的球队补强 |
| 8 | 选秀权拍卖 | 拍卖下届选秀权 |

### R8 选秀权拍卖算法

仅在选秀年（S2, S6, S10...）激活。最多 3 轮竞拍，佣金 5%。

#### 卖签决策（4因素相乘）

```
sell_prob = clamp(财务动机 × 签位留存 × 阵容系数 × 球队实力, 0, 0.90)
```

- **财务动机**: Bankrupt 0.70 → Wealthy 0.05
- **签位留存**: 状元签 0.10 → 13+号签 1.20（高签越不想卖）
- **阵容系数**: <5人 0.10, <7人 0.50, 7-8人 1.00, ≥9人 1.50
- **球队实力**: 弱队(<55)持高签(≤5) 0.50（保留重建），强队(>65)持高签 1.30（可出售回血）

#### 竞拍决策

- **预算**: Wealthy 40%, Healthy 30%, Tight 15%, Deficit 5%, Bankrupt 不参与
- **签位价值**: 14级梯度（100/92/85/78/72/65/58/52/45/40/35/30/25/20）
- **竞拍概率**: 签位价值/100 × 0.50 × 阵容需求 × 实力因素 × 财务信心 × 价格衰减 × 轮次系数
- **出价上限**: Wealthy ×1.5, Healthy ×1.3, Tight ×1.15, 其他 ×1.05

## 转会窗口关闭

R8 完成后，系统**不再自动关闭**转会窗口。需要手动确认关闭：

### 验证检查项

| 检查项 | issue_type | 触发条件 |
|--------|-----------|----------|
| 阵容过少 | `ROSTER_TOO_SMALL` | 活跃选手 < 5 |
| 阵容过多 | `ROSTER_TOO_LARGE` | 活跃选手 > 10 |
| 合同过期 | `INVALID_CONTRACT` | contract_end_season <= 当前赛季 |

### 关闭流程

```
R8 完成 → 点击"确认关闭转会窗口" → 验证检查
  ├─ 通过 → 标记 COMPLETED → 可推进到选秀
  └─ 不通过 → 显示问题列表
       ├─ 强制关闭 → 标记 COMPLETED
       └─ 返回修复
```

### 时间引擎集成

`game_flow.rs` 根据数据库实际状态判断转会期进度：

| 状态 | 条件 | PhaseStatus |
|------|------|-------------|
| 未初始化 | 无 transfer_window 记录 | NotInitialized |
| 进行中 | IN_PROGRESS 且 round > 0 | InProgress |
| 已完成 | COMPLETED | Completed |

推进到下一阶段前，`complete_and_advance` 会验证转会窗口必须为 COMPLETED 状态。

## 转会类型

| 类型 | 触发条件 | 说明 |
|------|----------|------|
| 合同到期 | 合同剩余0年 | 选手变为自由球员，无需转会费 |
| 主动求购 | AI判断需要补强 | 球队向其他球队报价 |
| 被动出售 | 财政困难/选手冗余 | 球队挂牌出售选手 |
| 退役 | 年龄过大/能力过低 | 选手离开职业圈 |
| 租借 | 替补选手外借 | 临时转会，赛季结束归还 |

## 转会意愿计算

### 匹配度 (match_score)

球队 AI 对选手的匹配评分（0-100），决定 R4/R5 中 offer 的优先级。

**9 个评估维度**，通过归一化加权计算：

| 维度 | 评分范围 | 说明 |
|------|---------|------|
| 能力匹配 | 10-100 | 按 ability 分段：90+=100, 80+=90, 75+=80... |
| 年龄匹配 | 40-100 | 受 AI 性格影响：youth_preference 偏好年轻，short_term_focus 偏好巅峰 |
| 财务匹配 | 0-100 | 基于 balance 的对数映射 |
| 位置需求度 | 5-100 | 0人=100, 1人=40, 2人=15, 3+=5 |
| 提升度 | 25-100 | 选手 vs 球队该位置最强选手的能力差 |
| 排名因子 | ×0.9~×1.1 | 弱队更渴望强援 |
| 潜力因素 | 40-100 | 23岁以下更看重；权重受 youth_preference 影响（0.05~0.15） |
| 稳定性因素 | 40-100 | 权重受 risk_tolerance 影响（0.05~0.10） |
| 成长标签 | ×0.95~×1.08 | Genius×1.08, Normal×1.0, Ordinary×0.95 |

**AI 性格权重分配**：

```
w_ability   = 0.25 + 0.15 × short_term_focus        // 0.25 ~ 0.40
w_age       = 0.15 + 0.15 × max(youth_pref, stf)    // 0.15 ~ 0.30
w_finance   = 0.10 + 0.10 × bargain_hunting          // 0.10 ~ 0.20
w_need      = 0.20                                    // 固定
w_upgrade   = 0.15 + 0.10 × short_term_focus         // 0.15 ~ 0.25
w_potential = 0.05 + 0.10 × youth_preference          // 0.05 ~ 0.15
w_stability = 0.05 + 0.05 × (1 - risk_tolerance)     // 0.05 ~ 0.10

match_score = (∑ score_i × w_i) / ∑ w_i × rank_factor × tag_multiplier
```

### 意愿度 (willingness)

选手是否接受报价的判定值（0-100），**>= 40** 才接受签约。

**8 因素 + 年龄优先级权重系统**：

| 因素 | 评分范围 | 说明 |
|------|---------|------|
| 薪资满意度 | 20-100 | 基于 offered/current 比值分段 |
| 球队竞争力 | 20-100 | 目标队排名：1-3名=100, 4-6名=80, 7-10名=60, 11-14名=40 |
| 首发机会 | 30-100 | 自己能力 vs 目标队该位置最强：明显强=100, 持平=85, 弱=30 |
| 球队声望 | 20-100 | 基于 team_reputation 线性映射 |
| 队友质量 | 30-100 | 目标队平均能力：≥70=100, ≥65=80, ≥60=65, <60=40 |
| 忠诚影响 | 0-50 | (100 - loyalty) × 0.5，固定权重 0.15 |
| 发展空间 | 30-100 | 仅对 ≤23 岁有效：有高能力同位置老将+队伍均值高=100 |
| 随机波动 | -8 ~ +8 | 增加不确定性 |

**年龄优先级权重**（关键机制）：

| 年龄段 | 薪资 | 竞争力 | 首发 | 声望 | 队友 | 发展 |
|--------|------|--------|------|------|------|------|
| 17-21（新秀） | 0.10 | 0.10 | 0.25 | 0.10 | 0.10 | 0.20 |
| 22-25（成长） | 0.15 | 0.20 | 0.20 | 0.10 | 0.10 | 0.10 |
| 26-28（巅峰） | 0.20 | 0.30 | 0.10 | 0.15 | 0.10 | 0.00 |
| 29-31（老将） | 0.35 | 0.15 | 0.10 | 0.15 | 0.10 | 0.00 |
| 32+（退役前） | 0.40 | 0.10 | 0.10 | 0.15 | 0.10 | 0.00 |

```
weighted_score = ∑ factor_i × w_i + loyalty × 0.15 + random_noise
cross_region_factor = 同赛区 ? 1.0 : (100 - region_loyalty) / 100
willingness = clamp(weighted_score × cross_region_factor, 0, 100)
```

## 合同年限规则

合同范围 **1-4 年**，由年龄、AI 性格、随机性三因素决定：

```
base_years: age ≤ 22 → 3, age 23-28 → 2, age 29+ → 1
personality_adj: long_term > 0.7 → +1, short_term > 0.7 → -1, 其他 → 0
random_adj: 30% 概率 +1, 25% 概率 -1, 45% 概率 0
contract_years = clamp(base + personality_adj + random_adj, 1, 4)
```

| 轮次 | 受 AI 性格影响 | 年限范围 |
|------|---------------|---------|
| R3 续约 | 否 | 1-4 年 |
| R4 自由球员 | 是 | 1-4 年 |
| R5 合同转会 | 是 | 1-4 年 |
| R7 紧急补人 | 否 | 1-2 年 |
| R8 选秀权拍卖 | 否 | - |

## AI球队性格系统

```rust
pub enum AITeamPersonality {
    Aggressive,      // 激进型 - 高价买人，追求即战力
    Conservative,    // 保守型 - 注重青训，不轻易买人
    Balanced,        // 均衡型 - 平衡当下和未来
    Development,     // 发展型 - 重视年轻选手
    WinNow,          // 即战力型 - 追求当下成绩
}
```

### 性格影响

| 性格 | 短期权重 | 长期权重 | 风险承受 | 年轻偏好 |
|------|---------|---------|---------|---------|
| Aggressive | 高 | 低 | 高 | 低 |
| Conservative | 低 | 高 | 低 | 中 |
| Balanced | 中 | 中 | 中 | 中 |
| Development | 低 | 高 | 中 | 高 |
| WinNow | 高 | 低 | 高 | 低 |

## 转会费计算

```
转会费 = 身价 × 供需系数 × 市场热度系数 × 合同系数
```

| 系数 | 范围 | 说明 |
|------|------|------|
| 供需系数 | 0.8-1.3 | 供大于求降价，供不应求涨价 |
| 市场热度 | 0.9-1.2 | 热门期涨价 |
| 合同系数 | 1.0-1.2 | 长合同涨价 |

## 球队声望系统

球队声望在转会缓存中批量计算，并在**选手意愿度评估**中作为关键因素使用。

```rust
pub struct TeamReputation {
    pub team_id: i64,
    pub overall: i64,         // 综合声望 (0-100)
    pub historical: i64,      // 历史声望 (0-100)
    pub recent: i64,          // 近期声望 (0-100)
    pub international: i64,   // 国际声望 (0-100)
}
```

**声望计算规则**:
- 历史声望: 累计荣誉（冠军+20, 亚军+10, 季军+5, 殿军+3），上限100
- 近期声望: 最近3个赛季积分，按 `pts / 200 × 100` 映射
- 综合声望（转会缓存简化版）: 历史×40% + 近期×60%
- 综合声望（完整版）: 历史×30% + 近期×40% + 国际×30%

**转会中的使用**:
- `TransferCache.team_reputations`: 在 `build()` 中批量计算，避免运行时查询
- `calculate_willingness`: 声望影响选手转会意愿（20-100 线性映射）
- 默认声望值: 30（查询不到时的兜底值）

## 自由球员市场

合同到期选手自动进入自由球员市场：

- 无需转会费，只需支付薪资
- 多支球队可同时报价
- **报价筛选**：该位置已有2人不报价；已有1人仅能力升级或青训新人（age<=23 & potential>=70）才报价；match_score < 50 不报价
- 选手通过 8 因素意愿度系统评估每个报价（详见「意愿度」章节）

### 市场竞争效应（R4）

当同一选手收到 ≥3 个报价时，触发市场溢价：

```
market_premium = 1.0 + (offer_count - 2) × 0.05
adjusted_expected_salary = expected_salary × market_premium
```

选手的薪资期望基准上调，意味着报价薪资需更高才能满足选手。

### 竞价升温（R5）

合同选手转会中，当 ≥2 支球队竞标时：

```
bid_premium = 1.0 + (bid_count - 1) × 0.08
transfer_fee = original_bid × bid_premium
```

竞争推高转会费，模拟真实市场中的抢人大战。

## API 接口

| 接口 | 描述 |
|------|------|
| `get_transfer_list()` | 获取转会名单 |
| `initiate_transfer(params)` | 发起转会 |
| `accept_transfer(offer_id)` | 接受报价 |
| `reject_transfer(offer_id)` | 拒绝报价 |
| `terminate_contract(player_id)` | 解约 |
| `loan_player(params)` | 租借球员 |
| `get_team_reputation(team_id)` | 获取球队声望 |
| `update_team_personality(team_id, personality)` | 更新球队性格 |
| `confirm_close_transfer_window(window_id, force?)` | 验证并关闭转会窗口 |

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/engines/transfer.rs` | 转会引擎（含关闭验证） |
| `src-tauri/src/engines/draft_auction.rs` | 选秀权拍卖引擎 |
| `src-tauri/src/commands/transfer_commands.rs` | 转会命令接口 |
| `src-tauri/src/commands/draft_auction_commands.rs` | 选秀权拍卖命令接口 |
| `src-tauri/src/models/transfer.rs` | 转会数据模型 |
| `src-tauri/src/services/game_flow.rs` | 时间引擎（转会期状态判断） |
| `src/stores/useTransferWindowStore.ts` | 前端转会窗口状态管理 |
| `src/stores/useTransferStoreTauri.ts` | 前端转会 Tauri 集成 |
| `src/stores/useDraftAuctionStore.ts` | 前端选秀权拍卖状态 |
| `src/views/TransferWindow.vue` | 转会窗口页面 |
| `src/views/Transfer.vue` | 转会系统页面 |
| `src/views/DraftAuction.vue` | 选秀权拍卖页面 |

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

球队 AI 对选手的匹配评分（0-100），决定 R4 中 offer 的优先级。

分项评分后通过**归一化加权**计算：

```
w_ability = 0.3 + 0.2 × short_term_focus       // 0.3 ~ 0.5
w_age     = 0.2 + 0.2 × max(youth_pref, short_term_focus)  // 0.2 ~ 0.4
w_finance = 0.15 + 0.15 × bargain_hunting       // 0.15 ~ 0.3

match_score = (ability_score × w_ability + age_score × w_age + finance_score × w_finance)
              / (w_ability + w_age + w_finance)
```

不同 AI 性格的权重偏向不同，但总分始终在 0-100 范围。

### 意愿度 (willingness)

选手是否接受报价的判定值（0-100），**>= 40** 才接受签约。

```
salary_score = 基于 offered_salary / current_salary 的分段评分（20-100）
loyalty_impact = (100 - loyalty) × 0.5
base = salary_score × 0.4 + loyalty_impact × 0.3 + 15 + random(-5, 5)

跨赛区惩罚:
  本赛区: willingness = base × 1.0
  跨赛区: willingness = base × (100 - region_loyalty) / 100
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

```rust
pub struct TeamReputation {
    pub team_id: u64,
    pub historical_score: u32,      // 历史声望 (0-100)
    pub recent_score: u32,          // 近期声望 (0-100)
    pub international_score: u32,   // 国际声望 (0-100)
    pub overall_score: u32,         // 综合声望 (0-100)
}
```

**声望计算规则**:
- 历史声望: 累计荣誉（冠军+20, 亚军+10, 季军+5）
- 近期声望: 最近3个赛季成绩
- 国际声望: 国际赛荣誉（每项+15）
- 综合声望: 历史×30% + 近期×40% + 国际×30%

## 自由球员市场

合同到期选手自动进入自由球员市场：

- 无需转会费，只需支付薪资
- 多支球队可同时报价
- 选手选择因素:
  - 薪资高低 (40%)
  - 球队竞争力 (30%)
  - 上场机会 (20%)
  - 随机因素 (10%)

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

-- TransferRounds 阶段（挖人）相关字段迁移
-- 添加转会轮次状态字段到 transfer_market_states 表
-- 添加挖人相关字段到 negotiations 表

-- ==================== 市场状态表新字段 ====================

-- 转会轮次阶段的当前轮次
ALTER TABLE transfer_market_states ADD COLUMN transfer_round INTEGER NOT NULL DEFAULT 0;

-- 转会轮次阶段的最大轮次
ALTER TABLE transfer_market_states ADD COLUMN max_transfer_rounds INTEGER NOT NULL DEFAULT 3;

-- 可挖人的有合同选手 ID 列表（想离队但有合同的选手）
ALTER TABLE transfer_market_states ADD COLUMN poachable_player_ids TEXT DEFAULT '[]';

-- 转会轮次是否稳定
ALTER TABLE transfer_market_states ADD COLUMN is_transfer_stable INTEGER NOT NULL DEFAULT 0;

-- 转会轮次连续无交易计数
ALTER TABLE transfer_market_states ADD COLUMN transfer_stable_rounds_count INTEGER NOT NULL DEFAULT 0;

-- ==================== 谈判表新字段 ====================

-- 是否为挖人谈判（有合同选手转会）
ALTER TABLE negotiations ADD COLUMN is_transfer INTEGER NOT NULL DEFAULT 0;

-- 转会费（万，仅挖人谈判有效）
ALTER TABLE negotiations ADD COLUMN transfer_fee INTEGER;

-- 最终转会费（万，仅挖人谈判）
ALTER TABLE negotiations ADD COLUMN final_transfer_fee INTEGER;

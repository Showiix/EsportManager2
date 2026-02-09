-- ============================================
-- 添加选手满意度字段
-- 版本: 012
-- 日期: 2026-02-09
-- 说明: 为 players 表添加 satisfaction 字段
-- ============================================

-- 添加 satisfaction 列（如果不存在）
-- 注意：SQLite 的 ALTER TABLE ADD COLUMN 如果列已存在会报错
-- 但迁移系统会处理这个错误

ALTER TABLE players ADD COLUMN satisfaction INTEGER NOT NULL DEFAULT 50;

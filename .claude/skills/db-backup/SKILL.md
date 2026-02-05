---
name: db-backup
description: 备份 EsportManager2 游戏数据库。当用户要求备份数据库、保存存档、或在重要操作前保存数据时使用此技能。
---

# 数据库备份

## Overview

备份 Tauri 应用的 SQLite 数据库文件，支持添加备注说明。用于在重要操作（如转会期、选秀、赛季结算）前保存数据状态。

## 数据库位置

| 类型 | 路径 |
|------|------|
| 运行时数据库 | `~/Library/Application Support/com.tauri.dev/esport_manager.db` |
| 备份目录 | `项目根目录/backups/` |

**注意**: 项目目录下的 `.db` 文件通常为空，实际数据存储在 Application Support 目录。

## 备份命令

### 基本备份（带时间戳）

```bash
TIMESTAMP=$(date +%Y%m%d_%H%M%S) && \
cp ~/Library/Application\ Support/com.tauri.dev/esport_manager.db \
   项目路径/backups/esport_manager_${TIMESTAMP}.db
```

### 带备注的备份

```bash
TIMESTAMP=$(date +%Y%m%d_%H%M%S) && \
cp ~/Library/Application\ Support/com.tauri.dev/esport_manager.db \
   项目路径/backups/esport_manager_${TIMESTAMP}_备注内容.db
```

## 常用备注模板

| 场景 | 备注 |
|------|------|
| 转会期前 | `转会市场开始前` |
| 选秀前 | `选秀开始前` |
| 赛季结算前 | `S{N}赛季结算前` |
| 重要测试前 | `测试_{功能名}前` |
| 版本发布前 | `v{版本号}_发布前` |

## 示例

### 用户请求示例

**用户**: "备份一下数据库"
```bash
TIMESTAMP=$(date +%Y%m%d_%H%M%S) && \
cp ~/Library/Application\ Support/com.tauri.dev/esport_manager.db \
   /Users/xxx/projects/agents-xxx/backups/esport_manager_${TIMESTAMP}.db
```

**用户**: "备份数据库，备注是转会市场开始前"
```bash
TIMESTAMP=$(date +%Y%m%d_%H%M%S) && \
cp ~/Library/Application\ Support/com.tauri.dev/esport_manager.db \
   /Users/xxx/projects/agents-xxx/backups/esport_manager_${TIMESTAMP}_转会市场开始前.db
```

## 查看备份列表

```bash
ls -lh 项目路径/backups/
```

## 恢复备份

```bash
cp 项目路径/backups/备份文件名.db \
   ~/Library/Application\ Support/com.tauri.dev/esport_manager.db
```

**警告**: 恢复前请确保应用已关闭，否则可能导致数据损坏。

## 注意事项

1. 备份前确保应用没有正在进行写入操作
2. 定期清理旧备份文件，避免占用过多磁盘空间
3. 重要备份建议额外复制到其他位置
4. 备份文件名使用时间戳格式：`YYYYMMDD_HHMMSS`

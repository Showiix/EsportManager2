---
name: commit-generator
description: 根据 Git 暂存区内容生成规范的 commit message。当用户要求生成 commit、提交代码、或查看暂存区变更时使用此技能。
---

# Commit Generator

## Overview

根据 Git 暂存区的变更内容，自动分析并生成符合项目规范的 commit message。支持 Conventional Commits 格式。

## 工作流程

### 1. 检查暂存区状态

```bash
git status
git diff --cached
```

如果 `git diff --cached` 输出过大，文件会保存到临时路径，使用 Read 工具读取。

### 2. 查看最近提交风格

```bash
git log --oneline -5
```

分析项目使用的 commit message 风格（中文/英文、是否使用 type 前缀等）。

### 3. 生成 Commit Message

#### 格式规范 (Conventional Commits)

```
<type>: <subject>

<body>

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
```

#### Type 类型

| Type | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `refactor` | 代码重构（不改变功能） |
| `style` | 样式/格式调整 |
| `docs` | 文档更新 |
| `test` | 测试相关 |
| `chore` | 构建/工具相关 |
| `perf` | 性能优化 |

#### Subject 规则

- 使用动词开头（中文：实现、修复、优化、添加、删除、重构）
- 简洁明了，不超过 50 字符
- 不使用句号结尾

#### Body 规则

- 列出主要变更点（使用 `-` 列表）
- 说明 "做了什么" 而非 "怎么做的"
- 每行不超过 72 字符

## 示例

### 单一功能变更

```
feat: 实现选手合同中心页面

- 新增 get_player_market_list 后端命令
- 添加 PlayerContractInfo 数据模型
- 支持多条件筛选、排序和分页功能

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
```

### Bug 修复

```
fix: 修复表格固定列滚动时内容溢出问题

- 添加 z-index 防止内容层叠
- 设置 overflow: hidden 截断溢出内容

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
```

### 多功能混合变更

```
feat: 实现转会系统核心功能

- 新增转会期管理命令（开始、执行、快进）
- 实现 AI 球队性格配置
- 添加球队声望计算系统
- 修复数据库迁移脚本执行顺序问题

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
```

## 注意事项

1. **不主动执行 commit** - 只生成 message，由用户决定是否执行
2. **遵循项目风格** - 根据 `git log` 判断项目使用中文还是英文
3. **Co-Authored-By** - 始终添加 Claude 署名
4. **敏感文件检查** - 如果暂存区包含 `.env`、credentials 等敏感文件，需要警告用户

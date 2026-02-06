---
name: debug-doc-writer
description: 记录 bug 修复文档到 debug 目录。当修复了 bug 或解决了问题后，使用此技能快速记录修复内容，方便后续追溯。
---

# Debug Doc Writer

## Overview

在 `debug/` 目录下记录每次 bug 修复的简要文档，便于追溯问题原因和修复方案。

## 文档目录

```
debug/
├── README.md          # 修复记录索引
└── YYYY-MM-DD-简短描述.md  # 单次修复记录
```

## 工作流程

### 1. 收集修复信息

从当前对话上下文或最近的 git commit 中提取：

```bash
git log --oneline -1
git diff HEAD~1
```

### 2. 创建修复文档

**文件命名**：`debug/YYYY-MM-DD-简短英文描述.md`

例如：`debug/2026-02-06-transfer-financial-integration.md`

### 3. 使用模板编写

```markdown
# 问题标题

**日期**: YYYY-MM-DD
**Commit**: `git commit hash`
**涉及文件**:
- `path/to/file1`
- `path/to/file2`

## 问题描述

简要描述发现的 bug 或问题现象。

## 原因分析

简要说明根本原因。

## 修复方案

列出具体做了哪些修改。

## 影响范围

说明修复影响了哪些功能或模块。
```

### 4. 更新索引

在 `debug/README.md` 中添加一行记录。索引格式：

```markdown
| 日期 | 问题 | 文档 |
|------|------|------|
| 2026-02-06 | 转会系统未写入财务记录 | [链接](2026-02-06-transfer-financial-integration.md) |
```

## 编写规范

1. **简洁为主** - 每篇文档控制在 30 行以内
2. **中文编写** - 专业术语保留英文
3. **重点突出** - 问题是什么、为什么发生、怎么修的
4. **不写废话** - 不需要背景介绍，直奔主题

## 注意事项

1. 如果 `debug/README.md` 不存在，先创建它
2. 文件名日期使用当天日期
3. 只记录 bug 修复和问题排查，不记录新功能开发

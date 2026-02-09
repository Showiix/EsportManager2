---
name: page-style
description: 页面视觉风格规范。当需要新建页面、重构页面样式、或确保页面风格一致性时使用此 skill。适用于：(1) 新建 Vue 页面 (2) 重构现有页面样式 (3) 检查页面是否符合设计规范 (4) 统一页面风格
---

# 页面风格规范

EsportManager2 项目的统一页面设计规范。所有数据展示类页面应遵循此规范。

## 设计原则

- 无阴影（`box-shadow: none`），用 `1px solid #e2e8f0` 边框替代
- 无渐变背景（页面标题、统计栏等均使用纯色）
- 表格不使用斑马纹（`stripe`）
- 颜色克制，大面积使用中性色，仅在关键数据点使用语义色
- 圆角统一：容器 `10px`，小元素 `6px`

## 色彩体系

| 用途 | 色值 |
|------|------|
| 主文本 | `#0f172a` |
| 次文本 | `#64748b` |
| 辅助文本/表头 | `#94a3b8` |
| 占位/空状态 | `#cbd5e1` |
| 正文/说明 | `#475569` |
| 强调/链接 | `#1e293b` |
| 主题色/强调 | `#6366f1`（indigo） |
| 边框 | `#e2e8f0` |
| 浅边框/分割线 | `#f1f5f9` |
| 悬停背景 | `#f8fafc` |
| 收入/正向 | `#10b981` |
| 支出/负向 | `#ef4444` |
| 警告 | `#f59e0b` |

## 页面结构

每个页面从上到下的标准结构：

```
根容器（padding: 0）
├── 页面标题区 (.page-header)
├── 提示条 (.notice-bar)         ← 可选
├── 统计栏 (.stats-bar)          ← 可选
├── 筛选区 (.filter-section)     ← 可选
└── 内容区 (.table-section 或 el-card)
    ├── 表格
    └── 分页 (.pagination-wrapper)
```

## 组件样式模板

### 页面标题 (.page-header)

```css
.page-header { margin-bottom: 20px; }
.page-header h1 {
  font-size: 24px; font-weight: 700; color: #0f172a;
  margin: 0 0 4px 0; letter-spacing: -0.3px;
}
.page-header p { font-size: 13px; color: #94a3b8; margin: 0; }
```

模板：
```html
<div class="page-header">
  <div>
    <h1>页面标题</h1>
    <p>页面描述文字</p>
  </div>
</div>
```

如需操作按钮，放在 `.page-header` 右侧，使用 flex 布局。

### 提示条 (.notice-bar)

左侧带 indigo 边框的提示信息：

```css
.notice-bar {
  padding: 10px 16px; background: #f8fafc;
  border-left: 3px solid #6366f1; border-radius: 0 8px 8px 0;
  font-size: 13px; color: #475569;
  margin-bottom: 16px; line-height: 1.6;
}
.notice-bar strong { color: #1e293b; }
```

### 统计栏 (.stats-bar)

水平排列的统计数据条：

```css
.stats-bar {
  display: flex; align-items: center;
  padding: 14px 24px; background: #ffffff;
  border: 1px solid #e2e8f0; border-radius: 10px;
  margin-bottom: 12px;
}
.stat-item {
  display: flex; align-items: baseline; gap: 6px;
  flex: 1; justify-content: center;
}
.stat-value {
  font-size: 20px; font-weight: 700; color: #0f172a;
  font-variant-numeric: tabular-nums;
}
.stat-label { font-size: 12px; color: #94a3b8; font-weight: 500; }
.stat-divider {
  width: 1px; height: 24px; background: #e2e8f0; flex-shrink: 0;
}
```

语义色修饰符：
- `.stat-value.highlight` → `color: #6366f1`
- `.stat-value.income` → `color: #10b981`
- `.stat-value.expense` → `color: #ef4444`

模板：
```html
<div class="stats-bar">
  <div class="stat-item">
    <span class="stat-value">123</span>
    <span class="stat-label">标签</span>
  </div>
  <div class="stat-divider"></div>
  <!-- 更多 stat-item -->
</div>
```

### 筛选区 (.filter-section)

无卡片包裹，直接使用 flex 排列：

```css
.filter-section { margin-bottom: 16px; }
.filter-row {
  display: flex; flex-wrap: wrap; gap: 10px; align-items: center;
}
.filter-group {
  display: flex; align-items: center; gap: 6px;
}
.filter-group label {
  font-size: 12px; color: #94a3b8; font-weight: 500; white-space: nowrap;
}
```

### 表格容器 (.table-section)

```css
.table-section {
  border: 1px solid #e2e8f0; border-radius: 10px; padding: 16px;
}
```

### 表格样式

通过 `:deep()` 覆盖 Element Plus 默认样式。表头用类名前缀区分（如 `.rankings-table`）：

```css
.xxx-table :deep(.el-table th.el-table__cell) {
  font-weight: 600; color: #94a3b8; font-size: 11px;
  text-transform: uppercase; letter-spacing: 0.5px;
  background: transparent; border-bottom: 1px solid #f1f5f9;
  padding: 10px 0;
}
.xxx-table :deep(.el-table__body tr) { transition: background-color 0.15s; }
.xxx-table :deep(.el-table__body tr td) {
  padding: 12px 0; border-bottom: 1px solid #f8fafc;
}
.xxx-table :deep(.el-table__body tr:hover > td) {
  background-color: #f8fafc !important;
}
.xxx-table :deep(.el-table__body tr:last-child td) { border-bottom: none; }
```

**注意**：表格不添加 `stripe` 属性。

### 分页 (.pagination-wrapper)

```css
.pagination-wrapper {
  margin-top: 16px; display: flex; justify-content: center;
}
```

### 详情按钮 (.detail-btn)

白底边框按钮，悬停变 indigo：

```css
.detail-btn {
  padding: 5px 14px; border: 1px solid #e2e8f0; border-radius: 6px;
  background: #ffffff; color: #475569; font-size: 12px;
  font-weight: 500; cursor: pointer; transition: all 0.15s;
}
.detail-btn:hover {
  border-color: #6366f1; color: #6366f1; background: #f5f3ff;
}
```

### 弹窗样式

```css
.xxx-dialog :deep(.el-dialog) { border-radius: 12px; overflow: hidden; }
.xxx-dialog :deep(.el-dialog__header) {
  border-bottom: 1px solid #f1f5f9; padding: 16px 24px;
}
.xxx-dialog :deep(.el-dialog__title) {
  font-weight: 700; font-size: 16px; color: #0f172a;
}
.xxx-dialog :deep(.el-dialog__body) {
  max-height: 60vh; overflow-y: auto; padding: 20px 24px;
}
```

## 数值样式

| 属性值范围 | 颜色/类名 |
|-----------|----------|
| 能力 ≥90 | `#f59e0b` (elite) |
| 能力 80-89 | `#3b82f6` (high) |
| 能力 70-79 | `#22c55e` (medium) |
| 能力 <70 | `#94a3b8` (low) |
| 年龄 ≤20 | `#22c55e` (young) |
| 年龄 21-27 | `#0f172a` (prime) |
| 年龄 ≥28 | `#ef4444` (old) |
| 满意度/忠诚度 ≥80 | `#22c55e` |
| 满意度/忠诚度 60-79 | `#f59e0b` |
| 满意度/忠诚度 <60 | `#ef4444` |

## 赛区标签色

| 赛区 | Element Plus tag type |
|------|----------------------|
| LPL | `danger` |
| LCK | `primary` |
| LEC | `success` |
| LCS | `warning` |

## 响应式断点

```css
@media (max-width: 1200px) {
  .stats-bar { flex-wrap: wrap; gap: 8px; }
  .stat-divider { display: none; }
  .filter-row { flex-direction: column; align-items: stretch; }
}
```

## 旧样式迁移检查清单

将旧页面迁移到新风格时，检查以下项：

- [ ] 移除渐变 `page-header`（`linear-gradient`），替换为纯文本标题
- [ ] 移除 `el-card` 包裹（如仅用于视觉容器），替换为 `.table-section` 或 `.stats-bar`
- [ ] 表格移除 `stripe` 属性
- [ ] 移除所有 `box-shadow`
- [ ] 统计卡片合并为 `.stats-bar` 横条
- [ ] 筛选区移除卡片包裹
- [ ] `el-button` 详情按钮替换为 `.detail-btn` 原生按钮
- [ ] 清理不再使用的组件 import
- [ ] 完整替换 `<style scoped>` 内容

## 已应用此风格的页面

- `Rankings.vue` - 积分排名
- `Finance.vue` - 财政中心
- `PlayerMarket.vue` - 选手合同中心
- `TransferMarketListings.vue` - 转会挂牌市场
- `TeamEvaluationCenter.vue` - 战队评估中心
- `PlayerEvaluationCenter.vue` - 选手评估中心
- `TransferBidAnalysis.vue` - 竞价分析
- `Honors.vue` - 荣誉殿堂
- `InternationalHall.vue` - 国际荣誉殿堂
- `PlayerHonorRankings.vue` - 选手荣誉榜
- `TeamHonorRankings.vue` - 战队荣誉榜

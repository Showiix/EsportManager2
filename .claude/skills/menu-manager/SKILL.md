---
name: menu-manager
description: 管理项目侧边栏菜单的 skill。当用户需要添加、删除、修改侧边栏菜单项时使用此 skill。适用于：(1) 添加新菜单项 (2) 删除菜单项 (3) 修改菜单项名称或图标 (4) 添加/删除子菜单 (5) 调整菜单顺序
---

# 菜单管理 Skill

管理 EsportManager2 项目的侧边栏菜单配置。

## 关键文件

项目有两个侧边栏组件，修改菜单时**必须同时更新两个文件**：

| 文件 | 用途 |
|------|------|
| `src/components/layout/Sidebar.vue` | 主侧边栏（深色主题） |
| `src/components/layout/MainLayout.vue` | 备用布局侧边栏（浅色主题） |

路由配置文件：
- `src/router/index.ts` - 添加新页面时需要同步添加路由

## 菜单结构

### Sidebar.vue 结构

```typescript
const menuItems: MenuItem[] = [
  { name: '菜单名', path: '/path', icon: 'iconName' },
  // 带子菜单
  {
    name: '父菜单',
    path: '/parent',
    icon: 'iconName',
    children: [
      { name: '子菜单1', path: '/parent/child1', icon: 'childIcon' },
    ]
  },
]
```

可用图标（定义在 `icons` 对象中）：
- dashboard, clock, trophy, users, gamepad, clipboard
- exchange, chart, wallet, stats, star, award
- medal, settings, wrench, chevron

子菜单图标：
- 国旗：cn, kr, eu, us（使用 emoji）
- 其他：preview, market, broadcast, robot（使用 emoji）

### MainLayout.vue 结构

使用 Element Plus 的 `el-menu` 组件：

```vue
<!-- 普通菜单项 -->
<el-menu-item index="/path" :disabled="isMenuDisabled('/path')">
  <el-icon><IconName /></el-icon>
  <span>菜单名</span>
</el-menu-item>

<!-- 带子菜单 -->
<el-sub-menu index="menu-id" :disabled="isMenuDisabled('/parent')">
  <template #title>
    <el-icon><IconName /></el-icon>
    <span>父菜单</span>
  </template>
  <el-menu-item index="/parent/child1">子菜单1</el-menu-item>
</el-sub-menu>
```

## 工作流程

### 添加菜单项

1. 在 `Sidebar.vue` 的 `menuItems` 数组中添加菜单项
2. 在 `MainLayout.vue` 的 `<el-menu>` 中添加对应的 `<el-menu-item>`
3. 如需新图标，在 `Sidebar.vue` 的 `icons` 对象中添加 SVG path
4. 在 `MainLayout.vue` 中 import 对应的 Element Plus 图标
5. 在 `src/router/index.ts` 中添加路由配置

### 删除菜单项

1. 从 `Sidebar.vue` 的 `menuItems` 数组中移除菜单项
2. 从 `MainLayout.vue` 的 `<el-menu>` 中移除对应元素
3. 可选：从 `src/router/index.ts` 中移除路由（如果页面也要删除）

### 添加子菜单

1. 在 `Sidebar.vue` 中，将菜单项改为带 `children` 数组的结构
2. 在 `MainLayout.vue` 中，将 `<el-menu-item>` 改为 `<el-sub-menu>` 结构
3. 确保 `expandedMenus` ref 中包含父菜单名称（如需默认展开）

## 注意事项

- 两个文件的菜单项必须保持一致
- 路径必须与路由配置匹配
- `isMenuDisabled` 用于控制未加载存档时禁用菜单
- `/settings` 和 `/dev-tools` 始终可用（不受存档加载限制）

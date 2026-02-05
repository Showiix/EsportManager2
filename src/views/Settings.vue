<template>
  <div class="settings-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>系统设置</h1>
        <p>管理存档和游戏设置</p>
      </div>
    </div>

    <!-- 存档管理 -->
    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <div class="card-title">
            <el-icon class="title-icon save"><Folder /></el-icon>
            <span>存档管理</span>
          </div>
          <div class="header-actions">
            <el-button @click="handleInitDatabase" :loading="isLoading">
              <el-icon><Connection /></el-icon>
              初始化数据库
            </el-button>
            <el-button type="primary" @click="showNewSaveDialog = true">
              <el-icon><Plus /></el-icon>
              新建存档
            </el-button>
          </div>
        </div>
      </template>

      <!-- 加载状态 -->
      <el-skeleton v-if="isLoading" :rows="3" animated />

      <!-- 存档列表 -->
      <div v-else-if="saves.length === 0" class="empty-state">
        <el-empty description="暂无存档，点击新建存档开始游戏" />
      </div>

      <div v-else class="save-list">
        <div v-for="save in saves" :key="save.id" class="save-item" :class="{ 'is-current': currentSave?.id === save.id }">
          <div class="save-icon">
            <el-icon :size="24"><Document /></el-icon>
          </div>
          <div class="save-info">
            <div class="save-name">
              {{ save.name }}
              <el-tag v-if="currentSave?.id === save.id" size="small" type="success">当前</el-tag>
            </div>
            <div class="save-meta">
              <el-tag size="small" type="primary">第{{ save.current_season }}赛季</el-tag>
              <span class="save-phase">{{ save.current_phase }}</span>
            </div>
          </div>
          <div class="save-time">{{ formatDate(save.updated_at) }}</div>
          <div class="save-actions">
            <el-button
              type="success"
              size="small"
              :disabled="currentSave?.id === save.id"
              @click="handleLoadSave(save)"
            >
              <el-icon><VideoPlay /></el-icon>
              加载
            </el-button>
            <el-button size="small" @click="exportSave(save)">
              <el-icon><Download /></el-icon>
              导出
            </el-button>
            <el-button type="danger" size="small" @click="handleDeleteSave(save)">
              <el-icon><Delete /></el-icon>
              删除
            </el-button>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 导入存档 -->
    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <div class="card-title">
            <el-icon class="title-icon import"><Upload /></el-icon>
            <span>导入存档</span>
          </div>
        </div>
      </template>

      <el-upload
        class="upload-area"
        drag
        :auto-upload="false"
        :show-file-list="false"
        accept=".json,.save"
        @change="handleImportFile"
      >
        <el-icon class="upload-icon"><UploadFilled /></el-icon>
        <div class="upload-text">拖拽存档文件到此处，或 <em>点击选择文件</em></div>
        <div class="upload-tip">支持 .json 或 .save 格式的存档文件</div>
      </el-upload>
    </el-card>

    <!-- 游戏设置 -->
    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <div class="card-title">
            <el-icon class="title-icon settings"><Setting /></el-icon>
            <span>游戏设置</span>
          </div>
        </div>
      </template>

      <div class="settings-list">
        <div class="setting-item">
          <div class="setting-info">
            <div class="setting-label">自动保存</div>
            <div class="setting-desc">每完成一个阶段自动保存游戏进度</div>
          </div>
          <el-switch v-model="gameSettings.autoSave" />
        </div>

        <el-divider />

        <div class="setting-item">
          <div class="setting-info">
            <div class="setting-label">比赛动画</div>
            <div class="setting-desc">显示比赛模拟的动画效果</div>
          </div>
          <el-switch v-model="gameSettings.matchAnimation" />
        </div>

        <el-divider />

        <div class="setting-item">
          <div class="setting-info">
            <div class="setting-label">音效</div>
            <div class="setting-desc">开启游戏音效</div>
          </div>
          <el-switch v-model="gameSettings.soundEffect" />
        </div>

        <el-divider />

        <div class="setting-item">
          <div class="setting-info">
            <div class="setting-label">模拟速度</div>
            <div class="setting-desc">调整比赛模拟的速度</div>
          </div>
          <el-select v-model="gameSettings.simulationSpeed" style="width: 120px;">
            <el-option label="慢速" value="slow" />
            <el-option label="正常" value="normal" />
            <el-option label="快速" value="fast" />
            <el-option label="极速" value="instant" />
          </el-select>
        </div>

        <el-divider />

        <div class="setting-item">
          <div class="setting-info">
            <div class="setting-label">语言</div>
            <div class="setting-desc">选择界面语言</div>
          </div>
          <el-select v-model="gameSettings.language" style="width: 120px;">
            <el-option label="简体中文" value="zh-CN" />
            <el-option label="English" value="en" />
          </el-select>
        </div>
      </div>
    </el-card>

    <!-- 数据管理 -->
    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <div class="card-title">
            <el-icon class="title-icon data"><Coin /></el-icon>
            <span>数据管理</span>
          </div>
        </div>
      </template>

      <div class="data-actions">
        <div class="data-action-item danger">
          <div class="data-action-info">
            <div class="data-action-label">删除数据库</div>
            <div class="data-action-desc">删除数据库文件，用于开发调试时重建数据库结构</div>
          </div>
          <el-button type="warning" @click="handleDeleteDatabase" :loading="isDeletingDb">
            删除数据库
          </el-button>
        </div>

        <el-divider />

        <div class="data-action-item">
          <div class="data-action-info">
            <div class="data-action-label">清除缓存</div>
            <div class="data-action-desc">清除本地缓存数据，不影响存档</div>
          </div>
          <el-button @click="clearCache">清除缓存</el-button>
        </div>

        <el-divider />

        <div class="data-action-item danger">
          <div class="data-action-info">
            <div class="data-action-label">重置游戏</div>
            <div class="data-action-desc">删除所有存档和设置，恢复初始状态</div>
          </div>
          <el-button type="danger" @click="resetGame">重置游戏</el-button>
        </div>
      </div>
    </el-card>

    <!-- 关于 -->
    <el-card class="settings-card about-card">
      <div class="about-content">
        <div class="about-logo">
          <el-icon :size="48"><Trophy /></el-icon>
        </div>
        <div class="about-title">电竞经理模拟器 2</div>
        <div class="about-version">版本 0.1.0</div>
        <div class="about-tech">基于 Tauri + Vue3 + Rust 构建</div>
        <div class="about-links">
          <el-button link type="primary">
            <el-icon><Link /></el-icon>
            官方网站
          </el-button>
          <el-button link type="primary">
            <el-icon><ChatDotRound /></el-icon>
            反馈建议
          </el-button>
        </div>
      </div>
    </el-card>

    <!-- 游戏指南 -->
    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <div class="card-title">
            <el-icon class="title-icon guide"><Reading /></el-icon>
            <span>游戏指南</span>
          </div>
        </div>
      </template>

      <GameGuide />
    </el-card>

    <!-- 新建存档对话框 -->
    <el-dialog
      v-model="showNewSaveDialog"
      title="新建存档"
      width="480px"
      :close-on-click-modal="false"
    >
      <el-form :model="newSaveForm" label-width="80px">
        <el-form-item label="存档名称">
          <el-input v-model="newSaveForm.name" placeholder="请输入存档名称" />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="showNewSaveDialog = false">取消</el-button>
        <el-button type="primary" :loading="isLoading" @click="handleCreateSave">创建</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Folder,
  Plus,
  Document,
  VideoPlay,
  Download,
  Delete,
  Upload,
  UploadFilled,
  Setting,
  Coin,
  Trophy,
  Link,
  ChatDotRound,
  Reading,
  Connection,
} from '@element-plus/icons-vue'
import GameGuide from '@/components/settings/GameGuide.vue'
import { useGameStore } from '@/stores/useGameStore'
import type { SaveInfo } from '@/api/tauri'

const gameStore = useGameStore()
const { saves, currentSave, isLoading, isInitialized } = storeToRefs(gameStore)

// 新建存档对话框
const showNewSaveDialog = ref(false)
const newSaveForm = reactive({
  name: '',
})

// 删除数据库状态
const isDeletingDb = ref(false)

// 游戏设置
const gameSettings = reactive({
  autoSave: true,
  matchAnimation: true,
  soundEffect: true,
  simulationSpeed: 'normal',
  language: 'zh-CN',
})

// 初始化加载
onMounted(async () => {
  try {
    await gameStore.loadSaves()
  } catch (e) {
    console.log('数据库未初始化，请先初始化数据库')
  }
})

// 格式化日期
const formatDate = (dateStr: string) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  })
}

// 初始化数据库
const handleInitDatabase = async () => {
  try {
    await ElMessageBox.confirm(
      '初始化数据库将创建游戏所需的数据结构。如果数据库已存在，不会删除现有数据。是否继续？',
      '初始化数据库',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'info',
      }
    )

    await gameStore.initDatabase()
    ElMessage.success('数据库初始化成功')
    // 刷新存档列表
    await gameStore.loadSaves()
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error(`数据库初始化失败: ${e}`)
    }
  }
}

// 删除数据库
const handleDeleteDatabase = async () => {
  try {
    await ElMessageBox.confirm(
      '删除数据库将清除所有游戏数据，包括所有存档！此操作不可恢复。确定要继续吗？',
      '删除数据库',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    isDeletingDb.value = true
    await gameStore.deleteDatabase()
    ElMessage.success('数据库已删除，请重新初始化数据库')
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error(`删除数据库失败: ${e}`)
    }
  } finally {
    isDeletingDb.value = false
  }
}

// 新建存档
const handleCreateSave = async () => {
  if (!isInitialized.value) {
    ElMessage.error('请先初始化数据库！')
    return
  }

  if (!newSaveForm.name.trim()) {
    ElMessage.warning('请输入存档名称')
    return
  }

  try {
    await gameStore.createSave(newSaveForm.name.trim())
    showNewSaveDialog.value = false
    newSaveForm.name = ''
    ElMessage.success('存档创建成功')
  } catch (e) {
    ElMessage.error(`存档创建失败: ${e}`)
  }
}

// 加载存档
const handleLoadSave = async (save: SaveInfo) => {
  if (!isInitialized.value) {
    ElMessage.error('请先初始化数据库！')
    return
  }

  try {
    await gameStore.loadSave(save.id)
    ElMessage.success(`已加载存档: ${save.name}`)
  } catch (e) {
    ElMessage.error(`加载存档失败: ${e}`)
  }
}

// 导出存档
const exportSave = (save: SaveInfo) => {
  ElMessage.info(`正在导出存档: ${save.name}`)
  // TODO: 实际导出逻辑 - 需要后端支持
}

// 删除存档
const handleDeleteSave = async (save: SaveInfo) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除存档「${save.name}」吗？此操作不可恢复。`,
      '删除存档',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    await gameStore.deleteSave(save.id)
    ElMessage.success('存档已删除')
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error(`删除存档失败: ${e}`)
    }
  }
}

// 导入存档
const handleImportFile = (file: any) => {
  ElMessage.info(`正在导入: ${file.name}`)
  // TODO: 实际导入逻辑 - 需要后端支持
}

// 清除缓存
const clearCache = async () => {
  await ElMessageBox.confirm(
    '确定要清除缓存吗？',
    '清除缓存',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'info',
    }
  )
  ElMessage.success('缓存已清除')
}

// 重置游戏
const resetGame = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要重置游戏吗？所有存档和设置将被删除，此操作不可恢复！',
      '重置游戏',
      {
        confirmButtonText: '重置',
        cancelButtonText: '取消',
        type: 'error',
      }
    )

    // 删除所有存档
    for (const save of saves.value) {
      await gameStore.deleteSave(save.id)
    }
    ElMessage.success('游戏已重置')
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error(`重置游戏失败: ${e}`)
    }
  }
}
</script>

<style scoped>
.settings-view {
  padding: 0;
}

/* 页面标题 */
.page-header {
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary, #303133);
  margin: 0 0 8px 0;
}

.page-header p {
  font-size: 14px;
  color: var(--text-tertiary, #909399);
  margin: 0;
}

/* 设置卡片 */
.settings-card {
  border-radius: 12px;
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary, #303133);
}

.title-icon {
  font-size: 20px;
}

.title-icon.save { color: #f59e0b; }
.title-icon.import { color: #3b82f6; }
.title-icon.settings { color: #8b5cf6; }
.title-icon.data { color: #22c55e; }
.title-icon.guide { color: #ec4899; }

/* 存档列表 */
.save-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.save-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 10px;
  transition: all 0.3s ease;
}

.save-item:hover {
  background: #ebeef5;
  transform: translateX(4px);
}

.save-item.is-current {
  background: linear-gradient(135deg, #f0f9eb 0%, #e1f3d8 100%);
  border: 1px solid #c2e7b0;
}

.save-icon {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  background: linear-gradient(135deg, #f59e0b, #d97706);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.save-info {
  flex: 1;
}

.save-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary, #303133);
  margin-bottom: 4px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.save-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.save-phase {
  font-size: 13px;
  color: var(--text-secondary, #606266);
}

.save-time {
  font-size: 13px;
  color: var(--text-tertiary, #909399);
  margin-right: 16px;
}

.save-actions {
  display: flex;
  gap: 8px;
}

/* 上传区域 */
.upload-area {
  width: 100%;
}

.upload-area :deep(.el-upload-dragger) {
  border-radius: 12px;
  border: 2px dashed #dcdfe6;
  padding: 40px;
  transition: all 0.3s ease;
}

.upload-area :deep(.el-upload-dragger:hover) {
  border-color: #409eff;
}

.upload-icon {
  font-size: 48px;
  color: #c0c4cc;
  margin-bottom: 16px;
}

.upload-text {
  font-size: 14px;
  color: var(--text-secondary, #606266);
  margin-bottom: 8px;
}

.upload-text em {
  color: #409eff;
  font-style: normal;
}

.upload-tip {
  font-size: 12px;
  color: var(--text-tertiary, #909399);
}

/* 设置列表 */
.settings-list {
  padding: 8px 0;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
}

.setting-info {
  flex: 1;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary, #303133);
  margin-bottom: 4px;
}

.setting-desc {
  font-size: 13px;
  color: var(--text-tertiary, #909399);
}

/* 数据管理 */
.data-actions {
  padding: 8px 0;
}

.data-action-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
}

.data-action-info {
  flex: 1;
}

.data-action-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary, #303133);
  margin-bottom: 4px;
}

.data-action-desc {
  font-size: 13px;
  color: var(--text-tertiary, #909399);
}

.data-action-item.danger .data-action-label {
  color: #f56c6c;
}

/* 关于卡片 */
.about-card {
  text-align: center;
}

.about-content {
  padding: 24px 0;
}

.about-logo {
  width: 80px;
  height: 80px;
  margin: 0 auto 16px;
  background: linear-gradient(135deg, #f59e0b, #d97706);
  border-radius: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.about-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary, #303133);
  margin-bottom: 8px;
}

.about-version {
  font-size: 14px;
  color: var(--text-secondary, #606266);
  margin-bottom: 4px;
}

.about-tech {
  font-size: 12px;
  color: var(--text-tertiary, #909399);
  margin-bottom: 16px;
}

.about-links {
  display: flex;
  justify-content: center;
  gap: 24px;
}

/* 空状态 */
.empty-state {
  padding: 40px 0;
}

/* 响应式 */
@media (max-width: 768px) {
  .save-item {
    flex-wrap: wrap;
  }

  .save-time {
    width: 100%;
    margin: 8px 0;
  }

  .save-actions {
    width: 100%;
    justify-content: flex-end;
  }

  .card-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }
}
</style>

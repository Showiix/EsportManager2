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
        <div class="about-subtitle">Esport Manager Simulator 2</div>
        <div class="about-version">v0.1.0</div>

        <div class="about-divider"></div>

        <div class="about-info-grid">
          <div class="about-info-item">
            <div class="about-info-label">开发者</div>
            <div class="about-info-value">Showiix</div>
          </div>
          <div class="about-info-item">
            <div class="about-info-label">技术栈</div>
            <div class="about-info-value">Tauri + Vue 3 + Rust</div>
          </div>
          <div class="about-info-item">
            <div class="about-info-label">数据库</div>
            <div class="about-info-value">SQLite</div>
          </div>
          <div class="about-info-item">
            <div class="about-info-label">UI 框架</div>
            <div class="about-info-value">Element Plus</div>
          </div>
        </div>

        <div class="about-divider"></div>

        <div class="about-copyright">
          &copy; 2024 Showiix. All rights reserved.
        </div>
      </div>
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
        <el-button @click="handleCustomCreate">自定义创建</el-button>
        <el-button type="primary" :loading="isLoading" @click="handleCreateSave">快速创建</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
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
  Coin,
  Trophy,
  Connection,
} from '@element-plus/icons-vue'
import { useGameStore } from '@/stores/useGameStore'
import type { SaveInfo } from '@/api/tauri'
import { createLogger } from '@/utils/logger'

const logger = createLogger('Settings')

const router = useRouter()
const gameStore = useGameStore()
const { saves, currentSave, isLoading, isInitialized } = storeToRefs(gameStore)

// 新建存档对话框
const showNewSaveDialog = ref(false)
const newSaveForm = reactive({
  name: '',
})

// 删除数据库状态
const isDeletingDb = ref(false)

// 初始化加载
onMounted(async () => {
  try {
    await gameStore.loadSaves()
  } catch (e) {
    logger.debug('数据库未初始化，请先初始化数据库')
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

// 自定义创建存档
const handleCustomCreate = () => {
  if (!isInitialized.value) {
    ElMessage.error('请先初始化数据库！')
    return
  }

  if (!newSaveForm.name.trim()) {
    ElMessage.warning('请输入存档名称')
    return
  }

  const name = newSaveForm.name.trim()
  showNewSaveDialog.value = false
  newSaveForm.name = ''
  router.push({ path: '/save-customize', query: { name } })
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
.title-icon.data { color: #22c55e; }

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
  background: linear-gradient(135deg, #eff6ff 0%, #f5f3ff 50%, #fdf2f8 100%);
}

.about-content {
  padding: 32px 0 24px;
}

.about-logo {
  width: 88px;
  height: 88px;
  margin: 0 auto 20px;
  background: linear-gradient(135deg, #3b82f6, #8b5cf6);
  border-radius: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 8px 24px rgba(99, 102, 241, 0.3);
}

.about-title {
  font-size: 22px;
  font-weight: 700;
  margin-bottom: 4px;
  letter-spacing: 1px;
  background: linear-gradient(135deg, #1e40af, #7c3aed);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.about-subtitle {
  font-size: 13px;
  color: var(--text-tertiary, #909399);
  margin-bottom: 4px;
  font-style: italic;
}

.about-version {
  display: inline-block;
  font-size: 12px;
  color: #8b5cf6;
  background: #f3f0ff;
  padding: 2px 12px;
  border-radius: 10px;
  font-weight: 500;
}

.about-divider {
  width: 60px;
  height: 2px;
  background: linear-gradient(90deg, transparent, #dcdfe6, transparent);
  margin: 20px auto;
}

.about-info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  max-width: 360px;
  margin: 0 auto;
  text-align: center;
}

.about-info-item {
  padding: 12px;
  background: white;
  border-radius: 10px;
  border: 1px solid #f0f0f0;
}

.about-info-label {
  font-size: 11px;
  color: var(--text-tertiary, #909399);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 4px;
}

.about-info-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary, #303133);
}

.about-copyright {
  font-size: 12px;
  color: var(--text-tertiary, #909399);
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

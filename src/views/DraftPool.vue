<template>
  <div class="draft-pool-view">
    <!-- 返回导航 -->
    <div class="back-nav">
      <button class="back-btn" @click="$router.push('/draft')">
        <el-icon><ArrowLeft /></el-icon>
        <span>返回选秀系统</span>
      </button>
    </div>

    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <div class="header-icon">
          <el-icon :size="28"><FolderOpened /></el-icon>
        </div>
        <div class="header-info">
          <h1 class="page-title">选手池管理</h1>
          <p class="page-desc">管理各赛区待选秀新秀数据</p>
        </div>
      </div>
      <div class="header-right">
        <el-dropdown trigger="click" @command="handleImportCommand">
          <el-button type="primary">
            <el-icon><Upload /></el-icon>
            导入新秀
            <el-icon class="el-icon--right"><ArrowDown /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="single">单个添加</el-dropdown-item>
              <el-dropdown-item command="file" divided>批量导入文件</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- 赛区切换标签 -->
    <div class="region-tabs">
      <button
        v-for="r in regionList"
        :key="r.code"
        class="region-tab"
        :class="{ active: selectedRegion === r.code }"
        @click="selectedRegion = r.code"
      >
        <span class="tab-badge" :class="r.code">{{ r.code.toUpperCase() }}</span>
        <span class="tab-name">{{ r.name }}</span>
        <span class="tab-count">{{ getRegionPoolCount(r.code) }}/14</span>
      </button>
    </div>

    <!-- 统计卡片 -->
    <div class="stats-row">
      <div class="stat-card">
        <div class="stat-icon genius">
          <el-icon><Star /></el-icon>
        </div>
        <div class="stat-info">
          <span class="stat-value">{{ geniusCount }}</span>
          <span class="stat-label">天才新秀</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon normal">
          <el-icon><User /></el-icon>
        </div>
        <div class="stat-info">
          <span class="stat-value">{{ normalCount }}</span>
          <span class="stat-label">普通新秀</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon mediocre">
          <el-icon><UserFilled /></el-icon>
        </div>
        <div class="stat-info">
          <span class="stat-value">{{ mediocreCount }}</span>
          <span class="stat-label">平庸新秀</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon total">
          <el-icon><Collection /></el-icon>
        </div>
        <div class="stat-info">
          <span class="stat-value">{{ currentPoolData.length }}/14</span>
          <span class="stat-label">选手池容量</span>
        </div>
      </div>
    </div>

    <!-- 选手池列表 -->
    <div class="pool-section">
      <div class="section-header">
        <div class="header-title">
          <h2>{{ currentRegionName }} 选手池</h2>
          <el-tag v-if="currentPoolData.length === 14" type="success">已满员</el-tag>
          <el-tag v-else type="warning">待补充</el-tag>
        </div>
        <div class="header-actions">
          <el-button @click="generateRandomPool" :disabled="currentPoolData.length >= 14">
            <el-icon><MagicStick /></el-icon>
            随机生成
          </el-button>
          <el-button type="danger" @click="clearPool" :disabled="currentPoolData.length === 0">
            <el-icon><Delete /></el-icon>
            清空选手池
          </el-button>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-if="currentPoolData.length === 0" class="empty-state">
        <div class="empty-icon">
          <el-icon :size="64"><FolderOpened /></el-icon>
        </div>
        <h3>选手池为空</h3>
        <p>请导入新秀数据或随机生成选手池</p>
        <div class="empty-actions">
          <el-button type="primary" @click="handleImportCommand('file')">
            <el-icon><Document /></el-icon>
            批量导入文件
          </el-button>
          <el-button @click="showImportDialog = true">
            <el-icon><Plus /></el-icon>
            单个添加
          </el-button>
          <el-button @click="generateRandomPool">
            <el-icon><MagicStick /></el-icon>
            随机生成
          </el-button>
        </div>
      </div>

      <!-- 选手网格 -->
      <div v-else class="pool-grid">
        <div
          v-for="(player, index) in currentPoolData"
          :key="player.id"
          class="pool-card"
          :class="getCardClass(player.tag)"
        >
          <div class="card-header">
            <span class="player-rank">#{{ index + 1 }}</span>
            <span class="player-tag" :class="player.tag.toLowerCase()">
              {{ getTagLabel(player.tag) }}
            </span>
          </div>
          <div class="card-body">
            <div class="player-name">{{ player.gameId }}</div>
            <div class="player-position">{{ player.position }}</div>
          </div>
          <div class="card-stats">
            <div class="stat-item">
              <span class="stat-value ability" :style="{ color: getAbilityColor(player.ability) }">
                {{ player.ability }}
              </span>
              <span class="stat-label">能力</span>
            </div>
            <div class="stat-item">
              <span class="stat-value potential">{{ player.potential }}</span>
              <span class="stat-label">潜力</span>
            </div>
          </div>
          <div class="card-footer">
            <el-button size="small" text type="primary" @click="editPlayer(player)">
              <el-icon><Edit /></el-icon>
            </el-button>
            <el-button size="small" text type="danger" @click="removePlayer(player.id)">
              <el-icon><Delete /></el-icon>
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 导入对话框 -->
    <el-dialog v-model="showImportDialog" title="导入新秀" width="500px">
      <el-form :model="importForm" label-width="80px">
        <el-form-item label="游戏ID">
          <el-input v-model="importForm.gameId" placeholder="请输入游戏ID" />
        </el-form-item>
        <el-form-item label="位置">
          <el-select v-model="importForm.position" placeholder="选择位置" style="width: 100%">
            <el-option label="上单" value="TOP" />
            <el-option label="打野" value="JUG" />
            <el-option label="中单" value="MID" />
            <el-option label="ADC" value="ADC" />
            <el-option label="辅助" value="SUP" />
          </el-select>
        </el-form-item>
        <el-form-item label="能力值">
          <el-slider v-model="importForm.ability" :min="30" :max="80" show-input />
        </el-form-item>
        <el-form-item label="潜力值">
          <el-slider v-model="importForm.potential" :min="50" :max="99" show-input />
        </el-form-item>
        <el-form-item label="天赋标签">
          <el-radio-group v-model="importForm.tag">
            <el-radio value="MEDIOCRE">平庸</el-radio>
            <el-radio value="NORMAL">一般</el-radio>
            <el-radio value="GENIUS">天才</el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showImportDialog = false">取消</el-button>
        <el-button type="primary" @click="importPlayer">确认导入</el-button>
      </template>
    </el-dialog>

    <!-- 编辑对话框 -->
    <el-dialog v-model="showEditDialog" title="编辑新秀" width="500px">
      <el-form :model="editForm" label-width="80px">
        <el-form-item label="游戏ID">
          <el-input v-model="editForm.gameId" placeholder="请输入游戏ID" />
        </el-form-item>
        <el-form-item label="位置">
          <el-select v-model="editForm.position" placeholder="选择位置" style="width: 100%">
            <el-option label="上单" value="TOP" />
            <el-option label="打野" value="JUG" />
            <el-option label="中单" value="MID" />
            <el-option label="ADC" value="ADC" />
            <el-option label="辅助" value="SUP" />
          </el-select>
        </el-form-item>
        <el-form-item label="能力值">
          <el-slider v-model="editForm.ability" :min="30" :max="80" show-input />
        </el-form-item>
        <el-form-item label="潜力值">
          <el-slider v-model="editForm.potential" :min="50" :max="99" show-input />
        </el-form-item>
        <el-form-item label="天赋标签">
          <el-radio-group v-model="editForm.tag">
            <el-radio value="MEDIOCRE">平庸</el-radio>
            <el-radio value="NORMAL">一般</el-radio>
            <el-radio value="GENIUS">天才</el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showEditDialog = false">取消</el-button>
        <el-button type="primary" @click="savePlayer">保存修改</el-button>
      </template>
    </el-dialog>

    <!-- 批量导入对话框 -->
    <el-dialog v-model="showBatchImportDialog" title="批量导入新秀" width="700px">
      <div class="batch-import-content">
        <!-- 上传区域 -->
        <div
          v-if="!parsedPlayers.length"
          class="upload-area"
          :class="{ 'is-dragover': isDragover }"
          @dragover.prevent="isDragover = true"
          @dragleave.prevent="isDragover = false"
          @drop.prevent="handleDrop"
          @click="triggerFileInput"
        >
          <div class="upload-icon">
            <el-icon :size="48"><UploadFilled /></el-icon>
          </div>
          <div class="upload-text">
            <p class="main-text">拖拽文件到此处，或<span class="link">点击上传</span></p>
            <p class="sub-text">支持 .json 或 .csv 格式文件</p>
          </div>
        </div>

        <!-- 格式说明 -->
        <div v-if="!parsedPlayers.length" class="format-guide">
          <h4>文件格式说明</h4>
          <div class="format-tabs">
            <div
              class="format-tab"
              :class="{ active: activeFormatTab === 'json' }"
              @click="activeFormatTab = 'json'"
            >
              JSON 格式
            </div>
            <div
              class="format-tab"
              :class="{ active: activeFormatTab === 'csv' }"
              @click="activeFormatTab = 'csv'"
            >
              CSV 格式
            </div>
          </div>

          <div v-if="activeFormatTab === 'json'" class="format-content">
            <pre class="code-block">[
  {
    "gameId": "Faker",
    "position": "MID",
    "ability": 75,
    "potential": 95,
    "tag": "GENIUS"
  },
  {
    "gameId": "Rookie",
    "position": "MID",
    "ability": 70,
    "potential": 90,
    "tag": "NORMAL"
  }
]</pre>
          </div>

          <div v-else class="format-content">
            <pre class="code-block">gameId,position,ability,potential,tag
Faker,MID,75,95,GENIUS
Rookie,MID,70,90,NORMAL
TheShy,TOP,72,88,GENIUS</pre>
          </div>

          <div class="field-desc">
            <h5>字段说明</h5>
            <table class="desc-table">
              <thead>
                <tr>
                  <th>字段名</th>
                  <th>说明</th>
                  <th>可选值</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><code>gameId</code></td>
                  <td>游戏ID</td>
                  <td>任意字符串</td>
                </tr>
                <tr>
                  <td><code>position</code></td>
                  <td>位置</td>
                  <td>TOP / JUG / MID / ADC / SUP</td>
                </tr>
                <tr>
                  <td><code>ability</code></td>
                  <td>能力值</td>
                  <td>30 ~ 80</td>
                </tr>
                <tr>
                  <td><code>potential</code></td>
                  <td>潜力值</td>
                  <td>50 ~ 99</td>
                </tr>
                <tr>
                  <td><code>tag</code></td>
                  <td>天赋标签</td>
                  <td>GENIUS / NORMAL / MEDIOCRE</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- 解析结果预览 -->
        <div v-else class="preview-area">
          <div class="preview-header">
            <div class="preview-info">
              <el-icon :size="20" color="#22c55e"><SuccessFilled /></el-icon>
              <span>已解析 <strong>{{ parsedPlayers.length }}</strong> 名选手数据</span>
            </div>
            <el-button text type="primary" @click="resetUpload">
              <el-icon><RefreshLeft /></el-icon>
              重新上传
            </el-button>
          </div>

          <div class="preview-tips">
            <p><span class="label">当前赛区：</span>{{ currentRegionName }}</p>
            <p><span class="label">剩余容量：</span>{{ 14 - currentPoolData.length }} 个名额</p>
            <p v-if="parsedPlayers.length > 14 - currentPoolData.length" class="warning">
              导入数量超过剩余容量，将只导入前 {{ 14 - currentPoolData.length }} 名选手
            </p>
          </div>

          <el-table
            :data="parsedPlayers.slice(0, 14 - currentPoolData.length)"
            max-height="300"
            border
            size="small"
          >
            <el-table-column type="index" label="#" width="50" />
            <el-table-column prop="gameId" label="游戏ID" min-width="100" />
            <el-table-column prop="position" label="位置" width="70" align="center" />
            <el-table-column prop="ability" label="能力" width="70" align="center">
              <template #default="{ row }">
                <span :style="{ color: getAbilityColor(row.ability), fontWeight: 600 }">
                  {{ row.ability }}
                </span>
              </template>
            </el-table-column>
            <el-table-column prop="potential" label="潜力" width="70" align="center">
              <template #default="{ row }">
                <span style="color: #8b5cf6; font-weight: 600">{{ row.potential }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="tag" label="天赋" width="80" align="center">
              <template #default="{ row }">
                <el-tag :type="getTagType(row.tag)" size="small">{{ getTagLabel(row.tag) }}</el-tag>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <input
          ref="fileInputRef"
          type="file"
          accept=".json,.csv"
          style="display: none"
          @change="handleFileUpload"
        />
      </div>
      <template #footer>
        <el-button @click="closeBatchImportDialog">取消</el-button>
        <el-button
          type="primary"
          @click="confirmBatchImport"
          :disabled="parsedPlayers.length === 0"
        >
          确认导入 {{ parsedPlayers.length > 0 ? `(${Math.min(parsedPlayers.length, 14 - currentPoolData.length)}名)` : '' }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import {
  ArrowLeft,
  ArrowDown,
  FolderOpened,
  Upload,
  UploadFilled,
  Star,
  User,
  UserFilled,
  Collection,
  MagicStick,
  Delete,
  Edit,
  Document,
  Plus,
  SuccessFilled,
  RefreshLeft,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { draftApi, queryApi } from '@/api/tauri'

// 赛区列表
const regionList = [
  { code: 'lpl', name: '中国' },
  { code: 'lck', name: '韩国' },
  { code: 'lec', name: '欧洲' },
  { code: 'lcs', name: '北美' },
]

// 赛区名称映射
const regionNames: Record<string, string> = {
  lpl: 'LPL',
  lck: 'LCK',
  lec: 'LEC',
  lcs: 'LCS',
}

// 状态
const selectedRegion = ref('lpl')
const showImportDialog = ref(false)
const showEditDialog = ref(false)
const showBatchImportDialog = ref(false)
const fileInputRef = ref<HTMLInputElement | null>(null)
const parsedPlayers = ref<Omit<PoolPlayer, 'id' | 'region'>[]>([])
const isDragover = ref(false)
const activeFormatTab = ref<'json' | 'csv'>('json')
const isLoading = ref(false)

// 导入表单
const importForm = ref({
  gameId: '',
  position: 'MID',
  ability: 50,
  potential: 70,
  tag: 'NORMAL',
})

// 编辑表单
const editForm = ref({
  id: '',
  gameId: '',
  position: 'MID',
  ability: 50,
  potential: 70,
  tag: 'NORMAL',
})

// 选手池数据
interface PoolPlayer {
  id: string
  gameId: string
  position: string
  ability: number
  potential: number
  tag: string
  region: string
}

// 从后端加载的选手池数据
const poolData = ref<PoolPlayer[]>([])

// 获取赛区ID
const getRegionId = async (regionCode: string): Promise<number> => {
  try {
    const regions = await queryApi.getAllRegions()
    const region = regions.find(r => r.code.toLowerCase() === regionCode.toLowerCase())
    return region?.id ?? 1
  } catch (e) {
    console.error('Failed to get region id:', e)
    return 1
  }
}

// 加载选手池数据
const loadPoolData = async (regionCode: string) => {
  isLoading.value = true
  try {
    const regionId = await getRegionId(regionCode)
    const players = await draftApi.getAvailableDraftPlayers(regionId)

    // 转换后端数据格式为前端格式
    const regionPlayers = players.map(p => ({
      id: String(p.id),
      gameId: p.name,
      position: p.position,
      ability: p.ability,
      potential: p.potential,
      tag: p.tag,
      region: regionCode,
    }))

    // 更新当前赛区的数据
    poolData.value = poolData.value.filter(p => p.region !== regionCode)
    poolData.value.push(...regionPlayers)
  } catch (e) {
    console.error('Failed to load pool data:', e)
    // 如果加载失败，清空该赛区数据
    poolData.value = poolData.value.filter(p => p.region !== regionCode)
  } finally {
    isLoading.value = false
  }
}

// 初始化加载
onMounted(async () => {
  await loadPoolData(selectedRegion.value)
})

// 监听赛区切换
watch(selectedRegion, async (newRegion) => {
  await loadPoolData(newRegion)
})

// 计算属性
const currentRegionName = computed(() => regionNames[selectedRegion.value] || '')

const currentPoolData = computed(() => {
  return poolData.value.filter(p => p.region === selectedRegion.value)
})

const geniusCount = computed(() => {
  return currentPoolData.value.filter(p => p.tag === 'GENIUS').length
})

const normalCount = computed(() => {
  return currentPoolData.value.filter(p => p.tag === 'NORMAL').length
})

const mediocreCount = computed(() => {
  return currentPoolData.value.filter(p => p.tag === 'MEDIOCRE').length
})

// 方法
const getRegionPoolCount = (region: string) => {
  return poolData.value.filter(p => p.region === region).length
}

const getCardClass = (tag: string) => {
  if (tag === 'GENIUS') return 'genius'
  if (tag === 'MEDIOCRE') return 'mediocre'
  return 'normal'
}

const getTagLabel = (tag: string) => {
  const labels: Record<string, string> = {
    'GENIUS': '天才',
    'NORMAL': '一般',
    'MEDIOCRE': '平庸',
  }
  return labels[tag] || tag
}

const getTagType = (tag: string) => {
  const types: Record<string, string> = {
    'GENIUS': 'warning',
    'NORMAL': '',
    'MEDIOCRE': 'info',
  }
  return types[tag] || ''
}

const getAbilityColor = (ability: number) => {
  if (ability >= 70) return '#22c55e'
  if (ability >= 60) return '#f59e0b'
  return '#ef4444'
}

const importPlayer = () => {
  if (!importForm.value.gameId.trim()) {
    ElMessage.warning('请输入游戏ID')
    return
  }

  if (currentPoolData.value.length >= 14) {
    ElMessage.warning('选手池已满，无法继续添加')
    return
  }

  const newPlayer: PoolPlayer = {
    id: Date.now().toString(),
    gameId: importForm.value.gameId,
    position: importForm.value.position,
    ability: importForm.value.ability,
    potential: importForm.value.potential,
    tag: importForm.value.tag,
    region: selectedRegion.value,
  }

  poolData.value.push(newPlayer)
  showImportDialog.value = false

  // 重置表单
  importForm.value = {
    gameId: '',
    position: 'MID',
    ability: 50,
    potential: 70,
    tag: 'NORMAL',
  }

  ElMessage.success('新秀导入成功')
}

const editPlayer = (player: PoolPlayer) => {
  editForm.value = { ...player }
  showEditDialog.value = true
}

const savePlayer = () => {
  const index = poolData.value.findIndex(p => p.id === editForm.value.id)
  if (index !== -1) {
    poolData.value[index] = { ...editForm.value, region: selectedRegion.value }
    showEditDialog.value = false
    ElMessage.success('保存成功')
  }
}

const removePlayer = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定要移除该新秀吗？', '确认移除', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    })

    const index = poolData.value.findIndex(p => p.id === id)
    if (index !== -1) {
      poolData.value.splice(index, 1)
      ElMessage.success('已移除')
    }
  } catch {
    // 取消操作
  }
}

const clearPool = async () => {
  try {
    await ElMessageBox.confirm('确定要清空当前赛区选手池吗？', '确认清空', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    })

    poolData.value = poolData.value.filter(p => p.region !== selectedRegion.value)
    ElMessage.success('选手池已清空')
  } catch {
    // 取消操作
  }
}

// 随机生成选手池 - 调用后端API
const generateRandomPool = async () => {
  const currentCount = currentPoolData.value.length

  if (currentCount >= 14) {
    ElMessage.warning('选手池已满')
    return
  }

  isLoading.value = true
  try {
    const regionId = await getRegionId(selectedRegion.value)
    // 调用后端API生成选秀池
    const players = await draftApi.generateDraftPool(regionId, 14)

    // 转换后端数据格式为前端格式
    const regionPlayers = players.map(p => ({
      id: String(p.id),
      gameId: p.name,
      position: p.position,
      ability: p.ability,
      potential: p.potential,
      tag: p.tag,
      region: selectedRegion.value,
    }))

    // 更新当前赛区的数据
    poolData.value = poolData.value.filter(p => p.region !== selectedRegion.value)
    poolData.value.push(...regionPlayers)

    ElMessage.success(`已生成 ${regionPlayers.length} 名新秀`)
  } catch (e) {
    console.error('Failed to generate draft pool:', e)
    ElMessage.error('生成选手池失败')
  } finally {
    isLoading.value = false
  }
}

// 处理导入命令
const handleImportCommand = (command: string) => {
  if (command === 'single') {
    showImportDialog.value = true
  } else if (command === 'file') {
    showBatchImportDialog.value = true
  }
}

// 触发文件选择
const triggerFileInput = () => {
  fileInputRef.value?.click()
}

// 处理拖放
const handleDrop = (event: DragEvent) => {
  isDragover.value = false
  const file = event.dataTransfer?.files?.[0]
  if (file) {
    processFile(file)
  }
}

// 处理文件上传
const handleFileUpload = async (event: Event) => {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (file) {
    processFile(file)
  }
  // 重置 input 以便可以再次选择同一文件
  input.value = ''
}

// 处理文件
const processFile = async (file: File) => {
  try {
    const content = await readFileContent(file)
    const fileName = file.name.toLowerCase()

    if (fileName.endsWith('.json')) {
      parseJSONContent(content)
    } else if (fileName.endsWith('.csv')) {
      parseCSVContent(content)
    } else {
      ElMessage.error('不支持的文件格式，请使用 JSON 或 CSV 文件')
      return
    }

    if (parsedPlayers.value.length === 0) {
      ElMessage.warning('未解析到有效的选手数据，请检查文件格式')
    }
  } catch (error) {
    ElMessage.error('文件解析失败，请检查文件格式')
    console.error(error)
  }
}

// 重置上传
const resetUpload = () => {
  parsedPlayers.value = []
}

// 关闭批量导入对话框
const closeBatchImportDialog = () => {
  showBatchImportDialog.value = false
  parsedPlayers.value = []
  isDragover.value = false
}

// 读取文件内容
const readFileContent = (file: File): Promise<string> => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = reject
    reader.readAsText(file)
  })
}

// 解析 JSON 内容
const parseJSONContent = (content: string) => {
  try {
    const data = JSON.parse(content)
    const players = Array.isArray(data) ? data : [data]
    parsedPlayers.value = validateAndNormalizePlayers(players)
  } catch {
    ElMessage.error('JSON 格式错误')
    parsedPlayers.value = []
  }
}

// 解析 CSV 内容
const parseCSVContent = (content: string) => {
  const lines = content.trim().split('\n')
  if (lines.length < 2) {
    ElMessage.error('CSV 文件格式错误，缺少数据行')
    parsedPlayers.value = []
    return
  }

  // 解析表头
  const headers = lines[0].split(',').map(h => h.trim().toLowerCase())
  const requiredHeaders = ['gameid', 'position', 'ability', 'potential', 'tag']

  // 检查必要字段
  const missingHeaders = requiredHeaders.filter(h => !headers.includes(h))
  if (missingHeaders.length > 0) {
    ElMessage.error(`CSV 缺少必要字段：${missingHeaders.join(', ')}`)
    parsedPlayers.value = []
    return
  }

  // 解析数据行
  const players = []
  for (let i = 1; i < lines.length; i++) {
    const line = lines[i].trim()
    if (!line) continue

    const values = line.split(',').map(v => v.trim())
    const player: Record<string, any> = {}

    headers.forEach((header, index) => {
      player[header] = values[index] || ''
    })

    players.push({
      gameId: player.gameid || player.game_id,
      position: player.position,
      ability: parseInt(player.ability, 10),
      potential: parseInt(player.potential, 10),
      tag: player.tag,
    })
  }

  parsedPlayers.value = validateAndNormalizePlayers(players)
}

// 验证和规范化选手数据
const validateAndNormalizePlayers = (players: any[]): Omit<PoolPlayer, 'id' | 'region'>[] => {
  const validPositions = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']
  const validTags = ['GENIUS', 'NORMAL', 'MEDIOCRE']

  return players
    .filter(p => {
      // 验证必填字段
      if (!p.gameId || typeof p.gameId !== 'string') return false
      if (!validPositions.includes(p.position?.toUpperCase())) return false
      if (isNaN(p.ability) || p.ability < 30 || p.ability > 80) return false
      if (isNaN(p.potential) || p.potential < 50 || p.potential > 99) return false
      if (!validTags.includes(p.tag?.toUpperCase())) return false
      return true
    })
    .map(p => ({
      gameId: p.gameId.trim(),
      position: p.position.toUpperCase(),
      ability: Number(p.ability),
      potential: Number(p.potential),
      tag: p.tag.toUpperCase(),
    }))
}

// 确认批量导入
const confirmBatchImport = () => {
  const remainingSlots = 14 - currentPoolData.value.length
  const playersToImport = parsedPlayers.value.slice(0, remainingSlots)

  playersToImport.forEach((player, index) => {
    const newPlayer: PoolPlayer = {
      id: Date.now().toString() + index,
      ...player,
      region: selectedRegion.value,
    }
    poolData.value.push(newPlayer)
  })

  ElMessage.success(`成功导入 ${playersToImport.length} 名新秀`)
  closeBatchImportDialog()
}
</script>

<style scoped lang="scss">
.draft-pool-view {
  padding: 0;
}

/* 返回导航 */
.back-nav {
  margin-bottom: 20px;
}

.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 0;
  background: none;
  border: none;
  color: #6b7280;
  font-size: 14px;
  cursor: pointer;
  transition: color 0.2s;

  &:hover {
    color: #3b82f6;
  }
}

/* 页面头部 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  border-radius: 14px;
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  color: white;
}

.header-info {
  .page-title {
    font-size: 24px;
    font-weight: 700;
    color: #1f2937;
    margin: 0 0 4px 0;
  }

  .page-desc {
    font-size: 14px;
    color: #6b7280;
    margin: 0;
  }
}

/* 赛区标签 */
.region-tabs {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
  padding: 16px;
  background: white;
  border-radius: 14px;
  border: 1px solid #e5e7eb;
}

.region-tab {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  padding: 12px 16px;
  background: #f9fafb;
  border: 2px solid transparent;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    background: #f3f4f6;
  }

  &.active {
    background: white;
    border-color: #3b82f6;
    box-shadow: 0 2px 8px rgba(59, 130, 246, 0.15);
  }

  .tab-badge {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 8px;
    font-size: 11px;
    font-weight: 700;
    color: white;

    &.lpl { background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%); }
    &.lck { background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%); }
    &.lec { background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%); }
    &.lcs { background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); }
  }

  .tab-name {
    font-size: 14px;
    font-weight: 600;
    color: #374151;
  }

  .tab-count {
    margin-left: auto;
    font-size: 12px;
    color: #9ca3af;
    padding: 2px 8px;
    background: #f3f4f6;
    border-radius: 10px;
  }
}

/* 统计卡片 */
.stats-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: white;
  border-radius: 12px;
  border: 1px solid #e5e7eb;

  .stat-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 48px;
    border-radius: 12px;
    font-size: 20px;

    &.genius {
      background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
      color: #d97706;
    }

    &.normal {
      background: linear-gradient(135deg, #dbeafe 0%, #bfdbfe 100%);
      color: #2563eb;
    }

    &.mediocre {
      background: linear-gradient(135deg, #f3f4f6 0%, #e5e7eb 100%);
      color: #6b7280;
    }

    &.total {
      background: linear-gradient(135deg, #dcfce7 0%, #bbf7d0 100%);
      color: #16a34a;
    }
  }

  .stat-info {
    display: flex;
    flex-direction: column;

    .stat-value {
      font-size: 24px;
      font-weight: 700;
      color: #1f2937;
    }

    .stat-label {
      font-size: 13px;
      color: #6b7280;
    }
  }
}

/* 选手池区块 */
.pool-section {
  background: white;
  border-radius: 16px;
  border: 1px solid #e5e7eb;
  padding: 24px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;

  .header-title {
    display: flex;
    align-items: center;
    gap: 12px;

    h2 {
      font-size: 18px;
      font-weight: 600;
      color: #1f2937;
      margin: 0;
    }
  }

  .header-actions {
    display: flex;
    gap: 10px;
  }
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 60px 20px;

  .empty-icon {
    color: #d1d5db;
    margin-bottom: 16px;
  }

  h3 {
    font-size: 18px;
    font-weight: 600;
    color: #374151;
    margin: 0 0 8px 0;
  }

  p {
    font-size: 14px;
    color: #6b7280;
    margin: 0 0 24px 0;
  }

  .empty-actions {
    display: flex;
    justify-content: center;
    gap: 12px;
  }
}

/* 选手池网格 */
.pool-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 14px;
}

.pool-card {
  padding: 16px;
  background: #f9fafb;
  border: 2px solid transparent;
  border-radius: 12px;
  transition: all 0.2s;

  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  }

  &.genius {
    background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);
    border-color: #fbbf24;
  }

  &.mediocre {
    background: linear-gradient(135deg, #f9fafb 0%, #f3f4f6 100%);
    border-color: #d1d5db;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;

    .player-rank {
      font-size: 12px;
      font-weight: 600;
      color: #6b7280;
    }

    .player-tag {
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 11px;
      font-weight: 600;

      &.genius {
        background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
        color: #92400e;
      }

      &.normal {
        background: #dbeafe;
        color: #1e40af;
      }

      &.mediocre {
        background: #f3f4f6;
        color: #6b7280;
      }
    }
  }

  .card-body {
    text-align: center;
    margin-bottom: 12px;

    .player-name {
      font-size: 15px;
      font-weight: 600;
      color: #1f2937;
      margin-bottom: 4px;
    }

    .player-position {
      font-size: 12px;
      color: #6b7280;
    }
  }

  .card-stats {
    display: flex;
    justify-content: center;
    gap: 20px;
    margin-bottom: 12px;

    .stat-item {
      text-align: center;

      .stat-value {
        display: block;
        font-size: 18px;
        font-weight: 700;
        line-height: 1;

        &.potential {
          color: #8b5cf6;
        }
      }

      .stat-label {
        font-size: 11px;
        color: #9ca3af;
      }
    }
  }

  .card-footer {
    display: flex;
    justify-content: center;
    gap: 8px;
    padding-top: 12px;
    border-top: 1px solid #e5e7eb;
  }
}

/* 批量导入对话框 */
.batch-import-content {
  .upload-area {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    border: 2px dashed #d1d5db;
    border-radius: 12px;
    background: #f9fafb;
    cursor: pointer;
    transition: all 0.2s;

    &:hover {
      border-color: #3b82f6;
      background: #eff6ff;
    }

    &.is-dragover {
      border-color: #3b82f6;
      background: #dbeafe;
      border-style: solid;
    }

    .upload-icon {
      color: #9ca3af;
      margin-bottom: 12px;
    }

    .upload-text {
      text-align: center;

      .main-text {
        font-size: 15px;
        color: #374151;
        margin: 0 0 4px 0;

        .link {
          color: #3b82f6;
          font-weight: 500;
        }
      }

      .sub-text {
        font-size: 13px;
        color: #9ca3af;
        margin: 0;
      }
    }
  }

  .format-guide {
    margin-top: 24px;

    h4 {
      font-size: 15px;
      font-weight: 600;
      color: #1f2937;
      margin: 0 0 12px 0;
    }

    .format-tabs {
      display: flex;
      gap: 8px;
      margin-bottom: 12px;

      .format-tab {
        padding: 8px 16px;
        font-size: 13px;
        font-weight: 500;
        color: #6b7280;
        background: #f3f4f6;
        border: none;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.2s;

        &:hover {
          background: #e5e7eb;
        }

        &.active {
          background: #3b82f6;
          color: white;
        }
      }
    }

    .format-content {
      .code-block {
        padding: 16px;
        background: #1f2937;
        color: #e5e7eb;
        border-radius: 8px;
        font-size: 12px;
        font-family: 'Monaco', 'Menlo', monospace;
        line-height: 1.6;
        overflow-x: auto;
        margin: 0;
      }
    }

    .field-desc {
      margin-top: 16px;

      h5 {
        font-size: 14px;
        font-weight: 600;
        color: #374151;
        margin: 0 0 10px 0;
      }

      .desc-table {
        width: 100%;
        border-collapse: collapse;
        font-size: 13px;

        th, td {
          padding: 10px 12px;
          text-align: left;
          border: 1px solid #e5e7eb;
        }

        th {
          background: #f9fafb;
          font-weight: 600;
          color: #374151;
        }

        td {
          color: #4b5563;

          code {
            padding: 2px 6px;
            background: #f3f4f6;
            border-radius: 4px;
            font-family: 'Monaco', 'Menlo', monospace;
            font-size: 12px;
            color: #dc2626;
          }
        }
      }
    }
  }

  .preview-area {
    .preview-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;

      .preview-info {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 15px;
        color: #374151;

        strong {
          color: #22c55e;
          font-weight: 700;
        }
      }
    }

    .preview-tips {
      padding: 12px 16px;
      background: #f9fafb;
      border-radius: 8px;
      margin-bottom: 16px;

      p {
        margin: 4px 0;
        font-size: 14px;
        color: #4b5563;

        .label {
          color: #6b7280;
        }

        &.warning {
          color: #dc2626;
          font-weight: 500;
        }
      }
    }
  }
}
</style>

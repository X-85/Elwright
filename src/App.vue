<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import CapabilityDetail from './components/CapabilityDetail.vue'
import CapabilityList from './components/CapabilityList.vue'
import ChatView from './components/ChatView.vue'
import SettingsCenter from './components/SettingsCenter.vue'
import TerminalPanel from './components/TerminalPanel.vue'
import { createBridge, type Bridge, type Capability } from './lib/bridge'
import { Blocks, MessagesSquare, PanelLeft, PanelRight, Settings2, Terminal } from 'lucide-vue-next'

const bridge: Bridge = createBridge()
const capabilities = ref<Capability[]>([])
const loadError = ref('')
const filter = ref<'all' | 'script' | 'knowledge' | 'skill'>('all')
const search = ref('')
const selectedId = ref('')
const showSettings = ref(false)
const settingsSection = ref<'general' | 'appearance' | 'model' | 'terminal'>('appearance')
// 一级视图：能力工具箱 ⇄ AI 对话（chat 阶段①）
const activeView = ref<'toolbox' | 'chat'>('toolbox')
const leftPanelVisible = ref(true)
const rightPanelVisible = ref(false)
const chatViewRef = ref<InstanceType<typeof ChatView> | null>(null)
const terminalRef = ref<InstanceType<typeof import('./components/TerminalPanel.vue').default> | null>(null)
// 应用 cwd：浏览器预览用空字符串，桌面用 process.cwd()（简单做法）
const cwd = ref('')

// 能力导入/删除的反馈（toast 式，几秒后自动消失）
const opMsg = ref('')
const opOk = ref(true)
let opTimer: ReturnType<typeof setTimeout> | undefined

function notify(message: string, ok: boolean) {
  opMsg.value = message
  opOk.value = ok
  clearTimeout(opTimer)
  opTimer = setTimeout(() => (opMsg.value = ''), 6000)
}

async function reload() {
  try {
    capabilities.value = await bridge.listCapabilities()
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e)
  }
}

// 导入：id 冲突时弹确认框，确认后带 force 重试
async function importCapability(force = false) {
  const result = await bridge.importCapability(force)
  notify(result.message, result.ok)
  if (result.ok) {
    selectedId.value = ''
    await reload()
  } else if (result.conflict && confirm('该能力已存在，覆盖导入吗？（覆盖后为自定义版本）')) {
    await importCapability(true)
  }
}

async function onDeleted() {
  selectedId.value = ''
  await reload()
}

// 检查更新：手动触发（不轮询，离网友好），结果展示在按钮下方
const checking = ref(false)
const updateMsg = ref('')
const updateUrl = ref('')

async function checkUpdate() {
  if (checking.value) return
  checking.value = true
  updateMsg.value = ''
  updateUrl.value = ''
  try {
    const info = await bridge.checkUpdate()
    if (info.updateAvailable) {
      updateMsg.value = `发现新版本 v${info.latest}（当前 v${info.current}）`
      updateUrl.value = info.releaseUrl
    } else {
      updateMsg.value = `已是最新版本（v${info.current}）`
    }
  } catch (e) {
    updateMsg.value = `检查更新失败：${e instanceof Error ? e.message : String(e)}`
  } finally {
    checking.value = false
  }
}

function openDownload() {
  if (updateUrl.value) bridge.openExternal(updateUrl.value)
}

onMounted(reload)

const filtered = computed(() => {
  const kw = search.value.trim().toLowerCase()
  return capabilities.value.filter((c) => {
    if (filter.value !== 'all' && c.type !== filter.value) return false
    if (!kw) return true
    return (
      c.id.toLowerCase().includes(kw) ||
      c.name.toLowerCase().includes(kw) ||
      (c.category ?? '').toLowerCase().includes(kw)
    )
  })
})

const selected = computed(
  () => capabilities.value.find((c) => c.id === selectedId.value) ?? null,
)

function select(id: string) {
  selectedId.value = selectedId.value === id ? '' : id
}

// 设置弹层关闭后刷新对话页的模型状态条（可能刚保存了配置）
function openSettings(section: 'general' | 'appearance' | 'model' | 'terminal' = 'appearance') {
  settingsSection.value = section
  showSettings.value = true
}

function onSettingsSaved() {
  chatViewRef.value?.refreshConfig()
}

function toggleTerminal() {
  // ZCode 式：无 tab 时新建一个（主目录）并展开；有 tab 纯展开/收起切换
  terminalRef.value?.toggleFromToolbar()
}
</script>

<template>
  <div class="layout">
    <header class="topbar">
      <div class="topbar-brand">Elwright</div>
      <nav class="topbar-nav" aria-label="主导航">
        <button
          :class="{ active: activeView === 'toolbox' }"
          title="能力工具箱"
          aria-label="能力工具箱"
          @click="activeView = 'toolbox'"
        >
          <Blocks :size="18" :stroke-width="1.8" />
        </button>
        <button
          :class="{ active: activeView === 'chat' }"
          title="AI 对话"
          aria-label="AI 对话"
          @click="activeView = 'chat'"
        >
          <MessagesSquare :size="18" :stroke-width="1.8" />
        </button>
      </nav>
      <div class="topbar-spacer"></div>
      <div class="topbar-actions">
        <button class="topbar-action" :class="{ active: leftPanelVisible }" :title="leftPanelVisible ? '隐藏左侧栏' : '显示左侧栏'" :aria-label="leftPanelVisible ? '隐藏左侧栏' : '显示左侧栏'" @click="leftPanelVisible = !leftPanelVisible">
          <PanelLeft :size="17" :stroke-width="1.8" />
        </button>
        <button class="topbar-action" :class="{ active: rightPanelVisible }" :title="rightPanelVisible ? '隐藏右侧栏' : '显示右侧栏'" :aria-label="rightPanelVisible ? '隐藏右侧栏' : '显示右侧栏'" @click="rightPanelVisible = !rightPanelVisible">
          <PanelRight :size="17" :stroke-width="1.8" />
        </button>
        <button v-if="bridge.kind === 'tauri'" class="topbar-action" title="打开或收起终端" aria-label="打开或收起终端" @click="toggleTerminal">
          <Terminal :size="17" :stroke-width="1.8" />
        </button>
        <button class="topbar-action" title="打开设置" aria-label="打开设置" @click="openSettings()">
          <Settings2 :size="17" :stroke-width="1.8" />
        </button>
      </div>
    </header>

    <div :class="['workspace-shell', { 'left-collapsed': !leftPanelVisible, 'right-collapsed': !rightPanelVisible, 'both-collapsed': !leftPanelVisible && !rightPanelVisible }]">
      <aside v-if="leftPanelVisible" class="sidebar">
        <nav v-if="activeView === 'toolbox'" class="filters">
          <button v-for="f in ['all', 'script', 'knowledge', 'skill'] as const" :key="f" :class="{ active: filter === f }" @click="filter = f">
            {{ { all: '全部', script: '脚本型', knowledge: '知识型', skill: '技能型' }[f] }}
          </button>
        </nav>
        <input v-if="activeView === 'toolbox'" v-model="search" class="search" placeholder="搜索 id / 名称 / 分类…" />
        <div v-if="activeView === 'toolbox'" class="sidebar-row">
          <button class="import-btn" @click="importCapability()">＋ 导入能力…</button>
        </div>
        <p v-if="activeView === 'toolbox'" class="count">{{ filtered.length }} / {{ capabilities.length }} 项</p>
        <transition name="fade"><p v-if="opMsg" :class="['op-toast', opOk ? 'op-ok' : 'op-err']">{{ opMsg }}</p></transition>
        <div class="update-box">
          <button class="update-btn" :disabled="checking" @click="checkUpdate">{{ checking ? '检查中…' : '检查更新' }}</button>
          <p v-if="updateMsg" class="update-msg">{{ updateMsg }}</p>
          <button v-if="updateUrl" class="update-link" @click="openDownload">前往下载 →</button>
        </div>
        <p class="bridge-badge">{{ bridge.kind === 'tauri' ? '桌面模式 · Tauri' : '预览模式 · 浏览器' }}</p>
      </aside>

      <main class="content">
        <ChatView v-if="activeView === 'chat'" ref="chatViewRef" :bridge="bridge" @open-settings="openSettings('model')" />
        <template v-else>
          <p v-if="loadError" class="error">加载失败：{{ loadError }}</p>
          <CapabilityList v-else :capabilities="filtered" :selected-id="selectedId" @select="select" />
          <CapabilityDetail v-if="selected" :cap="selected" :bridge="bridge" @notify="notify" @deleted="onDeleted" @open-settings="openSettings('model')" />
          <div v-else-if="!loadError" class="placeholder">← 选择一项能力查看详情</div>
        </template>
      </main>

      <aside v-if="rightPanelVisible" class="context-panel">
        <div class="context-placeholder">上下文面板</div>
      </aside>
    </div>

    <SettingsCenter v-if="showSettings" :bridge="bridge" :initial-section="settingsSection" @close="showSettings = false" @saved="onSettingsSaved" />

    <TerminalPanel ref="terminalRef" :bridge="bridge" :cwd="cwd" />
  </div>
</template>

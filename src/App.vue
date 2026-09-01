<script setup lang="ts">
import { computed, onMounted, ref, nextTick, watch } from 'vue'
import CapabilityDetail from './components/CapabilityDetail.vue'
import CapabilityList from './components/CapabilityList.vue'
import ChatView from './components/ChatView.vue'
import PeopleChatView from './components/PeopleChatView.vue'
import SettingsCenter from './components/SettingsCenter.vue'
import WorkbenchView from './components/WorkbenchView.vue'
import TerminalPanel from './components/TerminalPanel.vue'
import WorkspaceView from './components/WorkspaceView.vue'
import CodeBrowserView from './components/CodeBrowserView.vue'
import { createBridge, type Bridge, type Capability } from './lib/bridge'
import {
  Blocks,
  BookOpen,
  Columns3,
  Code2,
  ListTodo,
  Maximize2,
  MessageCircle,
  PanelBottom,
  PanelLeft,
  PanelRight,
  PanelTop,
  Settings2,
  Sparkles,
  Terminal,
} from 'lucide-vue-next'
import { currentMonitor, getCurrentWindow, LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize } from '@tauri-apps/api/window'

const bridge: Bridge = createBridge()
const capabilities = ref<Capability[]>([])
const loadError = ref('')
const filter = ref<'all' | 'script' | 'knowledge' | 'skill'>('all')
const search = ref('')
const selectedId = ref('')
const revealAllCapabilities = ref(localStorage.getItem('elwright-capability-reveal-all') === 'true')
function loadCapabilityUses(): Record<string, number> {
  try {
    const raw = JSON.parse(localStorage.getItem('elwright-capability-uses') ?? '{}')
    return raw && typeof raw === 'object' && !Array.isArray(raw) ? raw : {}
  } catch {
    return {}
  }
}

const capabilityUses = ref<Record<string, number>>(loadCapabilityUses())
const showSettings = ref(false)
const settingsSection = ref<'general' | 'appearance' | 'model' | 'terminal'>('appearance')
// 一级视图：能力工具箱 ⇄ AI 对话（chat 阶段①）
const activeView = ref<'toolbox' | 'workbench' | 'chat' | 'people' | 'workspace' | 'code'>('toolbox')
import { initializePreferences, resolveStartupView, saveLastView, preferences } from './lib/preferences'
import { growthSummary, newlyUnlocked } from './lib/growth'
import { recordRecent } from './lib/capabilityRecents'
watch(() => preferences.value.startupView, () => {}) // 保持模块热路径引用（初始化在 onMounted）
const leftPanelVisible = ref(true)
const rightPanelVisible = ref(false)
const chatViewRef = ref<InstanceType<typeof ChatView> | null>(null)
const codeBrowserRef = ref<InstanceType<typeof CodeBrowserView> | null>(null)
const terminalRef = ref<InstanceType<typeof import('./components/TerminalPanel.vue').default> | null>(null)
const appWindow = bridge.kind === 'tauri' ? getCurrentWindow() : null
const windowLayoutMenuOpen = ref(false)
const savedWindowBounds = ref<{ position: PhysicalPosition; size: PhysicalSize } | null>(null)
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

onMounted(() => {
  initializePreferences()
  activeView.value = resolveStartupView(preferences.value.startupView)
  if (preferences.value.autoUpdateCheck) void checkUpdate()
  reload()
})
watch(activeView, (v) => saveLastView(v))

const filtered = computed(() => {
  const kw = search.value.trim().toLowerCase()
  return capabilities.value.filter((c) => {
    if (!revealAllCapabilities.value && !isCapabilityUnlocked(c)) return false
    if (filter.value !== 'all' && c.type !== filter.value) return false
    if (!kw) return true
    return (
      c.id.toLowerCase().includes(kw) ||
      c.name.toLowerCase().includes(kw) ||
      (c.category ?? '').toLowerCase().includes(kw)
    )
  })
})

const lockedCount = computed(() => capabilities.value.filter((c) => !isCapabilityUnlocked(c)).length)
const totalCapabilityUses = computed(() => Object.values(capabilityUses.value).reduce((sum, count) => sum + count, 0))
// 成长提示（ADR-001）：距最近可解锁项的进度；无锁定项或均未开放条件时为 null
const nearestGrowth = computed(() => growthSummary(capabilities.value, totalCapabilityUses.value).nearest)

function isCapabilityUnlocked(cap: Capability): boolean {
  const tier = cap.releaseTier ?? 1
  if (tier <= 1) return true
  const required = cap.unlockAfterUses
  return required !== undefined && totalCapabilityUses.value >= required
}

function toggleRevealAll() {
  revealAllCapabilities.value = !revealAllCapabilities.value
  localStorage.setItem('elwright-capability-reveal-all', String(revealAllCapabilities.value))
}

const selected = computed(
  () => capabilities.value.find((item) => item.id === selectedId.value) ?? null,
)

function select(id: string) {
  selectedId.value = selectedId.value === id ? '' : id
  const cap = capabilities.value.find((item) => item.id === id)
  if (cap && isCapabilityUnlocked(cap)) {
    const prev = totalCapabilityUses.value
    capabilityUses.value[id] = (capabilityUses.value[id] ?? 0) + 1
    localStorage.setItem('elwright-capability-uses', JSON.stringify(capabilityUses.value))
    // 工作台「最近使用」（ADR：工作台二期）——与使用计数同时机记录
    recordRecent(id)
    // 成长提示（ADR-001）：跨过解锁门槛的瞬间给出反馈
    for (const name of newlyUnlocked(capabilities.value, prev, totalCapabilityUses.value)) {
      notify(`🎉 已解锁进阶能力「${name}」——在列表中即可使用`, true)
    }
  }
}

/** 工作台「常用能力」点击：跳到能力工具箱并打开详情。 */
function openCapabilityFromWorkbench(id: string) {
  activeView.value = 'toolbox'
  selectedId.value = id
}

// 设置弹层关闭后刷新对话页的模型状态条（可能刚保存了配置）
function openSettings(section: 'general' | 'appearance' | 'model' | 'terminal' = 'appearance') {
  settingsSection.value = section
  showSettings.value = true
}

function onSettingsSaved() {
  chatViewRef.value?.refreshConfig()
}

async function sendCodeToAi(payload: { title: string; text: string }) {
  activeView.value = 'chat'
  await nextTick()
  chatViewRef.value?.insertContext(payload.title, payload.text)
  notify('代码上下文已填入对话输入框，发送前可编辑', true)
}

async function openCodeInTerminal(dir: string) {
  if (bridge.kind !== 'tauri') {
    notify('【预览模式】终端仅桌面模式可用', false)
    return
  }
  await terminalRef.value?.openAtDir(dir)
  notify('已在终端中打开目录', true)
}

async function openCodeAt(absPath: string, line: number) {
  activeView.value = 'code'
  await nextTick()
  await codeBrowserRef.value?.openAbsolute(absPath, line)
}

function toggleTerminal() {
  // ZCode 式：无 tab 时新建一个（主目录）并展开；有 tab 纯展开/收起切换
  terminalRef.value?.toggleFromToolbar()
}

async function closeWindow() {
  try {
    await appWindow?.close()
  } catch (error) {
    console.warn('[window] close failed:', error)
  }
}

async function minimizeWindow() {
  try {
    await appWindow?.minimize()
  } catch (error) {
    console.warn('[window] minimize failed:', error)
  }
}

async function applyWindowLayout(
  layout: 'fullscreen' | 'left' | 'right' | 'top' | 'bottom' | 'fill' | 'three-column' | 'quarter-tl' | 'quarter-tr' | 'quarter-bl' | 'quarter-br' | 'restore',
) {
  if (!appWindow) return
  try {
    if (layout === 'fullscreen') {
      await appWindow.setFullscreen(true)
    } else {
      if (await appWindow.isFullscreen()) await appWindow.setFullscreen(false)
      if (layout === 'restore') {
        if (savedWindowBounds.value) {
          await appWindow.setPosition(savedWindowBounds.value.position)
          await appWindow.setSize(savedWindowBounds.value.size)
          await appWindow.setMinSize(new LogicalSize(960, 640))
          savedWindowBounds.value = null
        }
      } else {
        const monitor = await currentMonitor()
        if (!monitor) return
        if (!savedWindowBounds.value) {
          savedWindowBounds.value = {
            position: await appWindow.outerPosition(),
            size: await appWindow.outerSize(),
          }
        }
        const { position, size } = monitor.workArea
        const scale = monitor.scaleFactor
        const workX = position.x / scale
        const workY = position.y / scale
        const workWidth = size.width / scale
        const workHeight = size.height / scale
        const halfWidth = Math.floor(workWidth / 2)
        const halfHeight = Math.floor(workHeight / 2)
        let x = workX
        let y = workY
        let width = workWidth
        let height = workHeight
        if (layout === 'left') {
          width = halfWidth
        } else if (layout === 'right') {
          x = workX + halfWidth
          width = workWidth - halfWidth
        } else if (layout === 'top') {
          height = halfHeight
        } else if (layout === 'bottom') {
          y = workY + halfHeight
          height = workHeight - halfHeight
        } else if (layout === 'three-column') {
          width = Math.floor(workWidth / 3)
        } else if (layout === 'quarter-tl') {
          width = halfWidth
          height = halfHeight
        } else if (layout === 'quarter-tr') {
          x = workX + halfWidth
          width = halfWidth
          height = halfHeight
        } else if (layout === 'quarter-bl') {
          y = workY + halfHeight
          width = halfWidth
          height = halfHeight
        } else if (layout === 'quarter-br') {
          x = workX + halfWidth
          y = workY + halfHeight
          width = halfWidth
          height = halfHeight
        }
        // The app's normal 960px minimum is wider than half a Retina display.
        // Temporarily relax it so the snap can occupy the exact half work area.
        await appWindow.setMinSize(new LogicalSize(320, 400))
        await appWindow.setPosition(new LogicalPosition(x, y))
        await appWindow.setSize(new LogicalSize(width, height))
      }
    }
    windowLayoutMenuOpen.value = false
  } catch (error) {
    console.warn(`[window] ${layout} layout failed:`, error)
  }
}

async function startWindowDrag(event: MouseEvent) {
  if (!appWindow) return
  const target = event.target as HTMLElement
  if (target.closest('button')) return
  try {
    await appWindow.startDragging()
  } catch (error) {
    console.warn('[window] start dragging failed:', error)
  }
}

/** 原生绿点行为：点击切换全屏；macOS 退出全屏自动恢复原尺寸，无需手动 restore。 */
async function toggleFullscreen() {
  if (!appWindow) return
  try {
    if (await appWindow.isFullscreen()) {
      await appWindow.setFullscreen(false)
    } else {
      await appWindow.setFullscreen(true)
    }
    windowLayoutMenuOpen.value = false
  } catch (error) {
    console.warn('[window] fullscreen toggle failed:', error)
  }
}
</script>

<template>
  <div class="layout" @click="windowLayoutMenuOpen = false">
    <header :class="['app-chrome', { 'left-collapsed': !leftPanelVisible, 'right-collapsed': !rightPanelVisible || activeView === 'workspace', 'both-collapsed': !leftPanelVisible && (!rightPanelVisible || activeView === 'workspace') }]" @mousedown="startWindowDrag">
      <div class="chrome-left">
        <div class="window-controls" aria-label="窗口控制">
          <button class="window-button close" :disabled="bridge.kind !== 'tauri'" title="关闭窗口" aria-label="关闭窗口" @mousedown.stop @click.stop="closeWindow"><span class="window-dot"></span></button>
          <button class="window-button minimize" :disabled="bridge.kind !== 'tauri'" title="最小化窗口" aria-label="最小化窗口" @mousedown.stop @click.stop="minimizeWindow"><span class="window-dot"></span></button>
          <div
            class="window-layout-control"
            @mouseenter="bridge.kind === 'tauri' && (windowLayoutMenuOpen = true)"
            @mouseleave="windowLayoutMenuOpen = false"
          >
            <button class="window-button maximize" :disabled="bridge.kind !== 'tauri'" title="全屏（悬停显示移动与调整大小）" aria-label="全屏" @mousedown.stop @click.stop="toggleFullscreen"><span class="window-dot"></span></button>
            <div v-if="windowLayoutMenuOpen" class="window-layout-menu" @mousedown.stop @click.stop>
              <section class="window-layout-section">
                <h3>移动与调整大小</h3>
                <div class="window-layout-options">
                  <button title="移到屏幕左侧" aria-label="移到屏幕左侧" @click="applyWindowLayout('left')"><PanelLeft :size="18" /></button>
                  <button title="移到屏幕右侧" aria-label="移到屏幕右侧" @click="applyWindowLayout('right')"><PanelRight :size="18" /></button>
                  <button title="移到屏幕上半部" aria-label="移到屏幕上半部" @click="applyWindowLayout('top')"><PanelTop :size="18" /></button>
                  <button title="移到屏幕下半部" aria-label="移到屏幕下半部" @click="applyWindowLayout('bottom')"><PanelBottom :size="18" /></button>
                </div>
              </section>
              <section class="window-layout-section">
                <h3>填充与排列</h3>
                <div class="window-layout-options">
                  <button class="window-layout-quarter" title="填充屏幕左上角" aria-label="填充屏幕左上角" @click="applyWindowLayout('quarter-tl')">左上</button>
                  <button class="window-layout-quarter" title="填充屏幕右上角" aria-label="填充屏幕右上角" @click="applyWindowLayout('quarter-tr')">右上</button>
                  <button class="window-layout-quarter" title="填充屏幕左下角" aria-label="填充屏幕左下角" @click="applyWindowLayout('quarter-bl')">左下</button>
                  <button class="window-layout-quarter" title="填充屏幕右下角" aria-label="填充屏幕右下角" @click="applyWindowLayout('quarter-br')">右下</button>
                  <button title="填充屏幕" aria-label="填充屏幕" @click="applyWindowLayout('fill')"><Maximize2 :size="18" /></button>
                  <button title="三列排列" aria-label="三列排列" @click="applyWindowLayout('three-column')"><Columns3 :size="18" /></button>
                </div>
              </section>
              <button class="window-layout-fullscreen" title="进入或退出全屏" aria-label="进入或退出全屏" @click="toggleFullscreen">
                <Maximize2 :size="16" />
                <span>全屏</span>
              </button>
              <button class="window-layout-restore" title="恢复窗口大小" aria-label="恢复窗口大小" @click="applyWindowLayout('restore')">恢复窗口</button>
            </div>
          </div>
        </div>
        <div class="chrome-brand">Elwright</div>
        <button class="panel-tool chrome-panel-toggle" :title="leftPanelVisible ? '隐藏左侧栏' : '显示左侧栏'" :aria-label="leftPanelVisible ? '隐藏左侧栏' : '显示左侧栏'" @mousedown.stop @click.stop="leftPanelVisible = !leftPanelVisible"><PanelLeft :size="16" :stroke-width="1.8" /></button>
      </div>
      <div class="chrome-right">
        <button class="panel-tool" :disabled="bridge.kind !== 'tauri'" :title="bridge.kind === 'tauri' ? '打开或收起终端' : '终端仅桌面模式可用'" aria-label="终端" @mousedown.stop @click.stop="toggleTerminal"><Terminal :size="16" :stroke-width="1.8" /></button>
        <button class="panel-tool" title="打开设置" aria-label="打开设置" @mousedown.stop @click.stop="openSettings()"><Settings2 :size="16" :stroke-width="1.8" /></button>
        <button class="panel-tool" :class="{ active: rightPanelVisible }" :title="rightPanelVisible ? '隐藏右侧栏' : '显示右侧栏'" :aria-label="rightPanelVisible ? '隐藏右侧栏' : '显示右侧栏'" @mousedown.stop @click.stop="rightPanelVisible = !rightPanelVisible"><PanelRight :size="16" :stroke-width="1.8" /></button>
      </div>
    </header>
    <div :class="['workspace-shell', { 'left-collapsed': !leftPanelVisible, 'right-collapsed': !rightPanelVisible || activeView === 'workspace', 'both-collapsed': !leftPanelVisible && (!rightPanelVisible || activeView === 'workspace') }]">
      <aside v-if="leftPanelVisible" class="sidebar">
        <nav class="sidebar-nav" aria-label="主导航">
          <button :class="{ active: activeView === 'toolbox' }" title="能力工具箱" aria-label="能力工具箱" @click="activeView = 'toolbox'"><Blocks :size="16" :stroke-width="1.8" /><span>能力</span></button>
          <button :class="{ active: activeView === 'chat' }" title="AI 对话" aria-label="AI 对话" @click="activeView = 'chat'"><Sparkles :size="16" :stroke-width="1.8" /><span>对话</span></button>
          <button :class="{ active: activeView === 'workbench' }" title="工作台" aria-label="工作台" @click="activeView = 'workbench'"><ListTodo :size="16" :stroke-width="1.8" /><span>工作台</span></button>
          <button :class="{ active: activeView === 'people' }" title="消息会话" aria-label="消息会话" @click="activeView = 'people'"><MessageCircle :size="16" :stroke-width="1.8" /><span>消息</span></button>
          <button :class="{ active: activeView === 'workspace' }" title="资源与课题" aria-label="资源与课题" @click="activeView = 'workspace'"><BookOpen :size="16" :stroke-width="1.8" /><span>课题</span></button>
          <button :class="{ active: activeView === 'code' }" title="代码浏览器" aria-label="代码浏览器" @click="activeView = 'code'"><Code2 :size="16" :stroke-width="1.8" /><span>代码</span></button>
        </nav>

        <template v-if="activeView === 'toolbox'">
          <div class="sidebar-section">
            <div class="section-label">能力工具箱</div>
            <nav class="filters">
              <button v-for="f in ['all', 'script', 'knowledge', 'skill'] as const" :key="f" :class="{ active: filter === f }" @click="filter = f">
                {{ { all: '全部', script: '脚本型', knowledge: '知识型', skill: '技能型' }[f] }}
              </button>
            </nav>
            <input v-model="search" class="search" placeholder="搜索能力…" />
            <button class="import-btn" @click="importCapability()">＋ 导入能力</button>
            <button class="growth-toggle" :class="{ active: revealAllCapabilities }" @click="toggleRevealAll">{{ revealAllCapabilities ? '仅显示核心能力' : '查看全部能力' }}</button>
            <p v-if="lockedCount && nearestGrowth" class="growth-hint">
              距解锁「{{ nearestGrowth.name }}」还差 {{ nearestGrowth.remaining }} 次（累计已用
              {{ totalCapabilityUses }}/{{ nearestGrowth.threshold }} 次）；规则与记录仅保存在本机。
            </p>
            <p v-else-if="lockedCount" class="growth-hint">{{ lockedCount }} 项进阶能力待解锁；解锁条件与使用记录仅保存在本机。</p>
            <p class="count">{{ filtered.length }} / {{ capabilities.length }} 项</p>
          </div>
        </template>

        <!-- 报错/操作反馈 toast：必须放 toolbox 模板外，否则其他视图（代码浏览器等）的
             IPC 失败会完全静默（Q22 教训） -->
        <transition name="fade"><p v-if="opMsg" :class="['op-toast', opOk ? 'op-ok' : 'op-err']">{{ opMsg }}</p></transition>

        <div class="sidebar-foot">
          <div class="update-box">
            <button class="update-btn" :disabled="checking" @click="checkUpdate">{{ checking ? '检查中…' : '检查更新' }}</button>
            <p v-if="updateMsg" class="update-msg">{{ updateMsg }}</p>
            <button v-if="updateUrl" class="update-link" @click="openDownload">前往下载 →</button>
          </div>
          <p class="bridge-badge">{{ bridge.kind === 'tauri' ? '桌面模式 · Tauri' : '预览模式 · 浏览器' }}</p>
        </div>
      </aside>

      <main class="content">
        <WorkbenchView v-if="activeView === 'workbench'" :bridge="bridge" @open-code="openCodeAt" @open-capability="openCapabilityFromWorkbench" />
        <PeopleChatView v-else-if="activeView === 'people'" />
        <WorkspaceView v-else-if="activeView === 'workspace'" :bridge="bridge" :capabilities="capabilities" @notify="notify" />
        <CodeBrowserView v-else-if="activeView === 'code'" ref="codeBrowserRef" :bridge="bridge" @notify="notify" @send-to-ai="sendCodeToAi" @open-in-terminal="openCodeInTerminal" />
        <ChatView v-else-if="activeView === 'chat'" ref="chatViewRef" :bridge="bridge" @open-settings="openSettings('model')" />
        <template v-else>
          <p v-if="loadError" class="error">加载失败：{{ loadError }}</p>
          <CapabilityList v-else :capabilities="filtered" :selected-id="selectedId" :locked-ids="new Set(capabilities.filter(c => !isCapabilityUnlocked(c)).map(c => c.id))" :total-uses="totalCapabilityUses" @select="select" />
          <CapabilityDetail v-if="selected" :cap="selected" :bridge="bridge" :locked="!isCapabilityUnlocked(selected)" :total-uses="totalCapabilityUses" @notify="notify" @deleted="onDeleted" @open-settings="openSettings('model')" />
          <div v-else-if="!loadError" class="placeholder">← 选择一项能力查看详情</div>
        </template>
      </main>

      <aside v-if="rightPanelVisible && activeView !== 'workspace'" class="context-panel">
        <div class="context-head">
          <span>上下文</span>
        </div>
        <div class="context-placeholder">选择内容后查看上下文</div>
      </aside>

    </div>

    <SettingsCenter v-if="showSettings" :bridge="bridge" :initial-section="settingsSection" @close="showSettings = false" @saved="onSettingsSaved" />

    <TerminalPanel ref="terminalRef" :bridge="bridge" :cwd="cwd" />
  </div>
</template>

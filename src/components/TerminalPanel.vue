<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Plus, X } from 'lucide-vue-next'
import TerminalView from './TerminalView.vue'
import type { Bridge, TerminalSession } from '../lib/bridge'

interface Tab {
  id: number
  label: string
  session: TerminalSession
  /** 内部标记 tab 是否被用户关闭（区分 session 自动结束） */
  closed: boolean
}

const props = defineProps<{
  bridge: Bridge
  /** 默认 shell 列表（首项优先） */
  defaultShells?: string[]
  /** 应用 cwd 路径 */
  cwd?: string
}>()

const emit = defineEmits<{
  /** 「在终端中运行」请求（来自 CapabilityDetail） */
  (e: 'run-in-terminal', command: string): void
}>()

// 抽屉状态：收起时完全隐藏，但保留 DOM 以维持 xterm 会话。
const expanded = ref(false)
const heightPct = ref(40) // 20-85，百分比视口高
const tabs = ref<Tab[]>([])
const activeId = ref<number | null>(null)

const activeTab = computed(() => tabs.value.find((t) => t.id === activeId.value) ?? null)

watch(
  () => props.bridge.kind,
  (kind) => {
    // 桌面模式才允许打开；浏览器模式面板整块隐藏
  },
  { immediate: true },
)

/** 新建 tab 的默认 cwd：用户主目录（拿不到时交给后端 current_dir 兜底）。 */
const homeCwd = ref<string | undefined>(undefined)
onMounted(async () => {
  try {
    homeCwd.value = (await props.bridge.homeDir()) ?? undefined
  } catch {
    homeCwd.value = undefined
  }
})

/** 默认标签序号：终端 1、终端 2…（双击标签可重命名） */
let tabSeq = 0

async function openTab(label?: string) {
  if (props.bridge.kind !== 'tauri') return
  try {
    // 默认落在主目录（顶栏按钮/＋ 新建的语义）；「在终端中运行」显式传 props.cwd
    const session = await props.bridge.openTerminal({ cwd: homeCwd.value })
    const tab: Tab = {
      id: session.id,
      label: label ?? `终端 ${++tabSeq}`,
      session,
      closed: false,
    }
    tabs.value.push(tab)
    activeId.value = tab.id
    expanded.value = true
    return tab
  } catch (e) {
    alert(`打开终端失败：${e instanceof Error ? e.message : String(e)}`)
  }
}

async function closeTab(id: number) {
  const idx = tabs.value.findIndex((t) => t.id === id)
  if (idx < 0) return
  const tab = tabs.value[idx]
  tab.closed = true
  await tab.session.close()
  tabs.value.splice(idx, 1)
  if (activeId.value === id) {
    activeId.value = tabs.value[Math.max(0, idx - 1)]?.id ?? null
  }
  if (tabs.value.length === 0) {
    expanded.value = false
  }
}

function onTabExit(id: number) {
  // session 自动退出（PTY EOF）；保留 tab 显示历史，等用户手动关
  const tab = tabs.value.find((t) => t.id === id)
  if (tab) tab.label = `${tab.label} [已退出]`
}

function renameTab(id: number, newName: string) {
  const tab = tabs.value.find((t) => t.id === id)
  if (tab) tab.label = newName
}

function toggleExpand() {
  expanded.value = !expanded.value
}

/** 顶栏终端按钮（ZCode 式）：无 tab → 新建一个并展开；有 → 纯展开/收起切换。 */
async function toggleFromToolbar() {
  if (tabs.value.length === 0) {
    await openTab()
  } else {
    toggleExpand()
  }
}

// ---- 拖拽调高（ZCode 式：抓住表头上沿整体上下拖） ----
const panelRef = ref<HTMLElement | null>(null)
let dragging = false

function onDragStart(e: MouseEvent) {
  if (e.button !== 0) return
  dragging = true
  const startY = e.clientY
  const startPct = heightPct.value
  const onMove = (ev: MouseEvent) => {
    if (!dragging) return
    // 向上拖（clientY 减小）面板变高：高度 = startPct + (startY - clientY) / vh
    const deltaPct = ((startY - ev.clientY) / window.innerHeight) * 100
    heightPct.value = Math.min(85, Math.max(20, startPct + deltaPct))
  }
  const onUp = () => {
    dragging = false
    window.removeEventListener('mousemove', onMove)
    window.removeEventListener('mouseup', onUp)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
  }
  window.addEventListener('mousemove', onMove)
  window.addEventListener('mouseup', onUp)
  document.body.style.cursor = 'ns-resize'
  document.body.style.userSelect = 'none'
  e.preventDefault()
}

// 「在终端中运行」：open new tab + 写命令
async function runCommand(command: string) {
  const tab = await openTab(`运行: ${command.slice(0, 20)}`)
  if (!tab) return
  // 等待 PTY 启动；xterm.js 准备好后写命令 + \r
  setTimeout(() => {
    tab.session.write(command + '\r').catch(() => {})
  }, 150)
}

defineExpose({ openTab, runCommand, toggleExpand, toggleFromToolbar })

// 把面板的 runCommand 暴露给外部组件（CapabilityDetail 联动）。
// 通过 window 全局，避免 prop drilling；HMR 重载时 onBeforeUnmount 清理旧引用。
onMounted(() => {
  ;(window as unknown as { __elwrightTerminal?: { run: (cmd: string) => void } }).__elwrightTerminal = {
    run: (cmd) => runCommand(cmd),
  }
})
onBeforeUnmount(() => {
  const w = window as unknown as { __elwrightTerminal?: { run: (cmd: string) => void } }
  if (w.__elwrightTerminal) w.__elwrightTerminal = undefined
})
</script>

<template>
  <div v-if="bridge.kind === 'tauri'" ref="panelRef" class="terminal-panel" :class="{ expanded }" :style="{ height: expanded ? `${heightPct}vh` : '0px' }" :aria-hidden="!expanded">
    <div class="panel-header" title="拖动调整高度" @mousedown="onDragStart">
      <span class="panel-title" @mousedown.stop>终端</span>
      <div class="tabs">
        <div
          v-for="t in tabs"
          :key="t.id"
          :class="['tab', { active: t.id === activeId }]"
          @click="activeId = t.id"
        >
          <span
            class="label"
            :title="`双击重命名\n${t.label}`"
            @dblclick="$event.preventDefault(); renameTab(t.id, prompt('重命名', t.label) || t.label)"
            @mousedown.stop
          >{{ t.label }}</span>
          <button class="close" title="关闭" @click.stop="closeTab(t.id)">
            <X :size="12" :stroke-width="2" />
          </button>
        </div>
      </div>
      <button class="new-tab" title="新建终端标签" @mousedown.stop @click.stop="openTab()">
        <Plus :size="15" :stroke-width="1.8" />
      </button>
      <button class="collapse" title="收起面板（会话保留）" @mousedown.stop @click.stop="toggleExpand()">
        <X :size="15" :stroke-width="1.8" />
      </button>
    </div>
    <div v-show="expanded" class="panel-body">
      <TerminalView
        v-if="activeTab"
        :session="activeTab.session"
        :label="activeTab.label"
        @exit="onTabExit(activeTab.id)"
        @rename="(n: string) => renameTab(activeTab!.id, n)"
      />
      <div v-else class="empty">点击「＋」打开终端</div>
    </div>
  </div>
</template>

<style scoped>
.terminal-panel {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--panel);
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  transition: height 0.18s ease-out;
  visibility: hidden;
  opacity: 0;
  pointer-events: none;
  overflow: hidden;
  z-index: 50;
}
.terminal-panel.expanded {
  visibility: visible;
  opacity: 1;
  pointer-events: auto;
}
.panel-header {
  height: 32px;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 8px;
  border-bottom: 1px solid var(--border);
  color: var(--text);
  user-select: none;
  cursor: ns-resize;
}
.panel-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-dim);
  white-space: nowrap;
}
.new-tab {
  background: none;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}
.new-tab:hover {
  color: var(--text);
  background: var(--accent-soft);
}
.collapse {
  background: none;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}
.collapse:hover {
  color: var(--text);
  background: var(--accent-soft);
}
.tabs {
  display: flex;
  gap: 2px;
  flex: 1;
  overflow-x: auto;
}
.tab {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 6px;
  background: var(--bg);
  border: 1px solid transparent;
  border-radius: 3px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-dim);
}
.tab.active {
  background: var(--accent-soft);
  color: var(--text);
  border-color: var(--border);
}
.tab .label {
  max-width: 160px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tab .close {
  background: none;
  border: none;
  color: inherit;
  cursor: pointer;
  padding: 0 2px;
  display: inline-flex;
  align-items: center;
  border-radius: 3px;
}
.tab .close:hover {
  color: var(--text);
  background: var(--border);
}
.panel-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-dim);
  font-size: 13px;
}
</style>

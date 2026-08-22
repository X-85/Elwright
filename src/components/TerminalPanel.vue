<script setup lang="ts">
import { computed, ref, watch } from 'vue'
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

// 抽屉状态：最小化时只露标题栏
const expanded = ref(false)
const heightPct = ref(40) // 0-80
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

async function openTab(label = 'Terminal') {
  if (props.bridge.kind !== 'tauri') return
  try {
    const session = await props.bridge.openTerminal({ cwd: props.cwd })
    const tab: Tab = {
      id: session.id,
      label,
      session,
      closed: false,
    }
    tabs.value.push(tab)
    activeId.value = tab.id
    expanded.value = true
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

// 「在终端中运行」：open new tab + 写命令
async function runCommand(command: string) {
  const tab = await openTab(`运行: ${command.slice(0, 20)}`)
  if (!tab) return
  // 等待 PTY 启动；xterm.js 准备好后写命令 + \r
  setTimeout(() => {
    tab.session.write(command + '\r').catch(() => {})
  }, 150)
}

defineExpose({ openTab, runCommand })

// 暴露一个全局事件让外部触发（CapabilityDetail 联动）
;(window as unknown as { __elwrightTerminal?: { run: (cmd: string) => void } }).__elwrightTerminal = {
  run: (cmd) => runCommand(cmd),
}
</script>

<template>
  <div v-if="bridge.kind === 'tauri'" class="terminal-panel" :class="{ expanded }" :style="{ height: expanded ? `${heightPct}vh` : '32px' }">
    <div class="panel-header">
      <button class="toggle" :title="expanded ? '最小化' : '展开'" @click="toggleExpand">
        {{ expanded ? '▼' : '▲' }}
      </button>
      <button class="new-tab" title="新建终端标签" @click="openTab()">＋ 新建</button>
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
          >{{ t.label }}</span>
          <button class="close" title="关闭" @click.stop="closeTab(t.id)">×</button>
        </div>
      </div>
      <div v-if="expanded" class="resize-handle" title="拖动调整高度"></div>
    </div>
    <div v-if="expanded" class="panel-body">
      <TerminalView
        v-if="activeTab"
        :session="activeTab.session"
        :label="activeTab.label"
        @exit="onTabExit(activeTab.id)"
        @rename="(n: string) => renameTab(activeTab!.id, n)"
      />
      <div v-else class="empty">点击「＋ 新建」打开终端</div>
    </div>
  </div>
</template>

<style scoped>
.terminal-panel {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  background: #1e1e1e;
  border-top: 1px solid #3a3a3a;
  display: flex;
  flex-direction: column;
  transition: height 0.18s ease-out;
  z-index: 50;
}
.panel-header {
  height: 32px;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 8px;
  border-bottom: 1px solid #2a2a2a;
  user-select: none;
}
.toggle {
  background: none;
  border: none;
  color: #ccc;
  font-size: 12px;
  cursor: pointer;
  width: 22px;
}
.new-tab {
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  color: #ccc;
  padding: 2px 8px;
  border-radius: 3px;
  cursor: pointer;
  font-size: 12px;
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
  background: #2a2a2a;
  border: 1px solid transparent;
  border-radius: 3px;
  cursor: pointer;
  font-size: 12px;
  color: #888;
}
.tab.active {
  background: #3a3a3a;
  color: #eee;
  border-color: #555;
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
  color: #666;
  font-size: 13px;
}
.resize-handle {
  width: 4px;
  height: 16px;
  background: #444;
  cursor: ns-resize;
}
</style>
<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebglAddon } from '@xterm/addon-webgl'
import '@xterm/xterm/css/xterm.css'
import { resolvedThemeRef } from '../lib/theme'
import type { TerminalSession } from '../lib/bridge'

// xterm 主题跟随应用主题（system/light/dark 切换即生效）。
// 底色取 --panel 同值：终端区与表头同一平面（扁平一体，无色差块）。
const XTERM_THEMES = {
  dark: {
    background: '#202329',
    foreground: '#e6e9ef',
    cursor: '#e6e9ef',
    cursorAccent: '#202329',
    selectionBackground: '#3a4254',
  },
  light: {
    background: '#ffffff',
    foreground: '#24292f',
    cursor: '#24292f',
    cursorAccent: '#ffffff',
    selectionBackground: '#cce0fb',
  },
} as const

const props = defineProps<{
  session: TerminalSession
  /** 标签标题（用户可双击重命名） */
  label: string
}>()

const emit = defineEmits<{
  (e: 'exit'): void
  (e: 'rename', name: string): void
}>()

const containerRef = ref<HTMLDivElement | null>(null)
let term: Terminal | null = null
let fit: FitAddon | null = null
let webgl: WebglAddon | null = null
let ro: ResizeObserver | null = null

onMounted(() => {
  if (!containerRef.value) return

  term = new Terminal({
    fontFamily: 'Menlo, Consolas, "Liberation Mono", monospace',
    fontSize: 13,
    cursorBlink: true,
    scrollback: 10000,
    convertEol: true,
    theme: XTERM_THEMES[resolvedThemeRef.value],
    // macOS/Win 上让 option/alt 作 Meta 键而非发送 ESC；此处保持默认 ESC 由用户按习惯
  })

  fit = new FitAddon()
  term.loadAddon(fit)

  try {
    webgl = new WebglAddon()
    webgl.onContextLoss(() => webgl?.dispose())
    term.loadAddon(webgl)
  } catch (e) {
    // WebGL 不可用：xterm.js 自动回退 DOM 渲染，控制台 warn；用户无感
    console.warn('[terminal] WebGL renderer 不可用，回退 DOM:', e)
    webgl = null
  }

  term.open(containerRef.value)
  // 首屏 fit
  requestAnimationFrame(() => {
    fit?.fit()
    if (term) {
      const { cols, rows } = term
      props.session.resize(cols, rows).catch(() => {})
    }
  })

  // PTY 输出 → xterm
  const offOutput = props.session.onOutput((bytes) => {
    // PTY 输出是 UTF-8 文本字节流（TUI 也用 UTF-8）；直接解码写入
    term?.write(new TextDecoder('utf-8', { fatal: false }).decode(bytes))
  })

  const offExit = props.session.onExit(() => {
    term?.write('\r\n\x1b[31m[会话已结束]\x1b[0m\r\n')
    emit('exit')
  })

  // 用户按键 → PTY
  term.onData((data) => {
    props.session.write(data).catch((e) => {
      console.warn('[terminal] write 失败:', e)
    })
  })

  // 容器尺寸变化 → resize
  ro = new ResizeObserver(() => {
    if (!term || !fit) return
    fit.fit()
    const { cols, rows } = term
    props.session.resize(cols, rows).catch(() => {})
  })
  ro.observe(containerRef.value)

  // dispose hook：组件 unmount 时回收
  onBeforeUnmount(() => {
    offOutput()
    offExit()
    ro?.disconnect()
    ro = null
    // xterm.js dispose 顺序：先 addon 再 terminal 再 DOM
    try {
      webgl?.dispose()
    } catch {
      // ignore
    }
    webgl = null
    fit = null
    try {
      term?.dispose()
    } catch {
      // ignore
    }
    term = null
  })
})

// 监听外部传入的 session 替换（如父组件切换 tab）
watch(
  () => props.session,
  async (next) => {
    if (!term) return
    // 简单做法：fit 一次让新 session 的 PTY 与新 xterm 对齐
    requestAnimationFrame(() => {
      fit?.fit()
      const { cols, rows } = term!
      next.resize(cols, rows).catch(() => {})
    })
  },
)

// 双击标签触发重命名（由父组件 TerminalPanel 处理）
function onLabelDblClick() {
  const next = prompt('重命名标签', props.label)
  if (next && next.trim()) emit('rename', next.trim())
}
defineExpose({ onLabelDblClick })

// 主题切换：xterm options.theme 运行时可变，即时重绘
watch(resolvedThemeRef, (theme) => {
  if (term) term.options.theme = { ...XTERM_THEMES[theme] }
})
</script>

<template>
  <div class="terminal-view">
    <div ref="containerRef" class="terminal-host"></div>
  </div>
</template>

<style scoped>
.terminal-view {
  width: 100%;
  height: 100%;
  background: var(--panel);
}
.terminal-host {
  width: 100%;
  height: 100%;
  padding: 4px 8px;
}
</style>
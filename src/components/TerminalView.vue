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

// 一个 TerminalView 实例 = 一个 tab = 一个 PTY 会话，挂载时接线一次、
// session 永不替换（切换 tab 由父组件 v-show 显隐，状态各自保留）。
const props = defineProps<{
  session: TerminalSession
}>()

const emit = defineEmits<{
  (e: 'exit'): void
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
  // 首屏 fit（挂载时必然是活跃 tab，见父组件 openTab 语义）
  requestAnimationFrame(() => {
    fit?.fit()
    if (term) {
      const { cols, rows } = term
      props.session.resize(cols, rows).catch(() => {})
    }
  })

  // PTY 输出 → xterm（本实例专属 session；隐藏 tab 仍持续写入自己的缓冲，
  // 后台任务输出不丢失，切回即见）
  const offOutput = props.session.onOutput((bytes) => {
    // PTY 输出是 UTF-8 文本字节流（TUI 也用 UTF-8）；直接解码写入
    term?.write(new TextDecoder('utf-8', { fatal: false }).decode(bytes))
  })

  const offExit = props.session.onExit(() => {
    term?.write('\r\n\x1b[31m[会话已结束]\x1b[0m\r\n')
    emit('exit')
  })

  // 用户按键 → 本 tab 的 PTY
  term.onData((data) => {
    props.session.write(data).catch((e) => {
      console.warn('[terminal] write 失败:', e)
    })
  })

  // 容器尺寸变化 → resize。
  // v-show 隐藏（display:none）时 clientWidth 为 0：跳过 fit（0 列 resize
  // 无意义且 xterm fit 会除零）；重新显示时 RO 会再触发一次带真实尺寸。
  ro = new ResizeObserver(() => {
    if (!term || !fit) return
    const el = containerRef.value
    if (!el || el.clientWidth === 0) return
    fit.fit()
    const { cols, rows } = term
    props.session.resize(cols, rows).catch(() => {})
  })
  ro.observe(containerRef.value)

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

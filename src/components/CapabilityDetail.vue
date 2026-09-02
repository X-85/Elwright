<script setup lang="ts">
import { ref, watch } from 'vue'
import { marked } from 'marked'
import type { Bridge, Capability, InvokeResult, RunResult, ViewResult } from '../lib/bridge'

const props = defineProps<{
  cap: Capability
  bridge: Bridge
  locked?: boolean
  /** 累计使用次数（ADR-001：解锁进度提示） */
  totalUses?: number
}>()

const emit = defineEmits<{
  notify: [message: string, ok: boolean]
  deleted: []
  openSettings: []
}>()

const view = ref<ViewResult | null>(null)
const runArgs = ref('')
const runResult = ref<RunResult | null>(null)
const invokePrompt = ref('')
const invokeResult = ref<InvokeResult | null>(null)
const busy = ref(false)

// 导出/删除操作（删除仅自定义项可见）
async function onExport() {
  busy.value = true
  try {
    const result = await props.bridge.exportCapability(props.cap)
    emit('notify', result.message, result.ok)
  } finally {
    busy.value = false
  }
}

async function onDelete() {
  if (!confirm(`删除自定义能力「${props.cap.name}」（${props.cap.id}）？\n其引用的文件会一并清理。`)) return
  busy.value = true
  try {
    const result = await props.bridge.deleteCapability(props.cap)
    emit('notify', result.message, result.ok)
    if (result.ok) emit('deleted')
  } finally {
    busy.value = false
  }
}

// 知识型选中即加载文档；其余类型在操作后展示结果
watch(
  () => props.cap.id,
  async (_id) => {
    runResult.value = null
    invokeResult.value = null
    view.value = null
    if (props.cap.type === 'knowledge') {
      view.value = await props.bridge.viewDoc(props.cap)
    }
  },
  { immediate: true },
)

async function onRun() {
  if (props.locked) return
  busy.value = true
  try {
    runResult.value = await props.bridge.runScript(
      props.cap,
      runArgs.value.trim() ? runArgs.value.trim().split(/\s+/) : [],
    )
  } finally {
    busy.value = false
  }
}

async function onInvoke() {
  if (props.locked) return
  busy.value = true
  try {
    invokeResult.value = await props.bridge.invokeSkill(props.cap, invokePrompt.value.trim())
  } finally {
    busy.value = false
  }
}

/// 「在终端中运行」：拼出与 `ew run <id>` 等价的命令字符串，触发全局 terminalPanel
async function onRunInTerminal() {
  const win = window as unknown as { __elwrightTerminal?: { run: (cmd: string) => void } }
  const runner = win.__elwrightTerminal
  if (!runner) {
    emit('notify', '终端面板未就绪', false)
    return
  }
  const args = runArgs.value.trim()
  // 与 ew run 等价的命令：假设 terminal cwd 与 elwright 资源目录同级；
  // 这里直接以 entry 路径调用 python3 / bash 等，依赖 PATH 解释器探测（与 CLI 一致）
  const cmd = `${props.cap.id} ${args}`.trim()
  // 用 capability id 触发：TerminalPanel 收到后用 `ew run <id> [args]` 调用
  // —— 这样跨平台一致（不依赖终端解释器）
  // 更稳的写法：直接以 `ew` 二进制名调用，由 PATH 解析
  runner.run(`ew run ${cmd}`)
}

function renderMd(text: string): string {
  return marked.parse(text, { async: false }) as string
}
</script>

<template>
  <section class="detail">
    <header class="detail-head">
      <h2>{{ cap.name }}</h2>
      <span :class="['type-badge', cap.type]">{{ cap.type }}</span>
      <span v-if="cap.origin === 'custom'" class="custom-badge" title="来自用户叠加层 ~/.elwright/">自定义</span>
      <span v-if="cap.category" class="cap-cat">{{ cap.category }}</span>
      <code class="cap-id">{{ cap.id }}</code>
      <span class="detail-actions">
        <button :disabled="busy" @click="onExport">⬇ 导出</button>
        <button
          v-if="cap.origin === 'custom'"
          class="danger"
          :disabled="busy"
          @click="onDelete"
        >🗑 删除</button>
      </span>
    </header>

    <p v-if="locked" class="degrade-banner">
      <template v-if="cap.unlockAfterUses !== undefined">
        该能力尚未解锁：累计使用任意能力 {{ cap.unlockAfterUses }} 次后自动解锁（当前
        {{ totalUses ?? 0 }}/{{ cap.unlockAfterUses }}）。使用记录仅保存在本机。
      </template>
      <template v-else>该能力尚未解锁，暂未开放解锁条件。</template>
    </p>

    <!-- 脚本型 -->
    <template v-if="cap.type === 'script'">
      <p class="meta">入口：<code>{{ cap.entry ?? '（未配置）' }}</code></p>
      <div class="action-row">
        <input v-model="runArgs" class="search" placeholder="脚本参数（空格分隔）…" />
        <button class="primary" :disabled="busy || locked" @click="onRun">▶ 运行</button>
        <button
          v-if="bridge.kind === 'tauri' && cap.entry"
          class="terminal-btn"
          :disabled="busy || locked"
          :title="`在集成终端中执行：${cap.entry}${runArgs.trim() ? ' ' + runArgs.trim() : ''}`"
          @click="onRunInTerminal"
        >⌨ 在终端中运行</button>
      </div>
      <pre v-if="runResult" class="output">{{ runResult.output }}</pre>
    </template>

    <!-- 知识型 -->
    <template v-else-if="cap.type === 'knowledge'">
      <p v-if="view?.path" class="meta">来源：<code>{{ view.path }}</code></p>
      <div v-if="view?.ok" class="markdown" v-html="renderMd(view.content)"></div>
      <p v-else class="error">{{ view?.content }}</p>
    </template>

    <!-- 技能型 -->
    <template v-else-if="cap.type === 'skill'">
      <p class="meta">提示词模板：{{ cap.prompt ?? '（未配置）' }}</p>
      <p class="meta degrade-hint">
        {{ cap.offline ? '' : '离线时自动降级为 SOP 文档' }}
        <code v-if="cap.degradeDoc">{{ cap.degradeDoc }}</code>
      </p>
      <div class="action-row">
        <input v-model="invokePrompt" class="search" placeholder="附加输入（可选）…" />
        <button class="primary" :disabled="busy || locked" @click="onInvoke">⚡ 调用</button>
      </div>
      <div v-if="invokeResult">
        <p v-if="invokeResult.note" class="degrade-banner">
          {{ invokeResult.note }}
          <a
            v-if="invokeResult.source === 'degraded'"
            class="settings-link"
            @click="emit('openSettings')"
          >去配置模型 →</a>
        </p>
        <div v-if="invokeResult.source === 'degraded'" class="markdown" v-html="renderMd(invokeResult.content)"></div>
        <pre v-else class="output">{{ invokeResult.content }}</pre>
      </div>
    </template>

    <p v-else class="error">未知能力类型：{{ cap.type }}</p>
  </section>
</template>

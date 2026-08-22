<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'
import { renderChatMarkdown } from '../lib/safeMarkdown'
import type { Bridge, ChatMessage, LlmConfigInfo } from '../lib/bridge'

interface UiMessage {
  role: 'user' | 'assistant'
  content: string
  /** true = 调用失败的错误占位（重试时移除，不进入下一轮上下文） */
  error?: boolean
}

const props = defineProps<{
  bridge: Bridge
}>()

const emit = defineEmits<{
  openSettings: []
}>()

const messages = ref<UiMessage[]>([])
const input = ref('')
const sending = ref(false)
const chatScroll = ref<HTMLElement | null>(null)

const configInfo = ref<LlmConfigInfo | null>(null)
const configState = ref<'loading' | 'ready' | 'unconfigured' | 'preview'>('loading')

// 停止语义（阶段①）：序号递增，晚到的在途结果丢弃；Rust 侧请求照常
// 完成。真正的请求级取消与流式一起在阶段④实现。
let requestSeq = 0

const LONG_INPUT_HINT = 20000

async function refreshConfig() {
  configState.value = 'loading'
  try {
    configInfo.value = await props.bridge.getLlmConfig()
    configState.value = configInfo.value.baseUrl ? 'ready' : 'unconfigured'
  } catch {
    // 预览模式无法读用户配置链
    configState.value = 'preview'
  }
}

onMounted(refreshConfig)
// App.vue 在 ⚙ 设置弹层关闭后调用，刷新模型状态条
defineExpose({ refreshConfig })

function historyForRequest(): ChatMessage[] {
  return messages.value
    .filter((m) => !m.error)
    .map((m) => ({ role: m.role, content: m.content }))
}

async function send() {
  const text = input.value.trim()
  if (!text || sending.value) return
  messages.value.push({ role: 'user', content: text })
  input.value = ''
  await complete()
}

async function complete() {
  sending.value = true
  const seq = ++requestSeq
  try {
    const reply = await props.bridge.chat(historyForRequest())
    if (seq !== requestSeq) return
    messages.value.push({ role: 'assistant', content: reply })
  } catch (e) {
    if (seq !== requestSeq) return
    messages.value.push({
      role: 'assistant',
      content: e instanceof Error ? e.message : String(e),
      error: true,
    })
  } finally {
    if (seq === requestSeq) sending.value = false
    await nextTick()
    chatScroll.value?.scrollTo({ top: chatScroll.value.scrollHeight })
  }
}

function stop() {
  requestSeq++
  sending.value = false
}

/** 失败重试：移除尾部错误占位后用当前历史重发（原始 user 输入仍在列表中） */
async function retry() {
  while (messages.value.length && messages.value[messages.value.length - 1].error) {
    messages.value.pop()
  }
  await complete()
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey && !e.isComposing) {
    e.preventDefault()
    send()
  }
}

// 代码块复制（点击委托：按钮由 safeMarkdown 的 code renderer 注入）
async function onCopyCode(e: MouseEvent) {
  const btn = (e.target as HTMLElement).closest('.code-copy-btn') as HTMLElement | null
  if (!btn) return
  const code = btn.closest('.code-block')?.querySelector('code')
  if (!code) return
  try {
    await navigator.clipboard.writeText(code.textContent ?? '')
    btn.textContent = '已复制'
  } catch {
    btn.textContent = '复制失败'
  }
  setTimeout(() => {
    btn.textContent = '复制'
  }, 1500)
}

const render = renderChatMarkdown
</script>

<template>
  <section class="chat-view">
    <header class="chat-head">
      <h2>AI 对话</h2>
      <span
        v-if="configState === 'ready' && configInfo"
        class="chat-model"
        :title="`model 来源：${configInfo.source[2] || '未设置'} · base_url 来源：${configInfo.source[0] || '未设置'} · key：${configInfo.apiKeyMasked || '无'}`"
      >🤖 {{ configInfo.model || '（未设置 model）' }}</span>
      <span v-else-if="configState === 'unconfigured'" class="chat-model warn">未配置模型</span>
      <span v-else-if="configState === 'preview'" class="chat-model warn">预览模式 · 不可对话</span>
    </header>

    <div v-if="configState === 'unconfigured'" class="chat-guide">
      <p>尚未配置 LLM。配置后即可开始多轮对话；脚本、知识与技能能力不受影响，仍可离线使用。</p>
      <button class="primary" @click="emit('openSettings')">⚙ 去配置模型</button>
    </div>
    <p v-else-if="configState === 'preview'" class="chat-preview-note">
      【预览模式】AI 对话仅在桌面应用可用（需读取本机模型配置）。能力浏览不受影响。
    </p>

    <div ref="chatScroll" class="chat-messages" @click="onCopyCode">
      <p v-if="!messages.length && configState === 'ready'" class="chat-empty">
        输入消息开始对话。Enter 发送，Shift+Enter 换行；回复支持 Markdown，代码块可一键复制。
      </p>
      <div v-for="(m, i) in messages" :key="i" :class="['chat-msg', m.role, { error: m.error }]">
        <template v-if="m.role === 'assistant'">
          <div v-if="m.error" class="chat-error-box">
            <p class="error">{{ m.content }}</p>
            <button class="retry-btn" :disabled="sending" @click="retry">↻ 重试</button>
          </div>
          <div v-else class="markdown chat-md" v-html="render(m.content)"></div>
        </template>
        <p v-else class="chat-user-text">{{ m.content }}</p>
      </div>
      <div v-if="sending" class="chat-msg assistant pending">
        <span class="typing">生成中…</span>
        <button class="stop-btn" @click="stop">■ 停止</button>
      </div>
    </div>

    <div class="chat-input-row">
      <textarea
        v-model="input"
        class="chat-input"
        rows="3"
        placeholder="输入消息…（Enter 发送，Shift+Enter 换行）"
        @keydown="onKeydown"
      ></textarea>
      <div class="chat-actions">
        <p v-if="input.length > LONG_INPUT_HINT" class="chat-hint">
          输入较长（{{ input.length }} 字符），部分模型会截断或拒绝。
        </p>
        <button class="primary send-btn" :disabled="sending || !input.trim()" @click="send">发送</button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'
import { renderChatMarkdown } from '../lib/safeMarkdown'
import type { Bridge, ChatMessage, ChatSessionSummary, LlmConfigInfo } from '../lib/bridge'

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

// 会话列表
const sessions = ref<ChatSessionSummary[]>([])
const currentId = ref<string>('')
// 当前会话消息（UI 态，含 error 占位）
const messages = ref<UiMessage[]>([])
const renamingId = ref<string | null>(null)
const renameText = ref('')
const sessionsLoading = ref(false)
// 用户手动改过标题的会话：自动按首条消息算标题时跳过（保留用户命名的稳定性）
const userRenamed = ref<Set<string>>(new Set())

const input = ref('')
const sending = ref(false)
const chatScroll = ref<HTMLElement | null>(null)

const configInfo = ref<LlmConfigInfo | null>(null)
const configState = ref<'loading' | 'ready' | 'unconfigured' | 'preview'>('loading')

// 停止语义（阶段①）：序号递增，晚到的在途结果丢弃。
let requestSeq = 0
// 前端会话 id 生成：时间戳+计数器，与后端格式一致
let idCounter = 0

const LONG_INPUT_HINT = 20000

function newSessionId(): string {
  const ts = Date.now().toString(16)
  return `${ts}-${(idCounter++).toString(16)}`
}

function titleFor(msgs: UiMessage[]): string {
  const firstUser = msgs.find((m) => m.role === 'user' && !m.error)
  if (!firstUser) return '新对话'
  const t = firstUser.content.trim().replace(/\s+/g, ' ')
  return t.length > 24 ? t.slice(0, 24) + '…' : t || '新对话'
}

/** 持久化当前会话（消息变化后调用）。无消息时跳过（不写空会话）。 */
async function persistCurrent() {
  if (!currentId.value) return
  const clean = messages.value.filter((m) => !m.error)
  if (clean.length === 0) return
  // 用户改过标题则保留原名；否则按首条用户消息自动算
  const existing = sessions.value.find((s) => s.id === currentId.value)
  const title = userRenamed.value.has(currentId.value) && existing
    ? existing.title
    : titleFor(messages.value)
  try {
    await props.bridge.saveChatSession(currentId.value, title, clean)
    if (existing) {
      existing.title = title
      existing.updatedAt = new Date().toISOString()
      // 移到列表最前
      sessions.value = [existing, ...sessions.value.filter((s) => s.id !== currentId.value)]
    }
  } catch {
    // 持久化失败不影响对话进行（静默；真机可看 console）
  }
}

async function refreshConfig() {
  configState.value = 'loading'
  try {
    configInfo.value = await props.bridge.getLlmConfig()
    configState.value = configInfo.value.baseUrl ? 'ready' : 'unconfigured'
    // 控制台日志便于真机排错：未配置引导是否显示异常时看这里
    console.info(
      '[chat] refreshConfig: baseUrl=%s model=%s source=[%s,%s,%s] state=%s',
      JSON.stringify(configInfo.value.baseUrl),
      JSON.stringify(configInfo.value.model),
      configInfo.value.source[0] || 'unset',
      configInfo.value.source[1] || 'unset',
      configInfo.value.source[2] || 'unset',
      configState.value,
    )
  } catch (e) {
    configState.value = 'preview'
    console.warn('[chat] refreshConfig failed:', e)
  }
}

async function loadSessionList() {
  sessionsLoading.value = true
  try {
    sessions.value = await props.bridge.listChatSessions()
  } catch {
    sessions.value = []
  } finally {
    sessionsLoading.value = false
  }
}

async function selectSession(id: string) {
  if (id === currentId.value) return
  const loaded = await props.bridge.loadChatSession(id)
  if (!loaded) return
  currentId.value = id
  messages.value = loaded.messages.map((m) => ({ role: m.role, content: m.content }))
  renamingId.value = null
  // 重启后恢复：磁盘标题若与"按首条消息自动算"的标题不一致，视为用户改过名，
  // 之后 persistCurrent 不会再覆盖（保持跨重启的命名稳定性）。
  if (loaded.title && loaded.title !== titleFor(messages.value)) {
    userRenamed.value = new Set([...userRenamed.value, id])
    const s = sessions.value.find((x) => x.id === id)
    if (s) s.title = loaded.title
  }
  await nextTick()
  chatScroll.value?.scrollTo({ top: chatScroll.value.scrollHeight })
}

function newSession() {
  currentId.value = newSessionId()
  messages.value = []
  renamingId.value = null
  input.value = ''
}

async function deleteSession(id: string) {
  const target = sessions.value.find((s) => s.id === id)
  if (!target) return
  if (!confirm(`删除会话「${target.title}」？`)) return
  try {
    await props.bridge.deleteChatSession(id)
  } catch {
    // 忽略，仍从列表移除
  }
  sessions.value = sessions.value.filter((s) => s.id !== id)
  if (id === currentId.value) {
    if (sessions.value.length) {
      await selectSession(sessions.value[0].id)
    } else {
      newSession()
    }
  }
}

function startRename(id: string) {
  const s = sessions.value.find((x) => x.id === id)
  if (!s) return
  renamingId.value = id
  renameText.value = s.title
}

async function commitRename() {
  const id = renamingId.value
  if (!id) return
  const title = renameText.value.trim() || '新对话'
  renamingId.value = null
  const s = sessions.value.find((x) => x.id === id)
  if (!s) return
  s.title = title
  // 标记：此后按消息自动算标题时跳过该会话
  userRenamed.value = new Set([...userRenamed.value, id])
  if (id === currentId.value) {
    const clean = messages.value.filter((m) => !m.error)
    if (clean.length) {
      try {
        await props.bridge.saveChatSession(id, title, clean)
      } catch {
        // 静默
      }
    }
  }
}

onMounted(async () => {
  await refreshConfig()
  await loadSessionList()
  if (sessions.value.length) {
    await selectSession(sessions.value[0].id)
  } else {
    newSession()
  }
})
defineExpose({ refreshConfig })

function historyForRequest(): ChatMessage[] {
  return messages.value.filter((m) => !m.error).map((m) => ({ role: m.role, content: m.content }))
}

async function send() {
  const text = input.value.trim()
  if (!text || sending.value) return
  messages.value.push({ role: 'user', content: text })
  input.value = ''
  await persistCurrent()
  await complete()
}

async function complete() {
  sending.value = true
  const seq = ++requestSeq
  try {
    const reply = await props.bridge.chat(historyForRequest())
    if (seq !== requestSeq) return
    messages.value.push({ role: 'assistant', content: reply })
    await persistCurrent()
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
    <aside class="chat-sessions">
      <div class="sessions-head">
        <span>会话</span>
        <button class="new-btn" title="新建会话" @click="newSession">＋</button>
      </div>
      <ul class="session-list">
        <li
          v-for="s in sessions"
          :key="s.id"
          :class="['session-item', { active: s.id === currentId }]"
          @click="selectSession(s.id)"
        >
          <template v-if="renamingId === s.id">
            <input
              v-model="renameText"
              class="rename-input"
              autofocus
              @click.stop
              @keydown.enter="commitRename"
              @keydown.esc="renamingId = null"
              @blur="commitRename"
            />
          </template>
          <template v-else>
            <span class="session-title" @dblclick.stop="startRename(s.id)">{{ s.title }}</span>
            <button class="rename-btn" title="重命名" @click.stop="startRename(s.id)">✎</button>
            <button class="del-btn" title="删除" @click.stop="deleteSession(s.id)">×</button>
          </template>
        </li>
        <li v-if="!sessions.length && !sessionsLoading" class="session-empty">暂无会话</li>
      </ul>
    </aside>

    <div class="chat-main">
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
        【预览模式】AI 对话仅在桌面应用可用（需读取本机模型配置）。会话不会持久化，刷新即失。能力浏览不受影响。
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
    </div>
  </section>
</template>

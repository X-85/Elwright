<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'
import { MessageSquarePlus, Zap } from 'lucide-vue-next'
import { renderChatMarkdown } from '../lib/safeMarkdown'
import type { Bridge, Capability, ChatMessage, ChatSessionSummary, LlmConfigInfo } from '../lib/bridge'
import { isCapabilityResult, parseProposalId, resultFeedbackMessage, splitCapabilityResult } from '../lib/chatProposal'
import { extractFirstDiff } from '../lib/patch'
import PatchPreviewDialog from './PatchPreviewDialog.vue'

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

// 能力协作（阶段③）：能力清单 + 提议/调用确认卡片 + 执行/回灌
const capabilities = ref<Capability[]>([])
const capPickerOpen = ref(false)
const capArgs = ref<Record<number, string>>({})
const runningIdx = ref<number | null>(null)

const configInfo = ref<LlmConfigInfo | null>(null)
const configState = ref<'loading' | 'ready' | 'unconfigured' | 'preview'>('loading')

// 停止语义（阶段①）：序号递增，晚到的在途结果丢弃。
let requestSeq = 0
// 阶段④：当前在途流式请求 id（取消命令用）
let activeRequestId = 0
// 前端会话 id 生成：时间戳+计数器，与后端格式一致
let idCounter = 0

// 代码浏览器阶段④：最近项目根 + 补丁对话框
const recentProject = ref<string | null>(null)
const patchDialog = ref<{ patchText: string } | null>(null)

async function refreshRecentProject() {
  try {
    const r = await props.bridge.codeBrowserRecentLoad()
    recentProject.value = r.projects?.[0]?.rootPath ?? null
  } catch {
    recentProject.value = null
  }
}

onMounted(() => {
  refreshRecentProject()
})

function openPatchDialog(content: string) {
  const diff = extractFirstDiff(content)
  if (!diff) return
  patchDialog.value = { patchText: diff }
}

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
  props.bridge.listCapabilities().then((cs) => { capabilities.value = cs }).catch(() => {})
  await loadSessionList()
  if (sessions.value.length) {
    await selectSession(sessions.value[0].id)
  } else {
    newSession()
  }
})
function insertContext(title: string, text: string) {
  input.value = input.value ? input.value + '\n\n' : ''
  input.value += `【代码上下文】${title}\n\`\`\`\n${text}\n\`\`\`
`
}

defineExpose({ refreshConfig, insertContext })

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
  const history = historyForRequest()
  const streaming = props.bridge.kind === 'tauri'

  // 阶段④流式路径：增量渲染（50ms 节流），停止真取消后端读取
  if (streaming) {
    const requestId = Date.now() + seq
    activeRequestId = requestId
    messages.value.push({ role: 'assistant', content: '' })
    const idx = messages.value.length - 1
    let acc = ''
    let lastRender = 0
    try {
      await props.bridge.chatCompletionStream(requestId, history, (e) => {
        if (seq !== requestSeq) return
        if (e.type === 'delta' && e.text) {
          acc += e.text
          const now = Date.now()
          if (now - lastRender > 50) {
            lastRender = now
            messages.value[idx].content = acc
          }
        } else if (e.type === 'cancelled') {
          messages.value[idx].content = acc ? `${acc}\n\n（已停止）` : '（已停止，未收到内容）'
        } else if (e.type === 'error') {
          messages.value[idx] = {
            role: 'assistant',
            content: e.message ?? '未知错误',
            error: true,
          }
        }
      })
      if (seq !== requestSeq) return
      const finalMsg = messages.value[idx]
      if (!finalMsg.error && !finalMsg.content.endsWith('（已停止）')) {
        finalMsg.content = acc || '（空回复）'
      }
      if (!finalMsg.error) await persistCurrent()
    } catch (e) {
      if (seq !== requestSeq) return
      messages.value[idx] = {
        role: 'assistant',
        content: e instanceof Error ? e.message : String(e),
        error: true,
      }
    } finally {
      if (seq === requestSeq) sending.value = false
      await nextTick()
      chatScroll.value?.scrollTo({ top: chatScroll.value.scrollHeight })
    }
    return
  }

  // 浏览器预览：旧一次性路径（bridge.chat 抛预览模式错误 → 错误气泡）
  try {
    const reply = await props.bridge.chat(history)
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
  // 阶段④：真正取消后端读取（浏览器端静默忽略）
  void props.bridge.chatCancel(activeRequestId).catch(() => {})
}

// ---- 能力协作（阶段③）----

function proposalOf(m: UiMessage) {
  const id = parseProposalId(m.content)
  if (!id) return null
  const cap = capabilities.value.find((c) => c.id === id) ?? null
  return { id, cap }
}

function isResult(m: UiMessage) {
  return m.role === 'assistant' && isCapabilityResult(m.content)
}

function resultParts(m: UiMessage) {
  return splitCapabilityResult(m.content)
}

function pickCapability(cap: Capability) {
  capPickerOpen.value = false
  messages.value.push({ role: 'user', content: `【能力调用】\nid: ${cap.id}` })
  void persistCurrent()
}

async function runProposal(idx: number) {
  const m = messages.value[idx]
  if (!m || runningIdx.value !== null) return
  const id = parseProposalId(m.content)
  const cap = capabilities.value.find((c) => c.id === id)
  if (!cap) {
    notifyError(idx, `能力不存在或已被移除: ${id ?? ''}`)
    return
  }
  runningIdx.value = idx
  try {
    let header: string
    let body: string
    let note: string | undefined
    if (cap.type === 'script') {
      const args = (capArgs.value[idx] ?? '').split(/\s+/).filter(Boolean)
      const r = await props.bridge.runScript(cap, args)
      header = `【能力结果】${cap.name}（${cap.type}）`
      body = r.output
    } else if (cap.type === 'knowledge') {
      const r = await props.bridge.viewDoc(cap)
      header = `【能力结果】${cap.name}（${cap.type}）`
      body = r.content
    } else {
      const prompt = capArgs.value[idx] ?? ''
      const r = await props.bridge.invokeSkill(cap, prompt)
      header = `【能力结果】${cap.name}（${cap.type}${r.source === 'degraded' ? ' · 离线 SOP' : ''}）`
      body = r.content
      note = r.note
    }
    if (body.length > 4000) body = body.slice(0, 4000) + '\n…（超长截断）'
    if (note) body += `\n\n> ${note}`
    messages.value.push({ role: 'assistant', content: `${header}\n\n${body}` })
    await persistCurrent()
  } catch (e) {
    notifyError(idx, `能力执行失败：${e instanceof Error ? e.message : String(e)}`)
  } finally {
    runningIdx.value = null
    await nextTick()
    chatScroll.value?.scrollTo({ top: chatScroll.value.scrollHeight })
  }
}

function notifyError(idx: number, msg: string) {
  messages.value.splice(idx + 1, 0, { role: 'assistant', content: msg, error: true })
}

function tellAiResult(idx: number) {
  const m = messages.value[idx]
  if (!m) return
  const { header, body } = splitCapabilityResult(m.content)
  messages.value.push({ role: 'user', content: resultFeedbackMessage(header, body) })
  void persistCurrent()
  void complete()
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
        <button class="new-btn" title="新建会话" aria-label="新建会话" @click="newSession">
          <MessageSquarePlus :size="14" :stroke-width="1.8" />
        </button>
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
          <!-- 执行结果 -->
          <template v-if="!m.error && isResult(m)">
            <div class="cap-result">
              <div class="cap-result-head">{{ resultParts(m).header }}</div>
              <pre class="cap-result-body">{{ resultParts(m).body }}</pre>
              <button class="cap-tell-ai" :disabled="sending" @click="tellAiResult(i)">把结果告诉 AI</button>
            </div>
          </template>
          <!-- 能力提议/调用确认卡片 -->
          <template v-else-if="!m.error && proposalOf(m)">
            <div class="cap-proposal" role="group" aria-label="能力确认">
              <div class="cap-proposal-title">⚡ 能力{{ m.role === 'assistant' ? '提议' : '调用' }}</div>
              <template v-if="proposalOf(m)!.cap">
                <div><strong>{{ proposalOf(m)!.cap!.name }}</strong> <span class="cap-type">{{ proposalOf(m)!.cap!.type }}</span> <code class="cap-id">{{ proposalOf(m)!.cap!.id }}</code></div>
                <p class="cap-note">
                  {{ proposalOf(m)!.cap!.type === 'script' ? '将运行本地脚本（可在下方填写参数）。'
                    : proposalOf(m)!.cap!.type === 'knowledge' ? '将展示知识文档内容。'
                    : '将调用技能（可在下方填写输入；未配置模型时走离线 SOP）。' }}
                </p>
                <input
                  v-model="capArgs[i]"
                  class="cap-args"
                  :placeholder="proposalOf(m)!.cap!.type === 'script' ? '脚本参数（可选，空格分隔）' : '输入 / 参数（可选）'"
                  @keydown.enter.prevent
                />
                <div class="cap-actions">
                  <button class="cap-run" :disabled="runningIdx !== null" @click="runProposal(i)">
                    {{ runningIdx === i ? '执行中…' : '确认运行' }}
                  </button>
                  <span class="cap-muted">执行需你确认；模型不会自行运行。</span>
                </div>
              </template>
              <p v-else class="cap-note">能力不存在或已被移除：{{ proposalOf(m)!.id }}</p>
            </div>
          </template>
          <template v-else-if="m.role === 'assistant'">
            <div v-if="m.error" class="chat-error-box">
              <p class="error">{{ m.content }}</p>
              <button class="retry-btn" :disabled="sending" @click="retry">↻ 重试</button>
            </div>
            <div v-else>
              <div class="markdown chat-md" v-html="render(m.content)"></div>
              <button
                v-if="recentProject && extractFirstDiff(m.content)"
                class="patch-apply-btn"
                @click="openPatchDialog(m.content)"
              >
                预览并应用到代码
              </button>
            </div>
          </template>
          <p v-else class="chat-user-text">{{ m.content }}</p>
        </div>
        <div v-if="sending" class="chat-msg assistant pending">
          <span class="typing">生成中…</span>
          <button class="stop-btn" @click="stop">■ 停止</button>
        </div>
      </div>

      <PatchPreviewDialog
        v-if="patchDialog && recentProject"
        :bridge="props.bridge"
        :project-root="recentProject"
        :patch-text="patchDialog.patchText"
        @closed="patchDialog = null; refreshRecentProject()"
      />

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
          <div class="cap-picker-wrap">
            <button class="cap-picker-btn" title="选择能力（确认后执行）" @click="capPickerOpen = !capPickerOpen">
              <Zap :size="14" /> 能力
            </button>
            <ul v-if="capPickerOpen" class="cap-picker" role="listbox" aria-label="选择能力">
              <li v-for="c in capabilities" :key="c.id">
                <button class="cap-picker-item" @click="pickCapability(c)">
                  <strong>{{ c.name }}</strong> <span class="cap-type">{{ c.type }}</span>
                  <span class="cap-muted">{{ c.id }}</span>
                </button>
              </li>
              <li v-if="!capabilities.length" class="cap-muted cap-picker-empty">能力清单为空</li>
            </ul>
          </div>
          <button class="primary send-btn" :disabled="sending || !input.trim()" @click="send">发送</button>
        </div>
      </div>
    </div>
  </section>
</template>

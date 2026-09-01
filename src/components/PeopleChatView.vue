<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { Copy, ImagePlus, MessageCircle, Plus, QrCode, Send, SmilePlus, Trash2, UserPlus, X } from 'lucide-vue-next'
import type { Bridge, Contact } from '../lib/bridge'

type MessageKind = 'text' | 'image' | 'emoji'
type MessageStatus = 'sending' | 'local' | 'sent' | 'queued' | 'failed'
interface LocalMessage {
  id: string
  kind: MessageKind
  content: string
  createdAt: string
  status: MessageStatus
  /** peer = 对端发来（收件箱合并）；缺省 self = 本机 */
  direction?: 'self' | 'peer'
}
interface LocalConversation {
  id: string
  peer: string
  /** 绑定的联系人 ID（16 字符 base32）；缺省 = 纯本地会话 */
  peerId?: string
  messages: LocalMessage[]
  updatedAt: string
}

const props = defineProps<{ bridge: Bridge }>()

const STORAGE_KEY = 'elwright.people-conversations'
const CURSOR_KEY = 'elwright.people-inbox-cursor'
const conversations = ref<LocalConversation[]>([])
const currentId = ref('')
const draft = ref('')
const showEmoji = ref(false)
const showCreateDialog = ref(false)
const peerDraft = ref('')
const messagesScroll = ref<HTMLElement | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const emojis = ['👍', '✅', '🎉', '💡', '👀', '🚀', '🙏', '🙂']

// ---- 消息传输接线（phase 2；桌面可用，预览模式降级） ----
const desktop = ref(false)
const myId = ref('')
const contacts = ref<Contact[]>([])
const showInviteDialog = ref(false)
const inviteQr = ref('')
const inviteShort = ref('')
const showAddDialog = ref(false)
const addQr = ref('')
const addAlias = ref('')
const addError = ref('')
const transportNote = ref('')
let pollTimer: ReturnType<typeof setInterval> | null = null

const current = computed(() => conversations.value.find((item) => item.id === currentId.value) ?? null)

function persist() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(conversations.value))
}

function selectConversation(id: string) {
  currentId.value = id
  showEmoji.value = false
  nextTick(() => messagesScroll.value?.scrollTo({ top: messagesScroll.value.scrollHeight }))
}

function openCreateDialog() {
  showEmoji.value = false
  peerDraft.value = ''
  showCreateDialog.value = true
}

function closeCreateDialog() {
  showCreateDialog.value = false
  peerDraft.value = ''
}

function createConversation() {
  const peer = peerDraft.value.trim()
  if (!peer) return
  // 输入匹配联系人（别名或 ID）→ 绑定为传输会话
  const hit = contacts.value.find(
    (c) => c.alias === peer || c.peerId === peer || `${c.alias} (${c.peerId.slice(0, 8)}…)` === peer,
  )
  const conversation: LocalConversation = {
    id: `${Date.now().toString(16)}-${Math.random().toString(16).slice(2, 8)}`,
    peer: hit ? hit.alias : peer,
    peerId: hit?.peerId,
    messages: [],
    updatedAt: new Date().toISOString(),
  }
  conversations.value.unshift(conversation)
  selectConversation(conversation.id)
  persist()
  closeCreateDialog()
}

function startConversationWith(contact: Contact) {
  const existing = conversations.value.find((item) => item.peerId === contact.peerId)
  if (existing) {
    selectConversation(existing.id)
    return
  }
  const conversation: LocalConversation = {
    id: `${Date.now().toString(16)}-${Math.random().toString(16).slice(2, 8)}`,
    peer: contact.alias,
    peerId: contact.peerId,
    messages: [],
    updatedAt: new Date().toISOString(),
  }
  conversations.value.unshift(conversation)
  selectConversation(conversation.id)
  persist()
}

function removeConversation() {
  if (!current.value || !confirm(`删除与「${current.value.peer}」的本地会话？`)) return
  conversations.value = conversations.value.filter((item) => item.id !== currentId.value)
  currentId.value = conversations.value[0]?.id ?? ''
  persist()
}

function appendMessage(kind: MessageKind, content: string, direction: 'self' | 'peer' = 'self', status: MessageStatus = 'local') {
  if (!current.value || !content) return
  current.value.messages.push({
    id: `${Date.now().toString(16)}-${Math.random().toString(16).slice(2, 8)}`,
    kind,
    content,
    createdAt: new Date().toISOString(),
    status,
    direction,
  })
  current.value.updatedAt = new Date().toISOString()
  conversations.value.sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
  persist()
  nextTick(() => messagesScroll.value?.scrollTo({ top: messagesScroll.value.scrollHeight, behavior: 'smooth' }))
}

function statusLabel(message: LocalMessage) {
  if (message.direction === 'peer') return '已接收'
  switch (message.status) {
    case 'sending':
      return '发送中…'
    case 'sent':
      return '已送达'
    case 'queued':
      return '离线暂存 · 对端上线后补投'
    case 'failed':
      return '发送失败'
    default:
      return '本地保存'
  }
}

async function sendText() {
  const text = draft.value.trim()
  if (!text) return
  draft.value = ''
  // 已绑定联系人且桌面端 → 真实传输（先入离线发件箱再当场投递）
  if (desktop.value && current.value?.peerId) {
    const conversation = current.value
    const message: LocalMessage = {
      id: `${Date.now().toString(16)}-${Math.random().toString(16).slice(2, 8)}`,
      kind: 'text',
      content: text,
      createdAt: new Date().toISOString(),
      status: 'sending',
      direction: 'self',
    }
    conversation.messages.push(message)
    conversation.updatedAt = new Date().toISOString()
    persist()
    try {
      const result = await props.bridge.sendMessage(conversation.peerId!, text)
      message.status = result.status === 'sent' ? 'sent' : 'queued'
    } catch (e) {
      message.status = 'failed'
      transportNote.value = e instanceof Error ? e.message : String(e)
    }
    persist()
    nextTick(() => messagesScroll.value?.scrollTo({ top: messagesScroll.value.scrollHeight, behavior: 'smooth' }))
    return
  }
  appendMessage('text', text)
  draft.value = ''
}

function sendEmoji(emoji: string) {
  appendMessage('emoji', emoji)
  showEmoji.value = false
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
    event.preventDefault()
    sendText()
  }
}

function onCreateKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.isComposing) {
    event.preventDefault()
    createConversation()
  }
}

function chooseImage() {
  fileInput.value?.click()
}

function onImage(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return
  if (file.size > 2 * 1024 * 1024) {
    alert('图片不能超过 2 MB')
    return
  }
  const reader = new FileReader()
  reader.onload = () => appendMessage('image', String(reader.result ?? ''))
  reader.readAsDataURL(file)
  ;(event.target as HTMLInputElement).value = ''
}

function formatTime(value: string) {
  return new Intl.DateTimeFormat('zh-CN', { hour: '2-digit', minute: '2-digit' }).format(new Date(value))
}

// ---- 邀请互加 ----

async function openInviteDialog() {
  try {
    const invite = await props.bridge.createInvite()
    inviteQr.value = invite.qrPayload
    inviteShort.value = invite.shortCode
    showInviteDialog.value = true
  } catch (e) {
    transportNote.value = e instanceof Error ? e.message : String(e)
  }
}

async function copyInvite() {
  try {
    await navigator.clipboard.writeText(inviteQr.value)
    transportNote.value = '邀请原文已复制，发给对方即可'
  } catch {
    transportNote.value = '复制失败，请手动选择文本复制'
  }
}

async function submitAddContact() {
  addError.value = ''
  try {
    const contact = await props.bridge.addContact(addQr.value.trim(), addAlias.value.trim())
    contacts.value = await props.bridge.listContacts()
    startConversationWith(contact)
    showAddDialog.value = false
    addQr.value = ''
    addAlias.value = ''
  } catch (e) {
    addError.value = e instanceof Error ? e.message : String(e)
  }
}

// ---- 收件轮询：增量合并进本地会话 ----

async function pollInboxOnce() {
  if (!desktop.value) return
  try {
    const cursor = Number(localStorage.getItem(CURSOR_KEY) ?? '0') || 0
    const result = await props.bridge.pollInbox(cursor)
    for (const item of result.entries) {
      let conversation = conversations.value.find((c) => c.peerId === item.peerId)
      if (!conversation) {
        const contact = contacts.value.find((c) => c.peerId === item.peerId)
        conversation = {
          id: `${Date.now().toString(16)}-${Math.random().toString(16).slice(2, 8)}`,
          peer: contact?.alias ?? item.peerId.slice(0, 8),
          peerId: item.peerId,
          messages: [],
          updatedAt: new Date().toISOString(),
        }
        conversations.value.unshift(conversation)
      }
      conversation.messages.push({
        id: `inbox-${item.id}`,
        kind: 'text',
        content: item.text,
        createdAt: new Date(item.receivedAt * 1000).toISOString(),
        status: 'local',
        direction: 'peer',
      })
      conversation.updatedAt = new Date().toISOString()
      if (item.id === result.entries.at(-1)?.id && conversation.id === currentId.value) {
        nextTick(() => messagesScroll.value?.scrollTo({ top: messagesScroll.value.scrollHeight, behavior: 'smooth' }))
      }
    }
    if (result.entries.length) {
      localStorage.setItem(CURSOR_KEY, String(result.maxId))
      conversations.value.sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
      persist()
    }
  } catch {
    // 静默：轮询失败（未配置中继等）不打扰
  }
}

onMounted(async () => {
  try {
    const raw = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '[]')
    if (Array.isArray(raw)) conversations.value = raw
  } catch {
    conversations.value = []
  }
  currentId.value = conversations.value[0]?.id ?? ''

  // 消息传输接线：桌面端启动 listener + 身份/联系人加载；预览/失败静默降级为本地会话
  try {
    const identity = await props.bridge.getIdentity()
    if (identity) {
      desktop.value = true
      myId.value = identity.idBase32
      contacts.value = await props.bridge.listContacts()
      await props.bridge.startMessagingListener()
      await pollInboxOnce()
      pollTimer = setInterval(pollInboxOnce, 3000)
    }
  } catch {
    desktop.value = false // 预览模式或主目录不可定位：保持本地会话行为
  }
})

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer)
})

watch(conversations, persist, { deep: true })
</script>

<template>
  <section class="people-chat-view">
    <aside class="people-conversations">
      <header class="people-list-head">
        <div>
          <p class="people-kicker">Elwright</p>
          <h2>消息会话</h2>
          <small v-if="myId" class="my-id" :title="`我的 ID：${myId}`">我的 ID：{{ myId.slice(0, 8) }}…</small>
        </div>
        <div class="people-list-actions">
          <button v-if="desktop" class="icon-btn" title="邀请对方添加我" aria-label="邀请对方添加我" @click="openInviteDialog"><QrCode :size="17" /></button>
          <button v-if="desktop" class="icon-btn" title="通过邀请添加联系人" aria-label="通过邀请添加联系人" @click="showAddDialog = true; addError = ''"><UserPlus :size="17" /></button>
          <button class="icon-btn" title="新建会话" aria-label="新建会话" @click="openCreateDialog"><Plus :size="17" /></button>
        </div>
      </header>
      <div v-if="!conversations.length" class="people-empty-list">还没有消息会话</div>
      <button v-for="item in conversations" :key="item.id" class="people-conversation" :class="{ active: item.id === currentId }" @click="selectConversation(item.id)">
        <span class="peer-avatar"><MessageCircle :size="16" /></span>
        <span class="peer-summary">
          <strong>{{ item.peer }}</strong>
          <small>{{ item.messages.at(-1)?.content || '新会话' }}</small>
        </span>
        <small v-if="item.peerId" class="peer-online-dot" title="已绑定联系人 · 可端到端传输">⚡</small>
      </button>
      <!-- 联系人快捷区：桌面端显示，一键开聊 -->
      <div v-if="desktop && contacts.length" class="contacts-strip">
        <small class="contacts-title">联系人</small>
        <button v-for="c in contacts" :key="c.peerId" class="contact-chip" :title="c.peerId" @click="startConversationWith(c)">
          {{ c.alias }}
        </button>
      </div>
    </aside>

    <main v-if="current" class="people-chat-main">
      <header class="people-chat-head">
        <div>
          <h3>{{ current.peer }}</h3>
          <span class="local-status">{{ current.peerId ? (desktop ? '端到端加密 · 经消息中继传输' : '已绑定联系人 · 预览模式仅本机') : '本地会话 · 尚未连接消息服务' }}</span>
        </div>
        <button class="icon-btn danger-icon" title="删除本地会话" aria-label="删除本地会话" @click="removeConversation"><Trash2 :size="16" /></button>
      </header>
      <div ref="messagesScroll" class="people-messages">
        <div v-if="!current.messages.length" class="people-chat-empty">
          <MessageCircle :size="28" />
          <p>从工作上的一件小事开始聊起</p>
          <small>{{ current.peerId ? '消息端到端加密；对端不在线时自动离线暂存、上线补投。' : '当前内容只保存在本机；通过「邀请」互加联系人后可真实互发。' }}</small>
        </div>
        <article v-for="message in current.messages" :key="message.id" class="people-message" :class="message.direction === 'peer' ? 'peer' : 'self'">
          <div class="message-bubble">
            <img v-if="message.kind === 'image'" :src="message.content" alt="发送的图片" class="message-image" />
            <span v-else :class="{ 'message-emoji': message.kind === 'emoji' }">{{ message.content }}</span>
          </div>
          <small>{{ formatTime(message.createdAt) }} · {{ statusLabel(message) }}</small>
        </article>
      </div>
      <p v-if="transportNote" class="transport-note">{{ transportNote }}<button class="link-dismiss" @click="transportNote = ''">×</button></p>
      <footer class="people-compose">
        <div v-if="showEmoji" class="emoji-picker">
          <button v-for="emoji in emojis" :key="emoji" :aria-label="`发送${emoji}`" @click="sendEmoji(emoji)">{{ emoji }}</button>
        </div>
        <textarea v-model="draft" rows="2" :placeholder="current.peerId && desktop ? '输入工作消息…（端到端加密）' : '输入工作消息…（仅保存本机）'" @keydown="onKeydown"></textarea>
        <div class="compose-actions">
          <button class="icon-btn" title="发送图片（仅保存本机）" aria-label="发送图片" @click="chooseImage"><ImagePlus :size="17" /></button>
          <button class="icon-btn" title="发送表情" aria-label="发送表情" @click="showEmoji = !showEmoji"><SmilePlus :size="17" /></button>
          <button class="send-message" :disabled="!draft.trim()" @click="sendText"><Send :size="15" />发送</button>
        </div>
        <input ref="fileInput" type="file" accept="image/*" hidden @change="onImage" />
      </footer>
    </main>

    <main v-else class="people-no-conversation">
      <MessageCircle :size="34" />
      <h3>开始一段工作沟通</h3>
      <p>先创建一个消息会话；桌面端可通过邀请互加联系人真实互发（端到端加密）。</p>
      <button class="primary" @click="openCreateDialog">新建会话</button>
    </main>

    <div v-if="showCreateDialog" class="people-dialog-mask" @click.self="closeCreateDialog">
      <form class="people-dialog" aria-label="新建消息会话" @submit.prevent="createConversation">
        <header>
          <div><h3>新建消息会话</h3><p>输入联系人别名可绑定端到端传输；其他名称将创建纯本地会话。</p></div>
          <button class="icon-btn" type="button" title="关闭" aria-label="关闭" @click="closeCreateDialog"><X :size="17" /></button>
        </header>
        <label for="peer-name">对方名称或 Elwright 标识</label>
        <input id="peer-name" v-model="peerDraft" autofocus placeholder="例如：李明 或粘贴联系人 ID" @keydown="onCreateKeydown" />
        <footer><button class="secondary" type="button" @click="closeCreateDialog">取消</button><button class="primary" type="submit" :disabled="!peerDraft.trim()">创建会话</button></footer>
      </form>
    </div>

    <!-- 邀请弹窗：展示我的 v3 邀请原文 -->
    <div v-if="showInviteDialog" class="people-dialog-mask" @click.self="showInviteDialog = false">
      <form class="people-dialog" aria-label="我的邀请" @submit.prevent>
        <header>
          <div><h3>邀请对方添加我</h3><p>把下方原文发给对方；5 分钟内有效，对方「添加联系人」粘贴即可。</p></div>
          <button class="icon-btn" type="button" title="关闭" aria-label="关闭" @click="showInviteDialog = false"><X :size="17" /></button>
        </header>
        <small class="invite-short">短码 {{ inviteShort }}</small>
        <textarea class="invite-text" readonly rows="5" :value="inviteQr" @focus="($event.target as HTMLTextAreaElement).select()"></textarea>
        <footer>
          <button class="secondary" type="button" @click="showInviteDialog = false">关闭</button>
          <button class="primary" type="button" @click="copyInvite"><Copy :size="14" /> 复制邀请原文</button>
        </footer>
      </form>
    </div>

    <!-- 添加联系人弹窗：粘贴对方邀请原文 -->
    <div v-if="showAddDialog" class="people-dialog-mask" @click.self="showAddDialog = false">
      <form class="people-dialog" aria-label="添加联系人" @submit.prevent="submitAddContact">
        <header>
          <div><h3>通过邀请添加联系人</h3><p>粘贴对方发给你的邀请原文；校验签名与有效期后加入联系人。</p></div>
          <button class="icon-btn" type="button" title="关闭" aria-label="关闭" @click="showAddDialog = false"><X :size="17" /></button>
        </header>
        <label for="invite-input">邀请原文</label>
        <textarea id="invite-input" v-model="addQr" rows="5" placeholder="elwright-invite:v3:…" spellcheck="false"></textarea>
        <label for="invite-alias">备注名（可选）</label>
        <input id="invite-alias" v-model="addAlias" placeholder="例如：李明" />
        <p v-if="addError" class="add-error">{{ addError }}</p>
        <footer>
          <button class="secondary" type="button" @click="showAddDialog = false">取消</button>
          <button class="primary" type="submit" :disabled="!addQr.trim()">校验并添加</button>
        </footer>
      </form>
    </div>
  </section>
</template>

<style scoped>
.people-list-actions {
  display: flex;
  gap: 4px;
}
.my-id {
  display: block;
  font-size: 11px;
  opacity: 0.7;
}
.peer-online-dot {
  margin-left: auto;
  font-size: 11px;
}
.contacts-strip {
  margin-top: 8px;
  padding: 8px 10px 4px;
  border-top: 1px solid var(--border, rgba(128, 128, 128, 0.2));
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.contacts-title {
  width: 100%;
  font-size: 11px;
  opacity: 0.6;
}
.contact-chip {
  font-size: 12px;
  padding: 3px 10px;
  border-radius: 999px;
  border: 1px solid var(--border, rgba(128, 128, 128, 0.35));
  background: transparent;
  cursor: pointer;
}
.contact-chip:hover {
  border-color: var(--accent, #3a7bd5);
  color: var(--accent, #3a7bd5);
}
.people-message.peer {
  align-self: flex-start;
}
.people-message.peer .message-bubble {
  background: var(--bubble-peer, rgba(112, 128, 160, 0.18));
}
.transport-note {
  margin: 0 16px 6px;
  font-size: 12px;
  color: #b8863b;
  display: flex;
  align-items: center;
  gap: 6px;
}
.link-dismiss {
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
}
.invite-short {
  font-size: 12px;
  opacity: 0.75;
}
.invite-text {
  width: 100%;
  font-family: ui-monospace, monospace;
  font-size: 11.5px;
  word-break: break-all;
  resize: vertical;
}
.add-error {
  margin: 0;
  font-size: 12px;
  color: #d05353;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>

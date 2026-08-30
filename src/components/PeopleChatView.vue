<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { ImagePlus, MessageCircle, Plus, Send, SmilePlus, Trash2, X } from 'lucide-vue-next'

type MessageKind = 'text' | 'image' | 'emoji'
interface LocalMessage {
  id: string
  kind: MessageKind
  content: string
  createdAt: string
  status: 'local'
}
interface LocalConversation {
  id: string
  peer: string
  messages: LocalMessage[]
  updatedAt: string
}

const STORAGE_KEY = 'elwright.people-conversations'
const conversations = ref<LocalConversation[]>([])
const currentId = ref('')
const draft = ref('')
const showEmoji = ref(false)
const showCreateDialog = ref(false)
const peerDraft = ref('')
const messagesScroll = ref<HTMLElement | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const emojis = ['👍', '✅', '🎉', '💡', '👀', '🚀', '🙏', '🙂']

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
  const conversation: LocalConversation = {
    id: `${Date.now().toString(16)}-${Math.random().toString(16).slice(2, 8)}`,
    peer,
    messages: [],
    updatedAt: new Date().toISOString(),
  }
  conversations.value.unshift(conversation)
  selectConversation(conversation.id)
  persist()
  closeCreateDialog()
}

function removeConversation() {
  if (!current.value || !confirm(`删除与「${current.value.peer}」的本地会话？`)) return
  conversations.value = conversations.value.filter((item) => item.id !== currentId.value)
  currentId.value = conversations.value[0]?.id ?? ''
  persist()
}

function appendMessage(kind: MessageKind, content: string) {
  if (!current.value || !content) return
  current.value.messages.push({
    id: `${Date.now().toString(16)}-${Math.random().toString(16).slice(2, 8)}`,
    kind,
    content,
    createdAt: new Date().toISOString(),
    status: 'local',
  })
  current.value.updatedAt = new Date().toISOString()
  conversations.value.sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
  persist()
  nextTick(() => messagesScroll.value?.scrollTo({ top: messagesScroll.value.scrollHeight, behavior: 'smooth' }))
}

function sendText() {
  const text = draft.value.trim()
  if (!text) return
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

onMounted(() => {
  try {
    const raw = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '[]')
    if (Array.isArray(raw)) conversations.value = raw
  } catch {
    conversations.value = []
  }
  currentId.value = conversations.value[0]?.id ?? ''
})

watch(conversations, persist, { deep: true })
</script>

<template>
  <section class="people-chat-view">
    <aside class="people-conversations">
      <header class="people-list-head">
        <div><p class="people-kicker">Elwright</p><h2>消息会话</h2></div>
        <button class="icon-btn" title="新建会话" aria-label="新建会话" @click="openCreateDialog"><Plus :size="17" /></button>
      </header>
      <div v-if="!conversations.length" class="people-empty-list">还没有消息会话</div>
      <button v-for="item in conversations" :key="item.id" class="people-conversation" :class="{ active: item.id === currentId }" @click="selectConversation(item.id)">
        <span class="peer-avatar"><MessageCircle :size="16" /></span>
        <span class="peer-summary"><strong>{{ item.peer }}</strong><small>{{ item.messages.at(-1)?.content || '新会话' }}</small></span>
      </button>
    </aside>

    <main v-if="current" class="people-chat-main">
      <header class="people-chat-head">
        <div><h3>{{ current.peer }}</h3><span class="local-status">本地会话 · 尚未连接消息服务</span></div>
        <button class="icon-btn danger-icon" title="删除本地会话" aria-label="删除本地会话" @click="removeConversation"><Trash2 :size="16" /></button>
      </header>
      <div ref="messagesScroll" class="people-messages">
        <div v-if="!current.messages.length" class="people-chat-empty">
          <MessageCircle :size="28" />
          <p>从工作上的一件小事开始聊起</p>
          <small>当前内容只保存在本机，网络消息服务将在后续阶段接入。</small>
        </div>
        <article v-for="message in current.messages" :key="message.id" class="people-message self">
          <div class="message-bubble">
            <img v-if="message.kind === 'image'" :src="message.content" alt="发送的图片" class="message-image" />
            <span v-else :class="{ 'message-emoji': message.kind === 'emoji' }">{{ message.content }}</span>
          </div>
          <small>{{ formatTime(message.createdAt) }} · 本地保存</small>
        </article>
      </div>
      <footer class="people-compose">
        <div v-if="showEmoji" class="emoji-picker">
          <button v-for="emoji in emojis" :key="emoji" :aria-label="`发送${emoji}`" @click="sendEmoji(emoji)">{{ emoji }}</button>
        </div>
        <textarea v-model="draft" rows="2" placeholder="输入工作消息…" @keydown="onKeydown"></textarea>
        <div class="compose-actions">
          <button class="icon-btn" title="发送图片" aria-label="发送图片" @click="chooseImage"><ImagePlus :size="17" /></button>
          <button class="icon-btn" title="发送表情" aria-label="发送表情" @click="showEmoji = !showEmoji"><SmilePlus :size="17" /></button>
          <button class="send-message" :disabled="!draft.trim()" @click="sendText"><Send :size="15" />发送</button>
        </div>
        <input ref="fileInput" type="file" accept="image/*" hidden @change="onImage" />
      </footer>
    </main>

    <main v-else class="people-no-conversation">
      <MessageCircle :size="34" />
      <h3>开始一段工作沟通</h3>
      <p>先创建一个消息会话；后续可从这里升级到实时协作空间。</p>
      <button class="primary" @click="openCreateDialog">新建会话</button>
    </main>

    <div v-if="showCreateDialog" class="people-dialog-mask" @click.self="closeCreateDialog">
      <form class="people-dialog" aria-label="新建消息会话" @submit.prevent="createConversation">
        <header>
          <div><h3>新建消息会话</h3><p>先在本机创建会话，消息服务将在后续阶段接入。</p></div>
          <button class="icon-btn" type="button" title="关闭" aria-label="关闭" @click="closeCreateDialog"><X :size="17" /></button>
        </header>
        <label for="peer-name">对方名称或 Elwright 标识</label>
        <input id="peer-name" v-model="peerDraft" autofocus placeholder="例如：李明 或 elwright:liming" @keydown="onCreateKeydown" />
        <footer><button class="secondary" type="button" @click="closeCreateDialog">取消</button><button class="primary" type="submit" :disabled="!peerDraft.trim()">创建会话</button></footer>
      </form>
    </div>
  </section>
</template>

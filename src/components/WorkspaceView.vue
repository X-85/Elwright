<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Folder, FolderOpen, FolderPlus, Link2, FileText, Wrench, StickyNote, AppWindow, Plus, Trash2, Sparkles } from 'lucide-vue-next'
import { renderChatMarkdown } from '../lib/safeMarkdown'
import type { Bridge, Capability, WorkspaceData, WorkspaceFolder, WorkspaceResource, WorkspaceTopic } from '../lib/bridge'

const props = defineProps<{ bridge: Bridge; capabilities: Capability[] }>()
const emit = defineEmits<{ notify: [message: string, ok: boolean] }>()

const data = ref<WorkspaceData>({ folders: [], resources: [], topics: [] })
const mode = ref<'resources' | 'topics'>('resources')
const selectedFolder = ref<string | null>(null)
const selectedResource = ref<string | null>(null)
const selectedTopic = ref<string | null>(null)
const folderName = ref('')
const resourceTitle = ref('')
const resourceKind = ref<'url' | 'path' | 'capability' | 'note' | 'app'>('path')
const resourceValue = ref('')
const resourceNote = ref('')
const launchArgs = ref('')
const resourceIconName = ref('💻')
const topicTitle = ref('')
const topicQuestion = ref('')
const reportBusy = ref(false)

const folderRows = computed(() => {
  const rows: { folder: WorkspaceFolder; depth: number }[] = []
  const visit = (parentId: string | null, depth: number) => {
    data.value.folders.filter((f) => f.parentId === parentId).forEach((folder) => {
      rows.push({ folder, depth })
      visit(folder.id, depth + 1)
    })
  }
  visit(null, 0)
  return rows
})
const visibleResources = computed(() => data.value.resources.filter((r) => r.folderId === selectedFolder.value))
const appShortcuts = computed(() => visibleResources.value.filter((r) => r.kind === 'app'))
const currentTopic = computed(() => data.value.topics.find((t) => t.id === selectedTopic.value) ?? null)
const topicResources = computed(() => new Set(currentTopic.value?.resourceIds ?? []))
const resourceIcon = (kind: string) => kind === 'url' ? Link2 : kind === 'path' ? FileText : kind === 'capability' ? Wrench : kind === 'app' ? AppWindow : StickyNote

async function reload() {
  try { data.value = await props.bridge.loadWorkspace() } catch (e) { emit('notify', String(e), false) }
}

async function addFolder() {
  try {
    const folder = await props.bridge.createWorkspaceFolder(folderName.value, selectedFolder.value)
    folderName.value = ''
    selectedFolder.value = folder.id
    await reload()
  } catch (e) { emit('notify', String(e), false) }
}

async function removeFolder(folder: WorkspaceFolder) {
  if (!confirm(`删除文件夹「${folder.name}」？其中资源会移到未分类。`)) return
  try { await props.bridge.deleteWorkspaceFolder(folder.id); selectedFolder.value = folder.parentId; await reload() } catch (e) { emit('notify', String(e), false) }
}

async function addResource() {
  try {
    await props.bridge.createWorkspaceResource({ title: resourceTitle.value, kind: resourceKind.value, value: resourceValue.value, folderId: selectedFolder.value, note: resourceNote.value, launchArgs: resourceKind.value === 'app' ? launchArgs.value.trim().split(/\s+/).filter(Boolean) : [], icon: resourceKind.value === 'app' ? resourceIconName.value : '' })
    resourceTitle.value = ''; resourceValue.value = ''; resourceNote.value = ''; launchArgs.value = ''
    await reload()
  } catch (e) { emit('notify', String(e), false) }
}

async function chooseFile() {
  const path = await props.bridge.chooseWorkspaceFile()
  if (!path) {
    if (props.bridge.kind === 'browser') emit('notify', '【预览模式】浏览器无法读取本机文件路径，请在桌面应用中选择文件。', false)
    return
  }
  resourceKind.value = 'path'
  resourceValue.value = path
  if (!resourceTitle.value.trim()) {
    resourceTitle.value = path.split(/[\\/]/).pop() ?? path
  }
}

async function launchResource(resource: WorkspaceResource) {
  if (resource.kind !== 'app') return
  try { await props.bridge.launchWorkspaceApp(resource.id); emit('notify', `已启动「${resource.title}」`, true) } catch (e) { emit('notify', String(e), false) }
}

async function removeResource(id: string) {
  if (!confirm('移除这个收藏资源？不会删除原文件。')) return
  try { await props.bridge.deleteWorkspaceResource(id); selectedResource.value = null; await reload() } catch (e) { emit('notify', String(e), false) }
}

async function addTopic() {
  try {
    const topic = await props.bridge.createWorkspaceTopic(topicTitle.value, topicQuestion.value)
    topicTitle.value = ''; topicQuestion.value = ''; selectedTopic.value = topic.id; await reload()
  } catch (e) { emit('notify', String(e), false) }
}

async function saveTopic() {
  if (!currentTopic.value) return
  try { await props.bridge.updateWorkspaceTopic(currentTopic.value); emit('notify', '课题已保存', true); await reload() } catch (e) { emit('notify', String(e), false) }
}

async function toggleTopicResource(resource: WorkspaceResource) {
  if (!currentTopic.value) return
  const ids = new Set(currentTopic.value.resourceIds)
  ids.has(resource.id) ? ids.delete(resource.id) : ids.add(resource.id)
  currentTopic.value.resourceIds = [...ids]
  await saveTopic()
}

async function generateReport() {
  if (!currentTopic.value || reportBusy.value) return
  reportBusy.value = true
  try { const result = await props.bridge.generateTopicReport(currentTopic.value.id); currentTopic.value.report = result.content; emit('notify', result.note ?? '报告已生成', true); await reload() } catch (e) { emit('notify', String(e), false) } finally { reportBusy.value = false }
}

async function removeTopic() {
  if (!currentTopic.value || !confirm(`删除课题「${currentTopic.value.title}」？`)) return
  try { await props.bridge.deleteWorkspaceTopic(currentTopic.value.id); selectedTopic.value = null; await reload() } catch (e) { emit('notify', String(e), false) }
}

function selectCapability(cap: Capability) {
  resourceTitle.value = cap.name; resourceKind.value = 'capability'; resourceValue.value = cap.id
}

onMounted(reload)
</script>

<template>
  <section class="workspace-view">
    <header class="workspace-head">
      <div>
        <h1>资源与课题</h1>
        <p>把资料沉淀下来，再围绕问题形成可复用结论。</p>
      </div>
      <nav class="workspace-tabs">
        <button :class="{ active: mode === 'resources' }" @click="mode = 'resources'"><Folder :size="15" /> 收藏资源</button>
        <button :class="{ active: mode === 'topics' }" @click="mode = 'topics'"><Sparkles :size="15" /> 课题</button>
      </nav>
    </header>

    <div v-if="mode === 'resources'" class="workspace-grid">
      <aside class="workspace-tree">
        <div class="workspace-panel-title"><span>收藏夹</span><span class="muted">{{ data.folders.length }} 个</span></div>
        <div class="folder-row root" :class="{ selected: selectedFolder === null }" @click="selectedFolder = null"><Folder :size="15" /> 未分类 <span>{{ data.resources.filter(r => !r.folderId).length }}</span></div>
        <div v-for="row in folderRows" :key="row.folder.id" class="folder-row" :class="{ selected: selectedFolder === row.folder.id }" :style="{ paddingLeft: `${7 + row.depth * 16}px` }" @click="selectedFolder = row.folder.id"><Folder :size="15" /> {{ row.folder.name }} <span>{{ data.resources.filter(r => r.folderId === row.folder.id).length }}</span><button title="删除文件夹" @click.stop="removeFolder(row.folder)"><Trash2 :size="13" /></button></div>
        <form class="inline-form" @submit.prevent="addFolder"><input v-model="folderName" placeholder="新建文件夹" /><button title="新建文件夹" :disabled="!folderName.trim()"><FolderPlus :size="15" /></button></form>
        <p class="hint">最多三层嵌套；收藏只记录文件位置，不复制或删除原文件。</p>
      </aside>
      <section class="workspace-main">
        <div class="workspace-panel-title"><span>{{ selectedFolder ? data.folders.find(f => f.id === selectedFolder)?.name : '未分类' }}</span><span class="muted">{{ visibleResources.length }} 项</span></div>
        <div class="resource-list">
          <div v-if="appShortcuts.length" class="app-shortcuts"><button v-for="resource in appShortcuts" :key="resource.id" class="app-shortcut" :title="`启动 ${resource.title}`" @click="launchResource(resource)"><span class="app-shortcut-icon">{{ resource.icon || '💻' }}</span><small>{{ resource.title }}</small></button></div>
          <button v-for="resource in visibleResources" :key="resource.id" class="resource-row" :class="{ selected: selectedResource === resource.id }" @click="selectedResource = resource.id"><span v-if="resource.kind === 'app'" class="resource-icon-text">{{ resource.icon || '💻' }}</span><component v-else :is="resourceIcon(resource.kind)" :size="16" /><span><strong>{{ resource.title }}</strong><small>{{ resource.value }}</small></span></button>
          <p v-if="!visibleResources.length" class="empty-state">还没有收藏文件。选择右侧的“选择本地文件”后，把它整理到当前文件夹。</p>
        </div>
        <div v-if="selectedResource" class="resource-detail">
          <template v-if="data.resources.find(r => r.id === selectedResource)"><div class="detail-kicker">资源详情</div><h2>{{ data.resources.find(r => r.id === selectedResource)?.title }}</h2><code>{{ data.resources.find(r => r.id === selectedResource)?.value }}</code><p>{{ data.resources.find(r => r.id === selectedResource)?.note }}</p><button v-if="data.resources.find(r => r.id === selectedResource)?.kind === 'url'" class="save-topic" @click="bridge.openExternal(data.resources.find(r => r.id === selectedResource)!.value)"><Link2 :size="14" /> 打开链接</button><button v-if="data.resources.find(r => r.id === selectedResource)?.kind === 'app'" class="save-topic" @click="launchResource(data.resources.find(r => r.id === selectedResource)!)"><AppWindow :size="14" /> 启动软件</button><button class="danger-link" @click="removeResource(selectedResource!)"><Trash2 :size="14" /> 移除收藏</button></template>
        </div>
      </section>
      <aside class="workspace-form"><div class="workspace-panel-title">收藏文件</div><button class="file-picker" @click="chooseFile"><FolderOpen :size="16" /> 选择本地文件</button><p class="hint">文件保留在原位置，Elwright 只保存路径和分类。</p><div class="form-divider">或添加其他入口</div><input v-model="resourceTitle" placeholder="名称" /><select v-model="resourceKind"><option value="path">本地文件</option><option value="url">网页链接</option><option value="capability">Elwright 能力</option><option value="app">软件快捷方式</option><option value="note">文字笔记</option></select><textarea v-model="resourceValue" rows="3" :placeholder="resourceKind === 'path' ? '本地文件路径（也可通过上方选择）' : resourceKind === 'app' ? '应用路径或可执行文件命令' : 'URL、能力 ID 或笔记内容'" /><input v-if="resourceKind === 'app'" v-model="launchArgs" placeholder="启动参数（空格分隔，可选）" /><select v-if="resourceKind === 'app'" v-model="resourceIconName" aria-label="快捷方式图标"><option value="💻">💻 电脑</option><option value="🧰">🧰 工具</option><option value="🌐">🌐 浏览器</option><option value="✎">✎ 编辑器</option><option value="▣">▣ 终端</option></select><textarea v-model="resourceNote" rows="2" placeholder="备注（可选）" /><button class="primary wide" :disabled="!resourceTitle.trim() || !resourceValue.trim()" @click="addResource"><Plus :size="15" /> {{ resourceKind === 'app' ? '添加快捷方式' : resourceKind === 'path' ? '收藏文件' : '添加资源' }}</button><div v-if="resourceKind === 'capability'" class="cap-picks"><button v-for="cap in capabilities" :key="cap.id" @click="selectCapability(cap)">{{ cap.name }}</button></div></aside>
    </div>

    <div v-else class="topics-layout">
      <aside class="topic-list"><div class="workspace-panel-title"><span>我的课题</span><span class="muted">{{ data.topics.length }}</span></div><button v-for="topic in data.topics" :key="topic.id" :class="['topic-row', { selected: selectedTopic === topic.id }]" @click="selectedTopic = topic.id"><strong>{{ topic.title }}</strong><small>{{ topic.resourceIds.length }} 个资源</small></button><p v-if="!data.topics.length" class="empty-state">从一个具体问题开始。</p><form class="topic-create" @submit.prevent="addTopic"><input v-model="topicTitle" placeholder="新课题名称" /><textarea v-model="topicQuestion" rows="3" placeholder="你想弄清什么？" /><button class="primary wide" :disabled="!topicTitle.trim()"><Plus :size="15" /> 创建课题</button></form></aside>
      <section v-if="currentTopic" class="topic-detail"><div class="topic-title-row"><input v-model="currentTopic.title" class="topic-title-input" /><button class="primary" :disabled="reportBusy" @click="generateReport"><Sparkles :size="15" /> {{ reportBusy ? '生成中…' : '生成报告' }}</button></div><textarea v-model="currentTopic.question" class="topic-question" rows="3" placeholder="研究问题" /><div class="topic-resources"><div class="workspace-panel-title">关联资源 <span class="muted">勾选后将作为报告上下文</span></div><label v-for="resource in data.resources" :key="resource.id" class="topic-resource-check"><input type="checkbox" :checked="topicResources.has(resource.id)" @change="toggleTopicResource(resource)" /><span>{{ resource.title }}</span><small>{{ resource.kind }}</small></label><p v-if="!data.resources.length" class="hint">先在收藏资源中添加资料。</p></div><div v-if="currentTopic.report" class="report"><div class="detail-kicker">{{ currentTopic.report.startsWith('#') ? '研究报告' : '报告草稿' }}</div><div class="markdown" v-html="renderChatMarkdown(currentTopic.report)"></div></div><button class="save-topic" @click="saveTopic">保存课题</button><button class="danger-link" @click="removeTopic"><Trash2 :size="14" /> 删除课题</button></section><div v-else class="placeholder">← 选择或创建一个课题</div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { Bookmark, ChevronDown, ChevronRight, Code2, Copy, FileText, Folder, FolderOpen, ListTodo, RefreshCw, Search, SquareTerminal, Star, X } from 'lucide-vue-next'
import type { Bridge, CodeBrowserRecent, CodeDocument, CodeSearchHit, CodeSymbolHit, CodeTreeEntry } from '../lib/bridge'
import { highlightCode } from '../lib/codeHighlight'
import { codeLinkMarker } from '../lib/codeLinks'

const props = defineProps<{ bridge: Bridge }>()
const emit = defineEmits<{
  (e: 'notify', msg: string, ok: boolean): void
  (e: 'send-to-ai', payload: { title: string; text: string }): void
  (e: 'open-in-terminal', dir: string): void
}>()

const projectRoot = ref('')
const projectName = ref('')
// 注意：必须是普通对象而非 Map——Vue 3 v-for 遍历 Map 得到的是 [key, value] 对儿 +
// 数字下标，模板里的 (entries, rel) 会拿错（rel 恒为数字 → v-show 恒 false → 整棵树不可见）
const treeCache = ref<Record<string, CodeTreeEntry[]>>({})
const expanded = ref(new Set<string>())
const recentProjects = ref<CodeBrowserRecent['projects']>([])
const favorites = ref<CodeBrowserRecent['favorites']>([])
const bookmarks = ref<CodeBrowserRecent['bookmarks']>([])
const recentFiles = ref<CodeBrowserRecent['files']>([])

const previewNotice = ref('')

const query = ref('')
const searchMode = ref<'filename' | 'content'>('filename')
const searchHits = ref<CodeSearchHit[]>([])
const searching = ref(false)

const jumpQuery = ref('')
const symbolHits = ref<CodeSymbolHit[]>([])
const symbolsScanned = ref(false)
const jumping = ref(false)

interface OpenTab { path: string; doc: CodeDocument; targetLine: number }
const tabs = ref<OpenTab[]>([])
const activePath = ref('')

const activeTab = computed(() => tabs.value.find((t) => t.path === activePath.value) ?? null)
const activeLines = computed(() => {
  const doc = activeTab.value?.doc
  if (!doc || doc.sensitive || doc.truncated) return []
  return doc.content.split('\n').map((line, i) => ({ no: i + 1, html: highlightCode(line, doc.language) }))
})

async function notify(msg: string, ok = false) {
  emit('notify', msg, ok)
}

async function refreshRecent() {
  try {
    const r = await props.bridge.codeBrowserRecentLoad()
    recentProjects.value = r.projects
    recentFiles.value = r.files
    favorites.value = r.favorites
    bookmarks.value = r.bookmarks
  } catch { /* 最近列表读取失败不阻塞主流程 */ }
}
refreshRecent()

async function chooseProject() {
  previewNotice.value = ''
  const picked = await props.bridge.chooseProjectDirectory()
  if (!picked) {
    if (props.bridge.kind !== 'tauri') {
      previewNotice.value = '【预览模式】浏览器无法访问本机目录，请在桌面应用中使用代码浏览器。'
    }
    return
  }
  await openProject(picked)
}

async function openProject(root: string, rel = '') {
  try {
    // 先用一次树读取验证目录可用
    await props.bridge.codeBrowserTree(root, '')
    projectRoot.value = root
    projectName.value = root.split(/[\\/]/).filter(Boolean).pop() ?? root
    treeCache.value = {}
    expanded.value = new Set()
    tabs.value = []
    activePath.value = ''
    searchHits.value = []
    symbolHits.value = []
    symbolsScanned.value = false
    await loadTree('')
    const r = await props.bridge.codeBrowserRecentOpen(root, rel)
    recentProjects.value = r.projects
    recentFiles.value = r.files
    favorites.value = r.favorites
    bookmarks.value = r.bookmarks
  } catch (e) {
    notify(String(e), false)
  }
}

async function removeRecentProject(rootPath: string) {
  try {
    const r = await props.bridge.codeBrowserRecentRemoveProject(rootPath)
    recentProjects.value = r.projects
    recentFiles.value = r.files
    notify('已从最近项目移除（不影响磁盘上的项目文件）', true)
  } catch (e) {
    notify(String(e), false)
  }
}

async function loadTree(rel: string) {
  if (treeCache.value[rel] !== undefined) return
  const entries = await props.bridge.codeBrowserTree(projectRoot.value, rel)
  treeCache.value[rel] = entries
}

async function toggleDir(entry: CodeTreeEntry) {
  if (expanded.value.has(entry.path)) {
    expanded.value.delete(entry.path)
    expanded.value = new Set(expanded.value)
    return
  }
  try {
    await loadTree(entry.path)
    expanded.value.add(entry.path)
    expanded.value = new Set(expanded.value)
  } catch (e) {
    notify(String(e), false)
  }
}

/**
 * 扁平化可见树行：按 expanded 递归下钻 treeCache（懒加载，任意深度）。
 * 模板只渲染这一个列表——勿改回「外层 v-for treeCache + 模板手工嵌套」：
 * 外层循环会把每个已缓存层级再当顶级列表渲染一遍，展开后每级目录出现两次（Q23）。
 */
const visibleRows = computed(() => {
  const rows: { entry: CodeTreeEntry; depth: number }[] = []
  const walk = (rel: string, depth: number) => {
    for (const e of treeCache.value[rel] ?? []) {
      rows.push({ entry: e, depth })
      if (e.kind === 'dir' && expanded.value.has(e.path)) walk(e.path, depth + 1)
    }
  }
  walk('', 0)
  return rows
})

async function ensureSymbols() {
  if (symbolsScanned.value) return
  jumping.value = true
  try {
    symbolHits.value = await props.bridge.codeBrowserScanSymbols(projectRoot.value)
    symbolsScanned.value = true
  } catch (e) {
    notify(String(e), false)
  } finally {
    jumping.value = false
  }
}

async function openRel(rel: string, targetLine = 0) {
  const existing = tabs.value.find((t) => t.path === rel)
  if (existing) {
    activePath.value = rel
    if (targetLine) await scrollToLine(targetLine)
    return
  }
  try {
    const doc = await props.bridge.codeBrowserRead(projectRoot.value, rel)
    if (doc.sensitive) {
      notify(`已拒绝读取敏感文件：${rel}（${doc.notice}）`, false)
      return
    }
    tabs.value.push({ path: rel, doc, targetLine })
    if (tabs.value.length > 8) tabs.value.shift()
    activePath.value = rel
    const r = await props.bridge.codeBrowserRecentOpen(projectRoot.value, rel)
    recentFiles.value = r.files
    if (targetLine) await scrollToLine(targetLine)
  } catch (e) {
    notify(String(e), false)
  }
}

async function onTreeFileClick(entry: CodeTreeEntry) {
  if (!entry.readable) {
    notify(entry.sensitive ? `已拒绝读取敏感文件：${entry.name}` : `文件过大（${entry.size} 字节），不读取`, false)
    return
  }
  await openRel(entry.path)
}

async function scrollToLine(line: number) {
  await nextTick()
  document.getElementById(`ln-${line}`)?.scrollIntoView({ block: 'center' })
}

async function refreshTree() {
  delete treeCache.value['']
  expanded.value = new Set()
  await loadTree('')
}

async function doSearch() {
  searching.value = true
  try {
    searchHits.value = await props.bridge.codeBrowserSearch(projectRoot.value, query.value, searchMode.value)
    if (searchHits.value.length === 0) notify('无搜索结果', true)
  } catch (e) {
    notify(String(e), false)
  } finally {
    searching.value = false
  }
}

const jumpCandidates = computed(() => {
  const q = jumpQuery.value.trim().toLowerCase()
  if (!q) return []
  return symbolHits.value.filter((s) => s.name.toLowerCase().includes(q)).slice(0, 50)
})

async function doJump() {
  await ensureSymbols()
}

const isFavorite = (rel: string) => favorites.value.some((f) => f.path === rel && f.projectRoot === projectRoot.value)
const projectBookmarks = computed(() => bookmarks.value.filter((b) => b.projectRoot === projectRoot.value))
const bookmarkLines = computed(() => new Set(projectBookmarks.value.filter((b) => b.path === activePath.value).map((b) => b.line)))

async function toggleFavorite(rel: string) {
  try {
    favorites.value = await props.bridge.codeBrowserFavoritesToggle(projectRoot.value, rel)
    notify(isFavorite(rel) ? '已收藏' : '已取消收藏', true)
  } catch (e) {
    notify(String(e), false)
  }
}

async function toggleBookmark(line: number) {
  const doc = activeTab.value?.doc
  if (!doc) return
  const label = (doc.content.split('\n')[line - 1] ?? '').trim().slice(0, 40)
  try {
    bookmarks.value = await props.bridge.codeBrowserBookmarksToggle(projectRoot.value, doc.path, line, label)
  } catch (e) {
    notify(String(e), false)
  }
}

const absPath = (rel: string) => projectRoot.value.replace(/[\\/]+$/, '') + '/' + rel

async function createTodoFromLine() {
  const doc = activeTab.value?.doc
  if (!doc) return
  const sel = selectedCodeText.value
  const lineText = sel && sel.trim()
    ? sel.trim().split('\n')[0]
    : (doc.content.split('\n')[0] ?? '')
  const marker = codeLinkMarker(absPath(doc.path), 1)
  const text = `代码 ${marker} ${lineText.slice(0, 60)}`.trim()
  try {
    await props.bridge.todoAdd(text)
    notify('已创建 Todo（含代码位置，可在工作台点击跳回）', true)
  } catch (e) {
    notify(String(e), false)
  }
}

function openInTerminal() {
  const doc = activeTab.value?.doc
  if (!doc) return
  const dir = absPath(doc.path).split(/[\\/]/).slice(0, -1).join('/')
  emit('open-in-terminal', dir)
}

// ---- 发送到 AI：先确认（路径/范围/摘要），再交给 App 切到对话页预填 ----
const aiConfirm = ref<{ title: string; path: string; range: string; text: string } | null>(null)

const selectedCodeText = computed(() => {
  const sel = window.getSelection()
  if (!sel || sel.isCollapsed) return ''
  const text = sel.toString()
  // 只接受代码区内的选区
  const anchor = sel.anchorNode
  if (!anchor || !(anchor instanceof Element) ) {
    const parent = anchor?.parentElement
    return parent?.closest('.cb-code') ? text : ''
  }
  return (anchor as Element).closest('.cb-code') ? text : ''
})

function requestSendToAi() {
  const doc = activeTab.value?.doc
  if (!doc) return
  if (doc.sensitive) {
    notify('敏感文件不发送给 AI。', false)
    return
  }
  const selected = selectedCodeText.value
  const text = selected && selected.trim() ? selected : doc.content
  const capped = text.length > 8000 ? text.slice(0, 8000) + '\n…（超长截断）' : text
  const range = selected && selected.trim()
    ? `选中片段（${selected.split('\n').length} 行）`
    : `整个文件（${doc.content.split('\n').length} 行）`
  aiConfirm.value = { title: doc.path, path: doc.path, range, text: capped }
}

function confirmSendToAi() {
  if (!aiConfirm.value) return
  emit('send-to-ai', { title: aiConfirm.value.path, text: aiConfirm.value.text })
  aiConfirm.value = null
}

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    notify('已复制', true)
  } catch (e) {
    notify(`复制失败: ${e}`, false)
  }
}

function closeTab(path: string) {
  const idx = tabs.value.findIndex((t) => t.path === path)
  if (idx === -1) return
  tabs.value.splice(idx, 1)
  if (activePath.value === path) {
    activePath.value = tabs.value[Math.max(0, idx - 1)]?.path ?? ''
  }
}

/** Todo 联动跳回：按绝对路径打开文件（可带行号定位）。 */
async function openAbsolute(absPath: string, line = 0) {
  const norm = absPath.replace(/[\\/]+$/, '')
  const idx = Math.max(norm.lastIndexOf('/'), norm.lastIndexOf('\\'))
  if (idx === -1) return
  const dir = norm.slice(0, idx)
  const file = norm.slice(idx + 1)
  await openProject(dir, file)
  await openRel(file, line)
}

defineExpose({ openProject, openAbsolute })
</script>

<template>
  <section class="code-browser" aria-label="代码浏览器">
    <div class="cb-toolbar">
      <button class="cb-open" @click="chooseProject"><FolderOpen :size="15" /> 选择项目目录</button>
      <span v-if="projectName" class="cb-project">{{ projectName }}</span>
      <span v-else class="cb-hint">只读浏览；应用不会自行扫描磁盘，只访问你选择的目录。</span>
      <button v-if="projectRoot" class="cb-icon-btn" title="刷新当前目录树" @click="refreshTree"><RefreshCw :size="14" /></button>
    </div>

    <div v-if="!projectRoot" class="cb-recent">
      <div v-if="recentProjects.length" class="cb-recent-block">
        <div class="cb-panel-title">最近项目</div>
        <div v-for="p in recentProjects" :key="p.rootPath" class="cb-recent-row">
          <button class="cb-recent-open" :title="p.rootPath" @click="openProject(p.rootPath)">
            <Folder :size="14" /> {{ p.name }} <code class="cb-muted">{{ p.rootPath }}</code>
          </button>
          <button class="cb-recent-remove" :aria-label="'删除最近项目 ' + p.name" title="从最近列表移除（不影响磁盘上的项目文件）" @click.stop="removeRecentProject(p.rootPath)">×</button>
        </div>
      </div>
      <p v-if="previewNotice" class="cb-notice">{{ previewNotice }}</p>
      <p v-else class="cb-empty">还没有最近项目。选择一个本地项目目录开始浏览。</p>
    </div>

    <div v-else class="cb-workspace">
      <aside class="cb-tree">
        <div class="cb-search">
          <input v-model="query" class="cb-search-input" placeholder="文件名 / 内容搜索…" @keyup.enter="doSearch" />
          <select v-model="searchMode" class="cb-search-mode" aria-label="搜索模式">
            <option value="filename">文件名</option>
            <option value="content">内容</option>
          </select>
          <button class="cb-icon-btn" title="搜索" :disabled="searching || !query.trim()" @click="doSearch"><Search :size="14" /></button>
        </div>
        <ul v-if="searchHits.length" class="cb-hits">
          <li v-for="hit in searchHits" :key="hit.path + hit.line">
            <button class="cb-hit" @click="openRel(hit.path, hit.line)">
              <FileText :size="13" /> {{ hit.path }}<span v-if="hit.line" class="cb-muted">:{{ hit.line }}</span>
              <span v-if="hit.snippet" class="cb-snippet">{{ hit.snippet }}</span>
            </button>
          </li>
        </ul>

        <div class="cb-jump">
          <input v-model="jumpQuery" class="cb-search-input" placeholder="跳转符号（类 / 接口 / 方法）…" @focus="doJump" @input="doJump" />
          <ul v-if="jumpCandidates.length" class="cb-hits">
            <li v-for="(s, i) in jumpCandidates" :key="s.path + s.line + '-' + i">
              <button class="cb-hit" :title="s.declaration" @click="openRel(s.path, s.line)">
                <Code2 :size="13" /> <b>{{ s.kind }}</b> {{ s.name }} <span class="cb-muted">{{ s.path }}:{{ s.line }}</span>
              </button>
            </li>
          </ul>
          <p v-else-if="jumpQuery && symbolsScanned" class="cb-muted cb-jump-empty">无匹配符号；可用内容搜索兜底。</p>
        </div>

        <div class="cb-tree-body">
          <ul class="cb-dir">
            <li v-for="row in visibleRows" :key="row.entry.path">
              <button v-if="row.entry.kind === 'dir'" class="cb-row" :style="{ paddingLeft: 4 + row.depth * 14 + 'px' }" @click="toggleDir(row.entry)">
                <component :is="expanded.has(row.entry.path) ? ChevronDown : ChevronRight" :size="13" />
                <Folder :size="14" /> {{ row.entry.name }}
              </button>
              <button v-else class="cb-row" :class="{ 'cb-locked': !row.entry.readable }" :style="{ paddingLeft: 4 + row.depth * 14 + 'px' }" @click="onTreeFileClick(row.entry)">
                <FileText :size="14" /> {{ row.entry.name }}
                <span v-if="row.entry.sensitive" class="cb-tag">敏感</span>
                <span class="cb-fav" :class="{ on: isFavorite(row.entry.path) }" role="button" :aria-label="(isFavorite(row.entry.path) ? '取消收藏 ' : '收藏 ') + row.entry.name" @click.stop="toggleFavorite(row.entry.path)">★</span>
              </button>
            </li>
          </ul>
        </div>

        <div v-if="favorites.length || projectBookmarks.length" class="cb-marks">
          <div class="cb-panel-title">收藏与书签（当前项目）</div>
          <button v-for="f in favorites.filter((x) => x.projectRoot === projectRoot)" :key="'f-' + f.path" class="cb-hit" @click="openRel(f.path)">
            <Star :size="13" /> {{ f.path }}
          </button>
          <button v-for="b in projectBookmarks" :key="'b-' + b.path + b.line" class="cb-hit" :title="b.label" @click="openRel(b.path, b.line)">
            <Bookmark :size="13" /> {{ b.path }}:{{ b.line }} <span class="cb-muted">{{ b.label }}</span>
          </button>
        </div>

        <div v-if="recentFiles.length" class="cb-recent-files">
          <div class="cb-panel-title">最近文件</div>
          <button v-for="f in recentFiles.slice(0, 8)" :key="f.projectRoot + f.path" class="cb-hit" :title="f.projectRoot" @click="openProject(f.projectRoot, f.path).then(() => openRel(f.path))">
            <FileText :size="13" /> {{ f.path }}
          </button>
        </div>
      </aside>

      <main class="cb-viewer">
        <div class="cb-tabs">
          <div v-for="t in tabs" :key="t.path" :class="['cb-tab', { active: t.path === activePath }]" :title="t.path">
            <button class="cb-tab-name" @click="activePath = t.path">{{ t.doc.name }}</button>
            <button class="cb-tab-close" :aria-label="'关闭 ' + t.doc.name" @click="closeTab(t.path)"><X :size="12" /></button>
          </div>
          <span v-if="!tabs.length" class="cb-muted cb-viewer-hint">从左侧打开文件预览（只读）。</span>
        </div>

        <div v-if="activeTab" class="cb-doc">
          <div class="cb-doc-bar">
            <code>{{ activeTab.doc.path }}</code>
            <span class="cb-muted">{{ activeTab.doc.language }} · {{ activeTab.doc.size }} B</span>
            <button class="cb-icon-btn" :title="isFavorite(activeTab.doc.path) ? '取消收藏' : '收藏文件'" :class="{ 'cb-fav-on': isFavorite(activeTab.doc.path) }" @click="toggleFavorite(activeTab.doc.path)"><Star :size="13" /></button>
            <button class="cb-icon-btn" title="复制文件路径" @click="copyText(projectRoot + '/' + activeTab.doc.path)"><Copy :size="13" /></button>
            <button class="cb-send-ai" title="把当前文件创建为 Todo（含代码位置，可在工作台跳回）" @click="createTodoFromLine"><ListTodo :size="13" /> 转为 Todo</button>
            <button class="cb-send-ai" title="在终端中打开文件所在目录" @click="openInTerminal"><SquareTerminal :size="13" /> 终端定位</button>
            <button class="cb-send-ai" @click="requestSendToAi">发送到 AI</button>
          </div>
          <p v-if="activeTab.doc.notice" class="cb-notice">{{ activeTab.doc.notice }}</p>
          <div v-if="aiConfirm" class="cb-ai-confirm" role="dialog" aria-label="确认发送到 AI">
            <div>将发送到 AI 对话：<code>{{ aiConfirm.path }}</code></div>
            <div class="cb-muted">范围：{{ aiConfirm.range }} · {{ aiConfirm.text.length }} 字符。内容只会进入对话输入框，发送前可再编辑。</div>
            <div class="cb-ai-actions">
              <button class="cb-send-ai" @click="confirmSendToAi">确认并切换到对话</button>
              <button class="cb-icon-btn" @click="aiConfirm = null">取消</button>
            </div>
          </div>
          <div v-if="!activeTab.doc.sensitive && !activeTab.doc.truncated" class="cb-code">
            <div v-for="l in activeLines" :id="'ln-' + l.no" :key="l.no" :class="['cb-line', { 'cb-marked': bookmarkLines.has(l.no) }]">
              <span class="cb-lno" :title="bookmarkLines.has(l.no) ? '点击移除书签' : '点击添加书签'" @click="toggleBookmark(l.no)">{{ l.no }}</span>
              <!-- 内容已先整体 HTML 转义再着色（lib/codeHighlight），与 ADR-002 同源思路 -->
              <span class="cb-code-text" v-html="l.html"></span>
            </div>
          </div>
        </div>
      </main>
    </div>
  </section>
</template>

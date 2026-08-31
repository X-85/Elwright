<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { Bookmark, ChevronDown, ChevronRight, Code2, Copy, FileText, Folder, FolderOpen, RefreshCw, Search, Star, X } from 'lucide-vue-next'
import type { Bridge, CodeBrowserRecent, CodeDocument, CodeSearchHit, CodeSymbolHit, CodeTreeEntry } from '../lib/bridge'
import { highlightCode } from '../lib/codeHighlight'

const props = defineProps<{ bridge: Bridge }>()
const emit = defineEmits<{
  (e: 'notify', msg: string, ok: boolean): void
  (e: 'send-to-ai', payload: { title: string; text: string }): void
}>()

const projectRoot = ref('')
const projectName = ref('')
const treeCache = ref(new Map<string, CodeTreeEntry[]>())
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
    treeCache.value = new Map()
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

async function loadTree(rel: string) {
  if (treeCache.value.has(rel)) return
  const entries = await props.bridge.codeBrowserTree(projectRoot.value, rel)
  treeCache.value.set(rel, entries)
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
  treeCache.value.delete('')
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
const favoriteOf = (rel: string) => favorites.value.find((f) => f.path === rel && f.projectRoot === projectRoot.value)
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

defineExpose({ openProject })
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
        <button v-for="p in recentProjects" :key="p.rootPath" class="cb-recent-row" @click="openProject(p.rootPath)">
          <Folder :size="14" /> {{ p.name }} <code class="cb-muted">{{ p.rootPath }}</code>
        </button>
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
          <template v-for="(entries, rel) in treeCache" :key="rel">
            <ul v-show="rel === '' || expanded.has(rel)" class="cb-dir">
              <li v-for="e in entries" :key="e.path">
                <button v-if="e.kind === 'dir'" class="cb-row" @click="toggleDir(e)">
                  <component :is="expanded.has(e.path) ? ChevronDown : ChevronRight" :size="13" />
                  <Folder :size="14" /> {{ e.name }}
                </button>
                <button v-else class="cb-row" :class="{ 'cb-locked': !e.readable }" @click="onTreeFileClick(e)">
                  <span class="cb-indent"></span><FileText :size="14" /> {{ e.name }}
                  <span v-if="e.sensitive" class="cb-tag">敏感</span>
                  <span class="cb-fav" :class="{ on: isFavorite(e.path) }" role="button" :aria-label="(isFavorite(e.path) ? '取消收藏 ' : '收藏 ') + e.name" @click.stop="toggleFavorite(e.path)">★</span>
                </button>
                <template v-if="e.kind === 'dir' && expanded.has(e.path)">
                  <ul class="cb-dir cb-sub">
                    <li v-for="c in treeCache.get(e.path) ?? []" :key="c.path">
                      <button v-if="c.kind === 'dir'" class="cb-row" @click="toggleDir(c)">
                        <component :is="expanded.has(c.path) ? ChevronDown : ChevronRight" :size="13" />
                        <Folder :size="14" /> {{ c.name }}
                      </button>
                      <button v-else class="cb-row" :class="{ 'cb-locked': !c.readable }" @click="onTreeFileClick(c)">
                        <span class="cb-indent"></span><FileText :size="14" /> {{ c.name }}
                        <span v-if="c.sensitive" class="cb-tag">敏感</span>
                      </button>
                      <!-- 阶段①树最多展示两层懒加载展开，更深层级继续点开时复用同一机制 -->
                      <ul v-if="c.kind === 'dir' && expanded.has(c.path)" class="cb-dir cb-sub2">
                        <li v-for="gc in treeCache.get(c.path) ?? []" :key="gc.path">
                          <button v-if="gc.kind === 'file'" class="cb-row" :class="{ 'cb-locked': !gc.readable }" @click="onTreeFileClick(gc)">
                            <span class="cb-indent"></span><FileText :size="14" /> {{ gc.name }}
                            <span v-if="gc.sensitive" class="cb-tag">敏感</span>
                          </button>
                          <button v-else class="cb-row" @click="toggleDir(gc)">
                            <component :is="expanded.has(gc.path) ? ChevronDown : ChevronRight" :size="13" />
                            <Folder :size="14" /> {{ gc.name }}
                          </button>
                        </li>
                      </ul>
                    </li>
                  </ul>
                </template>
              </li>
            </ul>
          </template>
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

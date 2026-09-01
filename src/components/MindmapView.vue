<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import { ChevronDown, ChevronRight, ListTree, Network, Plus, RefreshCw, Trash2 } from 'lucide-vue-next'
import type { Bridge, MindmapDoc, MindmapSummary } from '../lib/bridge'
import { addChild, addSibling, indent, isHidden, outdent, removeSubtree, moveVertical } from '../lib/mindmap'

const props = defineProps<{ bridge: Bridge }>()

const maps = ref<MindmapSummary[]>([])
const corrupt = ref<string[]>([])
const current = ref<MindmapDoc | null>(null)
const selectedNodeId = ref('')
const loadDegraded = ref(false)
const loadError = ref('')
const statusText = ref('')
const newListTitle = ref('')
const showNew = ref(false)
const importing = ref(false)
const listLoading = ref(true)

const depthOfId = (id: string) => {
  if (!current.value) return 0
  let depth = 0
  let node = current.value.nodes.find((n) => n.id === id)
  const guard = current.value.nodes.length
  while (node?.parent && depth <= guard) {
    depth += 1
    node = current.value.nodes.find((n) => n.id === node!.parent)
  }
  return depth
}

const visibleRows = computed(() => {
  if (!current.value) return []
  return current.value.nodes
    .filter((n) => !isHidden(current.value!.nodes, n.id))
    .map((n) => ({ node: n, depth: depthOfId(n.id), childCount: current.value!.nodes.filter((c) => c.parent === n.id).length }))
})

function flash(msg: string) {
  statusText.value = msg
  setTimeout(() => {
    if (statusText.value === msg) statusText.value = ''
  }, 2500)
}

async function refreshList() {
  listLoading.value = true
  try {
    const result = await props.bridge.listMindmaps()
    maps.value = result.maps
    corrupt.value = result.corrupt
    loadDegraded.value = false
  } catch (e) {
    loadDegraded.value = true
    loadError.value = e instanceof Error ? e.message : String(e)
  } finally {
    listLoading.value = false
  }
}

async function openMap(id: string) {
  try {
    current.value = await props.bridge.loadMindmap(id)
    selectedNodeId.value = current.value.nodes[0]?.id ?? ''
    loadError.value = ''
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e)
  }
}

async function createMap() {
  const title = newListTitle.value.trim()
  if (!title) return
  try {
    const doc = await props.bridge.createMindmap(title)
    newListTitle.value = ''
    showNew.value = false
    await refreshList()
    current.value = doc
    selectedNodeId.value = doc.nodes[0]?.id ?? ''
  } catch (e) {
    flash(e instanceof Error ? e.message : String(e))
  }
}

async function deleteMap(id: string) {
  if (!confirm('删除这张脑图？该操作不可恢复。')) return
  try {
    await props.bridge.deleteMindmap(id)
    if (current.value?.id === id) current.value = null
    await refreshList()
  } catch (e) {
    flash(e instanceof Error ? e.message : String(e))
  }
}

let saveTimer: ReturnType<typeof setTimeout> | null = null
function scheduleSave() {
  if (!current.value || loadDegraded.value) return
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(async () => {
    if (!current.value) return
    try {
      current.value = await props.bridge.saveMindmap(current.value)
    } catch (e) {
      flash(e instanceof Error ? e.message : String(e))
    }
  }, 400)
}

function mutate(fn: (nodes: MindmapDoc['nodes']) => void) {
  if (!current.value) return
  fn(current.value.nodes)
  scheduleSave()
}

function addSiblingAndFocus(nodeId: string) {
  const node = current.value?.nodes.find((n) => n.id === nodeId)
  if (!node) return
  mutate((nodes) => {
    const created = addSibling(nodes, nodeId, '新节点')
    if (created) selectedNodeId.value = created.id
  })
  nextTick(() => document.getElementById(`mm-input-${selectedNodeId.value}`)?.focus())
}

function addChildTo(nodeId: string) {
  mutate((nodes) => {
    const created = addChild(nodes, nodeId, '新节点')
    if (created) {
      const parent = nodes.find((n) => n.id === nodeId)
      if (parent) parent.collapsed = false
      selectedNodeId.value = created.id
    }
  })
  nextTick(() => document.getElementById(`mm-input-${selectedNodeId.value}`)?.focus())
}

function removeNode(nodeId: string) {
  mutate((nodes) => {
    removeSubtree(nodes, nodeId)
    selectedNodeId.value = nodes[0]?.id ?? ''
  })
}

function toggleCollapse(nodeId: string) {
  mutate((nodes) => {
    const node = nodes.find((n) => n.id === nodeId)
    if (node) node.collapsed = !node.collapsed
  })
}

function onKeydown(event: KeyboardEvent, nodeId: string) {
  if (event.key === 'Enter' && !event.isComposing) {
    event.preventDefault()
    addSiblingAndFocus(nodeId)
  } else if (event.key === 'Tab') {
    event.preventDefault()
    if (event.shiftKey) {
      mutate((nodes) => {
        outdent(nodes, nodeId)
        selectedNodeId.value = nodeId
      })
    } else {
      mutate((nodes) => {
        indent(nodes, nodeId)
        selectedNodeId.value = nodeId
      })
    }
  } else if (event.key === 'ArrowUp' && (event.metaKey || event.ctrlKey)) {
    event.preventDefault()
    mutate((nodes) => {
      moveVertical(nodes, nodeId, true)
      selectedNodeId.value = nodeId
    })
  } else if (event.key === 'ArrowDown' && (event.metaKey || event.ctrlKey)) {
    event.preventDefault()
    mutate((nodes) => {
      moveVertical(nodes, nodeId, false)
      selectedNodeId.value = nodeId
    })
  }
}

/** 节点转 Todo：写入工作台 Todo 并打标记（ADR-001 §D4）。 */
async function convertToTodo(nodeId: string) {
  const node = current.value?.nodes.find((n) => n.id === nodeId)
  if (!node || node.convertedTodo || !node.text.trim()) return
  try {
    await props.bridge.todoAdd(node.text.trim())
    mutate((nodes) => {
      const target = nodes.find((n) => n.id === nodeId)
      if (target) target.convertedTodo = true
    })
    flash('已转为工作台 Todo')
  } catch (e) {
    flash(e instanceof Error ? e.message : String(e))
  }
}

/** 从 Todo 导入：未完成 Todo 挂到选中节点（无选中挂根）。 */
async function importFromTodos() {
  if (!current.value || importing.value) return
  importing.value = true
  try {
    const todos = await props.bridge.todoList()
    const open = todos.filter((t) => !t.done)
    if (!open.length) {
      flash('工作台没有未完成的 Todo')
      return
    }
    const anchor = selectedNodeId.value || current.value.nodes[0]?.id
    if (!anchor) return
    mutate((nodes) => {
      for (const t of open) {
        const created = addChild(nodes, anchor, t.text)
        if (created) created.convertedTodo = true
      }
    })
    flash(`已导入 ${open.length} 条 Todo`)
  } catch (e) {
    flash(e instanceof Error ? e.message : String(e))
  } finally {
    importing.value = false
  }
}

function formatTime(ts: number) {
  return new Intl.DateTimeFormat('zh-CN', { month: 'numeric', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(new Date(ts * 1000))
}

onMounted(refreshList)
</script>

<template>
  <section class="mindmap-view">
    <aside class="mindmap-side">
      <header class="mindmap-side-head">
        <div><p class="people-kicker">Elwright</p><h2>脑图</h2></div>
        <button class="icon-btn" title="新建脑图" aria-label="新建脑图" @click="showNew = !showNew; newListTitle = ''"><Plus :size="17" /></button>
      </header>
      <form v-if="showNew" class="mindmap-new" @submit.prevent="createMap">
        <input v-model="newListTitle" placeholder="脑图标题，如：部署思路" autofocus @keydown.escape="showNew = false" />
        <button class="primary" type="submit" :disabled="!newListTitle.trim()">创建</button>
      </form>
      <div v-if="listLoading" class="mindmap-empty">加载中…</div>
      <div v-else-if="!maps.length && !loadDegraded" class="mindmap-empty">还没有脑图</div>
      <button
        v-for="m in maps"
        :key="m.id"
        class="mindmap-item"
        :class="{ active: current?.id === m.id }"
        @click="openMap(m.id)"
      >
        <Network :size="15" />
        <span class="mindmap-item-body">
          <strong>{{ m.title }}</strong>
          <small>{{ m.nodeCount }} 节点 · {{ formatTime(m.updatedAt) }}</small>
        </span>
        <span class="mindmap-item-del" title="删除脑图" @click.stop="deleteMap(m.id)"><Trash2 :size="14" /></span>
      </button>
      <p v-if="corrupt.length" class="mindmap-corrupt">{{ corrupt.length }} 个损坏文件已跳过</p>
      <p v-if="loadDegraded" class="mindmap-degraded">{{ loadError }}</p>
    </aside>

    <main v-if="current" class="mindmap-main">
      <header class="mindmap-head">
        <h3>{{ current.title }}</h3>
        <span v-if="statusText" class="mindmap-status">{{ statusText }}</span>
        <button class="icon-btn" :disabled="importing" title="从工作台 Todo 导入（挂到选中节点）" aria-label="从工作台 Todo 导入" @click="importFromTodos">
          <RefreshCw :size="15" />
        </button>
      </header>
      <div class="mindmap-rows">
        <div
          v-for="row in visibleRows"
          :key="row.node.id"
          class="mindmap-row"
          :class="{ selected: row.node.id === selectedNodeId }"
          :style="{ paddingLeft: 12 + row.depth * 22 + 'px' }"
          @click="selectedNodeId = row.node.id"
        >
          <button
            v-if="row.childCount"
            class="icon-btn mm-caret"
            :aria-label="row.node.collapsed ? '展开' : '折叠'"
            @click.stop="toggleCollapse(row.node.id)"
          >
            <ChevronDown v-if="!row.node.collapsed" :size="14" />
            <ChevronRight v-else :size="14" />
          </button>
          <span v-else class="mm-caret-placeholder"></span>
          <input
            :id="`mm-input-${row.node.id}`"
            v-model="row.node.text"
            class="mm-input"
            :style="{ fontWeight: row.depth === 0 ? 600 : 400 }"
            @keydown="onKeydown($event, row.node.id)"
            @focus="selectedNodeId = row.node.id"
          />
          <button v-if="row.depth > 0" class="icon-btn mm-op" title="转为工作台 Todo" aria-label="转为工作台 Todo" :disabled="row.node.convertedTodo" @click.stop="convertToTodo(row.node.id)">
            <ListTree :size="14" />
          </button>
          <button v-if="row.depth > 0" class="icon-btn mm-op" title="删除节点（含子节点）" aria-label="删除节点" @click.stop="removeNode(row.node.id)">
            <Trash2 :size="14" />
          </button>
          <button class="icon-btn mm-op" title="加子节点" aria-label="加子节点" @click.stop="addChildTo(row.node.id)">
            <Plus :size="14" />
          </button>
          <span v-if="row.node.convertedTodo" class="mm-todo-tag" title="已转为工作台 Todo">todo</span>
        </div>
      </div>
      <p class="mindmap-hint">Enter 加兄弟节点 · Tab 缩进 · Shift+Tab 外提 · Ctrl/⌘+↑↓ 上下移动；改动自动保存</p>
    </main>

    <main v-else class="mindmap-no-selection">
      <Network :size="34" />
      <h3>用层级整理你的思路</h3>
      <p>新建一张脑图，或从左侧选择；节点可一键转为工作台 Todo。</p>
      <button class="primary" @click="showNew = true">新建脑图</button>
    </main>
  </section>
</template>

<style scoped>
.mindmap-view {
  display: grid;
  grid-template-columns: 260px 1fr;
  gap: 12px;
  height: 100%;
  min-height: 0;
}
.mindmap-side {
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow-y: auto;
}
.mindmap-side-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.mindmap-new {
  display: flex;
  gap: 6px;
}
.mindmap-new input {
  flex: 1;
  min-width: 0;
}
.mindmap-empty {
  font-size: 13px;
  opacity: 0.6;
  padding: 8px 4px;
}
.mindmap-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid transparent;
  background: transparent;
  cursor: pointer;
  text-align: left;
}
.mindmap-item.active {
  border-color: var(--accent, #3a7bd5);
  background: color-mix(in srgb, var(--accent, #3a7bd5) 8%, transparent);
}
.mindmap-item-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.mindmap-item-body strong {
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.mindmap-item-body small {
  font-size: 11px;
  opacity: 0.6;
}
.mindmap-item-del {
  opacity: 0;
  transition: opacity 0.15s;
}
.mindmap-item:hover .mindmap-item-del {
  opacity: 0.7;
}
.mindmap-corrupt,
.mindmap-degraded {
  font-size: 12px;
  opacity: 0.65;
  padding: 4px;
}
.mindmap-main {
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.mindmap-head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding-bottom: 8px;
}
.mindmap-status {
  font-size: 12px;
  opacity: 0.7;
}
.mindmap-rows {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.mindmap-row {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border-radius: 7px;
  min-height: 32px;
}
.mindmap-row.selected {
  background: color-mix(in srgb, var(--accent, #3a7bd5) 9%, transparent);
}
.mm-caret-placeholder {
  width: 22px;
  flex: none;
}
.mm-input {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  font-size: 13.5px;
  padding: 3px 4px;
  color: inherit;
}
.mm-input:focus {
  outline: none;
  background: color-mix(in srgb, var(--accent, #3a7bd5) 7%, transparent);
  border-radius: 5px;
}
.mm-op {
  opacity: 0;
  transition: opacity 0.15s;
}
.mindmap-row:hover .mm-op,
.mindmap-row.selected .mm-op {
  opacity: 0.75;
}
.mm-todo-tag {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 999px;
  border: 1px solid var(--accent, #3a7bd5);
  color: var(--accent, #3a7bd5);
}
.mindmap-hint {
  font-size: 11.5px;
  opacity: 0.55;
  margin: 6px 0 0;
}
.mindmap-no-selection {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  opacity: 0.8;
}
</style>

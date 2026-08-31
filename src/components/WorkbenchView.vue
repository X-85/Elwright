<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { renderChatMarkdown } from '../lib/safeMarkdown'
import type { Bridge, TodoItem } from '../lib/bridge'

import { parseCodeLinks } from '../lib/codeLinks'
const emit = defineEmits<{ (e: 'open-code', absPath: string, line: number): void }>()
const props = defineProps<{ bridge: Bridge }>()

// ---- Todo ----

const todos = ref<TodoItem[]>([])
const todoDraft = ref('')
const todoLoading = ref(true)
const todoError = ref('')

async function loadTodos() {
  try {
    todos.value = await props.bridge.todoList()
    todoError.value = ''
  } catch (e) {
    todoError.value = e instanceof Error ? e.message : String(e)
  } finally {
    todoLoading.value = false
  }
}

async function addTodo() {
  const text = todoDraft.value.trim()
  if (!text) return
  try {
    await props.bridge.todoAdd(text)
    todoDraft.value = ''
    await loadTodos()
  } catch (e) {
    todoError.value = e instanceof Error ? e.message : String(e)
  }
}

async function toggleTodo(item: TodoItem) {
  try {
    const updated = await props.bridge.todoToggle(item.id)
    const idx = todos.value.findIndex((t) => t.id === item.id)
    if (idx !== -1) todos.value[idx] = updated
  } catch (e) {
    todoError.value = e instanceof Error ? e.message : String(e)
  }
}

async function removeTodo(item: TodoItem) {
  try {
    await props.bridge.todoRemove(item.id)
    todos.value = todos.value.filter((t) => t.id !== item.id)
  } catch (e) {
    todoError.value = e instanceof Error ? e.message : String(e)
  }
}

const doneCount = computed(() => todos.value.filter((t) => t.done).length)

// ---- 今日记录 ----

/** 本地日期 YYYY-MM-DD（记录是「今天」的笔记，用本地时区而非 UTC）。 */
function localDate(d: Date): string {
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${y}-${m}-${day}`
}

const noteDate = ref(localDate(new Date()))
const noteText = ref('')
const noteLoaded = ref(false)
const notePreview = ref(false)
/** ''=无未保存变更；其他=保存状态徽标文案 */
const noteSaveState = ref<'dirty' | 'saving' | 'saved' | 'error'>('')
let saveTimer: ReturnType<typeof setTimeout> | undefined
let saveToken = 0

async function loadNote() {
  saveTimer && clearTimeout(saveTimer)
  noteLoaded.value = false
  noteSaveState.value = ''
  const token = ++saveToken
  try {
    const text = await props.bridge.noteGet(noteDate.value)
    if (token !== saveToken) return // 已切到别的日期，丢弃过期响应
    noteText.value = text ?? ''
    noteLoaded.value = true
  } catch (e) {
    if (token !== saveToken) return
    noteText.value = ''
    noteLoaded.value = true
    noteSaveState.value = 'error'
    noteError.value = e instanceof Error ? e.message : String(e)
  }
}

const noteError = ref('')

/** 防抖自动保存（800ms 静默后触发）。 */
function scheduleSave() {
  noteSaveState.value = 'dirty'
  saveTimer && clearTimeout(saveTimer)
  saveTimer = setTimeout(saveNote, 800)
}

async function saveNote() {
  const token = ++saveToken
  noteSaveState.value = 'saving'
  try {
    await props.bridge.noteSave(noteDate.value, noteText.value)
    if (token !== saveToken) return // 期间又有编辑/切页
    noteSaveState.value = 'saved'
  } catch (e) {
    if (token !== saveToken) return
    noteSaveState.value = 'error'
    noteError.value = e instanceof Error ? e.message : String(e)
  }
}

function shiftDate(days: number) {
  const d = new Date(noteDate.value + 'T12:00:00')
  d.setDate(d.getDate() + days)
  noteDate.value = localDate(d)
}

const noteSaveLabel = computed(
  () =>
    ({
      dirty: '未保存…',
      saving: '保存中…',
      saved: '已保存',
      '': '',
      error: '保存失败',
    })[noteSaveState.value],
)

const noteHtml = computed(() => renderChatMarkdown(noteText.value || '*（暂无记录）*'))

watch(noteDate, loadNote)
onMounted(() => {
  loadTodos()
  loadNote()
})
</script>

<template>
  <div class="workbench">
    <!-- Todo 列 -->
    <section class="wb-todo">
      <header class="wb-head">
        <h2>Todo</h2>
        <span class="wb-count">{{ doneCount }} / {{ todos.length }} 完成</span>
      </header>
      <form class="wb-add" @submit.prevent="addTodo">
        <input v-model="todoDraft" class="wb-input" placeholder="添加一条待办…" maxlength="500" />
        <button type="submit" class="primary" :disabled="!todoDraft.trim()">添加</button>
      </form>
      <p v-if="todoError" class="wb-error">{{ todoError }}</p>
      <p v-else-if="todoLoading" class="wb-empty">加载中…</p>
      <ul v-else class="todo-list">
        <li v-for="item in todos" :key="item.id" :class="['todo-item', { done: item.done }]">
          <label class="todo-main">
            <input type="checkbox" :checked="item.done" @change="toggleTodo(item)" />
            <span class="todo-text">
              <template v-for="(part, pi) in parseCodeLinks(item.text)" :key="pi">
                <button
                  v-if="part.kind === 'code' && part.path"
                  class="todo-code-link"
                  :title="'在代码浏览器中打开 ' + part.path"
                  @click="emit('open-code', part.path, part.line ?? 0)"
                >{{ part.path }}:{{ part.line }}</button>
                <template v-else>{{ part.text }}</template>
              </template>
            </span>
          </label>
          <button class="todo-del" title="删除" aria-label="删除" @click="removeTodo(item)">×</button>
        </li>
        <li v-if="todos.length === 0" class="wb-empty">没有待办，添加第一条吧</li>
      </ul>
    </section>

    <!-- 今日记录列 -->
    <section class="wb-note">
      <header class="wb-head">
        <h2>今日记录</h2>
        <span class="wb-note-nav">
          <button title="前一天" aria-label="前一天" @click="shiftDate(-1)">‹</button>
          <span class="wb-note-date">{{ noteDate }}</span>
          <button title="后一天" aria-label="后一天" @click="shiftDate(1)">›</button>
        </span>
        <span :class="['wb-save-state', noteSaveState]" aria-live="polite">{{ noteSaveLabel }}</span>
        <button
          class="wb-preview-toggle"
          :class="{ active: notePreview }"
          title="切换 Markdown 预览"
          @click="notePreview = !notePreview"
        >{{ notePreview ? '编辑' : '预览' }}</button>
      </header>
      <textarea
        v-show="!notePreview"
        v-model="noteText"
        class="wb-note-editor"
        :disabled="!noteLoaded"
        placeholder="记录今天的工作…（支持 Markdown，自动保存）"
        @input="scheduleSave"
      ></textarea>
      <div v-if="notePreview" class="wb-note-preview markdown" v-html="noteHtml"></div>
      <p v-if="props.bridge.kind === 'browser'" class="wb-preview-note">
        【预览模式】浏览器中修改不持久化，刷新即失；真实数据存于桌面应用（~/.elwright/notes/）。
      </p>
    </section>
  </div>
</template>

<style scoped>
.workbench {
  display: grid;
  grid-template-columns: minmax(260px, 5fr) minmax(320px, 7fr);
  gap: 16px;
  height: 100%;
  min-height: 0;
}
.wb-todo,
.wb-note {
  display: flex;
  flex-direction: column;
  min-height: 0;
  border: 1px solid var(--border, #ddd);
  border-radius: 8px;
  padding: 12px;
  background: var(--panel-bg, inherit);
}
.wb-head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}
.wb-head h2 {
  margin: 0;
  font-size: 15px;
}
.wb-count {
  font-size: 12px;
  opacity: 0.7;
  margin-right: auto;
}
.wb-add {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}
.wb-input {
  flex: 1;
}
.todo-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  flex: 1;
}
.todo-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 4px;
  border-bottom: 1px solid var(--border, #eee);
}
.todo-main {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
  cursor: pointer;
}
.todo-text {
  overflow-wrap: anywhere;
}
.todo-item.done .todo-text {
  text-decoration: line-through;
  opacity: 0.55;
}
.todo-del {
  border: none;
  background: none;
  cursor: pointer;
  font-size: 15px;
  opacity: 0.4;
  padding: 0 4px;
}
.todo-del:hover {
  opacity: 1;
  color: #c0392b;
}
.wb-empty,
.wb-error {
  font-size: 13px;
  opacity: 0.65;
  padding: 4px;
}
.wb-error {
  opacity: 1;
  color: #c0392b;
}
.wb-note-nav {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.wb-note-nav button {
  padding: 0 8px;
}
.wb-note-date {
  font-size: 13px;
  min-width: 92px;
  text-align: center;
  font-variant-numeric: tabular-nums;
}
.wb-save-state {
  font-size: 12px;
  opacity: 0.7;
  margin-left: auto;
}
.wb-save-state.saved {
  color: #27ae60;
  opacity: 1;
}
.wb-save-state.error {
  color: #c0392b;
  opacity: 1;
}
.wb-preview-toggle {
  font-size: 12px;
}
.wb-preview-toggle.active {
  border-color: var(--accent, #4b7bec);
}
.wb-note-editor {
  flex: 1;
  resize: none;
  font-family: inherit;
  line-height: 1.6;
  min-height: 0;
}
.wb-note-preview {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}
.wb-preview-note {
  font-size: 12px;
  opacity: 0.65;
  margin: 8px 0 0;
}
@media (max-width: 760px) {
  .workbench {
    grid-template-columns: 1fr;
  }
}
</style>

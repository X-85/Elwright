<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import CapabilityDetail from './components/CapabilityDetail.vue'
import CapabilityList from './components/CapabilityList.vue'
import { createBridge, type Bridge, type Capability } from './lib/bridge'

const bridge: Bridge = createBridge()
const capabilities = ref<Capability[]>([])
const loadError = ref('')
const filter = ref<'all' | 'script' | 'knowledge' | 'skill'>('all')
const search = ref('')
const selectedId = ref('')

// 能力导入/删除的反馈（toast 式，几秒后自动消失）
const opMsg = ref('')
const opOk = ref(true)
let opTimer: ReturnType<typeof setTimeout> | undefined

function notify(message: string, ok: boolean) {
  opMsg.value = message
  opOk.value = ok
  clearTimeout(opTimer)
  opTimer = setTimeout(() => (opMsg.value = ''), 6000)
}

async function reload() {
  try {
    capabilities.value = await bridge.listCapabilities()
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e)
  }
}

// 导入：id 冲突时弹确认框，确认后带 force 重试
async function importCapability(force = false) {
  const result = await bridge.importCapability(force)
  notify(result.message, result.ok)
  if (result.ok) {
    selectedId.value = ''
    await reload()
  } else if (result.conflict && confirm('该能力已存在，覆盖导入吗？（覆盖后为自定义版本）')) {
    await importCapability(true)
  }
}

async function onDeleted() {
  selectedId.value = ''
  await reload()
}

// 检查更新：手动触发（不轮询，离网友好），结果展示在按钮下方
const checking = ref(false)
const updateMsg = ref('')
const updateUrl = ref('')

async function checkUpdate() {
  if (checking.value) return
  checking.value = true
  updateMsg.value = ''
  updateUrl.value = ''
  try {
    const info = await bridge.checkUpdate()
    if (info.updateAvailable) {
      updateMsg.value = `发现新版本 v${info.latest}（当前 v${info.current}）`
      updateUrl.value = info.releaseUrl
    } else {
      updateMsg.value = `已是最新版本（v${info.current}）`
    }
  } catch (e) {
    updateMsg.value = `检查更新失败：${e instanceof Error ? e.message : String(e)}`
  } finally {
    checking.value = false
  }
}

function openDownload() {
  if (updateUrl.value) bridge.openExternal(updateUrl.value)
}

onMounted(reload)

const filtered = computed(() => {
  const kw = search.value.trim().toLowerCase()
  return capabilities.value.filter((c) => {
    if (filter.value !== 'all' && c.type !== filter.value) return false
    if (!kw) return true
    return (
      c.id.toLowerCase().includes(kw) ||
      c.name.toLowerCase().includes(kw) ||
      (c.category ?? '').toLowerCase().includes(kw)
    )
  })
})

const selected = computed(
  () => capabilities.value.find((c) => c.id === selectedId.value) ?? null,
)

function select(id: string) {
  selectedId.value = selectedId.value === id ? '' : id
}
</script>

<template>
  <div class="layout">
    <aside class="sidebar">
      <h1 class="brand">Elwright</h1>
      <p class="tagline">个人工作流工具箱</p>
      <nav class="filters">
        <button
          v-for="f in ['all', 'script', 'knowledge', 'skill'] as const"
          :key="f"
          :class="{ active: filter === f }"
          @click="filter = f"
        >
          {{ { all: '全部', script: '脚本型', knowledge: '知识型', skill: '技能型' }[f] }}
        </button>
      </nav>
      <input v-model="search" class="search" placeholder="搜索 id / 名称 / 分类…" />
      <button class="import-btn" @click="importCapability()">＋ 导入能力…</button>
      <p class="count">{{ filtered.length }} / {{ capabilities.length }} 项</p>
      <transition name="fade">
        <p v-if="opMsg" :class="['op-toast', opOk ? 'op-ok' : 'op-err']">{{ opMsg }}</p>
      </transition>
      <div class="update-box">
        <button class="update-btn" :disabled="checking" @click="checkUpdate">
          {{ checking ? '检查中…' : '检查更新' }}
        </button>
        <p v-if="updateMsg" class="update-msg">{{ updateMsg }}</p>
        <button
          v-if="updateUrl"
          class="update-link"
          @click="openDownload"
        >前往下载 →</button>
      </div>
      <p class="bridge-badge">
        {{ bridge.kind === 'tauri' ? '桌面模式 · Tauri' : '预览模式 · 浏览器' }}
      </p>
    </aside>

    <main class="content">
      <p v-if="loadError" class="error">加载失败：{{ loadError }}</p>
      <CapabilityList
        v-else
        :capabilities="filtered"
        :selected-id="selectedId"
        @select="select"
      />
      <CapabilityDetail
        v-if="selected"
        :cap="selected"
        :bridge="bridge"
        @notify="notify"
        @deleted="onDeleted"
      />
      <div v-else-if="!loadError" class="placeholder">← 选择一项能力查看详情</div>
    </main>
  </div>
</template>

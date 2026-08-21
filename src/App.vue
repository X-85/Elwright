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

onMounted(async () => {
  try {
    capabilities.value = await bridge.listCapabilities()
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e)
  }
})

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
      <p class="count">{{ filtered.length }} / {{ capabilities.length }} 项</p>
      <p class="bridge-badge">预览模式 · 浏览器</p>
    </aside>

    <main class="content">
      <p v-if="loadError" class="error">加载失败：{{ loadError }}</p>
      <CapabilityList
        v-else
        :capabilities="filtered"
        :selected-id="selectedId"
        @select="select"
      />
      <CapabilityDetail v-if="selected" :cap="selected" :bridge="bridge" />
      <div v-else-if="!loadError" class="placeholder">← 选择一项能力查看详情</div>
    </main>
  </div>
</template>

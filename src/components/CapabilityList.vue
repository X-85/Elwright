<script setup lang="ts">
import type { Capability } from '../lib/bridge'

defineProps<{
  capabilities: Capability[]
  selectedId: string
  lockedIds?: Set<string>
  /** 累计使用次数（ADR-001：解锁进度提示） */
  totalUses?: number
}>()

defineEmits<{ select: [id: string] }>()

const typeLabel: Record<string, string> = {
  script: '脚本',
  knowledge: '知识',
  skill: '技能',
}
</script>

<template>
  <ul class="cap-list">
    <li
      v-for="c in capabilities"
      :key="c.id"
      :class="['cap-item', { selected: c.id === selectedId, locked: lockedIds?.has(c.id) }]"
      @click="$emit('select', c.id)"
    >
      <div class="cap-main">
        <span class="cap-name">{{ c.name }}</span>
        <span :class="['type-badge', c.type]">{{ typeLabel[c.type] ?? c.type }}</span>
        <span v-if="c.origin === 'custom'" class="custom-badge" title="来自用户叠加层 ~/.elwright/">自定义</span>
        <span
          v-if="lockedIds?.has(c.id)"
          class="locked-badge"
          :title="c.unlockAfterUses !== undefined ? `累计使用任意能力 ${c.unlockAfterUses} 次后自动解锁（当前 ${totalUses ?? 0}/${c.unlockAfterUses}）` : '暂未开放解锁条件'"
        >待解锁</span>
      </div>
      <div class="cap-sub">
        <code>{{ c.id }}</code>
        <span v-if="c.category" class="cap-cat">{{ c.category }}</span>
        <span v-if="c.offline" class="offline" title="离网可用">⚡离网</span>
      </div>
    </li>
    <li v-if="capabilities.length === 0" class="cap-empty">没有匹配的能力</li>
  </ul>
</template>

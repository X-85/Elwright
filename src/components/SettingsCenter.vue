<script setup lang="ts">
import { ref } from 'vue'
import { Bot, CircleHelp, Palette, SlidersHorizontal, TerminalSquare } from 'lucide-vue-next'
import LlmSettings from './LlmSettings.vue'
import type { Bridge } from '../lib/bridge'
import { setThemePreference, themePreference, type ThemePreference } from '../lib/theme'

const props = defineProps<{ bridge: Bridge; initialSection?: 'general' | 'appearance' | 'model' | 'terminal' }>()
const emit = defineEmits<{ close: []; saved: [] }>()
const activeSection = ref<'general' | 'appearance' | 'model' | 'terminal'>(props.initialSection ?? 'appearance')

const sections = [
  { id: 'general', label: '常规', icon: SlidersHorizontal },
  { id: 'appearance', label: '外观', icon: Palette },
  { id: 'model', label: '模型设置', icon: Bot },
  { id: 'terminal', label: '终端', icon: TerminalSquare },
] as const

function chooseTheme(value: ThemePreference) {
  setThemePreference(value)
}
</script>

<template>
  <div class="settings-mask" @click.self="emit('close')">
    <section class="settings-center" role="dialog" aria-modal="true" aria-label="设置">
      <header class="settings-head">
        <div>
          <p class="settings-kicker">Elwright</p>
          <h2>设置</h2>
        </div>
        <button class="modal-close" title="关闭设置" aria-label="关闭设置" @click="emit('close')">×</button>
      </header>

      <div class="settings-body">
        <nav class="settings-nav" aria-label="设置分类">
          <button
            v-for="section in sections"
            :key="section.id"
            :class="{ active: activeSection === section.id }"
            @click="activeSection = section.id"
          >
            <component :is="section.icon" :size="17" :stroke-width="1.8" />
            <span>{{ section.label }}</span>
          </button>
          <div class="settings-nav-footer">
            <CircleHelp :size="15" /> <span>设置保存在本机</span>
          </div>
        </nav>

        <main class="settings-content">
          <template v-if="activeSection === 'general'">
            <h3>常规</h3>
            <p class="settings-muted">常用应用行为将在这里统一配置。</p>
            <div class="settings-placeholder">第一阶段暂不增加未经验证的通用偏好。</div>
          </template>

          <template v-else-if="activeSection === 'appearance'">
            <h3>外观</h3>
            <p class="settings-muted">选择 Elwright 的界面主题。</p>
            <div class="theme-options">
              <label v-for="option in [
                { value: 'system', label: '跟随系统', note: '根据操作系统自动切换' },
                { value: 'light', label: '浅色', note: '适合明亮环境' },
                { value: 'dark', label: '深色', note: '适合低光环境' },
              ] as const" :key="option.value" class="theme-option" :class="{ selected: themePreference === option.value }">
                <input :checked="themePreference === option.value" type="radio" name="theme" :value="option.value" @change="chooseTheme(option.value)" />
                <span class="theme-swatch" :class="`theme-swatch-${option.value}`"></span>
                <span><strong>{{ option.label }}</strong><small>{{ option.note }}</small></span>
              </label>
            </div>
          </template>

          <template v-else-if="activeSection === 'model'">
            <h3>模型设置</h3>
            <p class="settings-muted">配置 OpenAI 兼容模型端点，桌面应用与 CLI 共用。</p>
            <LlmSettings :bridge="bridge" embedded @saved="emit('saved')" />
          </template>

          <template v-else>
            <h3>终端</h3>
            <p class="settings-muted">终端字体、主题和滚动历史将在后续阶段提供。</p>
            <div class="settings-placeholder">当前终端行为保持现状。</div>
          </template>
        </main>
      </div>
    </section>
  </div>
</template>

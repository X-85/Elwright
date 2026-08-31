<script setup lang="ts">
import { ref } from 'vue'
import { Bot, CircleHelp, Palette, SlidersHorizontal, TerminalSquare } from 'lucide-vue-next'
import LlmSettings from './LlmSettings.vue'
import type { Bridge } from '../lib/bridge'
import { setThemePreference, themePreference, type ThemePreference } from '../lib/theme'
import { preferences, updatePreferences, TERMINAL_FONT_OPTIONS, TERMINAL_FONT_SIZE_OPTIONS, TERMINAL_SCROLLBACK_OPTIONS, UI_SCALE_OPTIONS, STARTUP_VIEW_OPTIONS } from '../lib/preferences'

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
            <p class="settings-muted">应用行为偏好，保存在本机。</p>
            <div class="pref-row">
              <label for="pref-startup">启动视图</label>
              <select id="pref-startup" :value="preferences.startupView" @change="updatePreferences({ startupView: ($event.target as HTMLSelectElement).value as typeof preferences.startupView })">
                <option v-for="o in STARTUP_VIEW_OPTIONS" :key="o.value" :value="o.value">{{ o.label }}</option>
              </select>
            </div>
            <div class="pref-row">
              <label for="pref-autoupd">启动时自动检查更新</label>
              <input id="pref-autoupd" type="checkbox" :checked="preferences.autoUpdateCheck" @change="updatePreferences({ autoUpdateCheck: ($event.target as HTMLInputElement).checked })" />
            </div>
            <div class="pref-row">
              <label for="pref-lang">界面语言</label>
              <select id="pref-lang" disabled>
                <option>中文（多语言需要 i18n 基建，暂未开放）</option>
              </select>
            </div>
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
            <div class="pref-row">
              <label for="pref-density">界面密度</label>
              <select id="pref-density" :value="preferences.density" @change="updatePreferences({ density: ($event.target as HTMLSelectElement).value as typeof preferences.density })">
                <option value="comfortable">舒适</option>
                <option value="compact">紧凑</option>
              </select>
            </div>
            <div class="pref-row">
              <label for="pref-scale">界面缩放</label>
              <select id="pref-scale" :value="preferences.uiScale" @change="updatePreferences({ uiScale: Number(($event.target as HTMLSelectElement).value) as typeof preferences.uiScale })">
                <option v-for="s in UI_SCALE_OPTIONS" :key="s" :value="s">{{ s }}%</option>
              </select>
            </div>
          </template>

          <template v-else-if="activeSection === 'model'">
            <h3>模型设置</h3>
            <p class="settings-muted">配置 OpenAI 兼容模型端点，桌面应用与 CLI 共用。</p>
            <LlmSettings :bridge="bridge" embedded @saved="emit('saved')" />
          </template>

          <template v-else>
            <h3>终端</h3>
            <p class="settings-muted">字号即时生效；滚动历史对新输出生效。主题已随界面主题自动切换。</p>
            <div class="pref-row">
              <label for="pref-tfont">终端字体</label>
              <select id="pref-tfont" :value="preferences.terminalFontFamily" @change="updatePreferences({ terminalFontFamily: ($event.target as HTMLSelectElement).value })">
                <option v-for="o in TERMINAL_FONT_OPTIONS" :key="o.value" :value="o.value">{{ o.label }}</option>
              </select>
            </div>
            <div class="pref-row">
              <label for="pref-tsize">终端字号</label>
              <select id="pref-tsize" :value="preferences.terminalFontSize" @change="updatePreferences({ terminalFontSize: Number(($event.target as HTMLSelectElement).value) })">
                <option v-for="s in TERMINAL_FONT_SIZE_OPTIONS" :key="s" :value="s">{{ s }} px</option>
              </select>
            </div>
            <div class="pref-row">
              <label for="pref-tscroll">滚动历史</label>
              <select id="pref-tscroll" :value="preferences.terminalScrollback" @change="updatePreferences({ terminalScrollback: Number(($event.target as HTMLSelectElement).value) })">
                <option v-for="s in TERMINAL_SCROLLBACK_OPTIONS" :key="s" :value="s">{{ s }} 行</option>
              </select>
            </div>
          </template>
        </main>
      </div>
    </section>
  </div>
</template>

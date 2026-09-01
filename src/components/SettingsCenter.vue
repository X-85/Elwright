<script setup lang="ts">
import { ref } from 'vue'
import { Bot, CircleHelp, Palette, Radio, SlidersHorizontal, TerminalSquare } from 'lucide-vue-next'
import LlmSettings from './LlmSettings.vue'
import MessagingSettings from './MessagingSettings.vue'
import type { Bridge } from '../lib/bridge'
import { setThemePreference, themePreference, type ThemePreference } from '../lib/theme'
import { preferences, updatePreferences, TERMINAL_FONT_OPTIONS, TERMINAL_FONT_SIZE_OPTIONS, TERMINAL_SCROLLBACK_OPTIONS, UI_SCALE_OPTIONS, STARTUP_VIEW_OPTIONS } from '../lib/preferences'
import { t, LOCALE_OPTIONS, type Locale } from '../lib/i18n'

const props = defineProps<{ bridge: Bridge; initialSection?: 'general' | 'appearance' | 'model' | 'terminal' | 'messaging' }>()
const emit = defineEmits<{ close: []; saved: [] }>()
const activeSection = ref<'general' | 'appearance' | 'model' | 'terminal' | 'messaging'>(props.initialSection ?? 'appearance')

const sections = [
  { id: 'general', label: 'settings.section.general', icon: SlidersHorizontal },
  { id: 'appearance', label: 'settings.section.appearance', icon: Palette },
  { id: 'model', label: 'settings.section.model', icon: Bot },
  { id: 'messaging', label: 'settings.section.messaging', icon: Radio },
  { id: 'terminal', label: 'settings.section.terminal', icon: TerminalSquare },
] as const

const themeOptions = [
  { value: 'system', label: 'settings.appearance.theme.system', note: 'settings.appearance.theme.systemNote' },
  { value: 'light', label: 'settings.appearance.theme.light', note: 'settings.appearance.theme.lightNote' },
  { value: 'dark', label: 'settings.appearance.theme.dark', note: 'settings.appearance.theme.darkNote' },
] as const

function chooseTheme(value: ThemePreference) {
  setThemePreference(value)
}

function chooseLanguage(value: Locale) {
  updatePreferences({ language: value })
}
</script>

<template>
  <div class="settings-mask" @click.self="emit('close')">
    <section class="settings-center" role="dialog" aria-modal="true" aria-label="设置">
      <header class="settings-head">
        <div>
          <p class="settings-kicker">Elwright</p>
          <h2>{{ t('settings.title') }}</h2>
        </div>
        <button class="modal-close" :title="t('settings.close')" :aria-label="t('settings.close')" @click="emit('close')">×</button>
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
            <span>{{ t(section.label) }}</span>
          </button>
          <div class="settings-nav-footer">
            <CircleHelp :size="15" /> <span>{{ t('settings.savedLocally') }}</span>
          </div>
        </nav>

        <main class="settings-content">
          <template v-if="activeSection === 'general'">
            <h3>{{ t('settings.section.general') }}</h3>
            <p class="settings-muted">{{ t('settings.general.desc') }}</p>
            <div class="pref-row">
              <label for="pref-startup">{{ t('settings.general.startupView') }}</label>
              <select id="pref-startup" :value="preferences.startupView" @change="updatePreferences({ startupView: ($event.target as HTMLSelectElement).value as typeof preferences.startupView })">
                <option v-for="o in STARTUP_VIEW_OPTIONS" :key="o.value" :value="o.value">{{ o.label }}</option>
              </select>
            </div>
            <div class="pref-row">
              <label for="pref-autoupd">{{ t('settings.general.autoUpdate') }}</label>
              <input id="pref-autoupd" type="checkbox" :checked="preferences.autoUpdateCheck" @change="updatePreferences({ autoUpdateCheck: ($event.target as HTMLInputElement).checked })" />
            </div>
            <div class="pref-row">
              <label for="pref-lang">{{ t('settings.general.language') }}</label>
              <select id="pref-lang" :value="preferences.language" @change="chooseLanguage(($event.target as HTMLSelectElement).value as Locale)">
                <option v-for="o in LOCALE_OPTIONS" :key="o.value" :value="o.value">{{ o.label }}</option>
              </select>
            </div>
          </template>

          <template v-else-if="activeSection === 'appearance'">
            <h3>{{ t('settings.section.appearance') }}</h3>
            <p class="settings-muted">{{ t('settings.appearance.desc') }}</p>
            <div class="theme-options">
              <label v-for="option in themeOptions" :key="option.value" class="theme-option" :class="{ selected: themePreference === option.value }">
                <input :checked="themePreference === option.value" type="radio" name="theme" :value="option.value" @change="chooseTheme(option.value)" />
                <span class="theme-swatch" :class="`theme-swatch-${option.value}`"></span>
                <span><strong>{{ t(option.label) }}</strong><small>{{ t(option.note) }}</small></span>
              </label>
            </div>
            <div class="pref-row">
              <label for="pref-density">{{ t('settings.appearance.density') }}</label>
              <select id="pref-density" :value="preferences.density" @change="updatePreferences({ density: ($event.target as HTMLSelectElement).value as typeof preferences.density })">
                <option value="comfortable">{{ t('settings.appearance.density.comfortable') }}</option>
                <option value="compact">{{ t('settings.appearance.density.compact') }}</option>
              </select>
            </div>
            <div class="pref-row">
              <label for="pref-scale">{{ t('settings.appearance.scale') }}</label>
              <select id="pref-scale" :value="preferences.uiScale" @change="updatePreferences({ uiScale: Number(($event.target as HTMLSelectElement).value) as typeof preferences.uiScale })">
                <option v-for="s in UI_SCALE_OPTIONS" :key="s" :value="s">{{ s }}%</option>
              </select>
            </div>
          </template>

          <template v-else-if="activeSection === 'model'">
            <h3>{{ t('settings.section.model') }}</h3>
            <p class="settings-muted">{{ t('settings.model.desc') }}</p>
            <LlmSettings :bridge="bridge" embedded @saved="emit('saved')" />
          </template>

          <template v-else-if="activeSection === 'messaging'">
            <h3>{{ t('settings.section.messaging') }}</h3>
            <p class="settings-muted">{{ t('settings.messaging.desc') }}</p>
            <MessagingSettings :bridge="bridge" />
          </template>

          <template v-else>
            <h3>{{ t('settings.section.terminal') }}</h3>
            <p class="settings-muted">{{ t('settings.terminal.desc') }}</p>
            <div class="pref-row">
              <label for="pref-tfont">{{ t('settings.terminal.fontFamily') }}</label>
              <select id="pref-tfont" :value="preferences.terminalFontFamily" @change="updatePreferences({ terminalFontFamily: ($event.target as HTMLSelectElement).value })">
                <option v-for="o in TERMINAL_FONT_OPTIONS" :key="o.value" :value="o.value">{{ o.label }}</option>
              </select>
            </div>
            <div class="pref-row">
              <label for="pref-tsize">{{ t('settings.terminal.fontSize') }}</label>
              <select id="pref-tsize" :value="preferences.terminalFontSize" @change="updatePreferences({ terminalFontSize: Number(($event.target as HTMLSelectElement).value) })">
                <option v-for="s in TERMINAL_FONT_SIZE_OPTIONS" :key="s" :value="s">{{ s }} px</option>
              </select>
            </div>
            <div class="pref-row">
              <label for="pref-tscroll">{{ t('settings.terminal.scrollback') }}</label>
              <select id="pref-tscroll" :value="preferences.terminalScrollback" @change="updatePreferences({ terminalScrollback: Number(($event.target as HTMLSelectElement).value) })">
                <option v-for="s in TERMINAL_SCROLLBACK_OPTIONS" :key="s" :value="s">{{ s }} {{ t('settings.terminal.scrollbackUnit') }}</option>
              </select>
            </div>
          </template>
        </main>
      </div>
    </section>
  </div>
</template>

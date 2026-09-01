<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { PlugZap } from 'lucide-vue-next'
import type { Bridge } from '../lib/bridge'
import { t } from '../lib/i18n'

const props = defineProps<{ bridge: Bridge }>()

const relayUrl = ref('')
const savedUrl = ref('')
const statusText = ref('')
const statusKind = ref<'ok' | 'err' | ''>('')
const testing = ref(false)
const loadDegraded = ref(false)

onMounted(async () => {
  try {
    const cfg = await props.bridge.getMessagingConfig()
    relayUrl.value = cfg.relayUrl
    savedUrl.value = cfg.relayUrl
  } catch (e) {
    // 浏览器预览等无本地配置环境：表单可见但明确降级
    loadDegraded.value = true
    statusText.value = e instanceof Error ? e.message : String(e)
    statusKind.value = 'err'
  }
})

async function save() {
  statusText.value = ''
  try {
    const cfg = await props.bridge.setMessagingRelayUrl(relayUrl.value.trim())
    savedUrl.value = cfg.relayUrl
    statusText.value = t('settings.messaging.saved')
    statusKind.value = 'ok'
  } catch (e) {
    statusText.value = e instanceof Error ? e.message : String(e)
    statusKind.value = 'err'
  }
}

async function test() {
  statusText.value = ''
  testing.value = true
  try {
    const msg = await props.bridge.testMessagingRelay(relayUrl.value.trim() || undefined)
    statusText.value = msg
    statusKind.value = 'ok'
  } catch (e) {
    statusText.value = e instanceof Error ? e.message : String(e)
    statusKind.value = 'err'
  } finally {
    testing.value = false
  }
}
</script>

<template>
  <div class="messaging-settings">
    <div class="pref-row">
      <label for="pref-relay">{{ t('settings.messaging.url') }}</label>
      <input
        id="pref-relay"
        v-model="relayUrl"
        type="text"
        :placeholder="t('settings.messaging.urlPlaceholder')"
        spellcheck="false"
        :disabled="loadDegraded"
      />
    </div>
    <p v-if="!relayUrl && !loadDegraded" class="settings-muted">{{ t('settings.messaging.notConfigured') }}</p>
    <div class="messaging-actions">
      <button class="btn" :disabled="loadDegraded" @click="save">{{ t('settings.messaging.save') }}</button>
      <button class="btn btn-secondary" :disabled="testing || loadDegraded" @click="test">
        <PlugZap :size="15" :stroke-width="1.8" />
        {{ testing ? t('settings.messaging.testing') : t('settings.messaging.test') }}
      </button>
    </div>
    <p v-if="statusText" class="messaging-status" :class="statusKind">{{ statusText }}</p>
  </div>
</template>

<style scoped>
.messaging-settings {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.messaging-settings input[type='text'] {
  flex: 1;
  min-width: 0;
}
.messaging-actions {
  display: flex;
  gap: 8px;
}
.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.messaging-status {
  font-size: 12.5px;
  white-space: pre-wrap;
  word-break: break-all;
}
.messaging-status.ok {
  color: var(--accent, #3a7bd5);
}
.messaging-status.err {
  color: #d05353;
}
</style>

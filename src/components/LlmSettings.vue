<script setup lang="ts">
import { onMounted, ref } from 'vue'
import type { Bridge, LlmConfigInfo } from '../lib/bridge'

const props = defineProps<{ bridge: Bridge }>()
const emit = defineEmits<{ close: [] }>()

const info = ref<LlmConfigInfo | null>(null)
const loadError = ref('')

// 表单值：api_key 特殊——打开时为空（不回传明文），留空保存 = 不改
const baseUrl = ref('')
const apiKey = ref('')
const model = ref('')
const apiKeyTouched = ref(false)

const testing = ref(false)
const saving = ref(false)
const testMsg = ref('')
const testOk = ref(false)
const saveMsg = ref('')
const saveOk = ref(false)

function fillForm(v: LlmConfigInfo) {
  baseUrl.value = v.baseUrl
  model.value = v.model
  apiKey.value = ''
  apiKeyTouched.value = false
}

function onApiKeyInput() {
  apiKeyTouched.value = true
}

onMounted(async () => {
  try {
    info.value = await props.bridge.getLlmConfig()
    fillForm(info.value)
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e)
  }
})

async function onTest() {
  if (testing.value || !baseUrl.value.trim()) return
  testing.value = true
  testMsg.value = ''
  try {
    // 未重新输入 key 时用已保存的（后端打码无法回传，桌面端语义：
    // 空串 = 不带鉴权测试；已保存 key 的场景建议重输后测试）
    testMsg.value = await props.bridge.testLlmConnection(
      baseUrl.value.trim(),
      apiKeyTouched.value ? apiKey.value.trim() : '',
      model.value.trim(),
    )
    testOk.value = true
  } catch (e) {
    testMsg.value = e instanceof Error ? e.message : String(e)
    testOk.value = false
  } finally {
    testing.value = false
  }
}

async function onSave() {
  if (saving.value) return
  saving.value = true
  saveMsg.value = ''
  try {
    // api_key 未动 = null（后端保留现值）；动了 = 新值（空串=清除）
    info.value = await props.bridge.setLlmConfig(
      baseUrl.value.trim(),
      apiKeyTouched.value ? apiKey.value.trim() : null,
      model.value.trim(),
    )
    fillForm(info.value)
    saveMsg.value = '已保存（写入用户层，桌面与 CLI 共用）'
    saveOk.value = true
  } catch (e) {
    saveMsg.value = e instanceof Error ? e.message : String(e)
    saveOk.value = false
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal">
      <header class="modal-head">
        <h3>⚙ 模型设置</h3>
        <button class="modal-close" @click="emit('close')">×</button>
      </header>

      <p v-if="loadError" class="error">{{ loadError }}</p>

      <template v-else>
        <p class="llm-hint">
          OpenAI 兼容端点（云端 API 或本地 Ollama / llama.cpp）。保存在
          <code>{{ info?.userConfigPath ?? '~/.elwright/config.json' }}</code
          >，桌面应用与 CLI（ew config）共用。
        </p>

        <label class="llm-field">
          <span>base_url <em v-if="info?.source[0]" class="src">来源：{{ info.source[0] }}</em></span>
          <input v-model="baseUrl" class="search" placeholder="如 https://api.xxx.com/v1 或 http://localhost:11434/v1" />
        </label>

        <label class="llm-field">
          <span>model <em v-if="info?.source[2]" class="src">来源：{{ info.source[2] }}</em></span>
          <input v-model="model" class="search" placeholder="如 gpt-4o-mini / qwen3:8b" />
        </label>

        <label class="llm-field">
          <span>
            api_key <em v-if="info?.source[1]" class="src">来源：{{ info.source[1] }}</em>
            <em v-if="info?.apiKeyMasked" class="src">已存：{{ info.apiKeyMasked }}</em>
          </span>
          <input
            v-model="apiKey"
            type="password"
            class="search"
            placeholder="留空 = 不修改已保存的 key"
            @input="onApiKeyInput"
          />
        </label>

        <div class="llm-actions">
          <button :disabled="testing || !baseUrl.trim()" @click="onTest">
            {{ testing ? '测试中…' : '测试连接' }}
          </button>
          <button class="primary" :disabled="saving" @click="onSave">
            {{ saving ? '保存中…' : '保存' }}
          </button>
        </div>

        <p v-if="testMsg" :class="testOk ? 'op-ok-inline' : 'error'">{{ testMsg }}</p>
        <p v-if="saveMsg" :class="saveOk ? 'op-ok-inline' : 'error'">{{ saveMsg }}</p>
      </template>
    </div>
  </div>
</template>

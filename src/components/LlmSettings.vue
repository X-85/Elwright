<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import type { Bridge, LlmConfigInfo, LlmProfileMeta } from '../lib/bridge'
import { validateProfileName as validateProfileNameShared } from '../lib/profileName'
import { t } from '../lib/i18n'

const props = withDefaults(defineProps<{ bridge: Bridge; embedded?: boolean }>(), { embedded: false })
const emit = defineEmits<{ close: []; saved: [] }>()

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

// Q19 模型档案
const profiles = ref<LlmProfileMeta[]>([])
const activeProfile = ref<string | null>(null)
const profileLoadError = ref('')
const profileActionBusy = ref(false)
const profileActionMsg = ref('')
const profileActionOk = ref(true)
// 新建档案对话框
const showAddProfile = ref(false)
const newProfileName = ref('')
const newProfileNameError = ref('')

// "__flat__" 哨兵表示当前未走档案（用 flat 字段）
const SELECT_FLAT = '__flat__'
const selectedProfile = ref<string>(SELECT_FLAT)
const activeIsFlat = computed(() => selectedProfile.value === SELECT_FLAT)
const currentProfileName = computed(() =>
  activeIsFlat.value ? null : selectedProfile.value,
)

function fillForm(v: LlmConfigInfo) {
  baseUrl.value = v.baseUrl
  model.value = v.model
  apiKey.value = ''
  apiKeyTouched.value = false
}

function onApiKeyInput() {
  apiKeyTouched.value = true
}

async function loadProfiles() {
  profileLoadError.value = ''
  try {
    profiles.value = await props.bridge.listLlmProfiles()
    activeProfile.value = await props.bridge.getActiveLlmProfile()
    selectedProfile.value = activeProfile.value ?? SELECT_FLAT
  } catch (e) {
    profileLoadError.value = e instanceof Error ? e.message : String(e)
    profiles.value = []
    activeProfile.value = null
    selectedProfile.value = SELECT_FLAT
  }
}

onMounted(async () => {
  try {
    info.value = await props.bridge.getLlmConfig()
    fillForm(info.value)
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e)
  }
  await loadProfiles()
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
    saveMsg.value = t('llm.saved')
    saveOk.value = true
    emit('saved')
    // 独立弹层保存后自动关闭；嵌入设置中心时保留当前页面。
    if (!props.embedded) emit('close')
  } catch (e) {
    saveMsg.value = e instanceof Error ? e.message : String(e)
    saveOk.value = false
  } finally {
    saving.value = false
  }
}

// Q19 档案切换：保存当前表单为 flat → 调用 setActive → 重新拉生效配置 + 档案列表
async function onProfileSwitch() {
  if (profileActionBusy.value) return
  profileActionBusy.value = true
  profileActionMsg.value = ''
  try {
    // 先把当前字段值保存到用户层（flat 字段同步更新，再切换激活）
    if (
      baseUrl.value.trim() ||
      model.value.trim() ||
      (apiKeyTouched.value && apiKey.value)
    ) {
      info.value = await props.bridge.setLlmConfig(
        baseUrl.value.trim(),
        apiKeyTouched.value ? apiKey.value.trim() : null,
        model.value.trim(),
      )
      fillForm(info.value)
    }
    if (!activeIsFlat.value) {
      await props.bridge.setActiveLlmProfile(currentProfileName.value!)
      profileActionMsg.value = t('llm.switchedTo').replace('{name}', currentProfileName.value!)
    } else {
      // 切回 flat：保存空 active 即可（后端提供 set_active 校验 name 存在；
      // 此处 UI 不允许切回 flat 时单独调 set_active，统一由下次 save profile 时维护）
      profileActionMsg.value = t('llm.usingFlat')
    }
    profileActionOk.value = true
    await loadProfiles()
  } catch (e) {
    profileActionMsg.value = e instanceof Error ? e.message : String(e)
    profileActionOk.value = false
  } finally {
    profileActionBusy.value = false
  }
}

function validateProfileName(name: string): string {
  return validateProfileNameShared(
    name,
    profiles.value.map((p) => p.name),
  )
}

function openAddProfile() {
  newProfileName.value = ''
  newProfileNameError.value = ''
  showAddProfile.value = true
}

async function onConfirmAddProfile() {
  const err = validateProfileName(newProfileName.value)
  if (err) {
    newProfileNameError.value = err
    return
  }
  profileActionBusy.value = true
  profileActionMsg.value = ''
  try {
    // 先把当前字段同步到用户层
    if (
      baseUrl.value.trim() ||
      model.value.trim() ||
      (apiKeyTouched.value && apiKey.value)
    ) {
      info.value = await props.bridge.setLlmConfig(
        baseUrl.value.trim(),
        apiKeyTouched.value ? apiKey.value.trim() : null,
        model.value.trim(),
      )
      fillForm(info.value)
    }
    await props.bridge.saveLlmProfile({
      name: newProfileName.value.trim().toLowerCase(),
      baseUrl: baseUrl.value.trim(),
      apiKey: apiKeyTouched.value ? apiKey.value.trim() : '',
      model: model.value.trim(),
    })
    profileActionMsg.value = t('llm.created').replace('{name}', newProfileName.value.trim().toLowerCase())
    profileActionOk.value = true
    showAddProfile.value = false
    await loadProfiles()
    // 自动选中新档案
    selectedProfile.value = newProfileName.value.trim().toLowerCase()
  } catch (e) {
    profileActionMsg.value = e instanceof Error ? e.message : String(e)
    profileActionOk.value = false
  } finally {
    profileActionBusy.value = false
  }
}

function cancelAddProfile() {
  showAddProfile.value = false
  newProfileNameError.value = ''
}

async function onDeleteProfile(name: string) {
  if (profileActionBusy.value) return
  if (!confirm(t('llm.confirmDelete').replace('{name}', name).replace('{activeNote}', activeProfile.value === name ? '\n' + t('llm.confirmDeleteActive') : ''))) {
    return
  }
  profileActionBusy.value = true
  profileActionMsg.value = ''
  try {
    await props.bridge.deleteLlmProfile(name)
    profileActionMsg.value = t('llm.deleted').replace('{name}', name)
    profileActionOk.value = true
    if (selectedProfile.value === name) selectedProfile.value = SELECT_FLAT
    await loadProfiles()
  } catch (e) {
    profileActionMsg.value = e instanceof Error ? e.message : String(e)
    profileActionOk.value = false
  } finally {
    profileActionBusy.value = false
  }
}
</script>

<template>
  <div :class="{ 'modal-mask': !embedded, 'embedded-settings-form': embedded }" @click.self="!embedded && emit('close')">
    <div :class="{ modal: !embedded }">
      <header v-if="!embedded" class="modal-head">
        <h3>{{ t('llm.title') }}</h3>
        <button class="modal-close" @click="emit('close')">×</button>
      </header>
      <div class="llm-settings-content">
        <p v-if="loadError" class="error">{{ loadError }}</p>

        <template v-else>
          <p class="llm-hint">
            {{ t('llm.hint') }}
            <code>{{ info?.userConfigPath ?? '~/.elwright/config.json' }}</code>
            {{ t('llm.hintTail') }}
          </p>

          <!-- Q19 模型档案 -->
          <div class="profile-bar">
            <label class="profile-select-wrap">
              <span>{{ t('llm.profile') }}</span>
              <select v-model="selectedProfile" :disabled="profileActionBusy" @change="onProfileSwitch">
                <option :value="SELECT_FLAT">{{ t('llm.flatOption') }}</option>
                <option v-for="p in profiles" :key="p.name" :value="p.name">
                  {{ p.active ? '★ ' : '  ' }}{{ p.name }}
                </option>
              </select>
            </label>
            <button class="profile-add-btn" :disabled="profileActionBusy" @click="openAddProfile">+ {{ t('llm.add') }}</button>
          </div>

          <label class="llm-field">
            <span>base_url <em v-if="info?.source[0]" class="src">{{ t('llm.source') }}{{ info.source[0] }}</em></span>
            <input v-model="baseUrl" class="search" :placeholder="t('llm.baseUrlPlaceholder')" />
          </label>

          <label class="llm-field">
            <span>model <em v-if="info?.source[2]" class="src">{{ t('llm.source') }}{{ info.source[2] }}</em></span>
            <input v-model="model" class="search" :placeholder="t('llm.modelPlaceholder')" />
          </label>

          <label class="llm-field">
            <span>
              api_key <em v-if="info?.source[1]" class="src">{{ t('llm.source') }}{{ info.source[1] }}</em>
              <em v-if="info?.apiKeyMasked" class="src">{{ t('llm.stored') }}{{ info.apiKeyMasked }}</em>
            </span>
            <input v-model="apiKey" type="password" class="search" :placeholder="t('llm.apiKeyPlaceholder')" @input="onApiKeyInput" />
          </label>

          <div class="llm-actions">
            <button :disabled="testing || !baseUrl.trim()" @click="onTest">{{ testing ? t('llm.testing') : t('llm.test') }}</button>
            <button class="primary" :disabled="saving" @click="onSave">{{ saving ? t('llm.saving') : t('llm.save') }}</button>
          </div>

          <p v-if="testMsg" :class="testOk ? 'op-ok-inline' : 'error'">{{ testMsg }}</p>
          <p v-if="saveMsg" :class="saveOk ? 'op-ok-inline' : 'error'">{{ saveMsg }}</p>
          <p v-if="profileActionMsg" :class="profileActionOk ? 'op-ok-inline' : 'error'">{{ profileActionMsg }}</p>
          <p v-if="profileLoadError" class="error">{{ profileLoadError }}</p>

          <!-- 已存在的档案清单（含删除按钮） -->
          <details v-if="profiles.length > 0" class="profile-list">
            <summary>{{ t('llm.configured') }}（{{ profiles.length }}）</summary>
            <ul>
              <li v-for="p in profiles" :key="p.name">
                <span :class="{ 'profile-active': p.active }">
                  {{ p.active ? '★ ' : '   ' }}{{ p.name }}
                </span>
                <button
                  class="profile-del-btn"
                  :disabled="profileActionBusy"
                  @click="onDeleteProfile(p.name)"
                >
                  {{ t('llm.delete') }}
                </button>
              </li>
            </ul>
          </details>

          <!-- 新建档案小弹窗（嵌入设置中心时直接渲染，不叠 modal-mask） -->
          <div v-if="showAddProfile" class="add-profile-modal" @click.self="cancelAddProfile">
            <div class="add-profile-card">
              <h4>{{ t('llm.addTitle') }}</h4>
              <p class="hint">{{ t('llm.addHint') }}</p>
              <input
                v-model="newProfileName"
                class="search"
                :placeholder="t('llm.namePlaceholder')"
                @keydown.enter.prevent="onConfirmAddProfile"
                @keydown.escape.prevent="cancelAddProfile"
              />
              <p v-if="newProfileNameError" class="error">{{ newProfileNameError }}</p>
              <div class="add-profile-actions">
                <button :disabled="profileActionBusy" @click="cancelAddProfile">{{ t('llm.cancel') }}</button>
                <button class="primary" :disabled="profileActionBusy" @click="onConfirmAddProfile">{{ t('llm.saveProfile') }}</button>
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

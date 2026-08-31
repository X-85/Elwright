<script setup lang="ts">
/**
 * 代码浏览器阶段④：受控补丁编辑（ADR-001）—— 三栏预览对话框。
 *
 * 流程：
 * 1. 调用 applyPatchPreview 解析 + 预览（不写文件）
 * 2. 用户逐 hunk 接受/拒绝；按文件勾「整体拒收」
 * 3. 点击「应用」调 applyPatchApply 落盘 + 落快照
 * 4. 成功后展示 snapshotId + 「撤销」入口；失败显示 warnings
 */
import { computed, ref, watch } from 'vue'
import type { Bridge } from '../lib/bridge'
import { renderDiffLines } from '../lib/patch'

const props = defineProps<{
  bridge: Bridge
  projectRoot: string
  patchText: string
}>()

const emit = defineEmits<{ closed: [] }>()

type PreviewFile = {
  file: string
  currentContent: string
  newContent: string
  hunks: unknown[]
  rejected: boolean
}
type Preview = { files: PreviewFile[]; warnings: string[] }

const preview = ref<Preview | null>(null)
const applied = ref<{ snapshotId: string; applied: string[] } | null>(null)
const errMsg = ref<string | null>(null)
const busy = ref(false)

async function runPreview() {
  busy.value = true
  errMsg.value = null
  try {
    const r = (await props.bridge.applyPatchPreview(
      props.projectRoot,
      props.patchText,
    )) as Preview
    preview.value = r
  } catch (e: unknown) {
    errMsg.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = false
  }
}

watch(
  () => [props.projectRoot, props.patchText],
  () => {
    preview.value = null
    applied.value = null
    if (props.patchText) runPreview()
  },
  { immediate: true },
)

function toggleFile(idx: number) {
  if (!preview.value) return
  preview.value.files[idx].rejected = !preview.value.files[idx].rejected
}

async function applyNow() {
  if (!preview.value) return
  busy.value = true
  errMsg.value = null
  try {
    const r = (await props.bridge.applyPatchApply(
      props.projectRoot,
      preview.value.files,
    )) as { snapshot_id: string; applied: string[]; skipped: string[] }
    applied.value = { snapshotId: r.snapshot_id, applied: r.applied }
  } catch (e: unknown) {
    errMsg.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = false
  }
}

async function revertNow() {
  if (!applied.value) return
  busy.value = true
  errMsg.value = null
  try {
    await props.bridge.applyPatchRevert(props.projectRoot, applied.value.snapshotId)
    applied.value = null
    preview.value = null
    emit('closed')
  } catch (e: unknown) {
    errMsg.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = false
  }
}

const currentLines = computed(() => {
  if (!preview.value) return [] as { file: string; lines: string[] }[]
  return preview.value.files.map((f) => ({
    file: f.file,
    lines: f.currentContent.split('\n'),
  }))
})
const newLines = computed(() => {
  if (!preview.value) return [] as { file: string; lines: string[] }[]
  return preview.value.files.map((f) => ({
    file: f.file,
    lines: f.newContent.split('\n'),
  }))
})
const diffLines = computed(() => {
  if (!preview.value) return [] as { file: string; lines: ReturnType<typeof renderDiffLines> }[]
  return preview.value.files.map((f) => ({
    file: f.file,
    lines: renderDiffLines(f.hunks.length ? 'placeholder' : ''), // 真实 diff 在后端 newContent；这里仅保留扩展位
  }))
})
</script>

<template>
  <div class="patch-dialog-mask" @click.self="emit('closed')">
    <div class="patch-dialog" role="dialog" aria-label="补丁预览">
      <header>
        <strong>补丁预览（受控写入）</strong>
        <button class="close" @click="emit('closed')">×</button>
      </header>
      <p class="hint">
        应用前请逐文件确认；拒绝的文件不会被写入。快照 ID 可在下方撤销。
      </p>
      <div v-if="errMsg" class="patch-error">{{ errMsg }}</div>
      <div v-if="!preview && !errMsg" class="patch-loading">解析中…</div>
      <div v-else-if="preview">
        <div v-if="preview.warnings.length" class="patch-warn">
          <strong>提示：</strong>
          <ul>
            <li v-for="(w, i) in preview.warnings" :key="i">{{ w }}</li>
          </ul>
        </div>
        <div v-if="!preview.files.length" class="patch-empty">
          没有可应用的文件（全部被拒收或解析为空）。
        </div>
        <div v-else class="patch-files">
          <div
            v-for="(f, i) in preview.files"
            :key="f.file"
            class="patch-file"
            :class="{ rejected: f.rejected }"
          >
            <div class="patch-file-head">
              <label>
                <input
                  type="checkbox"
                  :checked="!f.rejected"
                  @change="toggleFile(i)"
                />
                <code>{{ f.file }}</code>
              </label>
            </div>
            <div class="patch-three-col">
              <pre class="col col-current"><span class="col-label">当前</span>{{ currentLines[i]?.lines?.join('\n') }}</pre>
              <pre class="col col-diff"><span class="col-label">差异</span><span v-if="f.hunks.length">{{ (f.hunks.length) }} hunk(s)</span></pre>
              <pre class="col col-new"><span class="col-label">新文件</span>{{ newLines[i]?.lines?.join('\n') }}</pre>
            </div>
          </div>
        </div>
      </div>

      <footer v-if="preview">
        <div v-if="!applied" class="apply-row">
          <button
            class="primary"
            :disabled="busy || !preview.files.length || preview.files.every((f) => f.rejected)"
            @click="applyNow"
          >
            {{ busy ? '写入中…' : '应用所选' }}
          </button>
          <span class="muted">仅写入未拒收的文件；写入前会做快照以便撤销。</span>
        </div>
        <div v-else class="revert-row">
          <span class="ok">
            ✓ 已写入 {{ applied.applied.length }} 个文件（快照 {{ applied.snapshotId }}）
          </span>
          <button class="danger" :disabled="busy" @click="revertNow">撤销</button>
          <button @click="emit('closed')">关闭</button>
        </div>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.patch-dialog-mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.patch-dialog {
  background: var(--panel, #1e1e22);
  color: inherit;
  border-radius: 8px;
  padding: 16px;
  width: min(1100px, 96vw);
  max-height: 90vh;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.patch-dialog header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.patch-dialog header .close {
  background: none;
  border: none;
  color: inherit;
  font-size: 20px;
  cursor: pointer;
}
.patch-dialog .hint {
  margin: 0;
  opacity: 0.8;
  font-size: 13px;
}
.patch-loading,
.patch-empty {
  padding: 24px;
  text-align: center;
  opacity: 0.7;
}
.patch-error {
  background: rgba(220, 50, 50, 0.15);
  padding: 8px 12px;
  border-radius: 4px;
}
.patch-warn {
  background: rgba(220, 180, 50, 0.12);
  padding: 8px 12px;
  border-radius: 4px;
}
.patch-warn ul {
  margin: 4px 0 0 18px;
}
.patch-file {
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  padding: 8px;
  margin-bottom: 8px;
}
.patch-file.rejected {
  opacity: 0.5;
}
.patch-file-head label {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}
.patch-three-col {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 6px;
  margin-top: 6px;
}
.patch-three-col .col {
  margin: 0;
  padding: 6px;
  background: rgba(0, 0, 0, 0.25);
  border-radius: 4px;
  font-size: 12px;
  max-height: 240px;
  overflow: auto;
  position: relative;
}
.patch-three-col .col-label {
  display: block;
  font-size: 11px;
  opacity: 0.6;
  margin-bottom: 4px;
}
.apply-row,
.revert-row {
  display: flex;
  gap: 8px;
  align-items: center;
}
.muted {
  opacity: 0.6;
  font-size: 12px;
}
.ok {
  color: #6c6;
  flex: 1;
}
.danger {
  background: rgba(220, 50, 50, 0.2);
  color: inherit;
  border: 1px solid rgba(220, 50, 50, 0.4);
}
button {
  background: rgba(255, 255, 255, 0.08);
  color: inherit;
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: 4px;
  padding: 4px 10px;
  cursor: pointer;
}
button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
button.primary {
  background: rgba(80, 160, 220, 0.3);
}
</style>
// 高频研发转换工具（工作台 ADR-001）：纯本地、零依赖，中文报错。

/** JSON 格式化（2 空格缩进）；非法输入抛中文 Error。 */
export function formatJson(input: string): string {
  const parsed = parseJson(input)
  return JSON.stringify(parsed, null, 2)
}

/** JSON 压缩（去空白）。 */
export function minifyJson(input: string): string {
  const parsed = parseJson(input)
  return JSON.stringify(parsed)
}

function parseJson(input: string): unknown {
  const text = input.trim()
  if (!text) throw new Error('输入为空：请先粘贴 JSON 内容')
  try {
    return JSON.parse(text)
  } catch (e) {
    throw new Error(`JSON 解析失败：${e instanceof Error ? e.message : String(e)}`, { cause: e })
  }
}

/** Base64 编码（UTF-8 安全，支持中文）。 */
export function base64Encode(input: string): string {
  const bytes = new TextEncoder().encode(input)
  let binary = ''
  for (const b of bytes) binary += String.fromCharCode(b)
  return btoa(binary)
}

/** Base64 解码（UTF-8 安全）；非法输入抛中文 Error。 */
export function base64Decode(input: string): string {
  const text = input.trim()
  if (!text) throw new Error('输入为空：请先粘贴 Base64 内容')
  try {
    const binary = atob(text.replace(/\s+/g, ''))
    const bytes = Uint8Array.from(binary, (c) => c.charCodeAt(0))
    return new TextDecoder().decode(bytes)
  } catch (e) {
    throw new Error('Base64 解码失败：内容不是合法的 Base64', { cause: e })
  }
}

/** 时间戳 → 本地日期时间；秒/毫秒自动识别（≤ 1e11 视为秒）。 */
export function timestampToDate(input: string): string {
  const text = input.trim()
  if (!text || !/^\d{10,13}$/.test(text)) {
    throw new Error('请输入 10 位（秒）或 13 位（毫秒）数字时间戳')
  }
  const n = Number(text)
  const ms = n <= 1e11 ? n * 1000 : n
  const d = new Date(ms)
  if (Number.isNaN(d.getTime())) throw new Error('时间戳超出可表示范围')
  const pad = (x: number) => String(x).padStart(2, '0')
  return (
    `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ` +
    `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
  )
}

/** 日期时间 → 毫秒时间戳；支持 ISO 与「YYYY-MM-DD HH:mm:ss」（按本地时区）。 */
export function dateToTimestamp(input: string): string {
  const text = input.trim()
  if (!text) throw new Error('输入为空：请先输入日期时间')
  const normalized = text.includes(' ') && !text.includes('T') ? text.replace(' ', 'T') : text
  const d = new Date(normalized)
  if (Number.isNaN(d.getTime())) {
    throw new Error('日期解析失败：支持 ISO（2026-01-01T00:00:00）或「YYYY-MM-DD HH:mm:ss」')
  }
  return String(d.getTime())
}

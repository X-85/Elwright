/**
 * AI 对话阶段③（能力协作）：解析模型提议/用户调用/执行结果的消息标记。
 * 规则（docs/features/chat/behavior.md 能力协作节）：
 * - 模型只能以单独一行「【能力提议】id: <id>」提议，用户确认后由前端走既有路径执行。
 * - 用户主动选择能力时插入「【能力调用】id: <id>」消息，渲染为确认卡片。
 * - 执行结果为 assistant 消息「【能力结果】<名称>（<类型>）」，带「把结果告诉 AI」回灌。
 */

export interface ChatCapabilityProposal {
  id: string
}

/** 提取消息中的能力 id（提议/调用通用）；无则返回 null。 */
export function parseProposalId(content: string): string | null {
  const m = content.match(/【能力(?:提议|调用)】\s*\n?\s*id:\s*([^\s\n]+)/)
  return m ? m[1] : null
}

/** 是否为能力执行结果消息。 */
export function isCapabilityResult(content: string): boolean {
  return content.startsWith('【能力结果】')
}

/** 结果消息的首行（含能力名与类型），其余为正文。 */
export function splitCapabilityResult(content: string): { header: string; body: string } {
  const idx = content.indexOf('\n')
  if (idx === -1) return { header: content, body: '' }
  return { header: content.slice(0, idx), body: content.slice(idx + 1).trim() }
}

/** 结果回灌给 AI 的用户消息（截断超长输出，保护上下文）。 */
export function resultFeedbackMessage(header: string, body: string, maxChars = 2000): string {
  const capped = body.length > maxChars ? body.slice(0, maxChars) + '\n…（超长截断）' : body
  return `【能力执行结果 · 用户已确认】${header}\n${capped}\n请基于以上执行结果继续。`
}

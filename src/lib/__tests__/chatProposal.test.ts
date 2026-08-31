import { describe, expect, it } from 'vitest'

import { isCapabilityResult, parseProposalId, resultFeedbackMessage, splitCapabilityResult } from '../chatProposal'

describe('parseProposalId', () => {
  it('解析提议格式', () => {
    expect(parseProposalId('建议用这个：\n【能力提议】id: text-stats\n很合适')).toBe('text-stats')
  })

  it('解析用户调用格式', () => {
    expect(parseProposalId('【能力调用】\nid: weekly-report')).toBe('weekly-report')
  })

  it('普通消息返回 null', () => {
    expect(parseProposalId('【能力提议】没有 id 的坏格式')).toBeNull()
    expect(parseProposalId('普通回复')).toBeNull()
  })
})

describe('isCapabilityResult / splitCapabilityResult', () => {
  it('识别并拆分结果消息', () => {
    const content = '【能力结果】文本统计（script）\n\n总字符数：42'
    expect(isCapabilityResult(content)).toBe(true)
    const { header, body } = splitCapabilityResult(content)
    expect(header).toBe('【能力结果】文本统计（script）')
    expect(body).toBe('总字符数：42')
  })

  it('普通 assistant 消息不是结果', () => {
    expect(isCapabilityResult('普通回复')).toBe(false)
  })
})

describe('resultFeedbackMessage', () => {
  it('回灌消息带确认标记并截断超长正文', () => {
    const msg = resultFeedbackMessage('【能力结果】x（script）', 'a'.repeat(3000), 2000)
    expect(msg).toContain('【能力执行结果 · 用户已确认】')
    expect(msg).toContain('超长截断')
    expect(msg.length).toBeLessThan(3000)
  })
})

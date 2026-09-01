// lib/mindmap.ts 纯函数树操作测试（与 core::mindmap 同一套不变量：DFS 序、子树连续）

import { describe, expect, it } from 'vitest'
import {
  addChild,
  addSibling,
  depthOf,
  indent,
  isHidden,
  moveVertical,
  outdent,
  removeSubtree,
  type MindNode,
} from '../mindmap'

function n(id: string, parent: string | null): MindNode {
  return { id, text: id, parent, collapsed: false, convertedTodo: false }
}

/** 根(root) → a1, a2(→a2a), b1 */
function fixture(): MindNode[] {
  return [n('root', null), n('a1', 'root'), n('a2', 'root'), n('a2a', 'a2'), n('b1', 'root')]
}

const ids = (nodes: MindNode[]) => nodes.map((x) => x.id)

describe('mindmap tree ops', () => {
  it('addSibling/addChild 保持 DFS 序', () => {
    const nodes = fixture()
    const sib = addSibling(nodes, 'a1', 'a1b')
    expect(sib).not.toBeNull()
    expect(ids(nodes)).toEqual(['root', 'a1', sib!.id, 'a2', 'a2a', 'b1'])
    const child = addChild(nodes, sib!.id, 'a1bx')
    expect(child).not.toBeNull()
    expect(ids(nodes)).toEqual(['root', 'a1', sib!.id, child!.id, 'a2', 'a2a', 'b1'])
    expect(depthOf(nodes, sib!.id)).toBe(1)
    expect(depthOf(nodes, child!.id)).toBe(2)
  })

  it('removeSubtree 级联删除且拒绝根', () => {
    const nodes = fixture()
    expect(removeSubtree(nodes, 'root')).toBe(false)
    expect(removeSubtree(nodes, 'a2')).toBe(true)
    expect(ids(nodes)).toEqual(['root', 'a1', 'b1'])
  })

  it('moveVertical 整块交换', () => {
    const nodes = fixture()
    expect(moveVertical(nodes, 'a2', true)).toBe(true)
    expect(ids(nodes)).toEqual(['root', 'a2', 'a2a', 'a1', 'b1'])
    expect(moveVertical(nodes, 'b1', true)).toBe(true)
    expect(ids(nodes)).toEqual(['root', 'a2', 'a2a', 'b1', 'a1'])
    expect(moveVertical(nodes, 'a1', true)).toBe(true)
    expect(ids(nodes)).toEqual(['root', 'a2', 'a2a', 'a1', 'b1'])
    expect(moveVertical(nodes, 'root', true)).toBe(false)
  })

  it('indent/outdent 子树整体移动', () => {
    const nodes = fixture()
    expect(indent(nodes, 'a1')).toBe(false) // 第一个兄弟无法缩进
    expect(indent(nodes, 'a2')).toBe(true)
    expect(nodes.find((x) => x.id === 'a2')?.parent).toBe('a1')
    expect(depthOf(nodes, 'a2a')).toBe(3)
    expect(outdent(nodes, 'a2')).toBe(true)
    expect(nodes.find((x) => x.id === 'a2')?.parent).toBe('root')
    expect(ids(nodes)).toEqual(['root', 'a1', 'a2', 'a2a', 'b1'])
    expect(outdent(nodes, 'a1')).toBe(false) // 一级节点不能脱离根
  })

  it('isHidden 折叠祖先链', () => {
    const nodes = fixture()
    nodes.find((x) => x.id === 'a2')!.collapsed = true
    expect(isHidden(nodes, 'a2a')).toBe(true)
    expect(isHidden(nodes, 'a2')).toBe(false)
    expect(isHidden(nodes, 'b1')).toBe(false)
  })
})

// 脑图树操作纯函数（前端镜像 core::mindmap 的 DFS 不变量，ADR-001 §D2）。
//
// 约定与 Rust 侧一致：nodes 数组顺序即文档 DFS 序，子树连续；
// 所有操作原地修改并维持该不变量。根节点 parent=null 恒为 nodes[0]，
// 根不可删/不可移动/不可缩进；一级节点不可外提。

import type { MindNode } from './bridge'

export function depthOf(nodes: MindNode[], id: string): number {
  const node = nodes.find((n) => n.id === id)
  if (!node) return 0
  let depth = 0
  let current: MindNode | undefined = node
  while (current?.parent) {
    depth += 1
    if (depth > nodes.length) return depth // 环防御
    current = nodes.find((n) => n.id === current!.parent)
  }
  return depth
}

export function subtreeEnd(nodes: MindNode[], idx: number): number {
  const base = depthOf(nodes, nodes[idx].id)
  let end = idx + 1
  while (end < nodes.length && depthOf(nodes, nodes[end].id) > base) end += 1
  return end
}

function prevSiblingIdx(nodes: MindNode[], idx: number): number {
  const parent = nodes[idx].parent
  for (let i = idx - 1; i >= 0; i -= 1) {
    if (nodes[i].parent === parent) return i
  }
  return -1
}

function nextSiblingIdx(nodes: MindNode[], idx: number): number {
  const end = subtreeEnd(nodes, idx)
  if (end < nodes.length && nodes[end].parent === nodes[idx].parent) return end
  return -1
}

let seq = 0
export function newNodeId(): string {
  seq = (seq + 1) % 0xffff
  return `n${Date.now().toString(16)}${seq.toString(16).padStart(4, '0')}`
}

export function isRoot(nodes: MindNode[], id: string): boolean {
  const node = nodes.find((n) => n.id === id)
  return !node || node.parent === null
}

export function addSibling(nodes: MindNode[], targetId: string, text: string): MindNode | null {
  const idx = nodes.findIndex((n) => n.id === targetId)
  if (idx < 0 || isRoot(nodes, targetId)) return null
  const node: MindNode = {
    id: newNodeId(),
    text,
    parent: nodes[idx].parent,
    collapsed: false,
    convertedTodo: false,
  }
  nodes.splice(subtreeEnd(nodes, idx), 0, node)
  return node
}

export function addChild(nodes: MindNode[], parentId: string, text: string): MindNode | null {
  const idx = nodes.findIndex((n) => n.id === parentId)
  if (idx < 0) return null
  const node: MindNode = {
    id: newNodeId(),
    text,
    parent: parentId,
    collapsed: false,
    convertedTodo: false,
  }
  nodes.splice(subtreeEnd(nodes, idx), 0, node)
  return node
}

export function removeSubtree(nodes: MindNode[], id: string): boolean {
  if (isRoot(nodes, id)) return false
  const idx = nodes.findIndex((n) => n.id === id)
  if (idx < 0) return false
  nodes.splice(idx, subtreeEnd(nodes, idx) - idx)
  return true
}

export function moveVertical(nodes: MindNode[], id: string, up: boolean): boolean {
  if (isRoot(nodes, id)) return false
  const idx = nodes.findIndex((n) => n.id === id)
  if (idx < 0) return false
  const other = up ? prevSiblingIdx(nodes, idx) : nextSiblingIdx(nodes, idx)
  if (other < 0) return false
  const [a, b] = up ? [other, idx] : [idx, other]
  const endA = subtreeEnd(nodes, a)
  const endB = subtreeEnd(nodes, b)
  const blockA = nodes.slice(a, endA)
  const blockB = nodes.slice(endA, endB)
  nodes.splice(a, endB - a, ...blockB, ...blockA)
  return true
}

export function indent(nodes: MindNode[], id: string): boolean {
  if (isRoot(nodes, id)) return false
  const idx = nodes.findIndex((n) => n.id === id)
  if (idx < 0) return false
  const newParentIdx = prevSiblingIdx(nodes, idx)
  if (newParentIdx < 0) return false
  const newParentId = nodes[newParentIdx].id
  const end = subtreeEnd(nodes, idx)
  const block = nodes.splice(idx, end - idx)
  const at = subtreeEnd(nodes, nodes.findIndex((n) => n.id === newParentId))
  block[0] = { ...block[0], parent: newParentId }
  nodes.splice(at, 0, ...block)
  return true
}

export function outdent(nodes: MindNode[], id: string): boolean {
  const idx = nodes.findIndex((n) => n.id === id)
  if (idx < 0) return false
  const parentId = nodes[idx].parent
  if (!parentId) return false // 根不可外提
  const parentIdx = nodes.findIndex((n) => n.id === parentId)
  if (parentIdx < 0) return false
  const grandparent = nodes[parentIdx].parent
  if (grandparent === null) return false // 一级节点外提会脱离根
  const end = subtreeEnd(nodes, idx)
  const block = nodes.splice(idx, end - idx)
  const pIdx = nodes.findIndex((n) => n.id === parentId)
  const at = subtreeEnd(nodes, pIdx)
  block[0] = { ...block[0], parent: grandparent }
  nodes.splice(at, 0, ...block)
  return true
}

/** 折叠状态下某节点是否应被隐藏（任一祖先 collapsed）。 */
export function isHidden(nodes: MindNode[], id: string): boolean {
  let node = nodes.find((n) => n.id === id)
  let guard = 0
  while (node?.parent) {
    guard += 1
    if (guard > nodes.length) return false
    const parent = nodes.find((n) => n.id === node!.parent)
    if (!parent) return false
    if (parent.collapsed) return true
    node = parent
  }
  return false
}

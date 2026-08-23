# 阶段②（会话管理）验证记录

日期：2026-08-23（自动化部分；真机 GUI 验证待用户执行）

## 单元测试（cargo test）

- lib 31 通过（含阶段① llm mock 测试）+ main.rs 二进制 7 通过（chat_store 6 + update_info 1）= **38 通过、0 失败**。
- chat_store 测试覆盖：save/load/list/delete 往返、updated_at 降序排序、id 唯一性与非法字符拒绝、now_iso 字典序可排序、损坏文件在 list/load 中跳过、删除不存在文件幂等。
- 注：chat_store 属 main.rs 二进制 crate，`cargo test --lib` 跑不到，须全量 `cargo test`。

## 前端构建

- `cd src && npm run build` 通过（dist 产出正常，仅 chunk >500kB 提示，与阶段①相同）。

## IAB 浏览器预览（npm run dev :5199）

- 进入「💬 AI 对话」：两栏布局渲染正确——侧栏「会话 + ＋」、空态「暂无会话」；主区保留阶段①头部/提示/输入区。
- 布局度量（evaluate）：`.chat-view` display:flex / direction:row；`.chat-sessions` 宽 220px、右边框 1px；`.chat-main` 宽 ~830px。
- 点「＋」新建：会话不落盘不进列表（设计如此：空会话不持久化），无报错。
- 预览发送「你好」：用户气泡 + 【预览模式】错误气泡 + 重试按钮；侧栏仍「暂无会话」——确认浏览器不模拟持久化（save 静默忽略）。
- 截图存档：`.zcode/cli/artifacts/.../call_03801332-*.png`（本会话模型不看图，以 DOM 度量为准）。
- 备注：此 Vue 应用上 Playwright locator 曾超时（阶段①已知问题），交互走 `dom_cua.get_visible_dom()` + `click({node_id})`；首次 dom_cua 点击无效，reload 后恢复。

## 未验证（留真机）

- 桌面 app 内真实多会话流转：新建→发消息→自动落盘 `~/.elwright/chats/*.json`→切换→重命名→删除→重启后列表恢复。
- 与真实 LLM 端点联调下的会话持久化。

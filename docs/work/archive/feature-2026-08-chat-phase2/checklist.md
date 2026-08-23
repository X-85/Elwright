# 阶段②（会话管理）实现清单

## 存储（chat_store.rs）

- [x] ChatSession / ChatSessionSummary 结构（serde，snake_case JSON）
- [x] `~/.elwright/chats/` 目录（registry::user_root，支持 ELWRIGHT_USER_ROOT 覆盖）
- [x] id 生成：`{ts-hex}-{counter-hex}`（AtomicU64，零依赖）
- [x] now_iso：手写 UTC ISO8601（20 字符，字典序=时间序）
- [x] list_sessions：updated_at 降序、损坏文件跳过
- [x] load_session：缺失/损坏返回 None
- [x] save_session：upsert、保留原 created_at、pretty JSON
- [x] delete_session：幂等
- [x] 单测 6 个（roundtrip / 排序 / id 唯一 / 时间戳字典序 / 损坏跳过 / 删除幂等）

## IPC（main.rs）

- [x] chat_list_sessions / chat_load_session / chat_save_session / chat_delete_session
- [x] 注册进 invoke_handler

## 前端

- [x] bridge.ts：ChatSessionSummary / ChatSession 类型 + 四方法（tauri 字段映射；浏览器 list→[]、load→null、save/delete 静默）
- [x] ChatView.vue：两栏布局（chat-sessions 侧栏 + chat-main）
- [x] 新建（＋）/ 切换（点击加载+滚底）/ 重命名（✎/双击，Enter/Esc/blur）/ 删除（confirm + 回退选择）
- [x] 自动保存：用户发出与回复成功后 persistCurrent；错误占位不落盘
- [x] 标题：首条非错误用户消息 ≤24 字符；「新对话」兜底
- [x] 空会话不落盘（首条消息持久化后才进列表）
- [x] style.css：侧栏 220px、hover/active 态、重命名输入框、✎/× hover 显示

## 文档

- [x] chat feature README / behavior / architecture / changelog 阶段②条目
- [x] ROADMAP 进行中登记；AGENTS 当前进度同步

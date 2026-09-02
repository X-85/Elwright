# 架构

`core::workspace` 负责数据模型、校验和原子持久化。Tauri 命令只负责 IPC 参数与 LLM 调用，Vue 仅通过 `Bridge` 访问数据。

数据模型包含 `folders`、`resources`、`topics` 三个数组。资源通过 `folder_id` 归属文件夹，课题通过资源 id 列表引用资源。`app` 资源额外保存 `icon` 和 `launchArgs`，由桌面壳调用 `Command` 启动（macOS `.app` 使用 `open`）。报告生成使用现有 `llm::ConfigLayers` 和 blocking reqwest 客户端。

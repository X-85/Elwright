# 架构

Topic 台账是文件协议优先的轻量层，不依赖 Elwright 数据库或特定 Agent API。Skill 负责指导 Agent 读写 Markdown；`topic-viewer.html` 使用兼容浏览器的文件选择器读取 Topic 文件夹；Elwright 后续可以把同一协议接入桌面页面，但 MVP 不要求桌面端升级。

文件职责分离：`topic.md` 是当前状态，`events.md` 是过程，`decisions.md` 是稳定结论，`handoff.md` 是换 Chat/Agent 的最小上下文。安装脚本只复制 Skill 包，并在覆盖前备份目标目录。

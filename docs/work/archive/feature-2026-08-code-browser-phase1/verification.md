# 验证记录

## 自动化（2026-08-31）

- core 单测 8/8：路径穿越拒绝、敏感文件保护、语言识别、树排序与 target 跳过、
  文件名/内容搜索、符号扫描（类型/implements/方法、控制流与调用点排除）、
  最近列表提升与持久化上限。
- IPC 冒烟 1/1（mock runtime 真协议）：tree / read / search / scan_symbols、
  穿越在 IPC 层报错。项目根用临时目录，不碰真实数据。
- vitest 31（新增 codeHighlight 5：先转义后着色、恶意输入无裸 HTML、词边界）。
- e2e 10（新增：浏览器预览点「选择项目目录」出只读降级提示）。
- 全量：cargo 55+1+6+4+1 / clippy 0 error / fmt / vitest 31 / e2e 10 / build 全绿。

## 人工验证

- 【待用户】桌面壳（debug .app 或安装包）：选择真实 Java 项目 → 目录树展开 →
  打开 .java 文件看高亮 → 跳转 UserService 找到接口与实现候选 → 添加敏感文件
  （如 .env）确认拒读 → 最近项目/文件重开保留。

## 人工验证（2026-08-31）

- 用户指示跳过真机点验，自动化闸门全绿即合并（PR #2，CI 全绿）；真机点验项留档 PENDING-REAL-MACHINE-CHECKLIST.md B 节，待装 v0.1.8+ 后复验。

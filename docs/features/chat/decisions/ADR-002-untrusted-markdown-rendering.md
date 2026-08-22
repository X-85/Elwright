# ADR-002：模型输出按不可信 Markdown 渲染

## 状态

已接受（2026-08-22，随 chat 阶段① 实现落地并经注入样本验证）

## 决策

AI 对话的模型输出统一经 `src/lib/safeMarkdown.ts` 渲染（专用 `Marked` 实例，覆写三个 renderer）：

1. **`html`**：原始 HTML（块级/行内标签）原样 HTML 转义输出——标签以文本形式可见，不执行、不渲染。
2. **`link` / `image`**：href 仅放行 `http(s):`、`mailto:`、锚点 `#`、相对路径，且值内不允许引号/空白/尖括号；不合规链接降级为纯文本（图片降级为 alt 文本）。
3. **`code`**：覆写默认渲染加复制按钮外壳；代码内容自行转义。

不引入 sanitize 依赖（DOMPurify 等），保持零新依赖。

## 原因

- AGENTS.md 前端约定：marked 直出 `v-html` 仅限 `resources/` 可信本地文件；模型输出是不可信来源，必须另行收敛后再进 `v-html`。
- 立项时的候选方案「先转义 `<` 再走 marked」被否决：代码块内含 `<`（HTML/Java 泛型等）会显示为 `&lt;`，对开发工具不可接受。覆写 `html` renderer 只拦截原始 HTML 标签，代码块走 marked 默认转义，保真且安全面等价。
- 手写 DOM 后处理 sanitizer（DOMParser 删 script/on* 属性）也被否决：自研 sanitizer 存在已知 mXSS 绕过类别（svg/math/template 命名空间重解析差异），收益不比 renderer 覆写高。

## 验证

`renderChatMarkdown` 注入样本实测（2026-08-22）：`<script>`、`<img onerror>`、`javascript:` / `data:` 协议链接、href 带引号逃逸均被拦截；标题/链接/代码块/复制按钮正常。

## 影响

- 该 Marked 实例仅供对话页使用；CapabilityDetail 渲染可信本地文件仍走默认 marked，不受影响。
- 未来若渲染其他不可信来源（如远端注册表文档），复用 `renderChatMarkdown` 而不是默认 marked。

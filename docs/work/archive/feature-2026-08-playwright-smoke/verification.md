# Verification

## 自动化

- `npm ci`：通过，验证 CI 的干净依赖安装。
- `npx playwright install chromium`：通过。
- `npm run test:e2e`：通过（Chromium 3/3）。
- `npm test -- --run`：通过（22 tests）。
- `npm run build`：通过。
- `cargo test -q`：通过（41 tests）。
- `cargo fmt --check`：通过。
- CI YAML 解析：通过（Ruby `YAML.load_file`）。

## 安全边界

- 测试不读取或写入真实收藏文件、`~/.elwright/` 或项目业务资源。
- 测试只对浏览器上下文的 `localStorage` 写入虚拟工作区数据，资源值使用 `virtual://elwright-e2e/...` URI。
- 真实文件选择器、软件启动和终端 PTY 不在本阶段通过浏览器冒烟验证。

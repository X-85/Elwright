# 检查清单

- [x] ci.yml 新增 msi job（windows-latest，bash shell 构建，upload-artifact Elwright-windows-x64）
- [x] 更新 dmg job 上方过时注释（原「等公司机装 MSVC」）
- [x] AGENTS.md：CI 描述 + 待办行同步
- [x] 本地 YAML 校验（ruby yaml 安全加载）
- [x] CI 全绿（六 job，run 32538584086 / f908a4b）
- [x] Elwright-windows-x64 artifact 产出（7.4MB zip，含 Elwright_0.1.0_x64_en-US.msi）
- [ ] Windows 机器实际安装验证（用户在公司机确认）

## 排障过程（三跑）

1. a912d7b 首跑失败：tauri build 步骤挂，无日志（日志 API 403、网页超时）。
2. 6500441 加二分（cargo build --release 独立步骤）+ 失败时把日志尾部发为 ::error annotation（annotation API 公开可读）；确认 release 编译成功、失败在 WiX 打包段，真实错误 `failed to run light.exe`。
3. fa2ff26 尝试中文文件名改 ASCII（resources/docs/公司家里项目共建.md → company-home-collab.md）——未解决，但作为加固保留。
4. f908a4b 删除 resources/{tools,docs}/.gitkeep（重名 basename 在 msi 打包冲突）——**解决，全绿**。


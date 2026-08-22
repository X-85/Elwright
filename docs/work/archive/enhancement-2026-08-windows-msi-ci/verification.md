# 验证记录

## 结论

CI 六 job 全绿（run 32538584086，commit f908a4b）；`Elwright-windows-x64` artifact 产出 7.4MB zip，内含 `Elwright_0.1.0_x64_en-US.msi`。Windows 实机安装由用户在公司机确认后归档。

## 根因与修复

- 现象：`tauri build --bundles msi` 在 windows-latest 打包段失败，真实错误为 WiX `light.exe` 运行失败（candle 编译 main.wxs 成功）。
- 根因：`resources/tools/.gitkeep` 与 `resources/docs/.gitkeep` 重名 basename 在 msi 打包布局中冲突。两文件早已失去占位作用（目录均有真实内容），删除即解决。macOS dmg 打包（zip）对重名静默容错，故同一配置在 mac 上一直成功，掩盖了该问题。
- 顺带加固：`resources/docs/公司家里项目共建.md` 改名 `company-home-collab.md`（注册表 `doc` 字段同步；非根因，但资源文件名保持 ASCII 可避免 WiX ANSI 代码页类问题）。`ew view elwright-guide` 验证改名后仍可读。

## 无日志环境的诊断通道（可复用）

仓库公开但日志 API 需认证（403），本机直连 github.com 也不稳定。可用通道：

1. check-run annotations API 公开可读：失败步骤用 `::error::<行>` 把日志尾部转成 annotation，即可无 token 读到错误。
2. 把长步骤二分（cargo build --release 独立成步），用步骤结论定位失败段。
3. 步骤耗时（jobs API）可辅助判断：392s 失败 = 编译后段；92s 失败 = 打包段。

这些诊断步骤保留在 ci.yml（bisect 步骤顺带预热缓存，不增加总时长）。

## 验证明细

- YAML 本地校验：ruby `YAML.safe_load` 通过（每次改 workflow 后跑）。
- 三轮 CI（a912d7b 败 → 6500441 败但产出诊断信息 → fa2ff26 败 → f908a4b 全绿），失败轮的 bisect 与 annotation 均按预期工作。
- artifact 列表 API：两个制品均产出，msi 上传步骤 glob `bundle/msi/*.msi` 且 `if-no-files-found: error` 通过。

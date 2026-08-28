# 渐进式能力成长

Elwright 的能力可以按成熟度档位渐进式展示。MVP 位于能力注册表和桌面前端：核心档位默认可用，进阶能力可被用户主动查看；使用记录只保存在本机。

实现入口：`src-tauri/src/core/registry.rs`、`src/App.vue`、`src/components/CapabilityList.vue` 和 `src/components/CapabilityDetail.vue`。

后续阶段再建设完整解锁规则、成长提示和社区能力包审核/签名分发。

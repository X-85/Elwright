# ADR-001: LLM 客户端使用 reqwest blocking 而非 async

## 状态

已接受。

## 背景

架构方案 §9.2 锁定「自写 reqwest thin client，不引 async-openai」；§12.3 的待办写的是 `cargo add reqwest tokio`。reqwest 有 blocking 和 async 两种用法，需要定一个。

## 决策

使用 `reqwest` 的 **blocking** 特性（`--features blocking,json`），不显式依赖 tokio。

## 后果

- 优点：CLI（`ew`）是纯同步程序，blocking 免除 async/await 污染整个调用链，代码量最小，符合 pi-mono 极简哲学；reqwest blocking 内部自带运行时，功能无损。
- 缺点：阶段 3 桌面壳若需要并发调用或多请求场景，需评估切换 async 版本（同一 crate，迁移成本低）。
- 缓解：`chat()` 已收敛在 `LlmClient` 单一方法内，切换实现不影响调用方。

## 备选方案

- reqwest async + tokio：为阶段 3 预留，但当前无并发需求，属于提前加复杂度。
- async-openai crate：已在架构层面否决（结构耦合，只用一个端点）。

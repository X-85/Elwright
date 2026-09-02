//! ADR-004：对话长上下文——core 侧字符预算滑动窗口。
//!
//! `chat_completion` / `chat_completion_stream` 共用：system 提示由调用方固定前置，
//! 本模块只裁剪 user/assistant 历史。预算按消息 content 字符数计
//! （近似 token 量级，零依赖、纯本地，符合「LLM 可选、离线可用」主干红线）。

use crate::core::llm::ChatMessage;

/// 默认上下文预算（字符）。LLM 配置链 `contextBudgetChars` 可覆盖。
pub const DEFAULT_BUDGET_CHARS: usize = 24_000;

const TRUNCATE_MARKER: &str = "\n…（超长截断）\n";

fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// 中段截断：保留头尾各约一半预算，中间以标记替代（文件头/日志尾等两侧都有用）。
fn mid_truncate(content: &str, budget: usize) -> String {
    let total = char_len(content);
    if total <= budget {
        return content.to_string();
    }
    let half = budget.saturating_sub(char_len(TRUNCATE_MARKER)) / 2;
    let head: String = content.chars().take(half).collect();
    let tail: String = content.chars().skip(total - half).collect();
    format!("{head}{TRUNCATE_MARKER}{tail}")
}

/// 裁剪历史消息（ADR-004）：
/// - 最新一条（对话流末尾，正常为本次 user 提问）必留；超预算则中段截断；
/// - 更早的消息从新到旧整条保留，放不下的整条丢弃（半条上下文易误导模型）；
/// - 总长未超预算时原样返回，不做任何改动。
///
/// 返回 (裁剪后的历史, 是否发生裁剪)。
pub fn fit_messages(history: &[ChatMessage], budget: usize) -> (Vec<ChatMessage>, bool) {
    if history.is_empty() {
        return (Vec::new(), false);
    }
    let total: usize = history.iter().map(|m| char_len(&m.content)).sum();
    if total <= budget {
        return (history.to_vec(), false);
    }

    let last = history.len() - 1;
    let last_len = char_len(&history[last].content);
    let last_content = if last_len > budget {
        mid_truncate(&history[last].content, budget)
    } else {
        history[last].content.clone()
    };
    let mut used = last_len.min(budget);

    let mut keep = vec![false; last];
    for i in (0..last).rev() {
        let c = char_len(&history[i].content);
        if used + c <= budget {
            keep[i] = true;
            used += c;
        }
    }

    let mut kept: Vec<ChatMessage> = Vec::with_capacity(history.len());
    for (i, m) in history.iter().enumerate() {
        if i == last {
            let mut m = m.clone();
            m.content = last_content.clone();
            kept.push(m);
        } else if keep[i] {
            kept.push(m.clone());
        }
    }
    (kept, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(role: &str, content: &str) -> ChatMessage {
        ChatMessage::new(role, content)
    }

    #[test]
    fn within_budget_untouched() {
        let history = vec![
            msg("user", "你好"),
            msg("assistant", "你好！有什么可以帮你？"),
        ];
        let (out, trimmed) = fit_messages(&history, 10_000);
        assert!(!trimmed);
        assert_eq!(out, history);
    }

    #[test]
    fn empty_history_noop() {
        let (out, trimmed) = fit_messages(&[], 100);
        assert!(out.is_empty());
        assert!(!trimmed);
    }

    #[test]
    fn oldest_dropped_first_order_preserved() {
        let big = "x".repeat(800);
        let history = vec![
            msg("user", &format!("u0-{big}")),
            msg("assistant", &format!("a0-{big}")),
            msg("user", &format!("u1-{big}")),
            msg("assistant", &format!("a1-{big}")),
            msg("user", "最终问题"),
        ];
        let (out, trimmed) = fit_messages(&history, 2_000);
        assert!(trimmed);
        // 最终问题必留；更早的从最旧开始丢
        assert_eq!(out.last().unwrap().content, "最终问题");
        assert!(!out.iter().any(|m| m.content.starts_with("u0-")));
        assert!(!out.iter().any(|m| m.content.starts_with("a0-")));
        // 顺序与原始相对顺序一致
        let roles: Vec<&str> = out.iter().map(|m| m.role.as_str()).collect();
        assert_eq!(roles, ["user", "assistant", "user"]);
        // 预算不超
        let total: usize = out.iter().map(|m| char_len(&m.content)).sum();
        assert!(total <= 2_000, "total={total}");
    }

    #[test]
    fn latest_user_over_budget_mid_truncated() {
        let huge = format!("HEAD-{}-TAIL", "m".repeat(5_000));
        let history = vec![msg("user", &huge)];
        let (out, trimmed) = fit_messages(&history, 1_000);
        assert!(trimmed);
        assert_eq!(out.len(), 1);
        let content = &out[0].content;
        assert!(content.starts_with("HEAD-"), "应保留头部");
        assert!(content.ends_with("-TAIL"), "应保留尾部");
        assert!(content.contains("（超长截断）"));
        assert!(char_len(content) <= 1_000 + char_len(TRUNCATE_MARKER));
    }

    #[test]
    fn tiny_budget_keeps_only_trimmed_last() {
        let history = vec![
            msg("user", "旧问题"),
            msg("assistant", "旧回答内容不短"),
            msg("user", "新问题内容也不短"),
        ];
        let (out, trimmed) = fit_messages(&history, 5);
        assert!(trimmed);
        assert_eq!(out.len(), 1);
        assert!(out[0].content.contains("（超长截断）"));
    }

    #[test]
    fn assistant_last_still_kept_as_latest() {
        // 健壮性：末尾非 user（异常流）时同样按「最新必留」处理
        let history = vec![msg("user", "u"), msg("assistant", &"a".repeat(100))];
        let (out, trimmed) = fit_messages(&history, 10);
        assert!(trimmed);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].role, "assistant");
    }
}

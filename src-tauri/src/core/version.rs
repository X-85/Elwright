//! 版本号比较：桌面壳「检查更新」用，CLI 侧将来也可复用。
//! 纯函数、零依赖，语义按本项目发版习惯（`v0.1.0` tag ↔ `0.1.0` 三段式）从宽处理。

/// 去掉可选的 `v`/`V` 前缀
pub fn normalize(version: &str) -> &str {
    let v = version.trim();
    match v.strip_prefix(['v', 'V']) {
        Some(rest) => rest,
        None => v,
    }
}

/// latest 是否比 current 新。逐段按数值比较；段内非数字部分（如 `1-beta`
/// 的 `-beta`、`0.1.0rc1` 的 `rc1`）取前导数字、无数字视为 0；缺段视为 0。
pub fn is_newer(latest: &str, current: &str) -> bool {
    let l = segments(latest);
    let c = segments(current);
    for i in 0..l.len().max(c.len()) {
        let a = l.get(i).copied().unwrap_or(0);
        let b = c.get(i).copied().unwrap_or(0);
        if a != b {
            return a > b;
        }
    }
    false
}

fn segments(version: &str) -> Vec<u64> {
    normalize(version)
        .split(['.', '-', '+'])
        .map(|part| {
            let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse().unwrap_or(0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_v_prefix() {
        assert_eq!(normalize("v0.1.0"), "0.1.0");
        assert_eq!(normalize("0.1.0"), "0.1.0");
        assert_eq!(normalize(" V1.2 "), "1.2");
    }

    #[test]
    fn detects_patch_minor_major() {
        assert!(is_newer("v0.1.1", "0.1.0"));
        assert!(is_newer("0.2.0", "v0.1.9"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(!is_newer("0.1.0", "v0.1.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
    }

    #[test]
    fn numeric_not_lexicographic() {
        assert!(is_newer("0.10.0", "0.9.0"));
        assert!(!is_newer("0.9.0", "0.10.0"));
    }

    #[test]
    fn tolerates_short_and_prerelease_forms() {
        assert!(!is_newer("0.1", "0.1.0"));
        assert!(is_newer("0.1.1", "0.1"));
        // 预发布后缀取前导数字，视为同级
        assert!(!is_newer("0.2.0-beta", "0.2.0"));
        assert!(is_newer("0.2.1-rc1", "0.2.0"));
    }
}

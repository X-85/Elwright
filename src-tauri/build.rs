fn main() {
    tauri_build::build();
    embed_manifest_for_windows_tests();
}

/// Windows + MSVC 下给**测试二进制**嵌入 Common-Controls v6 manifest。
///
/// 不嵌入时，tauri::test 的 mock runtime 测试二进制在启动即报
/// STATUS_ENTRYPOINT_NOT_FOUND（0xc0000139）——tauri-apps/discussions#11179、
/// tauri#11028。tauri_build::build() 的 manifest 只作用于 bin target
/// （rustc-link-arg-bins），测试二进制拿不到，需自行补：
/// `rustc-link-arg-tests` 只影响 tests，不影响正式产物。
fn embed_manifest_for_windows_tests() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os == "windows" && target_env == "msvc" {
        let manifest =
            std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default())
                .join("windows-app-manifest.xml");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
}

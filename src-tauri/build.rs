fn main() {
    tauri_build::build();
    embed_manifest_for_windows_tests();
}

/// Windows 下给**测试二进制**嵌入 Common-Controls v6 manifest。
///
/// 不嵌入时，tauri::test 的 mock runtime 测试二进制在启动即报
/// STATUS_ENTRYPOINT_NOT_FOUND（0xc0000139）——tauri-apps/discussions#11179、
/// tauri#11028。tauri_build::build() 的 manifest 只作用于 bin target
/// （rustc-link-arg-bins），测试二进制拿不到，需自行补：
/// `rustc-link-arg-tests` 只影响 tests，不影响正式产物。
///
/// 链接器分语法：MSVC 用 /MANIFEST:EMBED + /MANIFESTINPUT；GNU（本机
/// 唯一工具链，CI-only MSVC）用 windres 预编译 .res 再交给 ld。
fn embed_manifest_for_windows_tests() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os != "windows" {
        return;
    }
    let manifest = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default())
        .join("windows-app-manifest.xml");
    println!("cargo:rerun-if-changed={}", manifest.display());
    match target_env.as_str() {
        "msvc" => {
            println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
            println!(
                "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",
                manifest.display()
            );
        }
        "gnu" => {
            let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
            let res = std::path::Path::new(&out_dir).join("app-manifest-tests.res");
            let rc = std::path::Path::new(&out_dir).join("app-manifest-tests.rc");
            let Ok(_) = std::fs::read(&manifest) else {
                println!(
                    "cargo:warning=windows-app-manifest.xml 不可读，跳过 GNU 测试 manifest 嵌入"
                );
                return;
            };
            std::fs::write(
                &rc,
                format!(
                    "#define RT_MANIFEST 24\n1 RT_MANIFEST \"{}\"",
                    manifest.display().to_string().replace('\\', "/")
                ),
            )
            .expect("写 rc 失败");
            // windres 在 MinGW 工具链 bin 目录（如 D:\mingw64\bin），随 PATH 命中；
            // 找不到时不阻塞构建，只警告（本地 GNU 主要编 ew/核心，测试可退回 CI）。
            let ok = std::process::Command::new("windres")
                .args(["--input"])
                .arg(&rc)
                .args(["--output"])
                .arg(&res)
                .args(["--output-format=coff"])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                println!("cargo:rustc-link-arg-tests={}", res.display());
            } else {
                println!("cargo:warning=windres 不可用，GNU 测试二进制未嵌 manifest（tauri mock runtime 测试会 ENTRYPOINT_NOT_FOUND）");
            }
        }
        _ => {}
    }
}

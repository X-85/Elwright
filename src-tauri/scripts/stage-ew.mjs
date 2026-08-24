// Staging script for the `ew` CLI sidecar.
//
// Runs as part of Tauri's `beforeBuildCommand` so that `ew.exe` is compiled and
// placed where `bundle.externalBin` expects it (binaries/ew-<triple>.exe) at
// build time. This keeps the packaging entirely local to the build step, so no
// CI workflow file needs to be touched.
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, copyFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const srcTauriDir = dirname(fileURLToPath(import.meta.url));

// `ew` is bundled for Windows only for now; the macOS desktop app is unaffected.
if (process.platform !== "win32") {
  console.log("[stage-ew] skipping: only Windows bundles the ew sidecar");
  process.exit(0);
}

// `ew` lives in the same cargo workspace as the desktop app.
execFileSync("cargo", ["build", "--release", "--bin", "ew"], {
  cwd: srcTauriDir,
  stdio: "inherit",
});

const src = join(srcTauriDir, "target", "release", "ew.exe");
if (!existsSync(src)) {
  console.error(`[stage-ew] build failed: ${src} not found`);
  process.exit(1);
}

const binDir = join(srcTauriDir, "binaries");
mkdirSync(binDir, { recursive: true });
const dst = join(binDir, "ew-x86_64-pc-windows-msvc.exe");
copyFileSync(src, dst);
console.log(`[stage-ew] staged ${dst}`);

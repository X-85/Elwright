#!/usr/bin/env bash
# install.sh — 从 GitHub Release 拉最新 Elwright macOS dmg 装到 /Applications。
#
# 用法：
#   curl -fsSL https://raw.githubusercontent.com/X-85/Elwright/main/install.sh | bash
#   curl -fsSL ... | ELWRIGHT_VERSION=v0.1.3 bash    # 指定版本
#   curl -fsSL ... | ELWRIGHT_INSTALL_DIR=~/Applications bash   # 用户目录
#
# 设计原则：
#   - 零依赖：只用 macOS 自带的 bash / curl / hdiutil / cp
#   - 幂等：覆盖前先卸掉老版本（拖出 /Applications + rm -rf）
#   - 不动 Gatekeeper：首次打开仍需右键 → 打开（未签名应用的官方建议）
#   - 失败清理：trap 保证 TMP 临时目录和挂载的 dmg 不残留
set -euo pipefail

REPO="${ELWRIGHT_REPO:-X-85/Elwright}"
APP_NAME="Elwright"
INSTALL_DIR="${ELWRIGHT_INSTALL_DIR:-/Applications}"

# 1. 探测系统（只支持 macOS；架构探测用于选 dmg）
if [ "$(uname -s)" != "Darwin" ]; then
  echo "✗ install.sh 只支持 macOS。Windows 请用 install.ps1。" >&2
  exit 1
fi
ARCH="$(uname -m)"
case "$ARCH" in
  arm64)  ASSET="aarch64.dmg" ;;
  x86_64) ASSET="x64.dmg" ;;
  *)      echo "✗ 不支持的架构: $ARCH（仅 arm64 / x86_64）" >&2; exit 1 ;;
esac

# 2. 决定要装的版本（默认 latest，可经环境变量覆盖）
if [ -n "${ELWRIGHT_VERSION:-}" ]; then
  TAG="v${ELWRIGHT_VERSION#v}"
else
  echo "→ 查询最新版本 ..."
  TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | python3 -c "import json,sys;print(json.load(sys.stdin)['tag_name'])")
fi
VERSION="${TAG#v}"
URL="https://github.com/${REPO}/releases/download/${TAG}/${APP_NAME}_${VERSION}_${ASSET}"
echo "→ 准备安装 ${APP_NAME} ${TAG}（${ARCH}）"

# 3. 准备临时目录（退出时自动清理）
TMP="$(mktemp -d -t elwright-install.XXXXXX)"
MOUNTED=""
cleanup() {
  if [ -n "$MOUNTED" ]; then hdiutil detach "$MOUNTED" >/dev/null 2>&1 || true; fi
  rm -rf "$TMP"
}
trap cleanup EXIT

# 4. 下载 dmg
echo "→ 下载 $URL"
if ! curl -fsSL -o "${TMP}/${APP_NAME}.dmg" "$URL"; then
  echo "✗ 下载失败（检查网络或 $TAG 是否存在）" >&2
  exit 1
fi

# 5. 挂载 dmg（-nobrowse + readonly 防止用户误点）
echo "→ 挂载 dmg ..."
MOUNT_OUT=$(hdiutil attach -nobrowse -readonly -mountpoint "${TMP}/mnt" \
            "${TMP}/${APP_NAME}.dmg" 2>&1) || {
  echo "✗ 挂载 dmg 失败：$MOUNT_OUT" >&2; exit 1; }
MOUNTED="${TMP}/mnt"

if [ ! -d "${MOUNTED}/${APP_NAME}.app" ]; then
  echo "✗ dmg 里没找到 ${APP_NAME}.app" >&2; exit 1
fi

# 6. 装到目标目录（覆盖式：先删老版本再复制）
TARGET="${INSTALL_DIR}/${APP_NAME}.app"
echo "→ 安装到 ${TARGET}"
mkdir -p "$INSTALL_DIR"
if [ -d "$TARGET" ]; then
  echo "  检测到已安装版本，先移除"
  rm -rf "$TARGET"
fi
# ditto 保留 extended attributes / 资源叉（cp -R 丢这些，Gatekeeper 会更严）
if ! ditto "${MOUNTED}/${APP_NAME}.app" "$TARGET"; then
  echo "✗ 复制失败（试试 ELWRIGHT_INSTALL_DIR=~/Applications 装到用户目录）" >&2
  exit 1
fi

# 7. 完成提示
cat <<EOF

✓ ${APP_NAME} ${TAG} 已装到 ${TARGET}

首次打开（未签名应用，Gatekeeper 会拦一次）：
  1. 在「应用程序」找到 Elwright
  2. 右键 → 打开 → 弹出框里再点「打开」
  3. 之后双击就能直接打开

卸载：
  rm -rf "${TARGET}"

EOF

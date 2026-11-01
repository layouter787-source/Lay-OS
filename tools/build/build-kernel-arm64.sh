#!/usr/bin/env bash
# Compila o kernel Linux do Lay-OS para arm64 (cross-compile). Resultado: out/Image-arm64
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="$ROOT/out"
VER="${LAYOS_KERNEL_VERSION:-6.6.63}"
MAJOR="${VER%%.*}"
SRC="$OUT/linux-arm64-$VER"
TARBALL="$OUT/linux-$VER.tar.xz"
URL="https://cdn.kernel.org/pub/linux/kernel/v${MAJOR}.x/linux-${VER}.tar.xz"
CROSS="aarch64-linux-gnu-"

command -v "${CROSS}gcc" >/dev/null || { echo "erro: instale gcc-aarch64-linux-gnu" >&2; exit 1; }

MAKEARGS=(ARCH=arm64 CROSS_COMPILE="$CROSS" CC="${CROSS}gcc -std=gnu11" HOSTCC="gcc -std=gnu11")

mkdir -p "$OUT"
if [[ ! -d "$SRC" ]]; then
  [[ -f "$TARBALL" ]] || { echo ">> baixando kernel $VER"; curl -fsSL -o "$TARBALL" "$URL"; }
  echo ">> extraindo"
  mkdir -p "$SRC.tmp"
  tar -xf "$TARBALL" -C "$SRC.tmp" --strip-components=1
  mv "$SRC.tmp" "$SRC"
fi

cd "$SRC"
echo ">> compilador: $(${CROSS}gcc --version | head -n1)"
echo ">> configurando arm64 (defconfig + fragmentos Lay-OS)"
make "${MAKEARGS[@]}" defconfig >/dev/null
ARCH=arm64 KCONFIG_CONFIG=.config ./scripts/kconfig/merge_config.sh -m .config \
  "$ROOT/kernel/configs/layos_x86.fragment" \
  "$ROOT/kernel/configs/layos_android.fragment" \
  "$ROOT/kernel/configs/layos_arm64.fragment" >/dev/null
make "${MAKEARGS[@]}" olddefconfig >/dev/null

for opt in ANDROID_BINDER_IPC ANDROID_BINDERFS USER_NS PID_NS CGROUPS SECCOMP SECURITY_LANDLOCK DEVTMPFS BLK_DEV_INITRD EXT4_FS VIRTIO_BLK SERIAL_AMBA_PL011_CONSOLE VIRTIO_MMIO; do
  if ! grep -q "^CONFIG_${opt}=y" .config; then
    echo "ERRO: CONFIG_${opt} nao ficou ativa no kernel arm64" >&2
    exit 1
  fi
done

echo ">> compilando arm64 com $(nproc) nucleos"
make "${MAKEARGS[@]}" -j"$(nproc)" Image

cp arch/arm64/boot/Image "$OUT/Image-arm64"
cp .config "$OUT/kernel-arm64.config"
echo "ok: $OUT/Image-arm64 ($(du -h "$OUT/Image-arm64" | cut -f1))"

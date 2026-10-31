#!/usr/bin/env bash
# Compila o kernel Linux do Lay-OS (x86_64) com os fragmentos de kernel/configs/.
# Resultado: out/bzImage
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="$ROOT/out"
VER="${LAYOS_KERNEL_VERSION:-6.6.63}"
MAJOR="${VER%%.*}"
SRC="$OUT/linux-$VER"
TARBALL="$OUT/linux-$VER.tar.xz"
URL="https://cdn.kernel.org/pub/linux/kernel/v${MAJOR}.x/linux-${VER}.tar.xz"

mkdir -p "$OUT"

if [[ ! -d "$SRC" ]]; then
  [[ -f "$TARBALL" ]] || { echo ">> baixando kernel $VER"; curl -fsSL -o "$TARBALL" "$URL"; }
  echo ">> extraindo"
  tar -xf "$TARBALL" -C "$OUT"
fi

cd "$SRC"
echo ">> configurando (defconfig + fragmentos Lay-OS)"
make defconfig >/dev/null
./scripts/kconfig/merge_config.sh -m .config \
  "$ROOT/kernel/configs/layos_x86.fragment" \
  "$ROOT/kernel/configs/layos_android.fragment" >/dev/null
make olddefconfig >/dev/null

# Confere se as opcoes essenciais do Lay-OS ficaram ativas
for opt in ANDROID_BINDER_IPC ANDROID_BINDERFS USER_NS PID_NS CGROUPS SECCOMP SECURITY_LANDLOCK DEVTMPFS BLK_DEV_INITRD EXT4_FS VIRTIO_BLK; do
  if ! grep -q "^CONFIG_${opt}=y" .config; then
    echo "ERRO: CONFIG_${opt} nao ficou ativa no kernel" >&2
    exit 1
  fi
done

echo ">> compilando com $(nproc) nucleos"
make -j"$(nproc)" bzImage

cp arch/x86/boot/bzImage "$OUT/bzImage"
cp .config "$OUT/kernel.config"
echo "ok: $OUT/bzImage ($(du -h "$OUT/bzImage" | cut -f1))"

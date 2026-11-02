#!/usr/bin/env bash
# Compila um BusyBox estatico para arm64 (cross-compile). Resultado: out/busybox-arm64
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="$ROOT/out"
VER="${LAYOS_BUSYBOX_VERSION:-1.36.1}"
SRC="$OUT/busybox-$VER"
TARBALL="$OUT/busybox-$VER.tar.bz2"
URL="https://busybox.net/downloads/busybox-${VER}.tar.bz2"
CROSS="aarch64-linux-gnu-"

command -v "${CROSS}gcc" >/dev/null || { echo "erro: instale gcc-aarch64-linux-gnu" >&2; exit 1; }
echo ">> compilador: $(${CROSS}gcc --version | head -n1)"

mkdir -p "$OUT"
if [[ ! -d "$SRC" ]]; then
  [[ -f "$TARBALL" ]] || { echo ">> baixando busybox $VER"; curl -fsSL -o "$TARBALL" "$URL"; }
  tar -xjf "$TARBALL" -C "$OUT"
fi

cd "$SRC"
MK=(ARCH=arm64 CROSS_COMPILE="$CROSS" HOSTCC="gcc -std=gnu11" EXTRA_CFLAGS="-std=gnu11")

echo ">> configurando busybox (defconfig estatico)"
make "${MK[@]}" defconfig >/dev/null
sed -i 's/^# CONFIG_STATIC is not set/CONFIG_STATIC=y/' .config
# 'tc' nao compila com headers novos e nao e necessario
sed -i 's/^CONFIG_TC=y/# CONFIG_TC is not set/' .config
sed -i 's/^CONFIG_FEATURE_TC_INGRESS=y/# CONFIG_FEATURE_TC_INGRESS is not set/' .config

# 'yes' termina com SIGPIPE (codigo 141); com pipefail isso derrubaria o script
( set +o pipefail; yes "" | make "${MK[@]}" oldconfig >/dev/null )

echo ">> compilando busybox com $(nproc) nucleos"
make "${MK[@]}" -j"$(nproc)" busybox

cp busybox "$OUT/busybox-arm64"
file "$OUT/busybox-arm64" 2>/dev/null || true
echo "ok: $OUT/busybox-arm64 ($(du -h "$OUT/busybox-arm64" | cut -f1))"

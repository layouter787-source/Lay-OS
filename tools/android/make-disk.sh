#!/usr/bin/env bash
# Cria out/android.img (ext4) a partir de out/android-rootfs.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RFS="$ROOT/out/android-rootfs"
IMG="$ROOT/out/android.img"

sudo test -d "$RFS" || { echo "erro: rode antes tools/android/fetch-rootfs.sh" >&2; exit 1; }

USED_MB="$(sudo du -sm "$RFS" | cut -f1)"
SIZE_MB=$(( USED_MB * 13 / 10 + 256 ))
echo ">> rootfs: ${USED_MB} MB, imagem: ${SIZE_MB} MB"

rm -f "$IMG"
truncate -s "${SIZE_MB}M" "$IMG"
sudo mke2fs -q -t ext4 -L android -d "$RFS" "$IMG"
sudo chown "$(id -u):$(id -g)" "$IMG"

echo "ok: $IMG ($(du -h "$IMG" | cut -f1))"

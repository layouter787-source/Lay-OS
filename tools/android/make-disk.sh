#!/usr/bin/env bash
# Cria out/android.img (ext4) a partir de out/android-rootfs.
# Tambem baixa um APK de teste e o coloca em /data/local/tmp/test.apk dentro da imagem,
# para ser instalado pelo shell do Android (sem depender de adb).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RFS="$ROOT/out/android-rootfs"
IMG="$ROOT/out/android.img"
APK_URL="${LAYOS_TEST_APK_URL:-https://f-droid.org/F-Droid.apk}"

sudo test -d "$RFS" || { echo "erro: rode antes tools/android/fetch-rootfs.sh" >&2; exit 1; }

rm -f "$ROOT/out/test.apk"
if curl -fsSL -o "$ROOT/out/test.apk" "$APK_URL"; then
  sudo mkdir -p "$RFS/data/local/tmp"
  sudo cp "$ROOT/out/test.apk" "$RFS/data/local/tmp/test.apk"
  sudo chown 2000:2000 "$RFS/data/local/tmp" "$RFS/data/local/tmp/test.apk"
  sudo chmod 755 "$RFS/data/local/tmp"
  sudo chmod 644 "$RFS/data/local/tmp/test.apk"
  echo ">> APK de teste incluido na imagem ($(du -h "$ROOT/out/test.apk" | cut -f1))"
else
  echo "AVISO: nao foi possivel baixar o APK de teste ($APK_URL)" >&2
fi

USED_MB="$(sudo du -sm "$RFS" | cut -f1)"
SIZE_MB=$(( USED_MB * 13 / 10 + 512 ))
echo ">> rootfs: ${USED_MB} MB, imagem: ${SIZE_MB} MB"

rm -f "$IMG"
truncate -s "${SIZE_MB}M" "$IMG"
sudo mke2fs -q -t ext4 -L android -d "$RFS" "$IMG"
sudo chown "$(id -u):$(id -g)" "$IMG"

echo "ok: $IMG ($(du -h "$IMG" | cut -f1))"

#!/usr/bin/env bash
# Cria out/android.img (ext4) a partir de out/android-rootfs.
# Tambem coloca na imagem:
#  - um APK de teste em /data/local/tmp/test.apk (instalado pelo shell do Android, sem adb);
#  - a chave publica do adb do host em /data/misc/adb/adb_keys;
#  - um script de init (vendor/etc/init/layos-adb.rc) que liga o adb por TCP (porta 5555) apos o boot.
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

# Chave do adb do host: o adbd do Android passa a confiar nesta maquina
if command -v adb >/dev/null 2>&1; then
  mkdir -p "$HOME/.android"
  [[ -f "$HOME/.android/adbkey" ]] || adb keygen "$HOME/.android/adbkey" || true
  if [[ -f "$HOME/.android/adbkey.pub" ]]; then
    sudo mkdir -p "$RFS/data/misc/adb"
    sudo cp "$HOME/.android/adbkey.pub" "$RFS/data/misc/adb/adb_keys"
    sudo chown -R 1000:2000 "$RFS/data/misc/adb"
    sudo chmod 750 "$RFS/data/misc/adb"
    sudo chmod 640 "$RFS/data/misc/adb/adb_keys"
    echo ">> chave do adb do host instalada em /data/misc/adb/adb_keys"
  fi
fi

# adb por TCP: sem service.adb.tcp.port o adbd so escuta em USB, e o QEMU aceita a conexao
# do host mesmo assim (o adb entao mostra 'device offline').
sudo mkdir -p "$RFS/vendor/etc/init"
sudo tee "$RFS/vendor/etc/init/layos-adb.rc" >/dev/null <<'EOF'
on property:sys.boot_completed=1
    setprop service.adb.tcp.port 5555
    setprop persist.adb.tcp.port 5555
    stop adbd
    start adbd
EOF
sudo chmod 644 "$RFS/vendor/etc/init/layos-adb.rc"
echo ">> adb por TCP (5555) ativado apos o boot via vendor/etc/init/layos-adb.rc"

USED_MB="$(sudo du -sm "$RFS" | cut -f1)"
SIZE_MB=$(( USED_MB * 13 / 10 + 512 ))
echo ">> rootfs: ${USED_MB} MB, imagem: ${SIZE_MB} MB"

rm -f "$IMG"
truncate -s "${SIZE_MB}M" "$IMG"
sudo mke2fs -q -t ext4 -L android -d "$RFS" "$IMG"
sudo chown "$(id -u):$(id -g)" "$IMG"

echo "ok: $IMG ($(du -h "$IMG" | cut -f1))"

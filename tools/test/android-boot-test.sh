#!/usr/bin/env bash
# Etapa 2 do Lay-Android: tenta iniciar o Android sobre o kernel do Lay-OS e mostra ate onde chegou.
# Retorna erro apenas se o init do Android nem chegou a rodar.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOG="$ROOT/out/android-boot.log"
mkdir -p "$ROOT/out"

LAYOS_INIT="$ROOT/system/init-android" \
LAYOS_INITRAMFS_OUT="$ROOT/out/initramfs-android.cpio.gz" \
LAYOS_RFS_NAME="rootfs-android" \
  "$ROOT/tools/build/build-initramfs.sh"

timeout "${LAYOS_ANDROID_TIMEOUT:-300}" "$ROOT/tools/qemu/run-android.sh" 2>&1 | tee "$LOG" >/dev/null || true

stage() { # descricao padrao
  if grep -q -- "$2" "$LOG"; then echo "OK      $1"; return 0; else echo "NAO     $1"; return 1; fi
}

echo "== Etapas do boot do Lay-Android =="
stage "disco Android montado"         "rootfs Android montado" || true
stage "binderfs montado"              "binderfs montado" || true
stage "controle passou ao init Android" "entregando o controle" || true
ANDROID_INIT=0
stage "init do Android rodou"          "init: " && ANDROID_INIT=1
stage "servicemanager iniciado"       "servicemanager" || true
stage "zygote iniciado"               "zygote" || true
stage "system_server / boot completo" "sys.boot_completed" || true

echo "== Ultimas linhas do log =="
tail -n 40 "$LOG"

[[ $ANDROID_INIT -eq 1 ]]

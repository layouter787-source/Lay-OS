#!/usr/bin/env bash
# Teste do Lay-OS arm64: gera o initramfs, sobe no QEMU virt e confere banner e arquitetura.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOG="$ROOT/out/arm64-boot-test.log"
SUMMARY="${GITHUB_STEP_SUMMARY:-/dev/null}"
mkdir -p "$ROOT/out"

BUSYBOX="$ROOT/out/busybox-arm64" \
LAYOS_APPLET_LIST_BUSYBOX="$(command -v busybox)" \
LAYOS_INITRAMFS_OUT="$ROOT/out/initramfs-arm64.cpio.gz" \
LAYOS_RFS_NAME="rootfs-arm64" \
  "$ROOT/tools/build/build-initramfs.sh" || exit 1

(
  sleep "${LAYOS_ARM64_BOOT_WAIT:-90}"
  printf 'layinfo\n'
  sleep 5
  printf 'poweroff -f\n'
  sleep 5
) | timeout "${LAYOS_ARM64_TIMEOUT:-300}" "$ROOT/tools/qemu/run-arm64.sh" 2>&1 | tee "$LOG" || true

fail=0
BANNER="nao"; ARCH="nao"
grep -q "Lay-OS 0.1.0 (fase 1)" "$LOG" && BANNER="sim" || fail=1
grep -q "Arq    : aarch64" "$LOG" && ARCH="sim" || fail=1

{
  echo "### Lay-OS arm64 (QEMU virt)"
  echo ""
  echo "| Estado | Verificacao |"
  echo "|---|---|"
  [[ $BANNER == sim ]] && echo "| OK | Lay-OS iniciou (banner) |" || echo "| NAO | Lay-OS iniciou (banner) |"
  [[ $ARCH == sim ]] && echo "| OK | kernel e sistema em aarch64 |" || echo "| NAO | kernel e sistema em aarch64 |"
  if [[ $fail -ne 0 ]]; then
    echo ""
    echo "<details><summary>Ultimas linhas do log arm64</summary>"
    echo ""
    echo '```'
    tail -n 30 "$LOG"
    echo '```'
    echo "</details>"
  fi
} >> "$SUMMARY"

if [[ $fail -eq 0 ]]; then
  echo "OK: Lay-OS arm64 iniciou e respondeu no QEMU"
else
  echo "FALHA: Lay-OS arm64 (banner=$BANNER, aarch64=$ARCH)"
  tail -n 30 "$LOG"
fi
exit $fail

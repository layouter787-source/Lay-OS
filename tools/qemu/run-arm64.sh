#!/usr/bin/env bash
# Executa o Lay-OS arm64 no QEMU (maquina 'virt', emulacao completa).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
KERNEL="${LAYOS_ARM64_KERNEL:-$ROOT/out/Image-arm64}"
INITRD="${LAYOS_ARM64_INITRD:-$ROOT/out/initramfs-arm64.cpio.gz}"

for f in "$KERNEL" "$INITRD"; do
  [[ -r "$f" ]] || { echo "erro: arquivo ausente: $f" >&2; exit 1; }
done

exec qemu-system-aarch64 \
  -M virt -cpu max \
  -m "${LAYOS_ARM64_RAM:-1024}" \
  -smp "${LAYOS_ARM64_CPUS:-2}" \
  -kernel "$KERNEL" \
  -initrd "$INITRD" \
  -append "console=ttyAMA0 rdinit=/init" \
  -nographic -no-reboot

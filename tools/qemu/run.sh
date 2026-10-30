#!/usr/bin/env bash
# Executa o Lay-OS no QEMU (x86_64). Use LAYOS_KERNEL para apontar um kernel proprio.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INITRD="$ROOT/out/initramfs.cpio.gz"

if [[ ! -f "$INITRD" ]]; then
  echo "initramfs nao encontrado. Rode: ./tools/build/build-initramfs.sh" >&2
  exit 1
fi

KERNEL="${LAYOS_KERNEL:-}"
if [[ -z "$KERNEL" ]]; then
  KERNEL="$(ls -1 /boot/vmlinuz-* 2>/dev/null | sort -V | tail -n1 || true)"
fi
if [[ -z "$KERNEL" || ! -r "$KERNEL" ]]; then
  echo "erro: kernel nao encontrado/legivel. Defina LAYOS_KERNEL=/caminho/bzImage" >&2
  exit 1
fi

ACCEL=()
if [[ -w /dev/kvm ]]; then
  ACCEL=(-enable-kvm -cpu host)
else
  ACCEL=(-cpu max)
fi

exec qemu-system-x86_64 \
  "${ACCEL[@]}" \
  -m "${LAYOS_RAM:-1024}" \
  -smp "${LAYOS_CPUS:-2}" \
  -kernel "$KERNEL" \
  -initrd "$INITRD" \
  -append "console=ttyS0 rdinit=/init quiet" \
  -nographic \
  -netdev user,id=n0 -device virtio-net-pci,netdev=n0

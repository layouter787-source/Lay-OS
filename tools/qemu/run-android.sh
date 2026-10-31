#!/usr/bin/env bash
# Executa o Lay-OS com a camada Android no QEMU (kernel do Lay-OS + disco android.img).
# A porta 5555 do Android (adb por TCP) fica disponivel em 127.0.0.1:5555 no host.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INITRD="$ROOT/out/initramfs-android.cpio.gz"
DISK="$ROOT/out/android.img"
KERNEL="${LAYOS_KERNEL:-$ROOT/out/bzImage}"

for f in "$INITRD" "$DISK" "$KERNEL"; do
  [[ -r "$f" ]] || { echo "erro: arquivo ausente: $f" >&2; exit 1; }
done

ACCEL=(-cpu max)
if [[ -w /dev/kvm ]]; then ACCEL=(-enable-kvm -cpu host); fi

exec qemu-system-x86_64 \
  "${ACCEL[@]}" \
  -m "${LAYOS_ANDROID_RAM:-4096}" \
  -smp "${LAYOS_ANDROID_CPUS:-4}" \
  -kernel "$KERNEL" \
  -initrd "$INITRD" \
  -append "console=ttyS0 loglevel=7 rdinit=/init androidboot.hardware=redroid androidboot.selinux=permissive androidboot.console=ttyS0" \
  -drive file="$DISK",format=raw,if=virtio \
  -netdev user,id=n0,hostfwd=tcp:127.0.0.1:5555-10.0.2.15:5555 \
  -device virtio-net-pci,netdev=n0 \
  -nographic -no-reboot

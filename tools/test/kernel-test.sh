#!/usr/bin/env bash
# Verifica se o kernel do Lay-OS iniciou com Binder e isolamento ativos.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOG="$ROOT/out/kernel-test.log"
mkdir -p "$ROOT/out"

(
  sleep "${LAYOS_BOOT_WAIT:-20}"
  printf 'echo BINDERFS=$(grep -c binder /proc/filesystems)\n'
  sleep 1
  printf 'echo USERNS=$(ls /proc/self/ns | grep -c user)\n'
  sleep 1
  printf 'echo CGROUP2=$(grep -c cgroup2 /proc/filesystems)\n'
  sleep 1
  printf 'echo KVER=$(uname -r)\n'
  sleep 2
  printf 'poweroff -f\n'
  sleep 3
) | timeout "${LAYOS_TIMEOUT:-120}" "$ROOT/tools/qemu/run.sh" 2>&1 | tee "$LOG" || true

fail=0
grep -q 'BINDERFS=1' "$LOG" || { echo "FALHA: binderfs ausente no kernel"; fail=1; }
grep -q 'USERNS=1' "$LOG" || { echo "FALHA: user namespace ausente"; fail=1; }
grep -q 'CGROUP2=1' "$LOG" || { echo "FALHA: cgroup v2 ausente"; fail=1; }

if [[ $fail -eq 0 ]]; then
  echo "OK: kernel do Lay-OS com Binder, namespaces e cgroup v2"
fi
exit $fail

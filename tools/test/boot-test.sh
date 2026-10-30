#!/usr/bin/env bash
# Teste automatizado: sobe o Lay-OS no QEMU (sem interface), roda 'layinfo' e confere o resultado.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOG="$ROOT/out/boot-test.log"
mkdir -p "$ROOT/out"

(
  sleep "${LAYOS_BOOT_WAIT:-20}"
  printf 'layinfo\n'
  sleep 2
  printf 'poweroff -f\n'
  sleep 3
) | timeout "${LAYOS_TIMEOUT:-120}" "$ROOT/tools/qemu/run.sh" 2>&1 | tee "$LOG" || true

fail=0
grep -q "Lay-OS 0.1.0 (fase 1)" "$LOG" || { echo "FALHA: banner do Lay-OS nao apareceu"; fail=1; }
grep -q "Kernel :" "$LOG" || { echo "FALHA: comando layinfo nao respondeu"; fail=1; }

if [[ $fail -eq 0 ]]; then
  echo "OK: Lay-OS iniciou e respondeu no QEMU"
fi
exit $fail

#!/usr/bin/env bash
# Etapa 2 do Lay-Android: inicia o Android sobre o kernel do Lay-OS, tenta falar com ele por adb
# e mostra ate onde chegou (tabela no log e no resumo da execucao do GitHub Actions).
# Detecta travamento: se o log do Android parar de crescer, encerra cedo.
# Retorna erro apenas se o init do Android nem chegou a rodar.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOG="$ROOT/out/android-boot.log"
SUMMARY="${GITHUB_STEP_SUMMARY:-/dev/null}"
SERIAL="127.0.0.1:5555"
STALL_LIMIT="${LAYOS_ANDROID_STALL:-150}"
mkdir -p "$ROOT/out"

LAYOS_INIT="$ROOT/system/init-android" \
LAYOS_INITRAMFS_OUT="$ROOT/out/initramfs-android.cpio.gz" \
LAYOS_RFS_NAME="rootfs-android" \
  "$ROOT/tools/build/build-initramfs.sh"

TIMEOUT="${LAYOS_ANDROID_TIMEOUT:-900}"
START=$SECONDS
timeout "$TIMEOUT" "$ROOT/tools/qemu/run-android.sh" >"$LOG" 2>&1 &
QPID=$!

HAVE_ADB=0; command -v adb >/dev/null 2>&1 && HAVE_ADB=1
ADB_UP=0; BOOTED=0; INFO=""; STALLED=0
last_size=0; last_change=$SECONDS
deadline=$((SECONDS + TIMEOUT - 15))

while (( SECONDS < deadline )) && kill -0 "$QPID" 2>/dev/null; do
  if (( HAVE_ADB )); then
    adb connect "$SERIAL" >/dev/null 2>&1
    if [[ "$(timeout 10 adb -s "$SERIAL" get-state 2>/dev/null | tr -d '\r')" == "device" ]]; then
      ADB_UP=1
      v="$(timeout 10 adb -s "$SERIAL" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')"
      if [[ "$v" == "1" ]]; then BOOTED=1; break; fi
    fi
  fi
  grep -q "sys.boot_completed=1" "$LOG" 2>/dev/null && { BOOTED=1; break; }

  size=$(stat -c %s "$LOG" 2>/dev/null || echo 0)
  if (( size != last_size )); then last_size=$size; last_change=$SECONDS
  elif (( SECONDS - last_change > STALL_LIMIT )); then STALLED=1; break; fi
  sleep 5
done
WALL=$((SECONDS - START))

if (( BOOTED && ADB_UP )); then
  REL="$(timeout 10 adb -s "$SERIAL" shell getprop ro.build.version.release 2>/dev/null | tr -d '\r')"
  SDK="$(timeout 10 adb -s "$SERIAL" shell getprop ro.build.version.sdk 2>/dev/null | tr -d '\r')"
  PKGS="$(timeout 20 adb -s "$SERIAL" shell pm list packages 2>/dev/null | wc -l)"
  INFO="Android $REL (SDK $SDK), $PKGS pacotes instalados"
  timeout 20 adb -s "$SERIAL" shell pm list packages 2>/dev/null | head -n 15 > "$ROOT/out/android-packages.txt"
fi

kill "$QPID" 2>/dev/null; sleep 1; pkill -f qemu-system-x86_64 2>/dev/null || true
wait "$QPID" 2>/dev/null || true

ROWS=()
ANDROID_INIT=0
stage() { # descricao padrao
  if grep -q -- "$2" "$LOG"; then ROWS+=("| OK | $1 |"); echo "OK      $1"; return 0
  else ROWS+=("| NAO | $1 |"); echo "NAO     $1"; return 1; fi
}

echo "== Etapas do boot do Lay-Android =="
stage "disco Android montado"           "rootfs Android montado" || true
stage "rede configurada (eth0)"         "rede configurada" || true
stage "binderfs montado"                "binderfs montado" || true
stage "controle passou ao init Android" "entregando o controle" || true
stage "init do Android rodou"           "init: " && ANDROID_INIT=1
stage "servicemanager iniciado"         "starting service 'servicemanager'" || true
stage "zygote iniciado"                 "starting service 'zygote" || true
if (( BOOTED )); then ROWS+=("| OK | boot completo (sys.boot_completed=1) |"); echo "OK      boot completo"
else ROWS+=("| NAO | boot completo (sys.boot_completed=1) |"); echo "NAO     boot completo"; fi
if (( ADB_UP )); then ROWS+=("| OK | adb conectado ao Android |"); echo "OK      adb conectado"
else ROWS+=("| NAO | adb conectado ao Android |"); echo "NAO     adb conectado (adb instalado: $HAVE_ADB)"; fi
[[ -n "$INFO" ]] && echo "INFO    $INFO"

# --- Diagnostico ---
ACCEL="$(grep -m1 'aceleracao:' "$LOG" || echo 'aceleracao: desconhecida')"
GUEST="$(grep -o '^\[ *[0-9]*\.[0-9]*\]' "$LOG" | tail -n1 | tr -d '[] ')"
SERVICES="$(grep -o "starting service '[^']*'" "$LOG" | uniq | sed "s/starting service //" | head -n 40 | tr '\n' ' ')"
ERRS="$(grep -E 'init: .*(fail|Fail|error|Error|denied|Unable|Could not|cannot|timed out|Wait for|wait_for)' "$LOG" | grep -v Untracked | head -n 30)"
CRASHES="$(grep -E 'received signal|Aborted|Fatal signal|SIGSEGV' "$LOG" | head -n 10)"
KEYS="$(grep -E 'servicemanager|apexd|vold|zygote|ueventd|loop|apex|selinux|SELinux|cgroup' "$LOG" | grep -v Untracked | head -n 40)"
EARLY="$(sed -n '/entregando o controle/,$p' "$LOG" | head -n 70)"
REASON="tempo limite"; (( STALLED )) && REASON="o log do Android parou de crescer por ${STALL_LIMIT}s (travou)"; (( BOOTED )) && REASON="boot completo"

echo "== Diagnostico =="
echo "$ACCEL"
echo "Motivo do fim: $REASON"
echo "Tempo real: ${WALL}s | tempo do Android (guest): ${GUEST:-?}s"
echo "Servicos iniciados: ${SERVICES:-nenhum}"
echo "-- Erros do init --"; echo "${ERRS:-nenhum encontrado}"
echo "-- Falhas de servicos --"; echo "${CRASHES:-nenhuma}"

{
  echo "### Lay-Android: boot do Android no kernel do Lay-OS"
  echo ""
  echo "| Estado | Etapa |"
  echo "|---|---|"
  printf '%s\n' "${ROWS[@]}"
  [[ -n "$INFO" ]] && { echo ""; echo "**$INFO**"; }
  echo ""
  echo "**Diagnostico**"
  echo ""
  echo "- $ACCEL"
  echo "- Motivo do fim do teste: $REASON"
  echo "- Tempo real: ${WALL}s | tempo do Android (guest): ${GUEST:-?}s"
  echo "- Servicos iniciados: ${SERVICES:-nenhum}"
  echo ""
  echo "<details><summary>Primeiras linhas do init do Android</summary>"
  echo ""
  echo '```'
  echo "${EARLY:-sem linhas}"
  echo '```'
  echo "</details>"
  echo ""
  echo "<details><summary>Linhas-chave (servicemanager, apexd, zygote, loop, cgroup...)</summary>"
  echo ""
  echo '```'
  echo "${KEYS:-nenhuma}"
  echo '```'
  echo "</details>"
  echo ""
  echo "<details><summary>Erros do init e falhas de servicos</summary>"
  echo ""
  echo '```'
  echo "${ERRS:-nenhum erro do init encontrado}"
  echo "---"
  echo "${CRASHES:-nenhuma falha de servico}"
  echo '```'
  echo "</details>"
  echo ""
  echo "<details><summary>Ultimas linhas do log</summary>"
  echo ""
  echo '```'
  tail -n 40 "$LOG"
  echo '```'
  echo "</details>"
} >> "$SUMMARY"

echo "== Ultimas linhas do log =="
tail -n 40 "$LOG"

[[ $ANDROID_INIT -eq 1 ]]

#!/usr/bin/env bash
# Etapa 2 do Lay-Android: inicia o Android sobre o kernel do Lay-OS e investiga por dentro.
#  - tenta falar com ele por adb;
#  - envia comandos ao shell do Android pela porta serial (getprop, ps, logcat) em horarios fixos;
#  - mostra o resultado na tabela do Summary do GitHub Actions.
# Retorna erro apenas se o init do Android nem chegou a rodar.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOG="$ROOT/out/android-boot.log"
FIFO="$ROOT/out/android-console.fifo"
SUMMARY="${GITHUB_STEP_SUMMARY:-/dev/null}"
SERIAL="127.0.0.1:5555"
DIAG1="${LAYOS_ANDROID_DIAG1:-90}"    # 1o diagnostico interno (s apos iniciar o QEMU)
DIAG2="${LAYOS_ANDROID_DIAG2:-210}"   # 2o diagnostico interno
END_AFTER="${LAYOS_ANDROID_END:-250}" # encerra aqui se nao completou o boot
mkdir -p "$ROOT/out"

LAYOS_INIT="$ROOT/system/init-android" \
LAYOS_INITRAMFS_OUT="$ROOT/out/initramfs-android.cpio.gz" \
LAYOS_RFS_NAME="rootfs-android" \
  "$ROOT/tools/build/build-initramfs.sh"

TIMEOUT="${LAYOS_ANDROID_TIMEOUT:-900}"
rm -f "$FIFO"; mkfifo "$FIFO"
exec 3<>"$FIFO"   # aberto para leitura e escrita: o QEMU nao ve EOF

START=$SECONDS
timeout "$TIMEOUT" "$ROOT/tools/qemu/run-android.sh" <&3 >"$LOG" 2>&1 &
QPID=$!

DIAGCMD='echo ==LAYOS-DIAG-BEGIN==; getprop | grep -E "init.svc|sys.boot|vold|ro.crypto|bootcomplete"; echo ==LAYOS-PS==; ps -A | head -n 70; echo ==LAYOS-LOGCAT==; logcat -d -t 120; echo ==LAYOS-DIAG-END=='
SENT1=0; SENT2=0

HAVE_ADB=0; command -v adb >/dev/null 2>&1 && HAVE_ADB=1
ADB_UP=0; BOOTED=0; INFO=""
deadline=$((START + TIMEOUT - 15))

while (( SECONDS < deadline )) && kill -0 "$QPID" 2>/dev/null; do
  el=$((SECONDS - START))
  if (( HAVE_ADB )); then
    adb connect "$SERIAL" >/dev/null 2>&1
    if [[ "$(timeout 10 adb -s "$SERIAL" get-state 2>/dev/null | tr -d '\r')" == "device" ]]; then
      ADB_UP=1
      v="$(timeout 10 adb -s "$SERIAL" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')"
      if [[ "$v" == "1" ]]; then BOOTED=1; break; fi
    fi
  fi
  grep -q "sys.boot_completed=1" "$LOG" 2>/dev/null && { BOOTED=1; break; }

  if (( ! SENT1 && el >= DIAG1 )); then printf '%s\n' "$DIAGCMD" >&3; SENT1=1; fi
  if (( ! SENT2 && el >= DIAG2 )); then printf '%s\n' "$DIAGCMD" >&3; SENT2=1; fi
  if (( el >= END_AFTER )); then break; fi
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
exec 3>&-; rm -f "$FIFO"

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
stage "shell do Android na serial"      "console:/ " || true
if (( BOOTED )); then ROWS+=("| OK | boot completo (sys.boot_completed=1) |"); echo "OK      boot completo"
else ROWS+=("| NAO | boot completo (sys.boot_completed=1) |"); echo "NAO     boot completo"; fi
if (( ADB_UP )); then ROWS+=("| OK | adb conectado ao Android |"); echo "OK      adb conectado"
else ROWS+=("| NAO | adb conectado ao Android |"); echo "NAO     adb conectado (adb instalado: $HAVE_ADB)"; fi
[[ -n "$INFO" ]] && echo "INFO    $INFO"

# --- Diagnostico ---
ACCEL="$(grep -m1 'aceleracao:' "$LOG" || echo 'aceleracao: desconhecida')"
GUEST="$(grep -o '^\[ *[0-9]*\.[0-9]*\]' "$LOG" | tail -n1 | tr -d '[] ')"
SERVICES="$(grep -o "starting service '[^']*'" "$LOG" | uniq | sed "s/starting service //" | head -n 60 | tr '\n' ' ')"
ERRS="$(grep -E 'init: .*(fail|Fail|error|Error|denied|Unable|Could not|cannot|timed out|Wait for|wait_for)' "$LOG" | grep -v Untracked | head -n 30)"
CRASHES="$(grep -E 'received signal|Aborted|Fatal signal|SIGSEGV' "$LOG" | head -n 10)"
INNER="$(sed -n '/^==LAYOS-DIAG-BEGIN==/,/^==LAYOS-DIAG-END==/p' "$LOG" | tr -d '\r')"
SVCSTATE="$(grep -E '^\[init\.svc\.' <<<"$INNER" | sort -u | head -n 90)"
SYSPROPS="$(grep -E '^\[(sys|vold|ro\.crypto|dev)\.' <<<"$INNER" | sort -u | head -n 30)"
PSOUT="$(sed -n '/^==LAYOS-PS==/,/^==LAYOS-LOGCAT==/p' <<<"$INNER" | tail -n 75)"
LOGCATOUT="$(sed -n '/^==LAYOS-LOGCAT==/,/^==LAYOS-DIAG-END==/p' <<<"$INNER" | tail -n 130)"
REASON="tempo limite"; (( END_AFTER <= WALL )) && REASON="diagnostico concluido em ${WALL}s sem boot completo"; (( BOOTED )) && REASON="boot completo"

echo "== Diagnostico =="
echo "$ACCEL"
echo "Motivo do fim: $REASON"
echo "Tempo real: ${WALL}s | tempo do Android (guest): ${GUEST:-?}s"
echo "Servicos iniciados: ${SERVICES:-nenhum}"
echo "-- Estado dos servicos (init.svc) --"; echo "${SVCSTATE:-sem dados (shell do Android nao respondeu)}"
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
  echo "<details open><summary>Estado dos servicos por dentro do Android (getprop init.svc)</summary>"
  echo ""
  echo '```'
  echo "${SVCSTATE:-sem dados: o shell do Android nao respondeu}"
  echo "---"
  echo "${SYSPROPS:-}"
  echo '```'
  echo "</details>"
  echo ""
  echo "<details><summary>Processos do Android (ps -A)</summary>"
  echo ""
  echo '```'
  echo "${PSOUT:-sem dados}"
  echo '```'
  echo "</details>"
  echo ""
  echo "<details><summary>logcat do Android (ultimas linhas)</summary>"
  echo ""
  echo '```'
  echo "${LOGCATOUT:-sem dados}"
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
  echo "<details><summary>Ultimas linhas do log serial</summary>"
  echo ""
  echo '```'
  tail -n 40 "$LOG"
  echo '```'
  echo "</details>"
} >> "$SUMMARY"

echo "== Ultimas linhas do log =="
tail -n 40 "$LOG"

[[ $ANDROID_INIT -eq 1 ]]

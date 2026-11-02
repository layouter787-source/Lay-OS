#!/usr/bin/env bash
# Lay-Android, etapas 2 e 3: inicia o Android sobre o kernel do Lay-OS, confirma o boot por adb,
# instala um APK de verdade e abre o app. Se o boot nao completar, investiga por dentro
# (getprop/ps/logcat enviados ao shell do Android pela porta serial).
# Retorna erro apenas se o init do Android nem chegou a rodar.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOG="$ROOT/out/android-boot.log"
FIFO="$ROOT/out/android-console.fifo"
SUMMARY="${GITHUB_STEP_SUMMARY:-/dev/null}"
SERIAL="127.0.0.1:5555"
APK_URL="${LAYOS_TEST_APK_URL:-https://f-droid.org/F-Droid.apk}"
APK_PKG="${LAYOS_TEST_APK_PKG:-org.fdroid.fdroid}"
DIAG1="${LAYOS_ANDROID_DIAG1:-90}"
DIAG2="${LAYOS_ANDROID_DIAG2:-210}"
END_AFTER="${LAYOS_ANDROID_END:-250}"
mkdir -p "$ROOT/out"

adbx() { timeout "${ADB_T:-30}" adb -s "$SERIAL" "$@" 2>&1 | tr -d '\r'; }

LAYOS_INIT="$ROOT/system/init-android" \
LAYOS_INITRAMFS_OUT="$ROOT/out/initramfs-android.cpio.gz" \
LAYOS_RFS_NAME="rootfs-android" \
  "$ROOT/tools/build/build-initramfs.sh"

TIMEOUT="${LAYOS_ANDROID_TIMEOUT:-900}"
rm -f "$FIFO"; mkfifo "$FIFO"
exec 3<>"$FIFO"

START=$SECONDS
timeout "$TIMEOUT" "$ROOT/tools/qemu/run-android.sh" <&3 >"$LOG" 2>&1 &
QPID=$!

DIAGCMD='echo ==LAYOS-DIAG-BEGIN==; getprop | grep -E "init.svc|sys.boot|vold|ro.crypto|bootcomplete"; echo ==LAYOS-PS==; ps -A | head -n 70; echo ==LAYOS-LOGCAT==; logcat -d -t 120; echo ==LAYOS-DIAG-END=='
SENT1=0; SENT2=0

HAVE_ADB=0; command -v adb >/dev/null 2>&1 && HAVE_ADB=1
ADB_UP=0; BOOTED=0
deadline=$((START + TIMEOUT - 15))

while (( SECONDS < deadline )) && kill -0 "$QPID" 2>/dev/null; do
  el=$((SECONDS - START))
  if (( HAVE_ADB )); then
    adb connect "$SERIAL" >/dev/null 2>&1
    if [[ "$(adbx get-state)" == "device" ]]; then
      ADB_UP=1
      v="$(ADB_T=10 adbx shell getprop sys.boot_completed)"
      if [[ "$v" == "1" ]]; then BOOTED=1; break; fi
    fi
  fi
  if (( ! SENT1 && el >= DIAG1 )); then printf '%s\n' "$DIAGCMD" >&3; SENT1=1; fi
  if (( ! SENT2 && el >= DIAG2 )); then printf '%s\n' "$DIAGCMD" >&3; SENT2=1; fi
  if (( el >= END_AFTER )); then break; fi
  sleep 5
done
BOOT_WALL=$((SECONDS - START))

# --- Etapa 3: com o Android no ar, confirmar versao, instalar um APK e abrir o app ---
REL=""; SDK=""; PKGS=0; INST_OK=0; APP_OK=0; DL_OK=0
INSTALL_OUT=""; FOCUS=""; UITEXT=""; MODEL=""; SIZE=""
if (( BOOTED )); then
  # espera o PackageManager ficar pronto
  for _ in $(seq 1 36); do
    PKGS="$(ADB_T=20 adbx shell pm list packages | grep -c '^package:')"
    (( PKGS > 20 )) && break
    sleep 5
  done
  REL="$(ADB_T=10 adbx shell getprop ro.build.version.release)"
  SDK="$(ADB_T=10 adbx shell getprop ro.build.version.sdk)"
  MODEL="$(ADB_T=10 adbx shell getprop ro.product.model)"
  SIZE="$(ADB_T=10 adbx shell wm size | tail -n1)"

  echo ">> baixando APK de teste: $APK_URL"
  if curl -fsSL -o "$ROOT/out/test.apk" "$APK_URL"; then DL_OK=1; fi
  if (( DL_OK )); then
    INSTALL_OUT="$(ADB_T=180 adbx install -r "$ROOT/out/test.apk" | tail -n 3)"
    grep -q 'Success' <<<"$INSTALL_OUT" && INST_OK=1
  else
    INSTALL_OUT="nao foi possivel baixar o APK"
  fi
  if (( INST_OK )); then
    ADB_T=30 adbx shell monkey -p "$APK_PKG" -c android.intent.category.LAUNCHER 1 >/dev/null
    sleep 12
    PID="$(ADB_T=10 adbx shell pidof "$APK_PKG")"
    [[ -n "$PID" ]] && APP_OK=1
    FOCUS="$(ADB_T=20 adbx shell dumpsys window | grep -m2 -E 'mCurrentFocus|mFocusedApp')"
    ADB_T=30 adbx shell uiautomator dump /sdcard/ui.xml >/dev/null
    UITEXT="$(ADB_T=20 adbx shell cat /sdcard/ui.xml | grep -o 'text="[^"]\+"' | head -n 15)"
    ADB_T=30 adbx exec-out screencap -p > "$ROOT/out/screen.png" 2>/dev/null || true
  fi
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
row() { # ok(0/1) descricao
  if (( $1 )); then ROWS+=("| OK | $2 |"); echo "OK      $2"; else ROWS+=("| NAO | $2 |"); echo "NAO     $2"; fi
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
row "$ADB_UP" "adb conectado ao Android"
row "$BOOTED" "boot completo confirmado por adb (sys.boot_completed=1)"
row "$(( PKGS > 20 ))" "PackageManager pronto ($PKGS pacotes)"
row "$DL_OK" "APK de teste baixado ($APK_PKG)"
row "$INST_OK" "APK instalado no Lay-Android (adb install)"
row "$APP_OK" "app abriu e esta em execucao"

INFO=""
[[ -n "$REL" ]] && INFO="Android $REL (SDK $SDK), modelo $MODEL, tela $SIZE, $PKGS pacotes"
[[ -n "$INFO" ]] && echo "INFO    $INFO"

ACCEL="$(grep -m1 'aceleracao:' "$LOG" || echo 'aceleracao: desconhecida')"
GUEST="$(grep -o '^\[ *[0-9]*\.[0-9]*\]' "$LOG" | tail -n1 | tr -d '[] ')"
SERVICES="$(grep -o "starting service '[^']*'" "$LOG" | uniq | sed "s/starting service //" | head -n 60 | tr '\n' ' ')"
ERRS="$(grep -E 'init: .*(fail|Fail|error|Error|denied|Unable|Could not|cannot|timed out|Wait for|wait_for)' "$LOG" | grep -v Untracked | head -n 30)"
CRASHES="$(grep -E 'received signal|Aborted|Fatal signal|SIGSEGV' "$LOG" | head -n 10)"
INNER="$(sed -n '/^==LAYOS-DIAG-BEGIN==/,/^==LAYOS-DIAG-END==/p' "$LOG" | tr -d '\r')"
SVCSTATE="$(grep -E '^\[init\.svc\.' <<<"$INNER" | sort -u | head -n 90)"
PSOUT="$(sed -n '/^==LAYOS-PS==/,/^==LAYOS-LOGCAT==/p' <<<"$INNER" | tail -n 75)"
LOGCATOUT="$(sed -n '/^==LAYOS-LOGCAT==/,/^==LAYOS-DIAG-END==/p' <<<"$INNER" | tail -n 130)"
REASON="diagnostico concluido em ${BOOT_WALL}s sem boot completo"; (( BOOTED )) && REASON="boot completo em ${BOOT_WALL}s"

echo "== Diagnostico =="
echo "$ACCEL | $REASON | guest: ${GUEST:-?}s"
echo "Instalacao do APK: ${INSTALL_OUT:-nao tentada}"
echo "Foco da janela: ${FOCUS:-n/d}"
echo "Textos na tela do app: ${UITEXT:-n/d}"

{
  echo "### Lay-Android: boot do Android + instalacao de APK"
  echo ""
  echo "| Estado | Etapa |"
  echo "|---|---|"
  printf '%s\n' "${ROWS[@]}"
  [[ -n "$INFO" ]] && { echo ""; echo "**$INFO**"; }
  echo ""
  echo "- $ACCEL"
  echo "- $REASON (tempo do Android: ${GUEST:-?}s)"
  echo "- Instalacao do APK: ${INSTALL_OUT:-nao tentada}"
  echo ""
  if (( INST_OK )); then
    echo "<details open><summary>App aberto: janela em foco e textos na tela</summary>"
    echo ""
    echo '```'
    echo "${FOCUS:-}"
    echo "---"
    echo "${UITEXT:-sem textos}"
    echo '```'
    echo "</details>"
    echo ""
  fi
  echo "<details><summary>Servicos iniciados (ordem)</summary>"
  echo ""
  echo '```'
  echo "${SERVICES:-nenhum}"
  echo '```'
  echo "</details>"
  if (( ! BOOTED )); then
    echo ""
    echo "<details open><summary>Estado dos servicos por dentro do Android (getprop init.svc)</summary>"
    echo ""
    echo '```'
    echo "${SVCSTATE:-sem dados: o shell do Android nao respondeu}"
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
  fi
  echo ""
  echo "<details><summary>Erros do init e falhas de servicos</summary>"
  echo ""
  echo '```'
  echo "${ERRS:-nenhum erro do init encontrado}"
  echo "---"
  echo "${CRASHES:-nenhuma falha de servico}"
  echo '```'
  echo "</details>"
} >> "$SUMMARY"

echo "== Ultimas linhas do log =="
tail -n 30 "$LOG"

[[ $ANDROID_INIT -eq 1 ]]

#!/usr/bin/env bash
# Lay-Android, etapas 2 e 3: inicia o Android sobre o kernel do Lay-OS, confirma o boot,
# instala um APK de verdade, abre o app e captura a tela.
# O controle e feito pelo SHELL DO ANDROID na porta serial (console:/ $), que e mais confiavel que adb;
# o adb e testado a parte, para o objetivo de depurar pelo Android Studio.
# Retorna erro apenas se o init do Android nem chegou a rodar.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOG="$ROOT/out/android-boot.log"
FIFO="$ROOT/out/android-console.fifo"
SUMMARY="${GITHUB_STEP_SUMMARY:-/dev/null}"
SERIAL="127.0.0.1:5555"
APK_PKG="${LAYOS_TEST_APK_PKG:-org.fdroid.fdroid}"
POLL_FROM="${LAYOS_ANDROID_POLL_FROM:-40}"
DIAG1="${LAYOS_ANDROID_DIAG1:-150}"
DIAG2="${LAYOS_ANDROID_DIAG2:-400}"
DIAG3="${LAYOS_ANDROID_DIAG3:-640}"
END_AFTER="${LAYOS_ANDROID_END:-700}"
mkdir -p "$ROOT/out"

adbx() { timeout "${ADB_T:-30}" adb -s "$SERIAL" "$@" 2>&1 | tr -d '\r'; }

LAYOS_INIT="$ROOT/system/init-android" \
LAYOS_INITRAMFS_OUT="$ROOT/out/initramfs-android.cpio.gz" \
LAYOS_RFS_NAME="rootfs-android" \
  "$ROOT/tools/build/build-initramfs.sh"

TIMEOUT="${LAYOS_ANDROID_TIMEOUT:-1200}"
rm -f "$FIFO"; mkfifo "$FIFO"
exec 3<>"$FIFO"

START=$SECONDS
timeout "$TIMEOUT" "$ROOT/tools/qemu/run-android.sh" <&3 >"$LOG" 2>&1 &
QPID=$!

# --- helpers do shell serial ---
ser()  { printf '%s\n' "$1" >&3; }
val()  { grep -a "^$1=" "$LOG" | tail -n1 | cut -d= -f2- | tr -d '\r'; }
have() { grep -aq "^$1=" "$LOG"; }
wait_val() { local t=$SECONDS; while (( SECONDS - t < $2 )); do have "$1" && return 0; sleep 2; done; return 1; }
wait_line() { local t=$SECONDS; while (( SECONDS - t < $2 )); do grep -aq "^$1" "$LOG" && return 0; sleep 3; done; return 1; }

DIAGCMD='echo ==LAYOS-DIAG-BEGIN==; getprop | grep -E "init.svc|sys.boot|vold|ro.crypto|bootcomplete"; echo ==LAYOS-PS==; ps -A | head -n 80; echo ==LAYOS-KEYLOG==; logcat -d -b all -t 800 | grep -E "FATAL|AndroidRuntime|SystemServer|Zygote|PackageManager|ActivityManager|Watchdog|dex2oat|died|crash|Exception" | tail -n 80; echo ==LAYOS-LOGCAT==; logcat -d -t 100; echo ==LAYOS-DIAG-END=='
SENT1=0; SENT2=0; SENT3=0

BOOTED=0; n=0
deadline=$((START + TIMEOUT - 20))
while (( SECONDS < deadline )) && kill -0 "$QPID" 2>/dev/null; do
  el=$((SECONDS - START))
  if (( el >= POLL_FROM )); then
    n=$((n + 1))
    ser "echo LAYOS_BC${n}=\$(getprop sys.boot_completed)"
    sleep 3
    if [[ "$(val LAYOS_BC${n})" == "1" ]]; then BOOTED=1; break; fi
  fi
  if (( ! SENT1 && el >= DIAG1 )); then ser "$DIAGCMD"; SENT1=1; fi
  if (( ! SENT2 && el >= DIAG2 )); then ser "$DIAGCMD"; SENT2=1; fi
  if (( ! SENT3 && el >= DIAG3 )); then ser "$DIAGCMD"; SENT3=1; fi
  if (( el >= END_AFTER )); then break; fi
  sleep 12
done
BOOT_WALL=$((SECONDS - START))

REL=""; SDK=""; MODEL=""; WM=""; PKGS=0
INST_OK=0; APP_OK=0; SHOT_OK=0; INSTALL_OUT=""; RES=""; AM_OUT=""; PIDV=""; FOCUS=""; RESUMED=""; LAYERS=""
APPLOG=""; PKGINFO=""; ADBPROPS=""; SHOTSIZE=""; ADBDIAG=""
if (( BOOTED )); then
  ser 'echo LAYOS_REL=$(getprop ro.build.version.release)'
  ser 'echo LAYOS_SDK=$(getprop ro.build.version.sdk)'
  ser 'echo LAYOS_MODEL=$(getprop ro.product.model)'
  ser 'echo LAYOS_WM=$(wm size 2>&1 | tail -n1)'
  wait_val LAYOS_WM 20
  REL="$(val LAYOS_REL)"; SDK="$(val LAYOS_SDK)"; MODEL="$(val LAYOS_MODEL)"; WM="$(val LAYOS_WM)"

  for i in $(seq 1 30); do
    ser "echo LAYOS_PK${i}=\$(pm list packages | wc -l | tr -d ' ')"
    wait_val "LAYOS_PK${i}" 15
    PKGS="$(val LAYOS_PK${i})"
    [[ "$PKGS" =~ ^[0-9]+$ ]] && (( PKGS > 20 )) && break
    PKGS=0
    sleep 4
  done

  # instala o APK que foi colocado em /data/local/tmp/test.apk na imagem
  ser "echo LAYOS_INST=\$(pm install -r /data/local/tmp/test.apk 2>&1 | tr '\\n' ' ')"
  wait_val LAYOS_INST 180
  INSTALL_OUT="$(val LAYOS_INST)"
  grep -q 'Success' <<<"$INSTALL_OUT" && INST_OK=1

  if (( INST_OK )); then
    # concede a permissao de notificacoes antes (senao o Android abre uma caixa de dialogo por cima do app)
    ser "pm grant $APK_PKG android.permission.POST_NOTIFICATIONS >/dev/null 2>&1"
    sleep 2
    # 'am start -a MAIN -c LAUNCHER' nao resolve apps cujo filtro nao tem CATEGORY_DEFAULT;
    # por isso descobrimos o componente com resolve-activity e abrimos com 'am start -n'.
    ser "echo LAYOS_RES=\$(cmd package resolve-activity --brief -a android.intent.action.MAIN -c android.intent.category.LAUNCHER $APK_PKG 2>&1 | tail -n 1)"
    wait_val LAYOS_RES 60
    RES="$(val LAYOS_RES)"
    if [[ "$RES" == */* ]]; then
      ser "echo LAYOS_AM=\$(am start -n $RES 2>&1 | tr '\\n' ' ')"
    else
      ser "echo LAYOS_AM=\$(am start -a android.intent.action.MAIN -c android.intent.category.LAUNCHER -p $APK_PKG 2>&1 | tr '\\n' ' ')"
    fi
    wait_val LAYOS_AM 90
    AM_OUT="$(val LAYOS_AM)"
    sleep 30
    ser "echo LAYOS_PID=\$(pidof $APK_PKG)"
    ser 'echo LAYOS_FOCUS=$(dumpsys window | grep -m1 mCurrentFocus)'
    ser 'echo LAYOS_RESUMED=$(dumpsys activity activities | grep -m1 -E "topResumedActivity|mResumedActivity")'
    ser "echo LAYOS_LAYERS=\$(dumpsys SurfaceFlinger --list | grep -i fdroid | tr '\\n' ' ')"
    wait_val LAYOS_LAYERS 30
    sleep 3

    # captura de tela (nativo, nao precisa de Java): vira texto base64 na serial e e decodificada no host
    ser 'screencap -p /data/local/tmp/screen.png 2>&1; echo LAYOS_SHOTSIZE=$(wc -c < /data/local/tmp/screen.png)'
    wait_val LAYOS_SHOTSIZE 40
    SHOTSIZE="$(val LAYOS_SHOTSIZE)"
    ser 'echo ==LAYOS-SHOT-BEGIN==; base64 /data/local/tmp/screen.png; echo ==LAYOS-SHOT-END=='
    wait_line '==LAYOS-SHOT-END==' 150
    sed -n '/^==LAYOS-SHOT-BEGIN==/,/^==LAYOS-SHOT-END==/p' "$LOG" | tr -d '\r' | grep -E '^[A-Za-z0-9+/=]+$' > "$ROOT/out/screen.b64.log"
    base64 -d "$ROOT/out/screen.b64.log" > "$ROOT/out/screen.png" 2>/dev/null || true
    head -c 8 "$ROOT/out/screen.png" 2>/dev/null | grep -aq 'PNG' && SHOT_OK=1

    ser "echo ==LAYOS-PKGINFO-BEGIN==; dumpsys package $APK_PKG | grep -E 'versionName|primaryCpuAbi|MAIN|LAUNCHER|Activity|enabled=|stopped=' | head -n 25; echo ==LAYOS-PKGINFO-END=="
    ser 'echo ==LAYOS-APPLOG-BEGIN==; logcat -d -b all -t 1500 | grep -E "fdroid|AndroidRuntime|FATAL|ActivityTaskManager" | tail -n 40; echo ==LAYOS-APPLOG-END=='
    sleep 15
    PIDV="$(val LAYOS_PID)"; FOCUS="$(val LAYOS_FOCUS)"; RESUMED="$(val LAYOS_RESUMED)"; LAYERS="$(val LAYOS_LAYERS)"
    [[ "$PIDV" =~ ^[0-9]+ ]] && APP_OK=1
  fi

  # diagnostico do adbd (esta reiniciando em loop?) e das portas TCP
  ser 'echo LAYOS_ADBPROPS=$(getprop | grep -i adb | tr "\n" " ")'
  ser 'echo ==LAYOS-ADBDIAG-BEGIN==; echo svc=$(getprop init.svc.adbd); ps -A | grep -i adbd; (netstat -tln 2>&1 || cat /proc/net/tcp6) | head -n 15; logcat -d -b all | grep -i adbd | tail -n 25; echo ==LAYOS-ADBDIAG-END=='
  wait_line '==LAYOS-ADBDIAG-END==' 40
  ADBPROPS="$(val LAYOS_ADBPROPS)"
  ADBDIAG="$(sed -n '/^==LAYOS-ADBDIAG-BEGIN==/,/^==LAYOS-ADBDIAG-END==/p' "$LOG" | tr -d '\r' | tail -n 50)"
  APPLOG="$(sed -n '/^==LAYOS-APPLOG-BEGIN==/,/^==LAYOS-APPLOG-END==/p' "$LOG" | tr -d '\r' | tail -n 45)"
  PKGINFO="$(sed -n '/^==LAYOS-PKGINFO-BEGIN==/,/^==LAYOS-PKGINFO-END==/p' "$LOG" | tr -d '\r' | tail -n 30)"
fi

# --- adb (para depurar pelo Android Studio no futuro): tenta reconectar algumas vezes ---
HAVE_ADB=0; command -v adb >/dev/null 2>&1 && HAVE_ADB=1
ADB_STATE="n/d"; ADB_SH="n/d"; ADB_OK=0
if (( HAVE_ADB && BOOTED )); then
  for _ in $(seq 1 8); do
    adb disconnect "$SERIAL" >/dev/null 2>&1
    adb kill-server >/dev/null 2>&1
    adb connect "$SERIAL" >/dev/null 2>&1
    sleep 3
    ADB_STATE="$(ADB_T=15 adbx get-state)"
    [[ "$ADB_STATE" == "device" ]] && break
    sleep 5
  done
  ADB_SH="$(ADB_T=20 adbx shell getprop sys.boot_completed)"
  [[ "$ADB_SH" == "1" ]] && ADB_OK=1
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
row "$BOOTED" "boot completo (sys.boot_completed=1, lido no shell do Android)"
row "$(( PKGS > 20 ))" "PackageManager pronto ($PKGS pacotes)"
row "$INST_OK" "APK instalado no Lay-Android (pm install)"
row "$APP_OK" "app abriu e esta em execucao ($APK_PKG)"
row "$SHOT_OK" "captura de tela do app (${SHOTSIZE:-?} bytes) salva em out/screen.png"
row "$ADB_OK" "adb shell funcional (estado: $ADB_STATE)"

INFO=""
[[ -n "$REL" ]] && INFO="Android $REL (SDK $SDK), modelo $MODEL, tela ${WM#*: }, $PKGS pacotes"
[[ -n "$INFO" ]] && echo "INFO    $INFO"

ACCEL="$(grep -a -m1 'aceleracao:' "$LOG" || echo 'aceleracao: desconhecida')"
GUEST="$(grep -a -o '^\[ *[0-9]*\.[0-9]*\]' "$LOG" | tail -n1 | tr -d '[] ')"
SERVICES="$(grep -a -o "starting service '[^']*'" "$LOG" | uniq | sed "s/starting service //" | head -n 60 | tr '\n' ' ')"
ERRS="$(grep -a -E 'init: .*(fail|Fail|error|Error|denied|Unable|Could not|cannot|timed out|Wait for|wait_for)' "$LOG" | grep -a -v Untracked | head -n 30)"
CRASHES="$(grep -a -E 'received signal|Aborted|Fatal signal|SIGSEGV' "$LOG" | head -n 10)"
INNER="$(awk '/^==LAYOS-DIAG-BEGIN==/{buf=""} {buf=buf $0 "\n"} /^==LAYOS-DIAG-END==/{last=buf} END{printf "%s", last}' "$LOG" | tr -d '\r')"
SVCSTATE="$(grep -E '^\[init\.svc\.' <<<"$INNER" | sort -u | grep -v -E '\[(stopped)\]' | head -n 90)"
PSOUT="$(sed -n '/^==LAYOS-PS==/,/^==LAYOS-KEYLOG==/p' <<<"$INNER" | tail -n 85)"
KEYLOG="$(sed -n '/^==LAYOS-KEYLOG==/,/^==LAYOS-LOGCAT==/p' <<<"$INNER" | tail -n 85)"
REASON="fim do teste em ${BOOT_WALL}s sem boot completo"; (( BOOTED )) && REASON="boot completo em ${BOOT_WALL}s"

echo "== Diagnostico =="
echo "$ACCEL | $REASON | ultimo log: ${GUEST:-?}s"
echo "Instalacao do APK: ${INSTALL_OUT:-nao tentada}"
echo "Componente: ${RES:-n/d} | am start: ${AM_OUT:-n/d}"
echo "PID do app: ${PIDV:-n/d}"
echo "Foco da janela: ${FOCUS:-n/d}"
echo "Atividade em primeiro plano: ${RESUMED:-n/d}"
echo "Camadas do SurfaceFlinger (fdroid): ${LAYERS:-nenhuma}"
echo "adb: estado=$ADB_STATE shell=$ADB_SH | props: ${ADBPROPS:-n/d}"

{
  echo "### Lay-Android: boot do Android + instalacao de APK"
  echo ""
  echo "| Estado | Etapa |"
  echo "|---|---|"
  printf '%s\n' "${ROWS[@]}"
  [[ -n "$INFO" ]] && { echo ""; echo "**$INFO**"; }
  echo ""
  echo "- $ACCEL"
  echo "- $REASON (ultimo log do kernel: ${GUEST:-?}s)"
  echo "- Instalacao do APK: ${INSTALL_OUT:-nao tentada}"
  echo "- Componente do app: \`${RES:-n/d}\` | am start: ${AM_OUT:-n/d}"
  echo "- Camadas de tela (SurfaceFlinger) do app: ${LAYERS:-nenhuma}"
  echo "- Atividade em primeiro plano: ${RESUMED:-n/d}"
  echo "- adb: estado=\`$ADB_STATE\` | resposta do shell=\`$ADB_SH\`"
  echo "- Propriedades adb do Android: ${ADBPROPS:-n/d}"
  echo ""
  if (( BOOTED )); then
    echo "<details open><summary>adbd: estado, portas TCP e log (por que o adb esta offline?)</summary>"
    echo ""
    echo '```'
    echo "${ADBDIAG:-sem dados}"
    echo '```'
    echo "</details>"
    echo ""
  fi
  if (( INST_OK )); then
    echo "<details><summary>App: PID, janela em foco e log</summary>"
    echo ""
    echo '```'
    echo "PID: ${PIDV:-n/d}"
    echo "${FOCUS:-}"
    echo "--- logcat (fdroid / AndroidRuntime / FATAL) ---"
    echo "${APPLOG:-sem dados}"
    echo '```'
    echo "</details>"
    echo ""
  fi
  if (( ! BOOTED )); then
    echo "<details open><summary>Erros do Android (logcat filtrado)</summary>"
    echo ""
    echo '```'
    echo "${KEYLOG:-sem dados}"
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
    echo "<details><summary>Servicos que NAO estao parados</summary>"
    echo ""
    echo '```'
    echo "${SVCSTATE:-sem dados}"
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

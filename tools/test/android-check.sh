#!/usr/bin/env bash
# Inspeciona o rootfs Android e confirma os componentes que o Lay-Android precisa.
# Resultado: out/android-check.log. Retorna erro se faltar algo essencial.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RFS="$ROOT/out/android-rootfs"
LOG="$ROOT/out/android-check.log"
: > "$LOG"

missing=0
check() { # descricao caminho
  if sudo test -e "$RFS/$2"; then echo "OK      $1 ($2)" | tee -a "$LOG"
  else echo "FALTA   $1 ($2)" | tee -a "$LOG"; missing=1; fi
}

echo "== Componentes essenciais ==" | tee -a "$LOG"
check "init do Android"            init
check "app_process 64 bits"        system/bin/app_process64
check "servicemanager (Binder)"   system/bin/servicemanager
check "zygote"                     system/bin/app_process
check "Bionic libc"                system/lib64/libc.so
check "PackageManager (pm)"       system/bin/pm
check "ActivityManager (am)"      system/bin/am
check "framework.jar"              system/framework/framework.jar
check "servidor do sistema"       system/framework/services.jar

echo "== ART (runtime) ==" | tee -a "$LOG"
if sudo find "$RFS/apex" "$RFS/system/apex" -maxdepth 1 -iname '*art*' 2>/dev/null | grep -q .; then
  echo "OK      APEX do ART encontrado" | tee -a "$LOG"
else
  echo "FALTA   APEX do ART" | tee -a "$LOG"; missing=1
fi

echo "== Tamanhos ==" | tee -a "$LOG"
for d in system vendor apex product system_ext; do
  if sudo test -d "$RFS/$d"; then
    echo "$d: $(sudo du -sh "$RFS/$d" | cut -f1)" | tee -a "$LOG"
  fi
done

if [[ $missing -eq 0 ]]; then echo "RESULTADO: rootfs Android completo" | tee -a "$LOG"
else echo "RESULTADO: faltam componentes (veja acima)" | tee -a "$LOG"; fi
exit $missing

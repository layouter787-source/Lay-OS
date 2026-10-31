#!/usr/bin/env bash
# Gera um initramfs com BusyBox + um init escolhido.
#   LAYOS_INIT           script de init (padrao: system/init)
#   LAYOS_INITRAMFS_OUT  arquivo de saida (padrao: out/initramfs.cpio.gz)
#   LAYOS_RFS_NAME       nome da pasta temporaria em out/ (padrao: rootfs)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="$ROOT/out"
INIT_SRC="${LAYOS_INIT:-$ROOT/system/init}"
OUT_FILE="${LAYOS_INITRAMFS_OUT:-$OUT/initramfs.cpio.gz}"
RFS="$OUT/${LAYOS_RFS_NAME:-rootfs}"

BUSYBOX="${BUSYBOX:-$(command -v busybox || true)}"
if [[ -z "$BUSYBOX" ]]; then
  echo "erro: busybox nao encontrado. Instale 'busybox-static' (Debian/Ubuntu) ou defina BUSYBOX=/caminho." >&2
  exit 1
fi

rm -rf "$RFS"
mkdir -p "$RFS"/{bin,sbin,usr/bin,usr/sbin,proc,sys,dev,run,tmp,root,etc,android}

cp "$BUSYBOX" "$RFS/bin/busybox"
chmod +x "$RFS/bin/busybox"

# Cria os links dos applets dentro do rootfs
(cd "$RFS" && ./bin/busybox --list-full 2>/dev/null | while read -r applet; do
  [[ "$applet" == "bin/busybox" ]] && continue
  mkdir -p "$(dirname "$applet")"
  depth=$(awk -F/ '{print NF-1}' <<<"$applet")
  rel=""
  for ((i=0; i<depth; i++)); do rel+="../"; done
  ln -sf "${rel}bin/busybox" "$applet"
done)

cp "$INIT_SRC" "$RFS/init"
chmod +x "$RFS/init"

echo "layos" > "$RFS/etc/hostname"
echo "root:x:0:0:root:/root:/bin/sh" > "$RFS/etc/passwd"

mkdir -p "$(dirname "$OUT_FILE")"
(cd "$RFS" && find . | cpio -o -H newc --quiet | gzip -9) > "$OUT_FILE"

echo "ok: $OUT_FILE ($(du -h "$OUT_FILE" | cut -f1))"

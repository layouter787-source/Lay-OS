#!/usr/bin/env bash
# Gera out/initramfs.cpio.gz com BusyBox + init do Lay-OS.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="$ROOT/out"
RFS="$OUT/rootfs"

BUSYBOX="${BUSYBOX:-$(command -v busybox || true)}"
if [[ -z "$BUSYBOX" ]]; then
  echo "erro: busybox nao encontrado. Instale 'busybox-static' (Debian/Ubuntu) ou defina BUSYBOX=/caminho." >&2
  exit 1
fi

rm -rf "$RFS"
mkdir -p "$RFS"/{bin,sbin,usr/bin,usr/sbin,proc,sys,dev,run,tmp,root,etc}

cp "$BUSYBOX" "$RFS/bin/busybox"
chmod +x "$RFS/bin/busybox"

# Cria os links dos applets dentro do rootfs
(cd "$RFS" && ./bin/busybox --list-full 2>/dev/null | while read -r applet; do
  [[ "$applet" == "bin/busybox" ]] && continue
  mkdir -p "$(dirname "$applet")"
  # caminho relativo ate /bin/busybox
  depth=$(awk -F/ '{print NF-1}' <<<"$applet")
  rel=""
  for ((i=0; i<depth; i++)); do rel+="../"; done
  ln -sf "${rel}bin/busybox" "$applet"
done)

cp "$ROOT/system/init" "$RFS/init"
chmod +x "$RFS/init"

echo "layos" > "$RFS/etc/hostname"
echo "root:x:0:0:root:/root:/bin/sh" > "$RFS/etc/passwd"

(cd "$RFS" && find . | cpio -o -H newc --quiet | gzip -9) > "$OUT/initramfs.cpio.gz"

echo "ok: $OUT/initramfs.cpio.gz ($(du -h "$OUT/initramfs.cpio.gz" | cut -f1))"

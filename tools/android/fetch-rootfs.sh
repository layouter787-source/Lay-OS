#!/usr/bin/env bash
# Baixa um rootfs Android de referencia (redroid, AOSP em container) para estudar/bootar no Lay-Android.
# Resultado: out/android-rootfs/
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
IMG="${LAYOS_ANDROID_IMAGE:-redroid/redroid:13.0.0-latest}"
DEST="$ROOT/out/android-rootfs"

command -v docker >/dev/null || { echo "erro: docker nao encontrado" >&2; exit 1; }

sudo rm -rf "$DEST"
mkdir -p "$DEST"

echo ">> baixando $IMG"
docker pull "$IMG"

cid="$(docker create "$IMG" /init)"
trap 'docker rm -f "$cid" >/dev/null 2>&1 || true' EXIT

echo ">> extraindo rootfs"
docker export "$cid" | sudo tar -x -C "$DEST" --exclude='dev/*' --exclude='proc/*' --exclude='sys/*'

echo "ok: $DEST ($(sudo du -sh "$DEST" | cut -f1))"

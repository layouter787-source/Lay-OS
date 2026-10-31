#!/usr/bin/env bash
# Pipeline completo de CI do Lay-OS. Chamado pelo workflow do GitHub Actions.
# Mantem a logica no repositorio: o workflow so precisa executar este script.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

echo "::group::Dependencias"
sudo apt-get update
sudo apt-get install -y qemu-system-x86 busybox-static cpio e2fsprogs \
  build-essential bc bison flex libssl-dev libelf-dev xz-utils curl
sudo apt-get install -y adb || echo "::warning::pacote adb indisponivel; a verificacao por adb sera pulada"
sudo chmod 666 /dev/kvm 2>/dev/null || true
echo "::endgroup::"

chmod +x tools/build/*.sh tools/qemu/*.sh tools/test/*.sh tools/ci/*.sh tools/android/*.sh system/init system/init-android

echo "::group::initramfs"
./tools/build/build-initramfs.sh
echo "::endgroup::"

echo "::group::Kernel Lay-OS"
./tools/build/build-kernel.sh
echo "::endgroup::"

export LAYOS_KERNEL="$ROOT/out/bzImage"

echo "::group::Teste de boot"
./tools/test/boot-test.sh
echo "::endgroup::"

echo "::group::Teste do kernel (Binder, namespaces, cgroups)"
./tools/test/kernel-test.sh
echo "::endgroup::"

# Libera espaco em disco antes da etapa Android
rm -rf out/linux-*/ out/linux-*.tar.xz

# --- Lay-Android (experimental: falha aqui vira aviso, nao derruba o pipeline) ---
echo "::group::Lay-Android: rootfs de referencia"
if ./tools/android/fetch-rootfs.sh; then
  ./tools/test/android-check.sh || echo "::warning::Lay-Android: faltam componentes no rootfs (veja acima)"
else
  echo "::warning::Lay-Android: nao foi possivel obter o rootfs Android"
fi
echo "::endgroup::"

if sudo test -d out/android-rootfs; then
  echo "::group::Lay-Android: boot do Android no kernel do Lay-OS (etapa 2)"
  docker image prune -af >/dev/null 2>&1 || true
  if ./tools/android/make-disk.sh && ./tools/test/android-boot-test.sh; then
    echo "Lay-Android: o init do Android rodou sobre o kernel do Lay-OS"
  else
    echo "::warning::Lay-Android: o boot do Android ainda nao funcionou (veja out/android-boot.log)"
  fi
  echo "::endgroup::"
fi

echo "Pipeline concluido com sucesso"

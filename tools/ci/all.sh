#!/usr/bin/env bash
# Pipeline completo de CI do Lay-OS. Chamado pelo workflow do GitHub Actions.
# Mantem a logica no repositorio: o workflow so precisa executar este script.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

echo "::group::Dependencias"
sudo apt-get update
sudo apt-get install -y qemu-system-x86 busybox-static cpio \
  build-essential bc bison flex libssl-dev libelf-dev xz-utils curl
echo "::endgroup::"

chmod +x tools/build/*.sh tools/qemu/*.sh tools/test/*.sh tools/ci/*.sh system/init

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

echo "Pipeline concluido com sucesso"

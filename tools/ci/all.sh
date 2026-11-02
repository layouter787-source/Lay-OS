#!/usr/bin/env bash
# Pipeline completo de CI do Lay-OS. Chamado pelo workflow do GitHub Actions.
# Mantem a logica no repositorio: o workflow so precisa executar este script.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

SUMMARY="${GITHUB_STEP_SUMMARY:-/dev/null}"

echo "::group::Dependencias"
sudo apt-get update
sudo apt-get install -y qemu-system-x86 qemu-system-arm busybox-static cpio e2fsprogs \
  build-essential bc bison flex libssl-dev libelf-dev xz-utils curl bzip2 file \
  gcc-aarch64-linux-gnu
sudo apt-get install -y adb || echo "::warning::pacote adb indisponivel; a verificacao por adb sera pulada"
sudo chmod 666 /dev/kvm 2>/dev/null || true
echo "::endgroup::"

chmod +x tools/build/*.sh tools/qemu/*.sh tools/test/*.sh tools/ci/*.sh tools/android/*.sh system/init system/init-android

echo "::group::initramfs"
./tools/build/build-initramfs.sh
echo "::endgroup::"

echo "::group::Kernel Lay-OS (x86_64)"
./tools/build/build-kernel.sh
echo "::endgroup::"

export LAYOS_KERNEL="$ROOT/out/bzImage"

echo "::group::Teste de boot"
./tools/test/boot-test.sh
echo "::endgroup::"

echo "::group::Teste do kernel (Binder, namespaces, cgroups)"
./tools/test/kernel-test.sh
echo "::endgroup::"

# Libera espaco em disco antes das etapas seguintes
rm -rf out/linux-*/ out/linux-*.tar.xz

# --- arm64 (experimental): primeiro passo rumo a celulares reais ---
# Roda antes do Android porque e mais rapido e o erro (se houver) aparece no Summary.
echo "::group::Lay-OS arm64 (QEMU virt)"
run_arm64() {
  ./tools/build/build-busybox-arm64.sh && ./tools/build/build-kernel-arm64.sh && ./tools/test/arm64-boot-test.sh
}
if run_arm64 2>&1 | tee out/arm64-build.log; then
  echo "Lay-OS arm64: iniciou no QEMU"
else
  echo "::warning::Lay-OS arm64: etapa experimental falhou (veja o Summary)"
  {
    echo "### Lay-OS arm64: a etapa falhou"
    echo ""
    echo "<details open><summary>Ultimas linhas do log (inclui o erro)</summary>"
    echo ""
    echo '```'
    tail -n 45 out/arm64-build.log
    echo '```'
    echo "</details>"
  } >> "$SUMMARY"
fi
echo "::endgroup::"
rm -rf out/linux-arm64-*/ out/linux-*.tar.xz out/busybox-1*/

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

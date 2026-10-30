# Lay-OS

Sistema operacional móvel **aberto, leve e poderoso**, com compatibilidade nativa com aplicativos Android (APK release e debug, Android Studio, Qt for Android, NDK) e suporte a apps Linux nativos (Qt, GTK, Rust, C++). Apps iOS estão fora do escopo.

> Status: **Fase 0/1**: arquitetura definida, ambiente de boot em QEMU em construção.

## Princípios

1. **Aberto de verdade**: 100% código livre, sem serviços Google obrigatórios, bootloader desbloqueável, usuário com controle total.
2. **Leve**: metas mensuráveis (ver `docs/ARCHITECTURE.md`): boot < 15 s e < 800 MB de RAM ociosa.
3. **Poderoso**: dois runtimes de aplicativos lado a lado (Linux nativo + Android), sem emulação de CPU.
4. **Seguro por padrão**: sandbox por app, permissões granulares, atualizações assinadas e atômicas.

## Arquitetura em uma imagem

```
┌──────────────────────────────────────────────────────────┐
│  Lay Shell (launcher, status bar, notificações, teclado) │
├───────────────────────────┬──────────────────────────────┤
│  Apps nativos             │  Lay-Android (runtime ART)   │
│  Qt / GTK / Rust / C++    │  APK, APK debug, NDK, JNI    │
├───────────────────────────┴──────────────────────────────┤
│  Lay Compositor (Wayland) · Lay Services (D-Bus/IPC)     │
├──────────────────────────────────────────────────────────┤
│  Init (PID 1) · Gerenciador de pacotes · Atualizações    │
├──────────────────────────────────────────────────────────┤
│  Kernel Linux (+ Binder) · Drivers · Firmware            │
└──────────────────────────────────────────────────────────┘
```

Detalhes em [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md), compatibilidade Android em [`docs/ANDROID_COMPAT.md`](docs/ANDROID_COMPAT.md) e plano em [`docs/ROADMAP.md`](docs/ROADMAP.md).

## Estrutura do repositório

| Pasta | Conteúdo |
|---|---|
| `kernel/` | Configuração e patches do kernel Linux |
| `system/` | Init, serviços base e rootfs mínimo |
| `shell/` | Lay Shell e compositor Wayland |
| `android-layer/` | Lay-Android: runtime ART, Binder, HALs, bridge de janelas |
| `sdk/` | SDK para desenvolvedores (nativo e Android) |
| `tools/` | Scripts de build e execução (QEMU) |
| `docs/` | Documentação e decisões de arquitetura |

## Começando (Fase 1: boot em QEMU)

Requisitos no host Linux: `qemu-system-x86_64`, `busybox-static`, `cpio`, `gzip`, um kernel em `/boot/vmlinuz-*` (ou defina `LAYOS_KERNEL`).

```bash
./tools/build/build-initramfs.sh   # gera out/initramfs.cpio.gz
./tools/qemu/run.sh                # inicia o Lay-OS no QEMU
```

Você deve ver o banner do Lay-OS e um shell mínimo.

## Licença

Código do Lay-OS sob licença MIT (ver `LICENSE`). O kernel Linux e componentes de terceiros mantêm suas próprias licenças (kernel: GPL-2.0).

# Arquitetura do Lay-OS

## 1. Objetivos mensuráveis

| Meta | Alvo inicial |
|---|---|
| Tempo de boot até o launcher | < 15 s |
| RAM ociosa (sem apps Android ativos) | < 800 MB |
| RAM ociosa (com runtime Android iniciado) | < 1,3 GB |
| Imagem do sistema base | < 2 GB |
| Compatibilidade APK (top 100 apps) | > 90% (sem Google Play Services) |

## 2. Decisão central: dual runtime

O Lay-OS **não** é um fork do AOSP. Ele é um sistema Linux próprio com dois runtimes de aplicativos que compartilham o mesmo kernel, compositor e serviços:

1. **Runtime nativo**: apps Wayland (Qt, GTK, Rust, C++), empacotados em bundles sandboxed.
2. **Lay-Android**: ART + Bionic + framework Android reduzido, rodando em namespaces Linux (sem VM e sem emulação de CPU). Executa qualquer APK (release/debug), incluindo apps criados no Android Studio e Qt for Android, e bibliotecas nativas via NDK/JNI.

Motivo: abertura e controle do Linux + ecossistema do Android, sem depender do Google.

## 3. Camadas

### 3.1 Kernel
- Linux LTS (mainline), configuração mínima por dispositivo em `kernel/configs/`.
- Módulos obrigatórios para o Lay-Android: `binder_linux`, `ashmem`/memfd, `ipc namespaces`, `cgroups v2`.
- Segurança: seccomp, Landlock, namespaces de usuário, dm-verity para partição de sistema.

### 3.2 Init e serviços
- PID 1 enxuto (BusyBox init na Fase 1; migração para s6/systemd-lite na Fase 3).
- Serviços: rede (iwd/NetworkManager), áudio (PipeWire), energia, telefonia (oFono/ModemManager), sensores, Bluetooth.
- IPC: D-Bus para serviços nativos; Binder para Lay-Android; ponte entre ambos (`lay-bridge`).

### 3.3 Gráficos e entrada
- **Lay Compositor**: compositor Wayland próprio (Rust + smithay), gestos touch, multitarefa em cards, janelas livres em modo desktop/dock.
- Janelas de apps Android são exportadas como superfícies Wayland (via `android-layer/window-bridge`).
- GPU: Mesa (Vulkan/GL ES) no host; passagem direta para o Lay-Android (sem virtualização).

### 3.4 Lay-Android
Ver [ANDROID_COMPAT.md](ANDROID_COMPAT.md).

### 3.5 Pacotes e atualizações
- Sistema imutável (rootfs somente leitura, A/B) com atualizações atômicas e rollback.
- Apps nativos: bundles (estilo Flatpak/OCI) assinados.
- Apps Android: instalação de APK local, repositórios abertos (F-Droid compatível), loja Lay Store (Fase 4).

### 3.6 Segurança e privacidade
- Cada app roda em sandbox próprio (namespaces + seccomp + Landlock).
- Permissões granulares e revogáveis (localização aproximada, câmera, microfone, contatos, rede por app).
- Sem telemetria por padrão; microG opcional para apps que exigem serviços Google.
- Secure boot com chaves do usuário; bootloader desbloqueável.

## 4. Shell (Lay Shell)
Launcher, barra de status, central de controle, notificações, teclado, configurações, bloqueio de tela. Escrito em Rust/Qt Quick, com temas e layout configuráveis.

## 5. Estratégia de dispositivos
1. **QEMU x86_64/aarch64**: desenvolvimento diário.
2. **1 dispositivo de referência** com suporte mainline Linux (decisão pendente: Pixel ou aparelho suportado pelo postmarketOS).
3. Expansão gradual via porte de device trees e HALs.

## 6. Decisões em aberto
- [ ] Dispositivo de referência
- [ ] Init definitivo (s6 vs systemd)
- [ ] Base do Lay-Android (AOSP mínimo próprio vs reaproveitar Waydroid)
- [ ] Licença do SDK

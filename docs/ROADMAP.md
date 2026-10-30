# Roadmap

## Fase 0: Fundação (atual)
- [x] Visão, arquitetura e metas
- [x] Estrutura do repositório e licença
- [ ] Definir dispositivo de referência

## Fase 1: Boot em QEMU
- [x] Script de init e initramfs mínimo
- [x] Script de execução em QEMU
- [ ] Kernel Lay-OS configurado (`kernel/configs/`)
- [ ] Rootfs com BusyBox + shell interativo
- [ ] CI (GitHub Actions) que gera e testa a imagem

## Fase 2: Interface gráfica
- [ ] Compositor Wayland (Rust/smithay)
- [ ] Lay Shell: launcher, barra de status, gestos
- [ ] Teclado virtual e entrada touch
- [ ] Rodar um app Qt nativo

## Fase 3: Dispositivo real
- [ ] Boot em 1 aparelho de referência
- [ ] Wi-Fi, Bluetooth, áudio, bateria, brilho
- [ ] Telefonia e SMS
- [ ] Atualizações atômicas A/B

## Fase 4: Compatibilidade Android
- [ ] Binder + ART em container
- [ ] Instalar e abrir um APK simples
- [ ] Window/Input bridge
- [ ] `lay-adbd`: depuração via Android Studio
- [ ] Suíte de compatibilidade (top apps)

## Fase 5: Ecossistema
- [ ] Lay Store e repositórios abertos
- [ ] SDK e documentação para desenvolvedores
- [ ] Câmera, GPS, sensores, NFC
- [ ] Versão beta pública

# Compatibilidade Android (Lay-Android)

## Escopo
Suporta todo app Android: APK de release e **debug**, apps feitos no Android Studio (Kotlin/Java), Qt Creator (Qt for Android), NDK/JNI (`.so` ARM/x86), Flutter, React Native, Unity e jogos. **Não** suporta iOS.

## Componentes
| Componente | Função |
|---|---|
| ART (Android Runtime) | Executa DEX/OAT dos apps |
| Bionic | libc usada por apps e libs nativas |
| Binder + servicemanager | IPC do framework Android (módulo de kernel) |
| Framework reduzido | ActivityManager, PackageManager, WindowManager (ponte), notificações |
| Window bridge | Converte superfícies Android em janelas Wayland |
| Input bridge | Touch, teclado, gamepad, sensores |
| HAL bridge | Câmera, áudio, GPU, sensores, GPS, telefonia mapeados para os serviços do Lay-OS |
| Compat de serviços Google | microG opcional (Play Services substituto) |

## Fluxo de instalação de um APK
1. `lay-pkg install app.apk` (ou toque no arquivo)
2. Verificação de assinatura e análise de permissões
3. Instalação no PackageManager do Lay-Android com UID isolado
4. Ícone aparece no Lay Shell como app comum

## Depuração (APK debug / Android Studio)
- `adb` compatível exposto via `lay-adbd` (USB e Wi-Fi)
- Android Studio e Qt Creator veem o Lay-OS como um dispositivo Android comum
- Suporte a `run-as`, logcat, profilers e attach de debugger nativo (lldb/gdb)

## Riscos e mitigação
| Risco | Mitigação |
|---|---|
| Apps que exigem Play Integrity/SafetyNet | Fora do escopo; documentar lista de apps incompatíveis |
| Desempenho de jogos 3D | Passagem direta de GPU, sem virtualização |
| Arquitetura ARM vs x86 | Tradução (libhoudini-like/Box64) apenas quando necessário |
| Consumo de RAM | Runtime Android sob demanda (inicia ao abrir o 1º APK) |

## Estratégia de implementação
1. Prototipar com Waydroid/LXC em Linux desktop.
2. Extrair e reduzir o framework para o mínimo necessário.
3. Integrar Binder, janelas e entrada ao compositor do Lay-OS.
4. Testar com suíte de apps (ver `docs/ROADMAP.md`).

# Lay-Android: runtime Android

## Estrategia

Construir o AOSP inteiro nao e viavel em CI gratuito (centenas de GB e horas de build). Por isso o Lay-Android evolui em etapas, partindo de um rootfs Android de referencia (redroid, AOSP 13 empacotado para rodar sobre kernel Linux com Binder) e substituindo/reduzindo pecas aos poucos.

## Etapas

| # | Etapa | Estado |
|---|---|---|
| 1 | Obter o rootfs Android e validar os componentes (ART, zygote, servicemanager, framework) | em CI (experimental) |
| 2 | Subir o Android dentro do kernel do Lay-OS no QEMU: binderfs, namespaces, `init` do Android | proximo |
| 3 | Instalar e abrir um APK simples (`pm install`, `am start`) | |
| 4 | window-bridge: superficies Android -> Wayland | |
| 5 | input-bridge e hal-bridge | |
| 6 | Reduzir o framework (remover o que nao for necessario) | |
| 7 | `lay-adbd`: depuracao pelo Android Studio / Qt Creator | |

## Requisitos do kernel (ja cobertos em `kernel/configs/layos_x86.fragment`)

- Binder + binderfs (`binder`, `hwbinder`, `vndbinder`)
- Namespaces (pid, ipc, uts, net, user) e cgroups v2
- memfd (substitui ashmem em Android 12+)
- Sem SELinux enforcing na fase inicial (politica propria do Lay-OS depois)

## Regra de produto

Apps Android aparecem como apps normais, sem selos (ver `docs/DESIGN_PRINCIPLES.md`).

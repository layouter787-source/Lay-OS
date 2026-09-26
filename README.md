# Lay OS

Sistema operacional próprio, desenvolvido do zero.

## Princípios
- Kernel e arquitetura próprios
- Baixo consumo de RAM, CPU e armazenamento
- Inicialização rápida
- Componentes modulares carregados conforme a necessidade
- Segurança por capacidades e isolamento
- Uma única edição adaptativa do Lay OS

## Primeira plataforma
- x86_64
- Inicialização via firmware/BIOS no protótipo inicial
- Kernel freestanding

## Estrutura inicial
- `boot/` — código de inicialização
- `kernel/` — núcleo do sistema
- `arch/x86_64/` — código específico da arquitetura
- `docs/` — decisões e arquitetura

A implementação começará pelo primeiro marco: firmware → bootloader Lay → entrada do kernel → execução segura do kernel.

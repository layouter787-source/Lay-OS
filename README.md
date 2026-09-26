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

## Estado atual

O primeiro núcleo funcional já possui:
- entrada x86_64 em long mode
- GDT controlada pelo kernel
- IDT com exceção e IRQs
- PIC remapeado
- PIT a 100 Hz
- console VGA
- alocador físico inicial de páginas
- base de scheduler orientado por ticks

### Próximas camadas
1. allocator de memória dinâmica
2. gerenciamento de processos/threads
3. troca de contexto preemptiva
4. syscalls e IPC
5. driver de armazenamento
6. filesystem LayFS
7. drivers de entrada e vídeo
8. compositor e Lay UI

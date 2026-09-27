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

## Estado atual — marco de 40%
- Boot BIOS x86_64 em long mode
- Carregamento de kernel em janela de desenvolvimento de 1024 setores
- Page tables de identidade para os primeiros 1 GiB
- GDT própria com segmentos de kernel, user mode e TSS
- IDT, PIC e PIT a 100 Hz
- Scheduler round-robin preemptivo
- Context switch restaurável
- Allocator inicial de páginas
- IPC por mailbox
- Syscalls via int 0x80
- Primeiro processo em Ring 3
- Código e stack de user mode com páginas marcadas como U/S
- Console VGA + porta de debug do QEMU
- CI com build e smoke test de execução

## Próximas camadas
1. Gerenciamento real de processos, threads e address spaces
2. Page-frame allocator baseado no mapa de memória do firmware
3. VFS e LayFS
4. Drivers de armazenamento e entrada
5. Rede TCP/IP e sockets
6. Graphics/compositor/Lay UI
7. Shell e serviços do sistema
8. Formato .layapp, SDK e pacote de aplicativos
9. Atualizações, recovery e suporte ARM64

O percentual é um marcador de progresso do projeto, não uma alegação de que o sistema já é um desktop completo.


## 80% milestone
Persistent ATA PIO block storage is validated by the QEMU CI data disk.

## CI trigger
QEMU persistent-storage smoke test trigger.

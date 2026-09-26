# Lay OS — Marco de 40%

Este marco considera o núcleo de execução dividido em camadas verificáveis.

## Entregas concluídas

- Boot BIOS x86_64 com leitura de uma janela de 1024 setores por transferências de até 127 setores.
- Entrada em long mode e page tables iniciais de identidade.
- GDT própria do kernel com segmentos de kernel, user mode e TSS.
- IDT, PIC, PIT e interrupções de timer/teclado.
- Scheduler round-robin preemptivo com contexto restaurável.
- Memória com allocator inicial de páginas acima das estruturas de boot.
- IPC por mailbox fixo.
- ABI de syscalls via int 0x80.
- Primeiro processo de user mode em Ring 3.
- Endereço virtual separado para código e stack do primeiro processo.
- Syscalls demonstradas: GETPID, GETTICKS, IPC_SEND, IPC_RECV, WRITE_CHAR e YIELD.
- Console VGA com espelhamento para a porta de debug do QEMU.
- Pipeline de build automático e smoke test de boot.

## Limites ainda abertos

Este marco ainda não inclui filesystem persistente, drivers de armazenamento completos, rede TCP/IP, áudio, compositor gráfico, shell, gerenciador de pacotes, formato .layapp, atualizações, recovery ou suporte ARM64.

O percentual é um marcador de progresso do projeto, não uma alegação de que o sistema já é um desktop completo.

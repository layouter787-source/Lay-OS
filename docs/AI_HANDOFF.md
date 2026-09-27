# Registro de handoff entre IAs - Lay OS

Este arquivo existe para que qualquer IA (Claude, ChatGPT, etc.) que
continue o desenvolvimento do Lay OS saiba exatamente onde o trabalho
parou e o que vem a seguir, sem precisar reconstruir o contexto do zero.
Ao terminar uma sessao de trabalho, adicione uma nova secao "Entrada"
no topo (mais recente primeiro) e nao apague as entradas antigas.

---

## Entrada - 2026-09-27 (Claude, via GitHub direto no repo)

### O que foi feito
- Deteccao de memoria real via BIOS E820 (`boot/boot.asm`, rotina
  `detect_memory`), chamada em modo real logo apos o carregamento do
  kernel do disco e antes da troca para modo protegido. O boot loader
  grava:
  - contagem de entradas (u16) em `0x8FF0`
  - ate 64 entradas E820 cruas de 24 bytes a partir de `0x9000`
  - contrato completo documentado em `docs/MEMORY_MAP.md`
- `kernel/memory.rs` reescrito: o allocator de paginas deixou de ser um
  bump allocator com janela fixa (2..32 MiB) e passou a ser um bitmap
  allocator que:
  - le o mapa E820 gravado pelo boot loader
  - libera apenas regioes tipo 1 (usavel) acima de 2 MiB
    (`RESERVED_BELOW`), preservando kernel, page tables iniciais e
    estruturas de boot
  - rastreia ate 128 MiB de RAM (`MAX_FRAMES = 32768` paginas de 4 KiB)
  - cai de volta para a janela fixa antiga se nao houver mapa de memoria
    (BIOS sem E820, ou boot loader antigo)
  - ganhou `free_page`, `free_bytes`, `total_bytes` (antes so existiam
    `alloc_page`/`used_bytes`)
- `kernel/console.rs`: novo `write_dec` para imprimir numeros em decimal
  (so existia `write_hex`).
- `kernel/main.rs`: a linha de boot da memoria agora mostra a RAM
  detectada em KiB.
- `kernel/shell.rs`: novo comando `meminfo` no Lay Shell (mostra
  total/usado/livre em KiB).

### Por que
Esta e a "camada 2" do roadmap do README ("Page-frame allocator baseado
no mapa de memoria do firmware"), a proxima logo depois do allocator
inicial fixo que ja existia (marco de 40%).

### Estado depois desta mudanca
- O allocator agora reflete a RAM real da maquina/QEMU (ate o teto de
  128 MiB rastreado), em vez de sempre assumir uma janela fixa de 30 MiB.
- `free_page` existe na API mas ainda nao e chamado por nenhum outro
  subsistema do kernel - nenhum processo/VFS libera paginas ainda. Isso e
  esperado nesta fase.
- Nao validei rodando em QEMU real nesta sessao (sem acesso a um
  ambiente de build/execucao a partir daqui); a revisao foi por leitura
  cuidadosa do codigo e comparacao com o padrao E820 conhecido. Rodar o
  `make` do projeto e testar no QEMU antes de seguir em frente e o
  primeiro passo recomendado para quem pegar isso a seguir.

### Proximos passos sugeridos (ordem sugerida)
1. **Validar em build real**: rodar `make` (ver `Makefile`) e o boot no
   QEMU, conferir a linha `memory: page allocator ready (N KiB usable)` e
   testar o comando `meminfo` no Lay Shell.
2. Usar `memory::alloc_page` de verdade para tabelas de pagina e pilhas
   de novos processos em `kernel/user.rs` / `kernel/process.rs` (hoje
   usam enderecos fixos hardcoded em vez do allocator).
3. Gerenciamento real de processos/threads (item 1 do roadmap do
   README): hoje `kernel/process.rs` so reconhece o pid 1 fixo.
4. VFS/LayFS persistente: conferir se `kernel/vfs.rs` e `kernel/block.rs`
   hoje persistem em disco de verdade ou só em RAM, e evoluir para um
   formato de disco real.
5. Se for necessario testar com mais de 128 MiB de RAM, aumentar
   `MAX_FRAMES` em `kernel/memory.rs` ou tornar o bitmap de tamanho
   dinamico (hoje é um array estatico fixo).

### Decisoes que quem continuar deve conhecer
- O contrato de enderecos fixos (0x8FF0/0x9000) para o mapa de memoria e
  deliberadamente simples porque o boot loader (NASM) e o kernel (Rust
  `no_std`) sao compilados separadamente e nao compartilham um struct de
  "boot info". Se um protocolo de boot mais robusto for adicionado no
  futuro (Multiboot2 ou um `BootInfo` proprio), atualizar
  `docs/MEMORY_MAP.md` junto.
- O teto de 128 MiB do bitmap foi uma escolha deliberada para manter o
  allocator simples (array estatico, sem heap) enquanto o projeto ainda
  esta em fase de prototipo x86_64 com RAM padrao do QEMU.

# Registro de handoff entre IAs - Lay OS

Este arquivo existe para que qualquer IA (Claude, ChatGPT, etc.) que
continue o desenvolvimento do Lay OS saiba exatamente onde o trabalho
parou e o que vem a seguir, sem precisar reconstruir o contexto do zero.
Ao terminar uma sessao de trabalho, adicione uma nova secao "Entrada"
no topo (mais recente primeiro) e nao apague as entradas antigas.

---

## Entrada - 2026-09-27 (Claude, sessao 2, via GitHub direto no repo)

### O que foi feito
- `kernel/user.rs`: a tabela de paginas (PT) do processo de usuario
  deixou de usar um endereco fisico hardcoded (0x103000) e passou a vir
  de `memory::alloc_page()`. PML4/PDPT/PD continuam fixos (sao as
  estruturas do boot loader, referenciadas por CR3), so o nivel extra
  adicionado para o processo de usuario agora e alocado de verdade.
  Comportamento observavel deveria ser identico (a primeira chamada de
  `alloc_page()` retorna deterministicamente 0x200000), mas o codigo
  nao assume mais silenciosamente que um frame fixo esta sempre livre.

### O que eu avaliei e decidi NAO fazer nesta sessao (leia antes de mexer em storage)
- Cheguei a considerar substituir `kernel/block.rs` (hoje um RAM-disk
  puro, ver `DISK: [[u8; 512]; 128]`) por um driver ATA/IDE PIO real
  (portas 0x1F0-0x1F7), que seria a "camada 4" do roadmap
  ("Drivers de armazenamento").
- **Nao fiz isso porque e arriscado sem conseguir rodar QEMU/CI a partir
  daqui para validar.** O motivo especifico: o unico disco que o QEMU
  usa hoje (ver `Makefile` e `.github/workflows/build.yml`, flag
  `-drive format=raw,file=build/lay-os.img`) e a PROPRIA imagem de
  boot+kernel - os primeiros ~1024 setores (LBA 1 em diante) sao o
  binario do kernel carregado no boot. O `block.rs` atual usa
  `BLOCK_COUNT = 128` (LBA 0-127 se fosse disco real), que colide
  direto com essa regiao. Um driver ATA real escrevendo ali corromperia
  a propria imagem de boot entre uma execucao e outra (o `make` recria
  a imagem do zero, entao nao quebraria o CI, mas seria uma pegadinha
  feia para quem reusa uma imagem local sem rebuildar, e semanticamente
  errado: nao sobra espaco livre nesses LBAs).
- **Se for implementar o driver ATA real, fazer nesta ordem:**
  1. Aumentar `IMAGE_SIZE` no `Makefile` para sobrar espaco livre depois
     do kernel (hoje kernel ocupa ate LBA ~1024; reservar um `LBA_BASE`
     bem acima disso, ex.: 2048, mais uma folga de seguranca).
  2. `kernel/block.rs`: LBA real usado = `LBA_BASE + block_index`, nunca
     `block_index` puro.
  3. Adicionar `inw`/`outw` em `kernel/arch.rs` (so existe `inb`/`outb`
     hoje) para os registradores de dados ATA (16 bits).
  4. Implementar leitura/escrita LBA28 em modo PIO com **timeout
     limitado** no polling de BSY/DRQ (nunca `loop` infinito esperando o
     status - se o disco nao responder, retornar `false` em vez de
     travar o boot).
  5. So depois disso: validar rodando localmente ou via CI antes de
     confiar no resultado - isso e algo que uma IA sem execucao real
     nao deveria declarar "pronto" sem essa validacao humana/CI.

### Estado depois desta mudanca
- `kernel/user.rs` usa o allocator real para a PT do processo de
  usuario. `kernel/process.rs` e `kernel/scheduler.rs` ainda tratam
  exatamente um processo de usuario fixo (pid 1); isso e o item 1 do
  roadmap do README ("gerenciamento real de processos, threads e
  address spaces") e ainda nao foi atacado - e uma mudanca bem maior
  (multiplos processos, carregamento de codigo variavel) que merece uma
  sessao propria.
- Ainda nao validei nada disso rodando em QEMU real (mesma limitacao da
  entrada anterior).

### Proximos passos sugeridos (ordem sugerida, atualizada)
1. **Validar em build real** as mudancas desta sessao e da anterior:
   `make` + QEMU, conferir boot completo e `meminfo` no Lay Shell.
2. Driver de armazenamento real (ver secao acima com o passo a passo e
   os riscos ja levantados) - camada 4 do roadmap.
3. Gerenciamento real de processos/threads (item 1 do roadmap): hoje
   `kernel/process.rs` so reconhece o pid 1 fixo; `kernel/scheduler.rs`
   tem `MAX_TASKS = 4` mas so preenche os slots 0 e 1.
4. VFS/LayFS persistente de verdade (depende do passo 2 primeiro).
5. Se for necessario testar com mais de 128 MiB de RAM, aumentar
   `MAX_FRAMES` em `kernel/memory.rs` ou tornar o bitmap de tamanho
   dinamico (hoje e um array estatico fixo).

---

## Entrada - 2026-09-27 (Claude, sessao 1, via GitHub direto no repo)

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
